// SPDX-License-Identifier: MIT OR Apache-2.0
//! Runs the shared parity fixtures (`fixtures/parity.json`) through the Rust
//! core. The Python and WebAssembly test suites run the same file through their
//! bindings and compare against the same expected values.
//!
//! Set `PARITY_PRINT=1` to print the actual output of every case; this is how
//! the expected values in the fixture were produced.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use core_::soc::{CellSpec, EKF_DEFAULTS};
use serde_json::{json, Value};
use tpt_algo_bindings_core as core_;

const FIXTURE: &str = include_str!("../fixtures/parity.json");
const TOLERANCE: f64 = 1e-9;

/// Returns `true` when `actual` matches `expected`: numbers within tolerance,
/// the same keys in objects, and the same length in arrays.
pub fn approx_eq(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            (a - b).abs() <= TOLERANCE * b.abs().max(1.0)
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| approx_eq(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && b.iter()
                    .all(|(k, y)| a.get(k).is_some_and(|x| approx_eq(x, y)))
        }
        _ => actual == expected,
    }
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn rows(v: &Value) -> Vec<Vec<f64>> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_array().unwrap().iter().map(f).collect())
        .collect()
}

fn pair_list(v: &Value) -> Vec<(f64, f64)> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| (f(&p[0]), f(&p[1])))
        .collect()
}

fn run_forest(input: &Value) -> Value {
    let data = rows(&input["data"]);
    let defaults = core_::forest_defaults();
    let n_trees = input["n_trees"]
        .as_u64()
        .map_or(defaults.n_trees, |v| v as usize);
    let subsample = input["subsample_size"]
        .as_u64()
        .map_or(defaults.subsample_size, |v| v as usize);
    let seed = input["seed"].as_u64().unwrap_or(defaults.seed);
    let forest = core_::Forest::fit(&data, n_trees, subsample, seed).unwrap();
    let queries = rows(&input["queries"]);
    json!({
        "dimension": forest.dimension(),
        "n_trees": forest.n_trees(),
        "scores": queries.iter().map(|q| forest.score(q).unwrap()).collect::<Vec<_>>(),
        "mean_path_length": queries
            .iter()
            .map(|q| forest.mean_path_length(q).unwrap())
            .collect::<Vec<_>>(),
    })
}

fn run_average_path_length(input: &Value) -> Value {
    let values: Vec<f64> = input["n"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| core_::average_path_length(n.as_u64().unwrap() as usize))
        .collect();
    json!({ "values": values })
}

fn run_ekf(input: &Value) -> Value {
    let cell_in = &input["cell"];
    let d = EKF_DEFAULTS;
    let cell = CellSpec {
        capacity_ah: f(&cell_in["capacity_ah"]),
        r0: f(&cell_in["r0"]),
        r1: f(&cell_in["r1"]),
        c1: f(&cell_in["c1"]),
        process_soc: cell_in["process_soc"].as_f64().unwrap_or(d.process_soc),
        process_v1: cell_in["process_v1"].as_f64().unwrap_or(d.process_v1),
        measurement: cell_in["measurement"].as_f64().unwrap_or(d.measurement),
    };
    let v1_variance = input["v1_variance"].as_f64().unwrap_or(d.v1_variance);
    let mut ekf = core_::Ekf::new(
        &pair_list(&input["ocv"]),
        cell,
        f(&input["soc0"]),
        f(&input["soc_variance"]),
        v1_variance,
    )
    .unwrap();
    let mut states = Vec::new();
    for step in input["steps"].as_array().unwrap() {
        match step["op"].as_str().unwrap() {
            "predict" => ekf.predict(f(&step["current"]), f(&step["dt"])).unwrap(),
            "update" => ekf
                .update(f(&step["voltage"]), f(&step["current"]))
                .unwrap(),
            other => panic!("unknown step {other}"),
        }
        let e = ekf.estimate();
        states.push(json!({ "soc": e.soc, "v1": e.v1, "soc_variance": e.soc_variance }));
    }
    let h = &input["helpers"];
    let t = &h["terminal"];
    let p = ekf.covariance();
    let params = ekf.cell_params();
    json!({
        "states": states,
        "covariance": [[p[0][0], p[0][1]], [p[1][0], p[1][1]]],
        "ocv_voltage": ekf.ocv_voltage(f(&h["ocv_voltage_soc"])),
        "terminal_voltage": ekf.terminal_voltage(f(&t[0]), f(&t[1]), f(&t[2])),
        "cell": {
            "capacity_ah": params.capacity_ah,
            "r0": params.r0,
            "r1": params.r1,
            "c1": params.c1,
        },
    })
}

