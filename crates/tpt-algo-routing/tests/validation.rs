// SPDX-License-Identifier: MIT OR Apache-2.0
//! Validation against public benchmark instances (not run by default).
//!
//! Set `TPT_DATASET_DIR` to a directory containing the instance files:
//! `A-n32-k5.vrp` (CVRPLIB, best known 784) and `C101.txt` (Solomon,
//! best known distance 828.94 with 10 vehicles). Gaps are indicative bounds
//! for a simple savings + local search heuristic, not optimality claims.
#![cfg(feature = "alloc")]
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use tpt_algo_routing::{solve, Config, Customer, Problem};

fn dataset(name: &str) -> Option<String> {
    let Ok(dir) = std::env::var("TPT_DATASET_DIR") else {
        eprintln!("TPT_DATASET_DIR is not set; skipping {name}");
        return None;
    };
    let path = PathBuf::from(dir).join(name);
    match std::fs::read_to_string(&path) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("cannot read {}: {e}; skipping", path.display());
            None
        }
    }
}

/// Parses a CVRPLIB `.vrp` file (EUC_2D, depot is node 1).
fn parse_cvrplib(text: &str) -> Problem {
    let mut capacity = 0.0;
    let mut coords: Vec<(f64, f64)> = Vec::new();
    let mut demands: Vec<f64> = Vec::new();
    let mut section = "";
    for line in text.lines().map(str::trim) {
        if let Some(v) = line.strip_prefix("CAPACITY") {
            capacity = v.trim_start_matches([' ', ':']).trim().parse().unwrap();
        } else if line.ends_with("_SECTION") {
            section = if line.starts_with("NODE_COORD") {
                "coord"
            } else if line.starts_with("DEMAND") {
                "demand"
            } else {
                ""
            };
        } else {
            let f: Vec<f64> = line
                .split_whitespace()
                .filter_map(|t| t.parse().ok())
                .collect();
            match (section, f.len()) {
                ("coord", 3) => coords.push((f[1], f[2])),
                ("demand", 2) => demands.push(f[1]),
                _ => {}
            }
        }
    }
    let customers = (1..coords.len())
        .map(|i| Customer::new(coords[i].0, coords[i].1, demands[i]))
        .collect();
    Problem::new(coords[0], capacity, customers).unwrap()
}

/// Parses a Solomon instance file (depot is the first customer row).
fn parse_solomon(text: &str) -> Problem {
    let rows: Vec<Vec<f64>> = text
        .lines()
        .map(|l| {
            l.split_whitespace()
                .filter_map(|t| t.parse().ok())
                .collect()
        })
        .collect();
    let capacity = rows.iter().find(|r| r.len() == 2).map(|r| r[1]).unwrap();
    let cust: Vec<&Vec<f64>> = rows.iter().filter(|r| r.len() == 7).collect();
    let depot = cust[0];
    let customers = cust[1..]
        .iter()
        .map(|r| Customer::new(r[1], r[2], r[3]).with_window(r[4], r[5], r[6]))
        .collect();
    Problem::new((depot[1], depot[2]), capacity, customers)
        .unwrap()
        .with_depot_due(depot[5])
        .unwrap()
}

#[test]
#[ignore = "requires dataset; set TPT_DATASET_DIR"]
fn cvrplib_a_n32_k5_gap() {
    let Some(text) = dataset("A-n32-k5.vrp") else {
        return;
    };
    let p = parse_cvrplib(&text);
    let s = solve(&p, &Config::default()).unwrap();
    s.validate(&p).unwrap();
    let gap = (s.cost(&p) - 784.0) / 784.0;
    eprintln!("A-n32-k5: cost {:.2}, gap {:.1}%", s.cost(&p), gap * 100.0);
    assert!(gap < 0.15, "gap {gap}");
}

#[test]
#[ignore = "requires dataset; set TPT_DATASET_DIR"]
fn solomon_c101_gap() {
    let Some(text) = dataset("C101.txt") else {
        return;
    };
    let p = parse_solomon(&text);
    let s = solve(&p, &Config::default()).unwrap();
    s.validate(&p).unwrap();
    let gap = (s.cost(&p) - 828.94) / 828.94;
    eprintln!(
        "C101: cost {:.2}, vehicles {}, gap {:.1}%",
        s.cost(&p),
        s.vehicles_used(),
        gap * 100.0
    );
    assert!(gap < 0.30, "gap {gap}");
}
