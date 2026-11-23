// SPDX-License-Identifier: MIT OR Apache-2.0
//! Battery model: OCV curve and first-order RC equivalent circuit.

use crate::SocError;

/// Piecewise-linear open-circuit-voltage curve with `N` breakpoints.
///
/// Breakpoints are `(soc, volts)` pairs with strictly increasing `soc`
/// inside `[0, 1]`.
///
/// # Example
///
/// ```
/// use tpt_algo_soc::OcvCurve;
///
/// let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)]).unwrap();
/// assert!((ocv.voltage(0.5) - 3.7).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct OcvCurve<const N: usize> {
    points: [(f64, f64); N],
}

impl<const N: usize> OcvCurve<N> {
    /// Validate and build a curve.
    ///
    /// # Errors
    /// [`SocError::InvalidOcvTable`] if `N < 2`, a value is non-finite, a SoC
    /// is outside `[0,1]`, or SoC is not strictly increasing.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::{OcvCurve, SocError};
    ///
    /// // SoC must be strictly increasing.
    /// assert!(matches!(
    ///     OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (0.5, 3.8)]),
    ///     Err(SocError::InvalidOcvTable(_))
    /// ));
    /// ```
    pub fn new(points: [(f64, f64); N]) -> Result<Self, SocError> {
        if N < 2 {
            return Err(SocError::InvalidOcvTable("need at least two points"));
        }
        let mut prev = f64::NEG_INFINITY;
        for &(s, v) in &points {
            if !s.is_finite() || !v.is_finite() {
                return Err(SocError::InvalidOcvTable("non-finite value"));
            }
            if !(0.0..=1.0).contains(&s) {
                return Err(SocError::InvalidOcvTable("soc outside [0,1]"));
            }
            if s <= prev {
                return Err(SocError::InvalidOcvTable("soc not strictly increasing"));
            }
            prev = s;
        }
        Ok(Self { points })
    }

    /// Index `i` of the segment `[points[i], points[i+1]]` used for `soc`.
    fn segment(&self, soc: f64) -> usize {
        let mut i = 0;
        while i + 2 < N && soc > self.points[i + 1].0 {
            i += 1;
        }
        i
    }

    /// OCV in volts at `soc`; `soc` is clamped to the table's range.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::OcvCurve;
    ///
    /// let ocv = OcvCurve::new([(0.0, 3.0), (1.0, 4.0)]).unwrap();
    /// assert!((ocv.voltage(0.25) - 3.25).abs() < 1e-12); // linear interpolation
    /// assert_eq!(ocv.voltage(2.0), 4.0); // clamped to the table
    /// ```
    #[must_use]
    pub fn voltage(&self, soc: f64) -> f64 {
        let lo = self.points[0].0;
        let hi = self.points[N - 1].0;
        let s = soc.clamp(lo, hi);
        let i = self.segment(s);
        let (s0, v0) = self.points[i];
        let (s1, v1) = self.points[i + 1];
        v0 + (v1 - v0) * (s - s0) / (s1 - s0)
    }

    /// Local slope `dOCV/dSoC` (volts per unit SoC) at `soc`.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::OcvCurve;
    ///
    /// let ocv = OcvCurve::new([(0.0, 3.0), (1.0, 4.0)]).unwrap();
    /// assert!((ocv.slope(0.5) - 1.0).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn slope(&self, soc: f64) -> f64 {
        let i = self.segment(soc);
        let (s0, v0) = self.points[i];
        let (s1, v1) = self.points[i + 1];
        (v1 - v0) / (s1 - s0)
    }
}

/// Scalar cell parameters.
///
/// # Example
///
/// ```
/// use tpt_algo_soc::CellParams;
///
/// let p = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
/// assert!(p.r1 > 0.0); // RC pair must have positive resistance and capacitance
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CellParams {
    /// Capacity in ampere-hours (> 0).
    pub capacity_ah: f64,
    /// Series resistance in ohms (>= 0).
    pub r0: f64,
    /// RC-pair resistance in ohms (> 0).
    pub r1: f64,
    /// RC-pair capacitance in farads (> 0).
    pub c1: f64,
}

