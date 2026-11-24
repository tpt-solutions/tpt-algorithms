// SPDX-License-Identifier: MIT OR Apache-2.0
//! Extended Kalman filter wrapper for battery state of charge.

use crate::error::{invalid_config, BindingError, ErrorKind, Result};
use tpt_algo_soc::{
    CellParams, Estimate, Filter, Initialized, Model, NoiseConfig, OcvCurve, SocError,
};

/// Smallest supported OCV table length.
pub const MIN_OCV_POINTS: usize = 2;
/// Largest supported OCV table length.
pub const MAX_OCV_POINTS: usize = 8;

/// Default noise and initial-variance values, shared by the Python and
/// WebAssembly bindings.
///
/// The SoC crate does not yet provide `Default` for `NoiseConfig` (Phase 19 in
/// `todo.md`), so these values are repeated here. The parity fixtures check
/// that both bindings use them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EkfDefaults {
    /// Initial variance of the RC-pair voltage.
    pub v1_variance: f64,
    /// Per-step process variance on SoC.
    pub process_soc: f64,
    /// Per-step process variance on the RC voltage.
    pub process_v1: f64,
    /// Terminal-voltage measurement variance.
    pub measurement: f64,
}

/// The binding defaults for the EKF.
pub const EKF_DEFAULTS: EkfDefaults = EkfDefaults {
    v1_variance: 1e-4,
    process_soc: 1e-9,
    process_v1: 1e-6,
    measurement: 1e-4,
};

/// Cell and noise parameters for [`Ekf::new`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellSpec {
    /// Capacity in ampere-hours (> 0).
    pub capacity_ah: f64,
    /// Series resistance in ohms (>= 0).
    pub r0: f64,
    /// RC-pair resistance in ohms (> 0).
    pub r1: f64,
    /// RC-pair capacitance in farads (> 0).
    pub c1: f64,
    /// Process variance on SoC per step (>= 0).
    pub process_soc: f64,
    /// Process variance on the RC voltage per step (>= 0).
    pub process_v1: f64,
    /// Terminal-voltage measurement variance (> 0).
    pub measurement: f64,
}

enum EkfInner {
    N2(Filter<Initialized, 2>),
    N3(Filter<Initialized, 3>),
    N4(Filter<Initialized, 4>),
    N5(Filter<Initialized, 5>),
    N6(Filter<Initialized, 6>),
    N7(Filter<Initialized, 7>),
    N8(Filter<Initialized, 8>),
}

/// Applies `$body` to the filter inside `$inner`, whatever its table length.
macro_rules! dispatch {
    ($inner:expr, $f:ident => $body:expr) => {
        match $inner {
            EkfInner::N2($f) => $body,
            EkfInner::N3($f) => $body,
            EkfInner::N4($f) => $body,
            EkfInner::N5($f) => $body,
            EkfInner::N6($f) => $body,
            EkfInner::N7($f) => $body,
            EkfInner::N8($f) => $body,
        }
    };
}

fn build<const N: usize>(
    table: &[(f64, f64)],
    cell: &CellSpec,
    soc0: f64,
    soc_variance: f64,
    v1_variance: f64,
) -> Result<Filter<Initialized, N>> {
    let points: [(f64, f64); N] = <[(f64, f64); N]>::try_from(table)
        .map_err(|_| invalid_config("OCV table length mismatch"))?;
    let ocv = OcvCurve::new(points).map_err(soc_error)?;
    let params = CellParams {
        capacity_ah: cell.capacity_ah,
        r0: cell.r0,
        r1: cell.r1,
        c1: cell.c1,
    };
    let model = Model::new(params, ocv).map_err(soc_error)?;
    let noise = NoiseConfig {
        process_soc: cell.process_soc,
        process_v1: cell.process_v1,
        measurement: cell.measurement,
    };
    Filter::new(model, noise)
        .and_then(|f| f.initialize(soc0, soc_variance, v1_variance))
        .map_err(soc_error)
}

fn soc_error(e: SocError) -> BindingError {
    let kind = match e {
        SocError::NonFinite(_) | SocError::InvalidTimeStep => ErrorKind::InvalidInput,
        _ => ErrorKind::InvalidConfig,
    };
    BindingError::new(kind, e.to_string())
}

/// Extended Kalman filter for battery state of charge, with a runtime-sized
/// OCV table.
pub struct Ekf {
    inner: EkfInner,
}

