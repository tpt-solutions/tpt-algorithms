// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt soc`: battery state-of-charge estimation with the EKF.
//!
//! Input columns: `time_s`, `current_a` (positive on discharge), `voltage_v`.
//! The OCV curve is the built-in 11-point table below; the cell parameters
//! and noise levels are options.

use super::{read_table, Report};
use crate::args::Args;
use crate::error::algo;
use crate::error::CliError;
use crate::json::Json;
use crate::source::Source;
use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve};

/// Built-in open-circuit-voltage table, SoC (0 to 1) against volts.
pub const DEFAULT_OCV: [(f64, f64); 11] = [
    (0.0, 3.00),
    (0.1, 3.45),
    (0.2, 3.60),
    (0.3, 3.67),
    (0.4, 3.71),
    (0.5, 3.75),
    (0.6, 3.80),
    (0.7, 3.86),
    (0.8, 3.95),
    (0.9, 4.06),
    (1.0, 4.20),
];

/// Runs the subcommand. `args` holds the tokens after `soc`.
pub fn run(mut args: Args, source: &mut dyn Source) -> Result<Report, CliError> {
    let input = args
        .take_value("--input")?
        .ok_or_else(|| CliError::Usage("soc needs --input <file|->".to_string()))?;
    let capacity_ah = args.take_parsed("--capacity-ah", 2.0)?;
    let r0 = args.take_parsed("--r0", 0.05)?;
    let r1 = args.take_parsed("--r1", 0.02)?;
    let c1 = args.take_parsed("--c1", 1000.0)?;
    let soc0 = args.take_parsed("--soc0", 0.5)?;
    let soc_variance = args.take_parsed("--soc-variance", 0.1)?;
    let v1_variance = args.take_parsed("--v1-variance", 1e-4)?;
    let process_soc = args.take_parsed("--process-soc", 1e-9)?;
    let process_v1 = args.take_parsed("--process-v1", 1e-6)?;
    let measurement = args.take_parsed("--measurement", 1e-4)?;
    if !args.positionals()?.is_empty() {
        return Err(CliError::Usage(
            "soc takes no positional arguments".to_string(),
        ));
    }

    let table = read_table(&input, source, true)?;
    for col in ["time_s", "current_a", "voltage_v"] {
        table.require(col)?;
    }

    let ocv = OcvCurve::new(DEFAULT_OCV).map_err(algo)?;
    let params = CellParams {
        capacity_ah,
        r0,
        r1,
        c1,
    };
    let noise = NoiseConfig {
        process_soc,
        process_v1,
        measurement,
    };
    let model = Model::new(params, ocv).map_err(algo)?;
    let mut filter = Filter::new(model, noise)
        .map_err(algo)?
        .initialize(soc0, soc_variance, v1_variance)
        .map_err(algo)?;

    let mut samples = Vec::with_capacity(table.len());
    let mut prev_time: Option<f64> = None;
    for row in 0..table.len() {
        let time = table.num(row, "time_s")?;
        let current = table.num(row, "current_a")?;
        let voltage = table.num(row, "voltage_v")?;
        if let Some(prev) = prev_time {
            let dt = time - prev;
            if dt < 0.0 {
                return Err(CliError::Input(format!(
                    "row {}: time_s decreases",
                    row + 1
                )));
            }
            if dt > 0.0 {
                filter.predict(current, dt).map_err(algo)?;
            }
        }
        prev_time = Some(time);
        filter.update(voltage, current).map_err(algo)?;
        let est = filter.estimate();
        samples.push(Json::obj([
            ("row", row.into()),
            ("time_s", time.into()),
            ("soc", est.soc.into()),
            ("v1", est.v1.into()),
            ("soc_variance", est.soc_variance.into()),
        ]));
    }

    let rows = Json::Array(samples);
    Ok(Report {
        json: Json::obj([("samples", rows.clone())]),
        csv: Some(rows),
    })
}
