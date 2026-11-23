// SPDX-License-Identifier: MIT OR Apache-2.0
//! Extended Kalman Filter with a type-state lifecycle.

use crate::{Model, SocError};
use core::marker::PhantomData;

/// Marker: filter has been configured but has no state yet.
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
/// let filter = Filter::new(model, noise)?; // Filter<Uninitialized, 3>
/// // filter.predict(...) does not exist until the filter is initialized.
/// let filter = filter.initialize(0.8, 0.05, 1e-4)?; // Filter<Initialized, 3>
/// assert_eq!(filter.estimate().soc, 0.8);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Uninitialized;

/// Marker: filter holds a validated state and covariance.
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
/// let filter = Filter::new(model, noise)?.initialize(0.8, 0.05, 1e-4)?;
/// assert!(filter.covariance()[0][0] > 0.0);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Initialized;

/// Process and measurement noise variances.
///
/// # Example
///
/// ```
/// use tpt_algo_soc::NoiseConfig;
///
/// let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
/// assert!(noise.measurement > 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoiseConfig {
    /// Per-step process variance on SoC (>= 0).
    pub process_soc: f64,
    /// Per-step process variance on `V1` (>= 0).
    pub process_v1: f64,
    /// Terminal-voltage measurement variance in V^2 (> 0).
    pub measurement: f64,
}

/// Snapshot of the filter's estimate.
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
/// let filter = Filter::new(model, noise)?.initialize(0.8, 0.05, 1e-4)?;
/// let est = filter.estimate();
/// assert_eq!(est.soc, 0.8);
/// assert_eq!(est.soc_variance, 0.05);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Estimate {
    /// State of charge in `[0, 1]`.
    pub soc: f64,
    /// RC-pair voltage in volts.
    pub v1: f64,
    /// Variance of the SoC estimate.
    pub soc_variance: f64,
}

/// Extended Kalman Filter over `[SoC, V1]`; `S` is the lifecycle state.
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
/// let mut filter = Filter::new(model, noise)?.initialize(0.8, 0.05, 1e-4)?;
/// filter.predict(2.0, 60.0)?; // 2 A discharge for one minute
/// filter.update(3.9, 2.0)?; // measured terminal voltage under the same current
/// assert!((0.0..=1.0).contains(&filter.estimate().soc));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Filter<S, const N: usize> {
    model: Model<N>,
    noise: NoiseConfig,
    x: [f64; 2],
    p: [[f64; 2]; 2],
    _state: PhantomData<S>,
}

fn nonneg(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}

impl<const N: usize> Filter<Uninitialized, N> {
    /// Create an uninitialised filter.
    ///
    /// # Errors
    /// [`SocError::InvalidNoise`] for negative/non-finite variances or a
    /// zero measurement variance.
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
    /// let filter = Filter::new(model, noise)?;
    /// let filter = filter.initialize(0.8, 0.05, 1e-4)?;
    /// assert_eq!(filter.estimate().soc, 0.8);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(model: Model<N>, noise: NoiseConfig) -> Result<Self, SocError> {
        if !(nonneg(noise.process_soc)
            && nonneg(noise.process_v1)
            && nonneg(noise.measurement)
            && noise.measurement > 0.0)
        {
            return Err(SocError::InvalidNoise);
        }
        Ok(Self {
            model,
            noise,
            x: [0.0; 2],
            p: [[0.0; 2]; 2],
            _state: PhantomData,
        })
    }

    /// Set the initial SoC (with `V1 = 0`) and diagonal covariance, producing
    /// a usable filter.
    ///
    /// # Errors
    /// [`SocError::InvalidInitialState`] if `soc` is outside `[0,1]` or a
    /// variance is negative or non-finite.
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
    /// let filter = Filter::new(model, noise)?.initialize(0.5, 0.01, 1e-4)?;
    /// assert_eq!(filter.estimate().v1, 0.0); // RC voltage starts at zero
    /// # Ok(())
    /// # }
    /// ```
    pub fn initialize(
        self,
        soc: f64,
        soc_variance: f64,
        v1_variance: f64,
    ) -> Result<Filter<Initialized, N>, SocError> {
        if !((0.0..=1.0).contains(&soc) && nonneg(soc_variance) && nonneg(v1_variance)) {
            return Err(SocError::InvalidInitialState);
        }
        Ok(Filter {
            model: self.model,
            noise: self.noise,
            x: [soc, 0.0],
            p: [[soc_variance, 0.0], [0.0, v1_variance]],
            _state: PhantomData,
        })
    }
}