/// Validated battery model: parameters plus OCV curve.
///
/// # Example
///
/// ```
/// use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};
/// # fn main() -> Result<(), SocError> {
/// let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)])?;
/// let params = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
/// let model = Model::new(params, ocv)?;
/// let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
/// assert_eq!(model.params().r0, 0.05);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Model<const N: usize> {
    pub(crate) params: CellParams,
    pub(crate) ocv: OcvCurve<N>,
}

impl<const N: usize> Model<N> {
    /// Validate parameters and build a model.
    ///
    /// # Errors
    /// [`SocError::InvalidCapacity`], [`SocError::InvalidResistance`] or
    /// [`SocError::InvalidCapacitance`] for out-of-range values.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::{CellParams, Model, OcvCurve, SocError};
    ///
    /// let ocv = OcvCurve::new([(0.0, 3.0), (1.0, 4.2)])?;
    /// let bad = CellParams { capacity_ah: 0.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
    /// assert!(matches!(Model::new(bad, ocv), Err(SocError::InvalidCapacity)));
    /// # Ok::<(), SocError>(())
    /// ```
    pub fn new(params: CellParams, ocv: OcvCurve<N>) -> Result<Self, SocError> {
        if !(params.capacity_ah.is_finite() && params.capacity_ah > 0.0) {
            return Err(SocError::InvalidCapacity);
        }
        if !(params.r0.is_finite() && params.r0 >= 0.0 && params.r1.is_finite() && params.r1 > 0.0)
        {
            return Err(SocError::InvalidResistance);
        }
        if !(params.c1.is_finite() && params.c1 > 0.0) {
            return Err(SocError::InvalidCapacitance);
        }
        Ok(Self { params, ocv })
    }

    /// Scalar parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};
    /// # fn main() -> Result<(), SocError> {
    /// let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)])?;
    /// let params = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
    /// let model = Model::new(params, ocv)?;
    /// let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
    /// assert_eq!(model.params().capacity_ah, 2.0);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn params(&self) -> &CellParams {
        &self.params
    }

    /// OCV curve.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};
    /// # fn main() -> Result<(), SocError> {
    /// let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)])?;
    /// let params = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
    /// let model = Model::new(params, ocv)?;
    /// let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
    /// assert_eq!(model.ocv().voltage(1.0), 4.2);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn ocv(&self) -> &OcvCurve<N> {
        &self.ocv
    }

    /// Terminal voltage `OCV(soc) - v1 - R0*I` for discharge current `i`.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};
    /// # fn main() -> Result<(), SocError> {
    /// let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)])?;
    /// let params = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
    /// let model = Model::new(params, ocv)?;
    /// let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
    /// // OCV(0.5) - v1 - R0 * i = 3.7 - 0.0 - 0.05 * 2.0
    /// let v = model.terminal_voltage(0.5, 0.0, 2.0);
    /// assert!((v - 3.6).abs() < 1e-12);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn terminal_voltage(&self, soc: f64, v1: f64, i: f64) -> f64 {
        self.ocv.voltage(soc) - v1 - self.params.r0 * i
    }

    /// Advance the true model one step (no noise); returns `(soc, v1)`.
    ///
    /// Useful for simulation and tests. `soc` is clamped to `[0,1]`.
    ///
    /// # Errors
    /// [`SocError::InvalidTimeStep`] or [`SocError::NonFinite`].
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};
    /// # fn main() -> Result<(), SocError> {
    /// let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)])?;
    /// let params = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
    /// let model = Model::new(params, ocv)?;
    /// let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
    /// let (soc, v1) = model.step(0.8, 0.0, 2.0, 360.0)?; // 2 A for 6 minutes
    /// assert!(soc < 0.8);
    /// assert!(v1 > 0.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn step(&self, soc: f64, v1: f64, i: f64, dt: f64) -> Result<(f64, f64), SocError> {
        if !(dt.is_finite() && dt > 0.0) {
            return Err(SocError::InvalidTimeStep);
        }
        if !i.is_finite() {
            return Err(SocError::NonFinite("current"));
        }
        let p = &self.params;
        let a = libm::exp(-dt / (p.r1 * p.c1));
        let soc_n = (soc - dt * i / (3600.0 * p.capacity_ah)).clamp(0.0, 1.0);
        let v1_n = v1 * a + p.r1 * (1.0 - a) * i;
        Ok((soc_n, v1_n))
    }
}
