// SPDX-License-Identifier: MIT OR Apache-2.0
//! WebAssembly API tests: options defaults, structured error kinds, and the
//! JSON-based entry points. Run with `wasm-pack test --node`.

#![cfg(target_arch = "wasm32")]

use js_sys::{Reflect, JSON};
use tpt_algo_wasm::{simulate_consensus, Ekf, IsolationForest, OrderBook};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

fn js(text: &str) -> JsValue {
    JSON::parse(text).unwrap()
}

fn kind_of(err: JsValue) -> String {
    Reflect::get(&err, &JsValue::from_str("kind"))
        .unwrap()
        .as_string()
        .unwrap()
}

fn ekf_options(extra: &str) -> JsValue {
    let base = r#"{"capacity_ah":2.0,"r0":0.05,"r1":0.02,"c1":1000.0,"soc0":0.8,"soc_variance":0.05"#;
    if extra.is_empty() {
        js(&format!("{base}}}"))
    } else {
        js(&format!("{base},{extra}}}"))
    }
}

const OCV: [f64; 6] = [0.0, 3.0, 0.5, 3.7, 1.0, 4.2];

#[wasm_bindgen_test]
fn forest_defaults_match_python_defaults() {
    let data = [0.0, 0.0, 0.1, 0.1, 0.2, 0.0, 0.0, 0.2, 0.1, 0.3, 50.0, 50.0];
    let with_defaults = IsolationForest::fit(&data, 2, js("{}")).unwrap();
    let explicit = IsolationForest::fit(&data, 2, js(r#"{"n_trees":100,"subsample_size":256,"seed":0}"#)).unwrap();
    assert_eq!(with_defaults.n_trees(), 100);
    assert_eq!(with_defaults.score(&[1.0, 1.0]).unwrap(), explicit.score(&[1.0, 1.0]).unwrap());
}

#[wasm_bindgen_test]
fn forest_rejects_bad_option_with_config_kind() {
    let err = IsolationForest::fit(&[1.0, 2.0], 1, js(r#"{"n_trees":"many"}"#)).err().unwrap();
    assert_eq!(kind_of(err), "invalid_config");
}

#[wasm_bindgen_test]
fn forest_zero_trees_is_config_error() {
    let err = IsolationForest::fit(&[1.0, 2.0], 1, js(r#"{"n_trees":0}"#)).err().unwrap();
    assert_eq!(kind_of(err), "invalid_config");
}

#[wasm_bindgen_test]
fn forest_ragged_data_is_input_error() {
    let err = IsolationForest::fit(&[1.0, 2.0, 3.0], 2, js("{}")).err().unwrap();
    assert_eq!(kind_of(err), "invalid_input");
}

#[wasm_bindgen_test]
fn ekf_requires_cell_options() {
    let err = Ekf::new(&OCV, js(r#"{"r0":0.05,"r1":0.02,"c1":1000.0,"soc0":0.8,"soc_variance":0.05}"#))
        .err()
        .unwrap();
    assert_eq!(kind_of(err), "invalid_config");
}

#[wasm_bindgen_test]
fn ekf_defaults_and_helpers() {
    let mut ekf = Ekf::new(&OCV, ekf_options("")).unwrap();
    let before = ekf.soc();
    ekf.predict(2.0, 600.0).unwrap();
    assert!(ekf.soc() < before);
    assert!((ekf.ocv_voltage(0.5) - 3.7).abs() < 1e-12);
    assert_eq!(ekf.covariance()[1], ekf.covariance()[2]);
    let model = ekf.model().unwrap();
    let capacity = Reflect::get(&model, &JsValue::from_str("capacity_ah")).unwrap();
    assert_eq!(capacity.as_f64(), Some(2.0));
}

#[wasm_bindgen_test]
fn ekf_unsupported_table_length_kind() {
    let nine: Vec<f64> = (0..9).flat_map(|i| [i as f64 / 8.0, 3.0]).collect();
    let err = Ekf::new(&nine, ekf_options("")).err().unwrap();
    assert_eq!(kind_of(err), "unsupported");
}

#[wasm_bindgen_test]
fn ekf_odd_length_ocv_is_input_error() {
    let err = Ekf::new(&[0.0, 3.0, 1.0], ekf_options("")).err().unwrap();
    assert_eq!(kind_of(err), "invalid_input");
}

#[wasm_bindgen_test]
fn order_book_batch_and_atomicity() {
    let mut book = OrderBook::new();
    let out = book
        .apply(r#"[{"type":"add","id":1,"side":"bid","price":99,"qty":10}]"#)
        .unwrap();
    assert!(out.contains("\"trades\":[]"));
    let err = book
        .apply(r#"[{"type":"cancel","id":1},{"type":"cancel","id":42}]"#)
        .err()
        .unwrap();
    assert_eq!(kind_of(err), "invalid_input");
    assert!(book.snapshot().contains("\"best_bid\":99"));
}

#[wasm_bindgen_test]
fn consensus_is_deterministic() {
    let cfg = r#"{"n":4,"seed":9,"ticks":60}"#;
    assert_eq!(simulate_consensus(cfg).unwrap(), simulate_consensus(cfg).unwrap());
}
