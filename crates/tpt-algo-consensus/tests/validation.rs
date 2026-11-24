// SPDX-License-Identifier: MIT OR Apache-2.0
//! Hand-derived HotStuff traces.
//!
//! `tests/data/hotstuff_traces.txt` holds expected commit timelines derived by
//! hand from the protocol rules (the derivation is in that file's header),
//! not produced by running the simulator. This test runs the simulator for
//! each line and checks that replica 0's committed views match the derivation.
//!
//! Line format: `<n> <seed> <ticks> <committed views, comma separated or ->`.
#![cfg(feature = "alloc")]

use std::path::PathBuf;

use tpt_algo_consensus::sim::{check_prefix_consistency, SimConfig, Simulator};
use tpt_algo_consensus::ReplicaId;

fn trace_file() -> PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "data",
        "hotstuff_traces.txt",
    ]
    .iter()
    .collect()
}

#[test]
fn hand_derived_hotstuff_traces() {
    let path = trace_file();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    let mut checked = 0usize;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 4, "bad line: {line}");
        let n: u16 = fields[0].parse().expect("n");
        let seed: u64 = fields[1].parse().expect("seed");
        let ticks: u64 = fields[2].parse().expect("ticks");
        let expected: Vec<u64> = if fields[3] == "-" {
            Vec::new()
        } else {
            fields[3]
                .split(',')
                .map(|v| v.parse().expect("view"))
                .collect()
        };

        let mut sim = Simulator::new(SimConfig::new(n, seed)).expect("config");
        sim.run(ticks);
        let report = sim.report();
        check_prefix_consistency(&report.honest_logs).expect("prefix consistency");
        let views: Vec<u64> = sim
            .replica(ReplicaId(0))
            .expect("replica 0 exists")
            .committed()
            .iter()
            .map(|b| b.view.0)
            .collect();
        assert_eq!(views, expected, "trace: {line}");
        checked += 1;
    }
    assert!(
        checked >= 8,
        "expected at least 8 trace lines, checked {checked}"
    );
}
