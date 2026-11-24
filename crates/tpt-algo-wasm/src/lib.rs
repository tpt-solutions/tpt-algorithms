// SPDX-License-Identifier: MIT OR Apache-2.0
//! WebAssembly bindings for the `tpt-algo` crates.
//!
//! Thin wrappers over `tpt-algo-bindings-core`, which holds the logic.
//!
//! - Numeric arrays cross as `Float64Array`. Rows are concatenated, with `dim`
//!   values per row.
//! - Options are plain objects with snake_case keys that match the Python
//!   keyword arguments, for example `{ n_trees: 50, seed: 7 }`. Missing keys take
//!   the same defaults as Python.
//! - Structured inputs and outputs (routing, order book, swarm, consensus) are
//!   JSON strings: pass `JSON.stringify(input)` and parse the result.
//! - Errors are thrown as `Error` objects with a `kind` property: `"invalid_input"`,
//!   `"invalid_config"`, `"unsupported"`, `"infeasible"` or `"unreachable"`.
//! - Seeds and counts must be within the safe integer range of JavaScript
//!   numbers (2^53 - 1).

use js_sys::{Object, Reflect};
use tpt_algo_bindings_core as core_;
use core_::{BindingError, ErrorKind};
use wasm_bindgen::prelude::*;

/// Converts a core error into a thrown `Error` with a `kind` property.
fn js_error(e: BindingError) -> JsValue {
    let err = js_sys::Error::new(&e.message);
    // Setting a plain property on a fresh Error object does not fail.
    let _ = Reflect::set(&err, &JsValue::from_str("kind"), &JsValue::from_str(e.kind.as_str()));
    err.into()
}

fn invalid_option(key: &str, what: &str) -> JsValue {
    js_error(BindingError::new(
        ErrorKind::InvalidConfig,
        format!("option '{key}' must be {what}"),
    ))
}

/// Reads a property of an options object; `undefined` and `null` mean absent.
fn option_value(options: &JsValue, key: &str) -> Result<Option<f64>, JsValue> {
    if options.is_undefined() || options.is_null() {
        return Ok(None);
    }
    let v = Reflect::get(options, &JsValue::from_str(key)).map_err(|_| invalid_option(key, "readable"))?;
    if v.is_undefined() || v.is_null() {
        return Ok(None);
    }
    v.as_f64()
        .map(Some)
        .ok_or_else(|| invalid_option(key, "a number"))
}

/// Optional non-negative integer option with a default.
fn option_uint(options: &JsValue, key: &str, default: u64) -> Result<u64, JsValue> {
    match option_value(options, key)? {
        None => Ok(default),
        Some(v) if v >= 0.0 && v.fract() == 0.0 && v <= 9_007_199_254_740_991.0 => Ok(v as u64),
        Some(_) => Err(invalid_option(key, "a non-negative integer")),
    }
}

/// Optional number option with a default.
fn option_float(options: &JsValue, key: &str, default: f64) -> Result<f64, JsValue> {
    Ok(option_value(options, key)?.unwrap_or(default))
}

/// Required number option.
fn required_float(options: &JsValue, key: &str) -> Result<f64, JsValue> {
    option_value(options, key)?.ok_or_else(|| {
        js_error(BindingError::new(
            ErrorKind::InvalidConfig,
            format!("option '{key}' is required"),
        ))
    })
}

fn split_rows(flat: &[f64], dim: usize) -> Result<Vec<Vec<f64>>, JsValue> {
    if dim == 0 || !flat.len().is_multiple_of(dim) {
        return Err(js_error(BindingError::new(
            ErrorKind::InvalidInput,
            "data length must be a positive multiple of the row width",
        )));
    }
    Ok(flat.chunks(dim).map(<[f64]>::to_vec).collect())
}

/// Anomaly scoring with a seeded Isolation Forest. Scores lie in `[0, 1]`.
#[wasm_bindgen]
pub struct IsolationForest {
    inner: core_::Forest,
}