impl Ekf {
    /// Builds and initializes a filter.
    ///
    /// `ocv` is the table of `(soc, volts)` pairs; its length must be between
    /// [`MIN_OCV_POINTS`] and [`MAX_OCV_POINTS`].
    ///
    /// # Errors
    /// Unsupported table length, or any validation error from the SoC crate.
    pub fn new(
        ocv: &[(f64, f64)],
        cell: CellSpec,
        soc0: f64,
        soc_variance: f64,
        v1_variance: f64,
    ) -> Result<Self> {
        let inner = match ocv.len() {
            2 => EkfInner::N2(build::<2>(ocv, &cell, soc0, soc_variance, v1_variance)?),
            3 => EkfInner::N3(build::<3>(ocv, &cell, soc0, soc_variance, v1_variance)?),
            4 => EkfInner::N4(build::<4>(ocv, &cell, soc0, soc_variance, v1_variance)?),
            5 => EkfInner::N5(build::<5>(ocv, &cell, soc0, soc_variance, v1_variance)?),
            6 => EkfInner::N6(build::<6>(ocv, &cell, soc0, soc_variance, v1_variance)?),
            7 => EkfInner::N7(build::<7>(ocv, &cell, soc0, soc_variance, v1_variance)?),
            8 => EkfInner::N8(build::<8>(ocv, &cell, soc0, soc_variance, v1_variance)?),
            n => {
                return Err(BindingError::new(
                    ErrorKind::Unsupported,
                    format!(
                        "OCV table must have {MIN_OCV_POINTS} to {MAX_OCV_POINTS} points, got {n}"
                    ),
                ))
            }
        };
        Ok(Self { inner })
    }

    /// Time update over `dt` seconds at discharge current `current` (amps).
    ///
    /// # Errors
    /// Invalid time step or non-finite current.
    pub fn predict(&mut self, current: f64, dt: f64) -> Result<()> {
        dispatch!(&mut self.inner, f => f.predict(current, dt).map_err(soc_error))
    }

    /// Measurement update with terminal voltage `voltage` under `current`.
    ///
    /// # Errors
    /// Non-finite input.
    pub fn update(&mut self, voltage: f64, current: f64) -> Result<()> {
        dispatch!(&mut self.inner, f => f.update(voltage, current).map_err(soc_error))
    }

    /// Current estimate.
    #[must_use]
    pub fn estimate(&self) -> Estimate {
        dispatch!(&self.inner, f => f.estimate())
    }

    /// The 2x2 error covariance of `(soc, v1)`, row-major.
    #[must_use]
    pub fn covariance(&self) -> [[f64; 2]; 2] {
        dispatch!(&self.inner, f => f.covariance())
    }

    /// The cell parameters the filter was built with.
    #[must_use]
    pub fn cell_params(&self) -> CellParams {
        dispatch!(&self.inner, f => *f.model().params())
    }

    /// Open-circuit voltage at `soc` from the piecewise-linear OCV curve.
    #[must_use]
    pub fn ocv_voltage(&self, soc: f64) -> f64 {
        dispatch!(&self.inner, f => f.model().ocv().voltage(soc))
    }

    /// Predicted terminal voltage for a state `(soc, v1)` at current `current`.
    #[must_use]
    pub fn terminal_voltage(&self, soc: f64, v1: f64, current: f64) -> f64 {
        dispatch!(&self.inner, f => f.model().terminal_voltage(soc, v1, current))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn cell() -> CellSpec {
        CellSpec {
            capacity_ah: 2.0,
            r0: 0.05,
            r1: 0.02,
            c1: 1000.0,
            process_soc: 1e-9,
            process_v1: 1e-6,
            measurement: 1e-4,
        }
    }

    #[test]
    fn ekf_supports_each_table_length() {
        for n in MIN_OCV_POINTS..=MAX_OCV_POINTS {
            let table: Vec<(f64, f64)> = (0..n)
                .map(|i| (i as f64 / (n - 1) as f64, 3.0 + i as f64 * 0.1))
                .collect();
            let mut ekf = Ekf::new(&table, cell(), 0.8, 0.05, 1e-4).unwrap();
            let before = ekf.estimate().soc;
            ekf.predict(2.0, 600.0).unwrap();
            assert!(ekf.estimate().soc < before, "table length {n}");
        }
    }

    #[test]
    fn ekf_rejects_unsupported_table_lengths() {
        let one = [(0.0, 3.0)];
        assert!(Ekf::new(&one, cell(), 0.5, 0.05, 1e-4).is_err());
        let nine: Vec<(f64, f64)> = (0..9).map(|i| (i as f64 / 8.0, 3.0)).collect();
        let err = Ekf::new(&nine, cell(), 0.5, 0.05, 1e-4).err().unwrap();
        assert_eq!(err.kind, ErrorKind::Unsupported);
        assert!(err.message.contains("got 9"));
    }

    #[test]
    fn ekf_reports_invalid_cell() {
        let table = [(0.0, 3.0), (1.0, 4.2)];
        let bad = CellSpec {
            capacity_ah: 0.0,
            ..cell()
        };
        let err = Ekf::new(&table, bad, 0.5, 0.05, 1e-4).err().unwrap();
        assert_eq!(err.kind, ErrorKind::InvalidConfig);
    }

    #[test]
    fn helpers_are_consistent() {
        let table = [(0.0, 3.0), (1.0, 4.2)];
        let ekf = Ekf::new(&table, cell(), 0.5, 0.05, 1e-4).unwrap();
        assert!((ekf.ocv_voltage(0.5) - 3.6).abs() < 1e-12);
        let p = ekf.covariance();
        assert_eq!(p[0][1], p[1][0]);
        assert_eq!(ekf.cell_params().capacity_ah, 2.0);
        let v = ekf.terminal_voltage(0.5, 0.0, 0.0);
        assert!((v - 3.6).abs() < 1e-12);
    }
}
