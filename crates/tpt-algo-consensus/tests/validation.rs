// SPDX-License-Identifier: MIT OR Apache-2.0
//! External-data validation.
//!
//! There is no public dataset for BFT consensus traces; validation is done by
//! the scripted known-answer scenarios in `tests/scenarios.rs`. This test
//! replays an optional external scenario file so that independently produced
//! expectations (for example hand-derived traces from the HotStuff paper) can
//! be checked without changing the crate.
//!
//! File format (`$TPT_DATASET_DIR/consensus/scenarios.txt`), one scenario per
//! line, `#` starts a comment:
//!
//! ```text
//! <n> <seed> <ticks> <expected committed views of replica 0, comma separated>
//! ```
#![cfg(feature = "alloc")]

use std::path::PathBuf;

use tpt_algo_consensus::sim::{check_prefix_consistency, SimConfig, Simulator};
use tpt_algo_consensus::ReplicaId;

#[test]
#[ignore = "requires dataset; set TPT_DATASET_DIR"]
fn external_scenarios() {
    let dir = std::env::var("TPT_DATASET_DIR").expect("TPT_DATASET_DIR not set");
    let path: PathBuf = [dir.as_str(), "consensus", "scenarios.txt"]
        .iter()
        .collect();
    let text = std::fs::read_to_string(&path).expect("cannot read scenarios.txt");
    let mut checked = 0;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 4, "bad line: {line}");
        let n: u16 = fields[0].parse().unwrap();
        let seed: u64 = fields[1].parse().unwrap();
        let ticks: u64 = fields[2].parse().unwrap();
        let expected: Vec<u64> = fields[3].split(',').map(|v| v.parse().unwrap()).collect();
        let mut sim = Simulator::new(SimConfig::new(n, seed)).unwrap();
        sim.run(ticks);
        let report = sim.report();
        check_prefix_consistency(&report.honest_logs).unwrap();
        let views: Vec<u64> = sim
            .replica(ReplicaId(0))
            .unwrap()
            .committed()
            .iter()
            .map(|b| b.view.0)
            .collect();
        assert_eq!(views, expected, "scenario: {line}");
        checked += 1;
    }
    assert!(checked > 0, "no scenarios in {}", path.display());
}