#[wasm_bindgen]
impl IsolationForest {
    /// Trains a forest on `data`, rows of width `dim` concatenated.
    ///
    /// `options` may hold `n_trees` (100), `subsample_size` (256) and `seed` (0).
    pub fn fit(data: &[f64], dim: usize, options: JsValue) -> Result<IsolationForest, JsValue> {
        let defaults = core_::forest_defaults();
        let n_trees = option_uint(&options, "n_trees", defaults.n_trees as u64)?;
        let subsample = option_uint(&options, "subsample_size", defaults.subsample_size as u64)?;
        let seed = option_uint(&options, "seed", defaults.seed)?;
        let rows = split_rows(data, dim)?;
        let n_trees = usize::try_from(n_trees).map_err(|_| invalid_option("n_trees", "a valid count"))?;
        let subsample =
            usize::try_from(subsample).map_err(|_| invalid_option("subsample_size", "a valid count"))?;
        core_::Forest::fit(&rows, n_trees, subsample, seed)
            .map(|inner| IsolationForest { inner })
            .map_err(js_error)
    }

    /// Number of features per row.
    #[wasm_bindgen(getter)]
    pub fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    /// Number of trees.
    #[wasm_bindgen(getter)]
    pub fn n_trees(&self) -> usize {
        self.inner.n_trees()
    }

    /// Anomaly score of one row.
    pub fn score(&self, row: &[f64]) -> Result<f64, JsValue> {
        self.inner.score(row).map_err(js_error)
    }

    /// Anomaly scores of concatenated rows, in order.
    pub fn score_samples(&self, data: &[f64]) -> Result<Vec<f64>, JsValue> {
        let rows = split_rows(data, self.inner.dimension())?;
        self.inner.score_samples(&rows).map_err(js_error)
    }

    /// Mean path length of one row over the trees. Short paths mean anomalous.
    pub fn mean_path_length(&self, row: &[f64]) -> Result<f64, JsValue> {
        self.inner.mean_path_length(row).map_err(js_error)
    }
}

/// Average path length `c(n)` of an unsuccessful search in a binary tree.
#[wasm_bindgen]
pub fn average_path_length(n: usize) -> f64 {
    core_::average_path_length(n)
}

/// Extended Kalman filter for battery state of charge.
#[wasm_bindgen]
pub struct Ekf {
    inner: core_::Ekf,
}

#[wasm_bindgen]
impl Ekf {
    /// `ocv` is `[soc0, volts0, soc1, volts1, ...]` with 2 to 8 pairs.
    ///
    /// `options` holds `capacity_ah`, `r0`, `r1`, `c1`, `soc0` and `soc_variance`
    /// (required), and `v1_variance` (1e-4), `process_soc` (1e-9),
    /// `process_v1` (1e-6) and `measurement` (1e-4) with defaults.
    #[wasm_bindgen(constructor)]
    pub fn new(ocv: &[f64], options: JsValue) -> Result<Ekf, JsValue> {
        if !ocv.len().is_multiple_of(2) {
            return Err(js_error(BindingError::new(
                ErrorKind::InvalidInput,
                "ocv must hold (soc, volts) pairs",
            )));
        }
        let d = core_::soc::EKF_DEFAULTS;
        let table: Vec<(f64, f64)> = ocv.chunks(2).map(|p| (p[0], p[1])).collect();
        let cell = core_::CellSpec {
            capacity_ah: required_float(&options, "capacity_ah")?,
            r0: required_float(&options, "r0")?,
            r1: required_float(&options, "r1")?,
            c1: required_float(&options, "c1")?,
            process_soc: option_float(&options, "process_soc", d.process_soc)?,
            process_v1: option_float(&options, "process_v1", d.process_v1)?,
            measurement: option_float(&options, "measurement", d.measurement)?,
        };
        let soc0 = required_float(&options, "soc0")?;
        let soc_variance = required_float(&options, "soc_variance")?;
        let v1_variance = option_float(&options, "v1_variance", d.v1_variance)?;
        core_::Ekf::new(&table, cell, soc0, soc_variance, v1_variance)
            .map(|inner| Ekf { inner })
            .map_err(js_error)
    }

