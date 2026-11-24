// SPDX-License-Identifier: MIT OR Apache-2.0
//! Runs the shared parity fixtures through the WebAssembly bindings, in Node
//! (`wasm-pack test --node`). The same file is run by the Rust core tests and
//! the Python suite; numbers must agree within 1e-9.

#![cfg(target_arch = "wasm32")]

use js_sys::{Reflect, JSON};
use serde_json::{json, Value};
use tpt_algo_wasm::{
    average_path_length, avoid, find_path, simulate_consensus, solve_routes, Ekf, IsolationForest,
    OrderBook,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

const FIXTURE: &str = include_str!("../../../internal/tpt-algo-bindings-core/fixtures/parity.json");
const TOLERANCE: f64 = 1e-9;

fn approx_eq(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            (a - b).abs() <= TOLERANCE * b.abs().max(1.0)
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| approx_eq(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len() && b.iter().all(|(k, y)| a.get(k).is_some_and(|x| approx_eq(x, y)))
        }
        _ => actual == expected,
    }
}

/// Converts a JSON value to a JavaScript value through `JSON.parse`.
fn js(v: &Value) -> JsValue {
    JSON::parse(&v.to_string()).unwrap()
}

fn json_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn kind_of(err: JsValue) -> String {
    Reflect::get(&err, &JsValue::from_str("kind"))
        .unwrap()
        .as_string()
        .unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn flat(rows: &Value) -> Vec<f64> {
    rows.as_array()
        .unwrap()
        .iter()
        .flat_map(|r| r.as_array().unwrap().iter().map(f))
        .collect()
}

fn width(rows: &Value) -> usize {
    rows[0].as_array().unwrap().len()
}

/// Keeps only the listed keys of an object, so absent options take defaults.
fn pick(src: &Value, keys: &[&str]) -> Value {
    let mut out = serde_json::Map::new();
    for k in keys {
        if let Some(v) = src.get(*k) {
            out.insert((*k).to_string(), v.clone());
        }
    }
    Value::Object(out)
}

fn run_forest(input: &Value) -> Value {
    let opts = pick(input, &["n_trees", "subsample_size", "seed"]);
    let forest =
        IsolationForest::fit(&flat(&input["data"]), width(&input["data"]), js(&opts)).unwrap();
    let queries = input["queries"].as_array().unwrap();
    json!({
        "dimension": forest.dimension(),
        "n_trees": forest.n_trees(),
        "scores": queries.iter().map(|q| forest.score(&flat(&json!([q]))).unwrap()).collect::<Vec<_>>(),
        "mean_path_length": queries
            .iter()
            .map(|q| forest.mean_path_length(&flat(&json!([q]))).unwrap())
            .collect::<Vec<_>>(),
    })
}

fn run_average_path_length(input: &Value) -> Value {
    let values: Vec<f64> = input["n"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| average_path_length(n.as_u64().unwrap() as usize))
        .collect();
    json!({ "values": values })
}

fn run_ekf(input: &Value) -> Value {
    let cell = &input["cell"];
    let mut opts = pick(cell, &["capacity_ah", "r0", "r1", "c1", "process_soc", "process_v1", "measurement"]);
    let extra = pick(input, &["soc0", "soc_variance", "v1_variance"]);
    if let (Value::Object(o), Value::Object(e)) = (&mut opts, extra) {
        o.extend(e);
    }
    let mut ekf = Ekf::new(&flat(&input["ocv"]), js(&opts)).unwrap();
    let mut states = Vec::new();
    for step in input["steps"].as_array().unwrap() {
        if step["op"] == "predict" {
            ekf.predict(f(&step["current"]), f(&step["dt"])).unwrap();
        } else {
            ekf.update(f(&step["voltage"]), f(&step["current"])).unwrap();
        }
        let e = ekf.estimate();
        states.push(json!({ "soc": e[0], "v1": e[1], "soc_variance": e[2] }));
    }
    let c = ekf.covariance();
    let h = &input["helpers"];
    let t = &h["terminal"];
    let model = ekf.model().unwrap();
    let model_field = |k: &str| Reflect::get(&model, &JsValue::from_str(k)).unwrap().as_f64().unwrap();
    json!({
        "states": states,
        "covariance": [[c[0], c[1]], [c[2], c[3]]],
        "ocv_voltage": ekf.ocv_voltage(f(&h["ocv_voltage_soc"])),
        "terminal_voltage": ekf.terminal_voltage(f(&t[0]), f(&t[1]), f(&t[2])),
        "cell": {
            "capacity_ah": model_field("capacity_ah"),
            "r0": model_field("r0"),
            "r1": model_field("r1"),
            "c1": model_field("c1"),
        },
    })
}

fn run_error(input: &Value) -> Value {
    let kind = match input["op"].as_str().unwrap() {
        "ekf_table" => {
            let n = input["points"].as_u64().unwrap() as usize;
            let table: Vec<f64> = (0..n).flat_map(|i| [i as f64 / 8.0, 3.0]).collect();
            let opts = json!({"capacity_ah": 2.0, "r0": 0.05, "r1": 0.02, "c1": 1000.0,
                              "soc0": 0.5, "soc_variance": 0.05});
            kind_of(Ekf::new(&table, js(&opts)).err().unwrap())
        }
        "ekf_capacity" => {
            let opts = json!({"capacity_ah": input["capacity_ah"], "r0": 0.05, "r1": 0.02,
                              "c1": 1000.0, "soc0": 0.5, "soc_variance": 0.05});
            kind_of(Ekf::new(&[0.0, 3.0, 1.0, 4.2], js(&opts)).err().unwrap())
        }
        "forest_trees" => kind_of(
            IsolationForest::fit(&[1.0, 2.0], 1, js(&json!({"n_trees": 0, "subsample_size": 2, "seed": 0})))
                .err()
                .unwrap(),
        ),
        "forest_empty" => kind_of(
            IsolationForest::fit(&[], 2, js(&json!({"n_trees": 10, "subsample_size": 2, "seed": 0})))
                .err()
                .unwrap(),
        ),
        "forest_dim" => {
            let forest = IsolationForest::fit(
                &[1.0, 2.0, 2.0, 3.0],
                2,
                js(&json!({"n_trees": 10, "subsample_size": 2})),
            )
            .unwrap();
            kind_of(forest.score(&[1.0]).err().unwrap())
        }
        "routing_infeasible" => kind_of(
            solve_routes(r#"{"depot":[0,0],"capacity":10,"customers":[{"x":1,"y":0,"demand":20}]}"#)
                .err()
                .unwrap(),
        ),
        "swarm_unreachable" => kind_of(
            find_path(r#"{"width":3,"height":1,"blocked":[[1,0]],"start":[0,0],"goal":[2,0],"connectivity":4}"#)
                .err()
                .unwrap(),
        ),
        "orderbook_unknown" => kind_of(
            OrderBook::new()
                .apply(r#"[{"type":"cancel","id":42}]"#)
                .err()
                .unwrap(),
        ),
        other => panic!("unknown error op {other}"),
    };
    json!({ "kind": kind })
}

fn run_case(case: &Value) -> Value {
    let input = &case["input"];
    match case["kind"].as_str().unwrap() {
        "forest" => run_forest(input),
        "average_path_length" => run_average_path_length(input),
        "ekf" => run_ekf(input),
        "routing" => json_of(&solve_routes(&input.to_string()).unwrap()),
        "orderbook" => json_of(&OrderBook::new().apply(&input["events"].to_string()).unwrap()),
        "swarm_avoid" => json_of(&avoid(&input.to_string()).unwrap()),
        "swarm_find_path" => json_of(&find_path(&input.to_string()).unwrap()),
        "consensus" => json_of(&simulate_consensus(&input.to_string()).unwrap()),
        "error" => run_error(input),
        other => panic!("unknown kind {other}"),
    }
}

#[wasm_bindgen_test]
fn shared_fixtures_match_expected() {
    let doc = json_of(FIXTURE);
    let mut failures = Vec::new();
    for case in doc["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let actual = run_case(case);
        if !approx_eq(&actual, &case["expected"]) {
            failures.push(format!("{name}: got {actual}, expected {}", case["expected"]));
        }
    }
    assert!(failures.is_empty(), "parity mismatches:\n{}", failures.join("\n"));
}
