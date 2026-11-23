// SPDX-License-Identifier: MIT OR Apache-2.0
//! Python bindings for `tpt-algo-fraud` and `tpt-algo-soc`.
//!
//! Thin wrappers over `tpt-algo-bindings-core`: the Rust logic lives there.
//! Errors raise `ValueError`.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tpt_algo_bindings_core as core_;

/// Anomaly scoring with a seeded Isolation Forest.
///
/// Scores lie in `[0, 1]`; values near 1 are anomalous.
#[pyclass(name = "IsolationForest", module = "tpt_algo")]
struct PyIsolationForest {
    inner: core_::Forest,
}

#[pymethods]
impl PyIsolationForest {
    /// Trains a forest on `data`, a list of equal-length rows of floats.
    #[staticmethod]
    #[pyo3(signature = (data, n_trees = 100, subsample_size = 256, seed = 0))]
    fn fit(
        data: Vec<Vec<f64>>,
        n_trees: usize,
        subsample_size: usize,
        seed: u64,
    ) -> PyResult<Self> {
        core_::Forest::fit(&data, n_trees, subsample_size, seed)
            .map(|inner| Self { inner })
            .map_err(PyValueError::new_err)
    }

    /// Number of features per row.
    #[getter]
    fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    /// Anomaly score of one row.
    fn score(&self, row: Vec<f64>) -> PyResult<f64> {
        self.inner.score(&row).map_err(PyValueError::new_err)
    }

    /// Anomaly scores of many rows, in order.
    fn score_samples(&self, data: Vec<Vec<f64>>) -> PyResult<Vec<f64>> {
        self.inner
            .score_samples(&data)
            .map_err(PyValueError::new_err)
    }
}

/// Extended Kalman filter for battery state of charge.
///
/// `ocv` is a list of `(soc, volts)` pairs with 2 to 8 entries and strictly
/// increasing `soc` in `[0, 1]`.
#[pyclass(name = "Ekf", module = "tpt_algo")]
struct PyEkf {
    inner: core_::Ekf,
}

#[pymethods]
impl PyEkf {
    #[new]
    #[pyo3(signature = (
        ocv,
        capacity_ah,
        r0,
        r1,
        c1,
        soc0,
        soc_variance,
        v1_variance = 1e-4,
        process_soc = 1e-9,
        process_v1 = 1e-6,
        measurement = 1e-4,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        ocv: Vec<(f64, f64)>,
        capacity_ah: f64,
        r0: f64,
        r1: f64,
        c1: f64,
        soc0: f64,
        soc_variance: f64,
        v1_variance: f64,
        process_soc: f64,
        process_v1: f64,
        measurement: f64,
    ) -> PyResult<Self> {
        let cell = core_::CellSpec {
            capacity_ah,
            r0,
            r1,
            c1,
            process_soc,
            process_v1,
            measurement,
        };
        core_::Ekf::new(&ocv, cell, soc0, soc_variance, v1_variance)
            .map(|inner| Self { inner })
            .map_err(PyValueError::new_err)
    }

    /// Time update over `dt` seconds at discharge current `current` (amps).
    fn predict(&mut self, current: f64, dt: f64) -> PyResult<()> {
        self.inner
            .predict(current, dt)
            .map_err(PyValueError::new_err)
    }

    /// Measurement update with terminal voltage `voltage` under `current`.
    fn update(&mut self, voltage: f64, current: f64) -> PyResult<()> {
        self.inner
            .update(voltage, current)
            .map_err(PyValueError::new_err)
    }

    /// State of charge in `[0, 1]`.
    #[getter]
    fn soc(&self) -> f64 {
        self.inner.estimate().soc
    }

    /// `(soc, v1, soc_variance)` as a tuple.
    fn estimate(&self) -> (f64, f64, f64) {
        let e = self.inner.estimate();
        (e.soc, e.v1, e.soc_variance)
    }
}

#[pymodule]
fn tpt_algo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIsolationForest>()?;
    m.add_class::<PyEkf>()?;
    Ok(())
}