    /// Time update over `dt` seconds at discharge current `current` (amps).
    pub fn predict(&mut self, current: f64, dt: f64) -> Result<(), JsValue> {
        self.inner.predict(current, dt).map_err(js_error)
    }

    /// Measurement update with terminal voltage `voltage` under `current`.
    pub fn update(&mut self, voltage: f64, current: f64) -> Result<(), JsValue> {
        self.inner.update(voltage, current).map_err(js_error)
    }

    /// State of charge in `[0, 1]`.
    #[wasm_bindgen(getter)]
    pub fn soc(&self) -> f64 {
        self.inner.estimate().soc
    }

    /// `[soc, v1, soc_variance]`.
    pub fn estimate(&self) -> Vec<f64> {
        let e = self.inner.estimate();
        vec![e.soc, e.v1, e.soc_variance]
    }

    /// The 2x2 error covariance of `(soc, v1)`, row-major: `[p00, p01, p10, p11]`.
    pub fn covariance(&self) -> Vec<f64> {
        let p = self.inner.covariance();
        vec![p[0][0], p[0][1], p[1][0], p[1][1]]
    }

    /// Cell parameters as an object with `capacity_ah`, `r0`, `r1` and `c1`.
    pub fn model(&self) -> Result<Object, JsValue> {
        let p = self.inner.cell_params();
        let obj = Object::new();
        for (key, value) in [
            ("capacity_ah", p.capacity_ah),
            ("r0", p.r0),
            ("r1", p.r1),
            ("c1", p.c1),
        ] {
            Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_f64(value))?;
        }
        Ok(obj)
    }

    /// Open-circuit voltage at `soc` from the OCV curve.
    pub fn ocv_voltage(&self, soc: f64) -> f64 {
        self.inner.ocv_voltage(soc)
    }

    /// Terminal voltage predicted for state `(soc, v1)` at current `current`.
    pub fn terminal_voltage(&self, soc: f64, v1: f64, current: f64) -> f64 {
        self.inner.terminal_voltage(soc, v1, current)
    }
}

/// Stateful limit order book. Events and results are JSON strings.
#[wasm_bindgen]
pub struct OrderBook {
    inner: core_::Book,
}

#[wasm_bindgen]
impl OrderBook {
    /// An empty book.
    #[wasm_bindgen(constructor)]
    pub fn new() -> OrderBook {
        OrderBook {
            inner: core_::Book::new(),
        }
    }

    /// Applies a JSON array of events atomically. Returns
    /// `{"trades": [...], "stats": {...}}` as JSON. `stats.traded` is quantity.
    pub fn apply(&mut self, events: &str) -> Result<String, JsValue> {
        self.inner.apply_json(events).map_err(js_error)
    }

    /// Top of book, depth and sequence number, as JSON.
    pub fn snapshot(&self) -> String {
        self.inner.snapshot_json()
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

/// Solves a routing problem given as JSON; returns the solution as JSON.
#[wasm_bindgen]
pub fn solve_routes(input: &str) -> Result<String, JsValue> {
    core_::routing::solve_json(input).map_err(js_error)
}

/// Collision-avoidance velocity for one agent, JSON in and out.
#[wasm_bindgen]
pub fn avoid(input: &str) -> Result<String, JsValue> {
    core_::swarm::avoid_json(input).map_err(js_error)
}

/// Shortest grid path, JSON in and out.
#[wasm_bindgen]
pub fn find_path(input: &str) -> Result<String, JsValue> {
    core_::swarm::find_path_json(input).map_err(js_error)
}

/// Runs the seeded consensus simulator, JSON in and out.
#[wasm_bindgen]
pub fn simulate_consensus(input: &str) -> Result<String, JsValue> {
    core_::consensus::simulate_json(input).map_err(js_error)
}
