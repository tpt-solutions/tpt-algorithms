// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt consensus`: seeded, deterministic run of the consensus simulator.
//!
//! This subcommand takes options instead of an input file: the simulator is
//! driven entirely by its seed and configuration. Output includes the
//! committed chain of every honest replica and a prefix-consistency check.
//!
//! Options: `--n` (replicas, default 4), `--seed` (default 0), `--ticks`
//! (default 200), `--gst` (global stabilisation tick, default 0),
//! `--drop-per-mille` (pre-GST drop rate, default 0), `--crash ID@TICK`
//! (repeatable), `--equivocate ID` (repeatable).

use super::Report;
use crate::args::Args;
use crate::error::{algo, CliError};
use crate::json::Json;
use tpt_algo_consensus::sim::{
    check_prefix_consistency, FaultKind, FaultSpec, SimConfig, Simulator,
};
use tpt_algo_consensus::ReplicaId;

/// Runs the subcommand. `args` holds the tokens after `consensus`.
pub fn run(mut args: Args) -> Result<Report, CliError> {
    let n: u16 = args.take_parsed("--n", 4)?;
    let seed: u64 = args.take_parsed("--seed", 0)?;
    let ticks: u64 = args.take_parsed("--ticks", 200)?;
    let gst: u64 = args.take_parsed("--gst", 0)?;
    let drop_per_mille: u16 = args.take_parsed("--drop-per-mille", 0)?;
    let mut faults = Vec::new();
    for raw in args.take_all("--crash")? {
        let (id, tick) = raw
            .split_once('@')
            .ok_or_else(|| CliError::Usage(format!("--crash expects ID@TICK, got {raw:?}")))?;
        faults.push(FaultSpec {
            replica: ReplicaId(parse_id(id)?),
            kind: FaultKind::Crash {
                at_tick: tick
                    .parse()
                    .map_err(|_| CliError::Usage(format!("bad tick in --crash {raw:?}")))?,
            },
        });
    }
    for raw in args.take_all("--equivocate")? {
        faults.push(FaultSpec {
            replica: ReplicaId(parse_id(&raw)?),
            kind: FaultKind::Equivocate,
        });
    }
    if !args.positionals()?.is_empty() {
        return Err(CliError::Usage(
            "consensus takes no positional arguments".to_string(),
        ));
    }

    let config = SimConfig {
        gst,
        pre_gst_drop_per_mille: drop_per_mille,
        faults,
        ..SimConfig::new(n, seed)
    };
    let mut sim = Simulator::new(config).map_err(algo)?;
    sim.run(ticks);
    let report = sim.report();
    let consistency = check_prefix_consistency(&report.honest_logs);

    let replicas: Vec<Json> = report
        .honest_logs
        .iter()
        .map(|(id, log)| {
            let ids: Vec<Json> = log.iter().map(|b| Json::Int(i128::from(b.id.0))).collect();
            Json::obj([
                ("replica", Json::Int(i128::from(id.0))),
                ("committed", log.len().into()),
                ("block_ids", Json::Array(ids)),
            ])
        })
        .collect();
    let summary: Vec<Json> = report
        .honest_logs
        .iter()
        .map(|(id, log)| {
            Json::obj([
                ("replica", Json::Int(i128::from(id.0))),
                ("committed", log.len().into()),
            ])
        })
        .collect();
    let stats = &report.stats;
    let json = Json::obj([
        ("n", Json::Int(i128::from(n))),
        ("seed", seed.into()),
        ("ticks", ticks.into()),
        ("now", report.now.into()),
        ("max_honest_view", report.max_honest_view.0.into()),
        (
            "stats",
            Json::obj([
                ("delivered", stats.delivered.into()),
                ("dropped", stats.dropped.into()),
                ("equivocations", stats.equivocations.into()),
                ("rejected_from_honest", stats.rejected_from_honest.into()),
                ("rejected_from_faulty", stats.rejected_from_faulty.into()),
                ("safety_violations", stats.safety_violations.into()),
            ]),
        ),
        ("prefix_consistent", consistency.is_ok().into()),
        (
            "consistency_error",
            match &consistency {
                Ok(()) => Json::Null,
                Err(e) => Json::Str(e.to_string()),
            },
        ),
        ("replicas", Json::Array(replicas)),
    ]);
    Ok(Report {
        json,
        csv: Some(Json::Array(summary)),
    })
}

fn parse_id(raw: &str) -> Result<u16, CliError> {
    raw.trim()
        .parse::<u16>()
        .map_err(|_| CliError::Usage(format!("replica id must be a number, got {raw:?}")))
}