fn run_routing(input: &Value) -> Value {
    serde_json::from_str(&core_::routing::solve_json(&input.to_string()).unwrap()).unwrap()
}

fn run_orderbook(input: &Value) -> Value {
    let mut book = core_::Book::new();
    let out = book.apply_json(&input["events"].to_string()).unwrap();
    serde_json::from_str(&out).unwrap()
}

fn run_swarm_avoid(input: &Value) -> Value {
    serde_json::from_str(&core_::swarm::avoid_json(&input.to_string()).unwrap()).unwrap()
}

fn run_swarm_find_path(input: &Value) -> Value {
    serde_json::from_str(&core_::swarm::find_path_json(&input.to_string()).unwrap()).unwrap()
}

fn run_consensus(input: &Value) -> Value {
    serde_json::from_str(&core_::consensus::simulate_json(&input.to_string()).unwrap()).unwrap()
}

/// Expected error kind for a fixed error case, as the bindings must report it.
fn run_error(input: &Value) -> Value {
    let kind = match input["op"].as_str().unwrap() {
        "ekf_table" => {
            let points: Vec<(f64, f64)> = (0..input["points"].as_u64().unwrap())
                .map(|i| (i as f64 / 8.0, 3.0))
                .collect();
            let cell = CellSpec {
                capacity_ah: 2.0,
                r0: 0.05,
                r1: 0.02,
                c1: 1000.0,
                process_soc: 1e-9,
                process_v1: 1e-6,
                measurement: 1e-4,
            };
            core_::Ekf::new(&points, cell, 0.5, 0.05, 1e-4).err().unwrap().kind
        }
        "ekf_capacity" => {
            let cell = CellSpec {
                capacity_ah: input["capacity_ah"].as_f64().unwrap(),
                r0: 0.05,
                r1: 0.02,
                c1: 1000.0,
                process_soc: 1e-9,
                process_v1: 1e-6,
                measurement: 1e-4,
            };
            core_::Ekf::new(&[(0.0, 3.0), (1.0, 4.2)], cell, 0.5, 0.05, 1e-4)
                .err()
                .unwrap()
                .kind
        }
        "forest_trees" => core_::Forest::fit(&[vec![1.0], vec![2.0]], 0, 2, 0)
            .err()
            .unwrap()
            .kind,
        "forest_empty" => core_::Forest::fit(&[], 10, 2, 0).err().unwrap().kind,
        "forest_dim" => {
            let forest = core_::Forest::fit(&[vec![1.0, 2.0], vec![2.0, 3.0]], 10, 2, 0).unwrap();
            forest.score(&[1.0]).err().unwrap().kind
        }
        "routing_infeasible" => core_::routing::solve_json(
            r#"{"depot":[0,0],"capacity":10,"customers":[{"x":1,"y":0,"demand":20}]}"#,
        )
        .err()
        .unwrap()
        .kind,
        "swarm_unreachable" => core_::swarm::find_path_json(
            r#"{"width":3,"height":1,"blocked":[[1,0]],"start":[0,0],"goal":[2,0],"connectivity":4}"#,
        )
        .err()
        .unwrap()
        .kind,
        "orderbook_unknown" => core_::Book::new()
            .apply_json(r#"[{"type":"cancel","id":42}]"#)
            .err()
            .unwrap()
            .kind,
        other => panic!("unknown error op {other}"),
    };
    json!({ "kind": kind.as_str() })
}

fn run_case(case: &Value) -> Value {
    let input = &case["input"];
    match case["kind"].as_str().unwrap() {
        "forest" => run_forest(input),
        "average_path_length" => run_average_path_length(input),
        "ekf" => run_ekf(input),
        "routing" => run_routing(input),
        "orderbook" => run_orderbook(input),
        "swarm_avoid" => run_swarm_avoid(input),
        "swarm_find_path" => run_swarm_find_path(input),
        "consensus" => run_consensus(input),
        "error" => run_error(input),
        other => panic!("unknown kind {other}"),
    }
}

#[test]
fn shared_fixtures_match_expected() {
    let doc: Value = serde_json::from_str(FIXTURE).unwrap();
    let print = std::env::var_os("PARITY_PRINT").is_some();
    let mut failures = Vec::new();
    for case in doc["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let actual = run_case(case);
        if print {
            println!("{name}\t{actual}");
            continue;
        }
        if !approx_eq(&actual, &case["expected"]) {
            failures.push(format!(
                "{name}: got {actual}, expected {}",
                case["expected"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "parity mismatches:\n{}",
        failures.join("\n")
    );
}
