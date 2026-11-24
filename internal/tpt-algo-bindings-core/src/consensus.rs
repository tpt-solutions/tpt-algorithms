// SPDX-License-Identifier: MIT OR Apache-2.0
//! Seeded, deterministic consensus simulator, as JSON. No live replicas are
//! exposed: a run is fully described by its input and gives the same output
//! every time.
//!
//! Input JSON:
//!
//! ```json
//! {
//!   "n": 4, "seed": 1, "ticks": 200,
//!   "base_timeout": 20, "max_backoff_shift": 4, "gst": 0,
//!   "pre_gst_drop_per_mille": 0, "pre_gst_max_delay": 1, "post_gst_max_delay": 1,
//!   "faults": [{"replica": 3, "kind": "equivocate"},
//!              {"replica": 2, "kind": "crash", "at_tick": 50}]
//! }
//! ```
//!
//! Only `n`, `ticks` and (optionally) `seed` are required in practice; the
//! rest default to [`SimConfig::new`]. Block ids are 16-digit hex strings, since
//! a `u64` does not fit a JavaScript number. Output JSON has `now`,
//! `max_honest_view`, `stats`, `honest_logs` (one block list per non-faulty
//! replica) and `prefix_consistent`.

use crate::error::{invalid_config, invalid_input, BindingError, ErrorKind, Result};
use crate::json::{array, field, parse, string, uint, uint_or};
use serde_json::{json, Value};
use tpt_algo_consensus::sim::{
    check_prefix_consistency, FaultKind, FaultSpec, SimConfig, Simulator,
};
use tpt_algo_consensus::{ConsensusError, ReplicaId};

/// Runs the simulator for `ticks` steps and returns the report as JSON.
///
/// # Errors
/// Malformed input, or a configuration the simulator rejects.
pub fn simulate_json(input: &str) -> Result<String> {
    let doc = parse(input)?;
    let n = u16::try_from(uint(&doc, "n")?)
        .map_err(|_| invalid_config("'n' is too large for a u16"))?;
    let ticks = uint(&doc, "ticks")?;
    let mut cfg = SimConfig::new(n, uint_or(&doc, "seed", 0)?);
    let defaults = SimConfig::new(n, 0);
    cfg.base_timeout = uint_or(&doc, "base_timeout", defaults.base_timeout)?;
    cfg.max_backoff_shift = u32::try_from(uint_or(
        &doc,
        "max_backoff_shift",
        u64::from(defaults.max_backoff_shift),
    )?)
    .map_err(|_| invalid_config("'max_backoff_shift' is too large"))?;
    cfg.gst = uint_or(&doc, "gst", defaults.gst)?;
    cfg.pre_gst_drop_per_mille = u16::try_from(uint_or(
        &doc,
        "pre_gst_drop_per_mille",
        u64::from(defaults.pre_gst_drop_per_mille),
    )?)
    .map_err(|_| invalid_config("'pre_gst_drop_per_mille' is too large"))?;
    cfg.pre_gst_max_delay = uint_or(&doc, "pre_gst_max_delay", defaults.pre_gst_max_delay)?;
    cfg.post_gst_max_delay = uint_or(&doc, "post_gst_max_delay", defaults.post_gst_max_delay)?;
    if field(&doc, "faults").is_some() {
        for f in array(&doc, "faults")? {
            cfg.faults.push(fault(f)?);
        }
    }

    let mut sim = Simulator::new(cfg).map_err(consensus_error)?;
    sim.run(ticks);
    let report = sim.report();
    let prefix_consistent = check_prefix_consistency(&report.honest_logs).is_ok();

    let logs: Vec<Value> = report
        .honest_logs
        .iter()
        .map(|(id, blocks)| {
            json!({
                "replica": id.0,
                "blocks": blocks.iter().map(|b| json!({
                    "id": format!("{:016x}", b.id.0),
                    "height": b.height,
                    "view": b.view.0,
                    "proposer": b.proposer.0,
                    "payload": b.payload,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let s = report.stats;
    Ok(json!({
        "now": report.now,
        "max_honest_view": report.max_honest_view.0,
        "prefix_consistent": prefix_consistent,
        "stats": {
            "delivered": s.delivered,
            "dropped": s.dropped,
            "equivocations": s.equivocations,
            "rejected_from_honest": s.rejected_from_honest,
            "rejected_from_faulty": s.rejected_from_faulty,
            "safety_violations": s.safety_violations,
        },
        "honest_logs": logs,
    })
    .to_string())
}

fn fault(v: &Value) -> Result<FaultSpec> {
    let replica = u16::try_from(uint(v, "replica")?)
        .map_err(|_| invalid_input("fault 'replica' is too large for a u16"))?;
    let kind = match string(v, "kind")? {
        "equivocate" => FaultKind::Equivocate,
        "crash" => FaultKind::Crash {
            at_tick: uint(v, "at_tick")?,
        },
        other => {
            return Err(invalid_input(format!(
                "fault kind must be 'crash' or 'equivocate', got '{other}'"
            )))
        }
    };
    Ok(FaultSpec {
        replica: ReplicaId(replica),
        kind,
    })
}

fn consensus_error(e: ConsensusError) -> BindingError {
    BindingError::new(ErrorKind::InvalidConfig, e.to_string())
}
