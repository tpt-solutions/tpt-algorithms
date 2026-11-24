// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native (pyo3) layer of the `tpt_algo` Python package.
//!
//! Thin wrappers over `tpt-algo-bindings-core`, which holds the logic. The
//! public, typed API lives in `python/tpt_algo/` (`__init__.py` and `.pyi`);
//! this module is imported as `tpt_algo._native`.
//!
//! Errors raise `tpt_algo.TptAlgoError`, a subclass of `ValueError` with a
//! `kind` attribute: `"invalid_input"`, `"invalid_config"`, `"unsupported"`,
//! `"infeasible"` or `"unreachable"`.

use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use tpt_algo_bindings_core as core_;

create_exception!(
    _native,
    TptAlgoError,
    PyValueError,
    "Error from tpt_algo. The `kind` attribute gives the category."
);

/// Converts a core error into `TptAlgoError` with its `kind` set.
fn py_err(py: Python<'_>, e: core_::BindingError) -> PyErr {
    let err = TptAlgoError::new_err(e.message.clone());
    // Setting an attribute on a freshly built exception cannot fail in practice;
    // if it did, the error would still be raised with its message.
    let _ = err.value(py).setattr("kind", e.kind.as_str());
    err
}

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
        py: Python<'_>,
        data: Vec<Vec<f64>>,
        n_trees: usize,
        subsample_size: usize,
        seed: u64,
    ) -> PyResult<Self> {
        core_::Forest::fit(&data, n_trees, subsample_size, seed)
            .map(|inner| Self { inner })
            .map_err(|e| py_err(py, e))
    }

    /// Number of features per row.
    #[getter]
    fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    /// Number of trees.
    #[getter]
    fn n_trees(&self) -> usize {
        self.inner.n_trees()
    }

    /// Anomaly score of one row.
    fn score(&self, py: Python<'_>, row: Vec<f64>) -> PyResult<f64> {
        self.inner.score(&row).map_err(|e| py_err(py, e))
    }

    /// Anomaly scores of many rows, in order.
    fn score_samples(&self, py: Python<'_>, data: Vec<Vec<f64>>) -> PyResult<Vec<f64>> {
        self.inner.score_samples(&data).map_err(|e| py_err(py, e))
    }

    /// Mean path length of one row over the trees. Short paths mean anomalous.
    fn mean_path_length(&self, py: Python<'_>, row: Vec<f64>) -> PyResult<f64> {
        self.inner.mean_path_length(&row).map_err(|e| py_err(py, e))
    }
}

/// Average path length `c(n)` of an unsuccessful search in a binary tree.
#[pyfunction]
fn average_path_length(n: usize) -> f64 {
    core_::average_path_length(n)
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
        py: Python<'_>,
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
            .map_err(|e| py_err(py, e))
    }

    /// Time update over `dt` seconds at discharge current `current` (amps).
    fn predict(&mut self, py: Python<'_>, current: f64, dt: f64) -> PyResult<()> {
        self.inner.predict(current, dt).map_err(|e| py_err(py, e))
    }

    /// Measurement update with terminal voltage `voltage` under `current`.
    fn update(&mut self, py: Python<'_>, voltage: f64, current: f64) -> PyResult<()> {
        self.inner
            .update(voltage, current)
            .map_err(|e| py_err(py, e))
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

    /// The 2x2 error covariance of `(soc, v1)` as a nested list.
    fn covariance(&self) -> Vec<Vec<f64>> {
        let p = self.inner.covariance();
        vec![p[0].to_vec(), p[1].to_vec()]
    }

    /// Cell parameters as a dict with `capacity_ah`, `r0`, `r1` and `c1`.
    fn model<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let p = self.inner.cell_params();
        let d = PyDict::new(py);
        d.set_item("capacity_ah", p.capacity_ah)?;
        d.set_item("r0", p.r0)?;
        d.set_item("r1", p.r1)?;
        d.set_item("c1", p.c1)?;
        Ok(d)
    }

    /// Open-circuit voltage at `soc` from the OCV curve.
    fn ocv_voltage(&self, soc: f64) -> f64 {
        self.inner.ocv_voltage(soc)
    }

    /// Terminal voltage predicted for state `(soc, v1)` at current `current`.
    fn terminal_voltage(&self, soc: f64, v1: f64, current: f64) -> f64 {
        self.inner.terminal_voltage(soc, v1, current)
    }
}

/// Stateful limit order book. Events and results are JSON strings; the
/// `tpt_algo.OrderBook` wrapper in Python converts to and from dicts.
#[pyclass(name = "OrderBookCore", module = "tpt_algo._native")]
struct PyOrderBookCore {
    inner: core_::Book,
}

#[pymethods]
impl PyOrderBookCore {
    #[new]
    fn new() -> Self {
        Self {
            inner: core_::Book::new(),
        }
    }

    /// Applies a JSON array of events atomically; returns JSON.
    fn apply_json(&mut self, py: Python<'_>, events: &str) -> PyResult<String> {
        self.inner.apply_json(events).map_err(|e| py_err(py, e))
    }

    /// Top of book and counters as JSON.
    fn snapshot_json(&self) -> String {
        self.inner.snapshot_json()
    }
}

/// Solves a routing problem given as JSON; returns the solution as JSON.
#[pyfunction]
fn solve_routes_json(py: Python<'_>, input: &str) -> PyResult<String> {
    core_::routing::solve_json(input).map_err(|e| py_err(py, e))
}

/// Collision-avoidance velocity for one agent, JSON in and out.
#[pyfunction]
fn avoid_json(py: Python<'_>, input: &str) -> PyResult<String> {
    core_::swarm::avoid_json(input).map_err(|e| py_err(py, e))
}

/// Shortest grid path, JSON in and out.
#[pyfunction]
fn find_path_json(py: Python<'_>, input: &str) -> PyResult<String> {
    core_::swarm::find_path_json(input).map_err(|e| py_err(py, e))
}

/// Runs the seeded consensus simulator, JSON in and out.
#[pyfunction]
fn simulate_consensus_json(py: Python<'_>, input: &str) -> PyResult<String> {
    core_::consensus::simulate_json(input).map_err(|e| py_err(py, e))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIsolationForest>()?;
    m.add_class::<PyEkf>()?;
    m.add_class::<PyOrderBookCore>()?;
    m.add_function(wrap_pyfunction!(average_path_length, m)?)?;
    m.add_function(wrap_pyfunction!(solve_routes_json, m)?)?;
    m.add_function(wrap_pyfunction!(avoid_json, m)?)?;
    m.add_function(wrap_pyfunction!(find_path_json, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_consensus_json, m)?)?;
    m.add("TptAlgoError", m.py().get_type::<TptAlgoError>())?;
    Ok(())
}
