// SPDX-License-Identifier: MIT OR Apache-2.0
//! Validation of the SoC pipeline: CSV loading, OCV / R0 / R1 / C1 fitting,
//! EKF replay and RMSE against a reference SoC.
//!
//! The offline test generates a CSV from a known cell model, then runs the
//! same loader, fit, replay and RMSE code that the dataset test uses. The
//! NASA Battery Dataset test is `#[ignore]`d and needs `TPT_DATASET_DIR`
//! (see `scripts/fetch-dataset.sh` and `docs/datasets.md`).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fmt::Write as _;
use std::path::PathBuf;

use tpt_algo_soc::{CellParams, Filter, Initialized, Model, NoiseConfig, OcvCurve, SocError};

/// Fixed SoC nodes of the fitted OCV table.
const NODES: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
/// Nominal capacity of the cell (ampere-hours). Taken from the cell datasheet,
/// not fitted: the OCV/R0/R1/C1 fit does not observe capacity directly.
const CAPACITY_AH: f64 = 2.0;
/// Bound on RMSE (SoC fraction) for the dataset replay. Placeholder until the
/// bound claimed by the cited paper is recorded in `docs/datasets.md`.
const DATASET_RMSE_BOUND: f64 = 0.03;

#[derive(Debug, Clone, Copy)]
struct Sample {
    /// Seconds since the start of the trace.
    t: f64,
    /// Discharge current in amps (positive on discharge).
    i: f64,
    /// Measured terminal voltage in volts.
    v: f64,
    /// Reference (Coulomb-counted) state of charge in `[0, 1]`.
    soc_ref: f64,
}

/// Parse `time_s,current_a,voltage_v,soc_ref` CSV text.
fn parse_csv(text: &str) -> Result<Vec<Sample>, String> {
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    let header = lines.next().ok_or("empty CSV")?;
    if header != "time_s,current_a,voltage_v,soc_ref" {
        return Err(format!("unexpected header: {header}"));
    }
    let mut out: Vec<Sample> = Vec::new();
    for (row, line) in lines.enumerate() {
        let line_no = row + 2;
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        if fields.len() != 4 {
            return Err(format!(
                "line {line_no}: expected 4 fields, got {}",
                fields.len()
            ));
        }
        let mut vals = [0.0f64; 4];
        for (slot, field) in vals.iter_mut().zip(&fields) {
            *slot = field
                .parse::<f64>()
                .map_err(|e| format!("line {line_no}: {field:?}: {e}"))?;
            if !slot.is_finite() {
                return Err(format!("line {line_no}: non-finite value"));
            }
        }
        let [t, i, v, soc_ref] = vals;
        if !(0.0..=1.0).contains(&soc_ref) {
            return Err(format!("line {line_no}: soc_ref outside [0,1]"));
        }
        if let Some(prev) = out.last() {
            if t <= prev.t {
                return Err(format!("line {line_no}: time not strictly increasing"));
            }
        }
        out.push(Sample { t, i, v, soc_ref });
    }
    if out.len() < 2 {
        return Err("need at least two samples".to_owned());
    }
    Ok(out)
}

/// Fitted cell: OCV at [`NODES`], R0, R1 and C1.
#[derive(Debug, Clone, Copy)]
struct Fit {
    ocv: [f64; 5],
    r0: f64,
    r1: f64,
    c1: f64,
    /// RMSE of the static fit (terminal voltage residual, volts).
    voltage_rmse: f64,
}

/// Hat-function weights of the piecewise-linear OCV over [`NODES`].
fn hat_weights(soc: f64) -> [f64; 5] {
    let x = soc.clamp(0.0, 1.0) * 4.0;
    let seg = (x.floor() as usize).min(3);
    let frac = x - seg as f64;
    let mut w = [0.0; 5];
    w[seg] = 1.0 - frac;
    w[seg + 1] = frac;
    w
}

/// RC-pair response to unit R1 for time constant `tau = R1 * C1`:
/// `V1_k = R1 * u_k`. Uses the same discretisation as `Model::step`.
fn unit_rc_response(samples: &[Sample], tau: f64) -> Vec<f64> {
    let mut u = vec![0.0; samples.len()];
    for k in 0..samples.len() - 1 {
        let dt = samples[k + 1].t - samples[k].t;
        let a = (-dt / tau).exp();
        u[k + 1] = u[k] * a + (1.0 - a) * samples[k].i;
    }
    u
}

