// SPDX-License-Identifier: MIT OR Apache-2.0
//! WebAssembly bindings for `tpt-algo-fraud` and `tpt-algo-soc`.
//!
//! Thin wrappers over `tpt-algo-bindings-core`. Data crosses the boundary as
//! flat `Float64Array`s: rows are concatenated, with `dim` values per row.
//! Errors become JavaScript `Error` objects.

use tpt_algo_bindings_core as core_;
use wasm_bindgen::prelude::*;

fn js_error(message: String) -> JsError {
    JsError::new(&message)
}

fn split_rows(flat: &[f64], dim: usize) -> Result<Vec<Vec<f64>>, JsError> {
    if dim == 0 || !flat.len().is_multiple_of(dim) {
        return Err(JsError::new(
            "data length must be a positive multiple of the row width",
        ));
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
    pub fn fit(
        data: &[f64],
        dim: usize,
        n_trees: usize,
        subsample_size: usize,
        seed: u64,
    ) -> Result<IsolationForest, JsError> {
        let rows = split_rows(data, dim)?;
        core_::Forest::fit(&rows, n_trees, subsample_size, seed)
            .map(|inner| IsolationForest { inner })
            .map_err(js_error)
    }

    /// Number of features per row.
    #[wasm_bindgen(getter)]
    pub fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    /// Anomaly score of one row.
    pub fn score(&self, row: &[f64]) -> Result<f64, JsError> {
        self.inner.score(row).map_err(js_error)
    }

    /// Anomaly scores of concatenated rows, in order.
    pub fn score_samples(&self, data: &[f64]) -> Result<Vec<f64>, JsError> {
        let rows = split_rows(data, self.inner.dimension())?;
        self.inner.score_samples(&rows).map_err(js_error)
    }
}

/// Extended Kalman filter for battery state of charge.
#[wasm_bindgen]
pub struct Ekf {
    inner: core_::Ekf,
}

#[wasm_bindgen]
impl Ekf {
    /// `ocv` is `[soc0, volts0, soc1, volts1, ...]` with 2 to 8 pairs.
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ocv: &[f64],
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
    ) -> Result<Ekf, JsError> {
        if !ocv.len().is_multiple_of(2) {
            return Err(JsError::new("ocv must hold (soc, volts) pairs"));
        }
        let table: Vec<(f64, f64)> = ocv.chunks(2).map(|p| (p[0], p[1])).collect();
        let cell = core_::CellSpec {
            capacity_ah,
            r0,
            r1,
            c1,
            process_soc,
            process_v1,
            measurement,
        };
        core_::Ekf::new(&table, cell, soc0, soc_variance, v1_variance)
            .map(|inner| Ekf { inner })
            .map_err(js_error)
    }

    /// Time update over `dt` seconds at discharge current `current` (amps).
    pub fn predict(&mut self, current: f64, dt: f64) -> Result<(), JsError> {
        self.inner.predict(current, dt).map_err(js_error)
    }

    /// Measurement update with terminal voltage `voltage` under `current`.
    pub fn update(&mut self, voltage: f64, current: f64) -> Result<(), JsError> {
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
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn forest_scores_outlier_higher() {
        let mut data: Vec<f64> = Vec::new();
        for i in 0..200 {
            data.push(f64::from(i % 10) * 0.1);
            data.push(f64::from(i % 7) * 0.1);
        }
        data.extend_from_slice(&[50.0, 50.0]);
        let forest = IsolationForest::fit(&data, 2, 100, 256, 7).unwrap();
        let scores = forest.score_samples(&data).unwrap();
        assert!(scores[200] > scores[0]);
    }

    #[wasm_bindgen_test]
    fn ekf_discharge_lowers_soc() {
        let ocv = [0.0, 3.0, 0.5, 3.7, 1.0, 4.2];
        let mut ekf = Ekf::new(
            &ocv, 2.0, 0.05, 0.02, 1000.0, 0.8, 0.05, 1e-4, 1e-9, 1e-6, 1e-4,
        )
        .unwrap();
        let before = ekf.soc();
        ekf.predict(2.0, 600.0).unwrap();
        assert!(ekf.soc() < before);
    }
}