impl<const N: usize> Filter<Initialized, N> {
    /// Time update: propagate state and covariance over `dt` seconds with
    /// discharge current `current` (amps).
    ///
    /// # Errors
    /// [`SocError::InvalidTimeStep`] or [`SocError::NonFinite`]; the filter is
    /// unchanged on error.
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
    /// let mut filter = Filter::new(model, noise)?.initialize(0.8, 0.05, 1e-4)?;
    /// let before = filter.estimate().soc;
    /// filter.predict(2.0, 600.0)?; // 2 A discharge for ten minutes
    /// assert!(filter.estimate().soc < before);
    /// # Ok(())
    /// # }
    /// ```
    pub fn predict(&mut self, current: f64, dt: f64) -> Result<(), SocError> {
        // Invariants maintained by `initialize` and `update`.
        debug_assert!(
            (0.0..=1.0).contains(&self.x[0]),
            "precondition failed: SoC in [0, 1]"
        );
        debug_assert!(
            self.p[0][0] >= 0.0 && self.p[1][1] >= 0.0,
            "precondition failed: covariance diagonal non-negative"
        );
        let (soc, v1) = self.model.step(self.x[0], self.x[1], current, dt)?;
        let prm = self.model.params;
        let a = libm::exp(-dt / (prm.r1 * prm.c1));
        // F = diag(1, a); P' = F P F^T + Q
        let p = self.p;
        let p00 = p[0][0] + self.noise.process_soc;
        let p01 = a * p[0][1];
        let p11 = a * a * p[1][1] + self.noise.process_v1;
        debug_assert!(
            p00 >= 0.0 && p11 >= 0.0,
            "postcondition failed: predicted covariance diagonal non-negative"
        );
        self.p = [[p00, p01], [p01, p11]];
        self.x = [soc, v1];
        Ok(())
    }

    /// Measurement update with terminal voltage `voltage` (volts) observed
    /// under discharge current `current`. Uses the Joseph-form covariance
    /// update.
    ///
    /// # Errors
    /// [`SocError::NonFinite`]; the filter is unchanged on error.
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
    /// let mut filter = Filter::new(model, noise)?.initialize(0.5, 0.05, 1e-4)?;
    /// let before = filter.covariance()[0][0];
    /// filter.update(3.9, 0.0)?; // a measurement is informative, so variance shrinks
    /// assert!(filter.covariance()[0][0] < before);
    /// # Ok(())
    /// # }
    /// ```
    pub fn update(&mut self, voltage: f64, current: f64) -> Result<(), SocError> {
        if !voltage.is_finite() {
            return Err(SocError::NonFinite("voltage"));
        }
        if !current.is_finite() {
            return Err(SocError::NonFinite("current"));
        }
        let r = self.noise.measurement;
        let h = [self.model.ocv.slope(self.x[0]), -1.0];
        let y = voltage - self.model.terminal_voltage(self.x[0], self.x[1], current);
        let p = self.p;
        let ph = [
            p[0][0] * h[0] + p[0][1] * h[1],
            p[1][0] * h[0] + p[1][1] * h[1],
        ];
        let s = h[0] * ph[0] + h[1] * ph[1] + r;
        if !(s.is_finite() && s > 0.0) {
            return Err(SocError::NonFinite("innovation covariance"));
        }
        let k = [ph[0] / s, ph[1] / s];
        // A = I - K H;  P = A P A^T + K R K^T
        let a = [
            [1.0 - k[0] * h[0], -k[0] * h[1]],
            [-k[1] * h[0], 1.0 - k[1] * h[1]],
        ];
        let mut ap = [[0.0; 2]; 2];
        for (i, row) in ap.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = a[i][0] * p[0][j] + a[i][1] * p[1][j];
            }
        }
        let mut np = [[0.0; 2]; 2];
        for (i, row) in np.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = ap[i][0] * a[j][0] + ap[i][1] * a[j][1] + k[i] * r * k[j];
            }
        }
        let off = 0.5 * (np[0][1] + np[1][0]);
        debug_assert!(
            np[0][0] >= 0.0 && np[1][1] >= 0.0,
            "postcondition failed: updated covariance diagonal non-negative"
        );
        self.p = [[np[0][0], off], [off, np[1][1]]];
        self.x = [(self.x[0] + k[0] * y).clamp(0.0, 1.0), self.x[1] + k[1] * y];
        Ok(())
    }

    /// Current estimate.
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
    /// let filter = Filter::new(model, noise)?.initialize(0.8, 0.05, 1e-4)?;
    /// assert_eq!(filter.estimate().soc, 0.8);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn estimate(&self) -> Estimate {
        Estimate {
            soc: self.x[0],
            v1: self.x[1],
            soc_variance: self.p[0][0],
        }
    }

    /// Full 2x2 state covariance (symmetric), ordered `[SoC, V1]`.
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
    /// let filter = Filter::new(model, noise)?.initialize(0.8, 0.05, 1e-4)?;
    /// let p = filter.covariance();
    /// assert_eq!(p[0][1], p[1][0]); // symmetric
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn covariance(&self) -> [[f64; 2]; 2] {
        self.p
    }

    /// The underlying model.
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
    /// let filter = Filter::new(model, noise)?.initialize(0.8, 0.05, 1e-4)?;
    /// assert_eq!(filter.model().params().capacity_ah, 2.0);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn model(&self) -> &Model<N> {
        &self.model
    }
}