/// Solve a 6x6 linear system by Gaussian elimination with partial pivoting.
fn solve6(mut a: [[f64; 6]; 6], mut b: [f64; 6]) -> Option<[f64; 6]> {
    for col in 0..6 {
        let pivot = (col..6).max_by(|&x, &y| a[x][col].abs().total_cmp(&a[y][col].abs()))?;
        if a[pivot][col].abs() < 1e-14 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..6 {
            let f = a[row][col] / a[col][col];
            for c in col..6 {
                a[row][c] -= f * a[col][c];
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = [0.0; 6];
    for row in (0..6).rev() {
        let mut s = b[row];
        for c in row + 1..6 {
            s -= a[row][c] * x[c];
        }
        x[row] = s / a[row][row];
    }
    Some(x)
}

/// Linear least squares for the OCV nodes and R0 with R1 and the RC response
/// fixed: `v_k + R1*u_k = sum_j w_j(soc_k) OCV_j - R0 * i_k`.
fn fit_static(samples: &[Sample], u: &[f64], r1: f64) -> Option<([f64; 5], f64, f64)> {
    let mut ata = [[0.0; 6]; 6];
    let mut atb = [0.0; 6];
    let rows: Vec<([f64; 6], f64)> = samples
        .iter()
        .zip(u)
        .map(|(s, &uk)| {
            let w = hat_weights(s.soc_ref);
            let row = [w[0], w[1], w[2], w[3], w[4], -s.i];
            (row, s.v + r1 * uk)
        })
        .collect();
    for (row, b) in &rows {
        for p in 0..6 {
            for q in 0..6 {
                ata[p][q] += row[p] * row[q];
            }
            atb[p] += row[p] * b;
        }
    }
    let theta = solve6(ata, atb)?;
    let ocv = [theta[0], theta[1], theta[2], theta[3], theta[4]];
    let r0 = theta[5];
    let sse: f64 = rows
        .iter()
        .map(|(row, b)| {
            let pred: f64 = row.iter().zip(&theta).map(|(a, t)| a * t).sum();
            (pred - b).powi(2)
        })
        .sum();
    Some((ocv, r0, (sse / rows.len() as f64).sqrt()))
}

/// Log-spaced grid of `n` points over `[lo, hi]`.
fn log_grid(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    let (a, b) = (lo.ln(), hi.ln());
    (0..n)
        .map(|k| (a + (b - a) * k as f64 / (n - 1) as f64).exp())
        .collect()
}

/// Fit OCV, R0, R1, C1 to a discharge trace. Grid search over the RC time
/// constant and R1 (both log-spaced, coarse then refined); for each pair the
/// remaining parameters are solved exactly by linear least squares.
fn fit_cell(samples: &[Sample]) -> Option<Fit> {
    let mut best: Option<(f64, f64, [f64; 5], f64, f64)> = None;
    let consider = |tau: f64, r1: f64, best: &mut Option<(f64, f64, [f64; 5], f64, f64)>| {
        let u = unit_rc_response(samples, tau);
        if let Some((ocv, r0, rmse)) = fit_static(samples, &u, r1) {
            if best.as_ref().is_none_or(|b| rmse < b.4) {
                *best = Some((tau, r1, ocv, r0, rmse));
            }
        }
    };
    for &tau in &log_grid(1.0, 500.0, 40) {
        for &r1 in &log_grid(1e-3, 0.5, 40) {
            consider(tau, r1, &mut best);
        }
    }
    let (tau0, r10, ..) = best?;
    for &tau in &log_grid(tau0 / 1.2, tau0 * 1.2, 21) {
        for &r1 in &log_grid(r10 / 1.2, r10 * 1.2, 21) {
            consider(tau, r1, &mut best);
        }
    }
    let (tau, r1, ocv, r0, voltage_rmse) = best?;
    Some(Fit {
        ocv,
        r0,
        r1,
        c1: tau / r1,
        voltage_rmse,
    })
}

fn build_model(fit: &Fit) -> Result<Model<5>, SocError> {
    let points: [(f64, f64); 5] = core::array::from_fn(|j| (NODES[j], fit.ocv[j]));
    let ocv = OcvCurve::new(points)?;
    let params = CellParams {
        capacity_ah: CAPACITY_AH,
        r0: fit.r0,
        r1: fit.r1,
        c1: fit.c1,
    };
    Model::new(params, ocv)
}

/// Replay the trace through the EKF, starting from a deliberately wrong SoC
/// guess. Returns the RMSE of the estimated SoC against the reference SoC.
fn replay_rmse(samples: &[Sample], fit: &Fit, initial_soc: f64) -> Result<f64, SocError> {
    let model = build_model(fit)?;
    let noise = NoiseConfig {
        process_soc: 1e-9,
        process_v1: 1e-6,
        measurement: 1e-4,
    };
    let mut f: Filter<Initialized, 5> =
        Filter::new(model, noise)?.initialize(initial_soc, 0.05, 1e-4)?;
    let mut sq = 0.0;
    for (k, s) in samples.iter().enumerate() {
        if k > 0 {
            let prev = samples[k - 1];
            f.predict(prev.i, s.t - prev.t)?;
        }
        f.update(s.v, s.i)?;
        sq += (f.estimate().soc - s.soc_ref).powi(2);
    }
    Ok((sq / samples.len() as f64).sqrt())
}

/// Known-model CSV: an RC cell (OCV, R0 = 0.05, R1 = 0.02, C1 = 1000 F) driven
/// by 1 A pulses (25 samples on, 5 off, 10 s per sample) until SoC is about
/// 0.03. Voltage carries seeded uniform noise of +-1 mV.
fn synthetic_csv() -> String {
    let ocv: OcvCurve<5> = OcvCurve::new(core::array::from_fn(|j| {
        (NODES[j], [3.0, 3.5, 3.7, 3.95, 4.2][j])
    }))
    .unwrap();
    let params = CellParams {
        capacity_ah: CAPACITY_AH,
        r0: 0.05,
        r1: 0.02,
        c1: 1000.0,
    };
    let model = Model::new(params, ocv).unwrap();
    let dt = 10.0;
    let mut rng = SplitMix64(42);
    let (mut soc, mut v1) = (1.0, 0.0);
    let mut out = String::from("time_s,current_a,voltage_v,soc_ref\n");
    for k in 0..840 {
        let i = if k % 30 < 25 { 1.0 } else { 0.0 };
        let v = model.terminal_voltage(soc, v1, i) + 1e-3 * rng.sym();
        writeln!(out, "{},{},{:.9},{:.9}", k as f64 * dt, i, v, soc).unwrap();
        (soc, v1) = model.step(soc, v1, i, dt).unwrap();
    }
    out
}

/// Seeded SplitMix64 for the synthetic measurement noise.
struct SplitMix64(u64);
impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [-1, 1).
    fn sym(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

#[test]
fn csv_loader_accepts_valid_and_rejects_malformed_input() {
    let good = "time_s,current_a,voltage_v,soc_ref\n0,1.0,3.9,0.9\n10,1.0,3.88,0.89\n";
    let samples = parse_csv(good).unwrap();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[1].t, 10.0);
    assert_eq!(samples[1].soc_ref, 0.89);

    assert!(
        parse_csv("a,b,c,d\n0,1,2,3\n0,1,2,3\n").is_err(),
        "bad header"
    );
    assert!(parse_csv(&format!(
        "{}0,1,2\n",
        "time_s,current_a,voltage_v,soc_ref\n"
    ))
    .is_err());
    assert!(parse_csv("time_s,current_a,voltage_v,soc_ref\n0,1,nan,0.5\n1,1,3,0.5\n").is_err());
    assert!(parse_csv("time_s,current_a,voltage_v,soc_ref\n0,1,3,1.5\n1,1,3,0.5\n").is_err());
    assert!(parse_csv("time_s,current_a,voltage_v,soc_ref\n5,1,3,0.5\n5,1,3,0.5\n").is_err());
    assert!(parse_csv("time_s,current_a,voltage_v,soc_ref\n0,1,3,0.5\n").is_err());
}

#[test]
fn synthetic_known_model_fit_and_replay_recover_truth() {
    let csv = synthetic_csv();
    let samples = parse_csv(&csv).expect("generated CSV must parse");
    assert_eq!(samples.len(), 840);

    let fit = fit_cell(&samples).expect("fit must succeed on a rich trace");
    // Truth: R0 = 0.05, R1 = 0.02, C1 = 1000 (tau = 20 s).
    assert!(
        (fit.r0 - 0.05).abs() < 0.005,
        "R0 {} not within 5 mOhm of 0.05",
        fit.r0
    );
    assert!(
        (fit.r1 - 0.02).abs() / 0.02 < 0.25,
        "R1 {} not within 25% of 0.02",
        fit.r1
    );
    let tau = fit.r1 * fit.c1;
    assert!(
        (tau - 20.0).abs() / 20.0 < 0.25,
        "tau {tau} not within 25% of 20 s"
    );
    // Fitted OCV nodes should track the truth curve.
    for (fitted, truth) in fit.ocv.iter().zip([3.0, 3.5, 3.7, 3.95, 4.2]) {
        assert!(
            (fitted - truth).abs() < 0.02,
            "OCV node {fitted} vs {truth}"
        );
    }
    assert!(
        fit.voltage_rmse < 0.002,
        "static fit residual {}",
        fit.voltage_rmse
    );

    let rmse = replay_rmse(&samples, &fit, 0.9).unwrap();
    assert!(rmse < 0.02, "replay SoC RMSE {rmse} exceeds 0.02");
}

#[test]
#[ignore = "requires NASA Battery Dataset; set TPT_DATASET_DIR (see scripts/fetch-dataset.sh, docs/datasets.md)"]
fn nasa_battery_dataset_soc_rmse() {
    let Some(dir) = std::env::var_os("TPT_DATASET_DIR").map(PathBuf::from) else {
        panic!("TPT_DATASET_DIR is not set; fetch the dataset with scripts/fetch-dataset.sh");
    };
    // The loader expects a CSV export of one reference discharge with the
    // columns time_s,current_a,voltage_v,soc_ref. Converting the .mat files
    // is a manual step and is not part of this repository.
    let path = dir.join("nasa_discharge.csv");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let samples = parse_csv(&text).unwrap_or_else(|e| panic!("bad CSV: {e}"));
    let fit = fit_cell(&samples).expect("fit must succeed on the dataset trace");
    let rmse = replay_rmse(&samples, &fit, samples[0].soc_ref).unwrap();
    assert!(
        rmse <= DATASET_RMSE_BOUND,
        "replay SoC RMSE {rmse} exceeds bound {DATASET_RMSE_BOUND}"
    );
}
