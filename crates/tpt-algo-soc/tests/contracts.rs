// SPDX-License-Identifier: MIT OR Apache-2.0
//! Range and quantity contracts of the EKF, checked at the public API.
//!
//! `Filter::predict` debug-asserts that SoC is in `[0, 1]` and that the
//! covariance diagonal is non-negative. These tests drive the filter with
//! extreme but valid inputs and check the same contracts on every step, so a
//! violation fails in release builds as well (the debug asserts only fire in
//! debug builds).
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use tpt_algo_soc::{CellParams, Filter, Initialized, Model, NoiseConfig, OcvCurve, SocError};

type F = Filter<Initialized, 3>;

fn model() -> Model<3> {
    let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)]).unwrap();
    let params = CellParams {
        capacity_ah: 2.0,
        r0: 0.05,
        r1: 0.02,
        c1: 1000.0,
    };
    Model::new(params, ocv).unwrap()
}

fn noise() -> NoiseConfig {
    NoiseConfig {
        process_soc: 1e-9,
        process_v1: 1e-6,
        measurement: 1e-4,
    }
}

fn filter(soc: f64, soc_var: f64, v1_var: f64) -> F {
    Filter::new(model(), noise())
        .unwrap()
        .initialize(soc, soc_var, v1_var)
        .unwrap()
}

/// Checks the contracts that must hold after every public call.
fn assert_contracts(f: &F) {
    let est = f.estimate();
    assert!(
        est.soc.is_finite() && (0.0..=1.0).contains(&est.soc),
        "soc {}",
        est.soc
    );
    assert!(est.v1.is_finite(), "v1 {}", est.v1);
    let p = f.covariance();
    assert!(p[0][0] >= 0.0 && p[1][1] >= 0.0, "diag {p:?}");
    assert_eq!(p[0][1], p[1][0], "covariance must be symmetric");
    assert!(
        p[0][0] * p[1][1] - p[0][1] * p[1][0] >= -1e-12,
        "covariance not PSD {p:?}"
    );
    assert_eq!(est.soc_variance, p[0][0]);
}

#[test]
fn initialize_accepts_range_endpoints_and_rejects_outside() {
    for soc in [0.0, 1.0] {
        let f = filter(soc, 0.0, 0.0);
        assert_contracts(&f);
    }
    let base = Filter::new(model(), noise()).unwrap();
    assert_eq!(
        base.clone().initialize(-1e-12, 0.1, 0.1).err(),
        Some(SocError::InvalidInitialState)
    );
    assert_eq!(
        base.clone().initialize(1.0 + 1e-12, 0.1, 0.1).err(),
        Some(SocError::InvalidInitialState)
    );
    assert_eq!(
        base.initialize(0.5, -1e-9, 0.1).err(),
        Some(SocError::InvalidInitialState)
    );
}

#[test]
fn soc_stays_in_unit_interval_under_extreme_discharge() {
    // Full-scale discharge and charge at a large time step; SoC must clamp.
    let mut f = filter(0.02, 0.1, 1e-4);
    for _ in 0..50 {
        f.predict(1e6, 1e4).unwrap();
        f.update(-100.0, 1e6).unwrap();
        assert_contracts(&f);
    }

    let mut f = filter(0.98, 0.1, 1e-4);
    for _ in 0..50 {
        f.predict(-1e6, 1e4).unwrap();
        f.update(100.0, -1e6).unwrap();
        assert_contracts(&f);
    }
}

#[test]
fn covariance_diagonal_stays_non_negative_for_tiny_steps_and_noise() {
    // Very small dt and very small measurement variance stress the Joseph
    // update; the diagonal must remain non-negative after every call.
    let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)]).unwrap();
    let params = CellParams {
        capacity_ah: 2.0,
        r0: 0.0,
        r1: 1e-6,
        c1: 1.0,
    };
    let tiny = NoiseConfig {
        process_soc: 0.0,
        process_v1: 0.0,
        measurement: 1e-12,
    };
    let mut f = Filter::new(Model::new(params, ocv).unwrap(), tiny)
        .unwrap()
        .initialize(0.5, 1e-12, 1e-12)
        .unwrap();
    for k in 0..200 {
        let i = if k % 2 == 0 { 1e-9 } else { -1e-9 };
        f.predict(i, 1e-9).unwrap();
        assert_contracts(&f);
        f.update(3.7 + 1e-9 * k as f64, i).unwrap();
        assert_contracts(&f);
    }
}

#[test]
fn rejected_inputs_leave_state_unchanged() {
    // Errors must not partially apply a step (the contract is "unchanged on error").
    let mut f = filter(0.5, 0.05, 1e-4);
    let before = (f.estimate(), f.covariance());
    assert!(f.predict(1.0, f64::NAN).is_err());
    assert!(f.update(f64::INFINITY, 1.0).is_err());
    assert_eq!((f.estimate(), f.covariance()), before);
    assert_contracts(&f);
}

proptest! {
    /// Random valid initial states and arbitrary finite inputs (including
    /// large magnitudes) keep the range and quantity contracts at every step.
    #[test]
    fn contracts_hold_for_arbitrary_finite_inputs(
        soc0 in 0.0f64..=1.0,
        var0 in 0.0f64..1.0,
        v1_var0 in 0.0f64..1.0,
        steps in proptest::collection::vec(
            (-1e4f64..1e4, 1e-3f64..1e4, -10.0f64..10.0),
            1..40,
        ),
    ) {
        let mut f = filter(soc0, var0, v1_var0);
        for (current, dt, volts) in steps {
            f.predict(current, dt).unwrap();
            assert_contracts(&f);
            f.update(3.5 + volts, current).unwrap();
            assert_contracts(&f);
        }
    }
}
