// SPDX-License-Identifier: MIT OR Apache-2.0
//! Run the EKF over a simulated constant-current discharge.
use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};

fn main() -> Result<(), SocError> {
    let ocv = OcvCurve::new([(0.0, 3.0), (0.2, 3.5), (0.5, 3.7), (0.8, 3.95), (1.0, 4.2)])?;
    let params = CellParams {
        capacity_ah: 2.0,
        r0: 0.05,
        r1: 0.02,
        c1: 1000.0,
    };
    let model = Model::new(params, ocv)?;
    let noise = NoiseConfig {
        process_soc: 1e-9,
        process_v1: 1e-6,
        measurement: 1e-4,
    };
    // Deliberately wrong initial guess (0.5) vs true SoC (0.9).
    let mut filter = Filter::new(model.clone(), noise)?.initialize(0.5, 0.1, 1e-4)?;
    let (mut soc, mut v1) = (0.9, 0.0);
    for k in 0..1800 {
        let (i, dt) = (1.0, 1.0);
        (soc, v1) = model.step(soc, v1, i, dt)?;
        filter.predict(i, dt)?;
        filter.update(model.terminal_voltage(soc, v1, i), i)?;
        if k % 300 == 0 {
            println!("t={k:4}s true={soc:.4} est={:.4}", filter.estimate().soc);
        }
    }
    Ok(())
}
