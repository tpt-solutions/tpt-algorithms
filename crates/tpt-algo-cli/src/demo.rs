// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt demo <crate>`: runs a subcommand on small bundled datasets.
//!
//! The data files under `data/demo/` are embedded in the binary, so a demo
//! needs no files on disk and always prints the same output. Because the
//! inputs come from a [`MapSource`], a demo can also be recorded and replayed.

use crate::args::Args;
use crate::commands;
use crate::error::CliError;
use crate::source::MapSource;

const SOC_CSV: &str = include_str!("../data/demo/soc.csv");
const FRAUD_CSV: &str = include_str!("../data/demo/fraud.csv");
const ORDERBOOK_CSV: &str = include_str!("../data/demo/orderbook.csv");
const ROUTE_CSV: &str = include_str!("../data/demo/route.csv");
const SWARM_CSV: &str = include_str!("../data/demo/swarm.csv");

/// Names accepted by `tpt demo`.
pub const DEMO_NAMES: [&str; 6] = ["soc", "fraud", "orderbook", "route", "swarm", "consensus"];

/// Runs `tpt demo <crate> [--format json|csv]`. `tokens` follows `demo`.
pub fn run(tokens: Vec<String>) -> Result<String, CliError> {
    let mut args = Args::new(tokens);
    let format = args.take_value("--format")?;
    let positionals = args.positionals()?;
    let name = match positionals.as_slice() {
        [name] => name.as_str(),
        _ => {
            return Err(CliError::Usage(format!(
                "usage: tpt demo <{}> [--format json|csv]",
                DEMO_NAMES.join("|")
            )))
        }
    };

    let (mut command, files): (Vec<String>, Vec<(&str, &str)>) = match name {
        "soc" => (
            tokens_of(&[
                "soc",
                "--input",
                "demo/soc.csv",
                "--soc0",
                "0.9",
                "--soc-variance",
                "0.05",
            ]),
            vec![("demo/soc.csv", SOC_CSV)],
        ),
        "fraud" => (
            tokens_of(&[
                "fraud",
                "score",
                "--input",
                "demo/fraud.csv",
                "--trees",
                "100",
                "--subsample",
                "64",
                "--seed",
                "7",
            ]),
            vec![("demo/fraud.csv", FRAUD_CSV)],
        ),
        "orderbook" => (
            tokens_of(&["orderbook", "--input", "demo/orderbook.csv"]),
            vec![("demo/orderbook.csv", ORDERBOOK_CSV)],
        ),
        "route" => (
            tokens_of(&[
                "route",
                "--input",
                "demo/route.csv",
                "--depot",
                "0,0",
                "--capacity",
                "10",
            ]),
            vec![("demo/route.csv", ROUTE_CSV)],
        ),
        "swarm" => (
            tokens_of(&["swarm", "avoid", "--input", "demo/swarm.csv"]),
            vec![("demo/swarm.csv", SWARM_CSV)],
        ),
        "consensus" => (
            tokens_of(&["consensus", "--n", "4", "--seed", "7", "--ticks", "200"]),
            Vec::new(),
        ),
        other => {
            return Err(CliError::Usage(format!(
                "unknown demo {other:?}; choose one of {}",
                DEMO_NAMES.join(", ")
            )))
        }
    };
    if let Some(f) = format {
        command.push("--format".to_string());
        command.push(f);
    }
    let mut source = MapSource::from_pairs(&files);
    commands::run(command, &mut source)
}

fn tokens_of(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}
