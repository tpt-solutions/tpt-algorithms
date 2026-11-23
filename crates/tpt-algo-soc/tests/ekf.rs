// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unit, property and synthetic-scenario tests for the SoC EKF.
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use tpt_algo_soc::{CellParams, Filter, Initialized, Model, NoiseConfig, OcvCurve, SocError};

type F = Filter<Initialized, 5>;

fn curve() -> OcvCurve<5> {
    OcvCurve::new([(0.0, 3.0), (0.2, 3.5), (0.5, 3.7), (0.8, 3.95), (1.0, 4.2)]).unwrap()
}

fn params() -> CellParams {
    CellParams {
        capacity_ah: 2.0,
        r0: 0.05,
        r1: 0.02,
        c1: 1000.0,
    }
}

fn noise() -> NoiseConfig {
    NoiseConfig {
        process_soc: 1e-9,
        process_v1: 1e-6,
        measurement: 1e-4,
    }
}

fn model() -> Model<5> {
    Model::new(params(), curve()).unwrap()
}

fn filter(soc: f64) -> F {
    Filter::new(model(), noise())
        .unwrap()
        .initialize(soc, 0.1, 1e-4)
        .unwrap()
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

fn is_psd(p: [[f64; 2]; 2]) -> bool {
    let tol = 1e-12;
    (p[0][1] - p[1][0]).abs() <= tol
        && p[0][0] >= -tol
        && p[1][1] >= -tol
        && p[0][0] * p[1][1] - p[0][1] * p[1][0] >= -tol
}

#[test]
fn ocv_interpolation_and_slope() {
    let c = curve();
    assert!((c.voltage(0.0) - 3.0).abs() < 1e-12);
    assert!((c.voltage(0.1) - 3.25).abs() < 1e-12);
    assert!((c.voltage(1.0) - 4.2).abs() < 1e-12);
    assert!((c.voltage(2.0) - 4.2).abs() < 1e-12); // clamped
    assert!((c.slope(0.1) - 2.5).abs() < 1e-12);
    assert!((c.slope(0.65) - 0.833_333_333_333).abs() < 1e-9);
}

#[test]
fn rejects_bad_ocv() {
    let bad = OcvCurve::new([(0.0, 3.0), (0.5, 3.5), (0.5, 3.7)]);
    assert!(matches!(bad, Err(SocError::InvalidOcvTable(_))));
    let bad = OcvCurve::new([(0.0, 3.0), (1.5, 3.5)]);
    assert!(matches!(bad, Err(SocError::InvalidOcvTable(_))));
    let bad = OcvCurve::new([(0.0, f64::NAN), (1.0, 3.5)]);
    assert!(matches!(bad, Err(SocError::InvalidOcvTable(_))));
    let bad = OcvCurve::new([(0.5, 3.0)]);
    assert!(matches!(bad, Err(SocError::InvalidOcvTable(_))));
}

#[test]
fn rejects_bad_params() {
    let mk = |f: fn(&mut CellParams)| {
        let mut p = params();
        f(&mut p);
        Model::new(p, curve())
    };
    assert_eq!(mk(|p| p.capacity_ah = 0.0), Err(SocError::InvalidCapacity));
    assert_eq!(
        mk(|p| p.capacity_ah = f64::NAN),
        Err(SocError::InvalidCapacity)
    );
    assert_eq!(mk(|p| p.r1 = 0.0), Err(SocError::InvalidResistance));
    assert_eq!(mk(|p| p.r0 = -1.0), Err(SocError::InvalidResistance));
    assert_eq!(mk(|p| p.c1 = -5.0), Err(SocError::InvalidCapacitance));
}

#[test]
fn rejects_bad_noise_and_init() {
    let mut n = noise();
    n.measurement = 0.0;
    assert_eq!(Filter::new(model(), n).err(), Some(SocError::InvalidNoise));
    let f = Filter::new(model(), noise()).unwrap();
    assert_eq!(
        f.clone().initialize(1.5, 0.1, 0.1).err(),
        Some(SocError::InvalidInitialState)
    );
    assert_eq!(
        f.initialize(0.5, f64::INFINITY, 0.1).err(),
        Some(SocError::InvalidInitialState)
    );
}

#[test]
fn rejects_bad_runtime_inputs() {
    let mut f = filter(0.5);
    assert_eq!(f.predict(1.0, 0.0), Err(SocError::InvalidTimeStep));
    assert_eq!(f.predict(1.0, -1.0), Err(SocError::InvalidTimeStep));
    assert_eq!(
        f.predict(f64::NAN, 1.0),
        Err(SocError::NonFinite("current"))
    );
    assert_eq!(f.update(f64::NAN, 1.0), Err(SocError::NonFinite("voltage")));
    assert_eq!(
        f.update(3.7, f64::INFINITY),
        Err(SocError::NonFinite("current"))
    );
    assert!((f.estimate().soc - 0.5).abs() < 1e-15);
}

#[test]
fn coulomb_counting_known_answer() {
    // 2 Ah cell, 2 A for 1800 s removes 1 Ah = 0.5 SoC.
    let mut f = filter(1.0);
    for _ in 0..1800 {
        f.predict(2.0, 1.0).unwrap();
    }
    assert!((f.estimate().soc - 0.5).abs() < 1e-9);
}

#[test]
fn v1_steady_state_known_answer() {
    // Constant current: V1 -> R1 * I.
    let mut f = filter(1.0);
    for _ in 0..1000 {
        f.predict(1.0, 100.0).unwrap();
    }
    assert!((f.estimate().v1 - 0.02).abs() < 1e-9);
}

#[test]
fn converges_from_wrong_initial_soc_noiseless() {
    let m = model();
    let mut f = filter(0.5);
    let (mut soc, mut v1) = (0.9, 0.0);
    for _ in 0..1800 {
        (soc, v1) = m.step(soc, v1, 1.0, 1.0).unwrap();
        f.predict(1.0, 1.0).unwrap();
        f.update(m.terminal_voltage(soc, v1, 1.0), 1.0).unwrap();
    }
    assert!((f.estimate().soc - soc).abs() < 5e-3);
}

#[test]
fn synthetic_noisy_discharge_rmse() {
    let m = model();
    let mut rng = SplitMix64(42);
    let mut f = filter(0.6);
    let (mut soc, mut v1) = (0.95, 0.0);
    let mut sq = 0.0;
    let mut n = 0.0;
    for k in 0..3000 {
        // Pulsed discharge profile.
        let i = if (k / 100) % 2 == 0 { 2.0 } else { 0.5 };
        (soc, v1) = m.step(soc, v1, i, 1.0).unwrap();
        // Uniform noise in +-10 mV (std ~ 5.8 mV; filter assumes 10 mV).
        let z = m.terminal_voltage(soc, v1, i) + 0.01 * rng.sym();
        f.predict(i, 1.0).unwrap();
        f.update(z, i).unwrap();
        if k >= 500 {
            sq += (f.estimate().soc - soc).powi(2);
            n += 1.0;
        }
    }
    let rmse = (sq / n).sqrt();
    assert!(rmse < 0.02, "rmse {rmse}");
}

proptest! {
    #[test]
    fn soc_in_unit_interval_and_cov_psd(
        soc0 in 0.0f64..=1.0,
        seed in any::<u64>(),
        currents in proptest::collection::vec(-5.0f64..5.0, 1..200),
        dt in 0.1f64..60.0,
    ) {
        let mut rng = SplitMix64(seed);
        let mut f = filter(soc0);
        for i in currents {
            f.predict(i, dt).unwrap();
            // Arbitrary, possibly wildly inconsistent, measurement.
            let z = 2.5 + 2.0 * (rng.sym() + 1.0);
            f.update(z, i).unwrap();
            let e = f.estimate();
            prop_assert!((0.0..=1.0).contains(&e.soc));
            prop_assert!(e.v1.is_finite());
            prop_assert!(is_psd(f.covariance()), "{:?}", f.covariance());
        }
    }
}
