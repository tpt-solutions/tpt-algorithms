// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt`: command-line front end for the tpt-algorithms crates.
//!
//! Reads CSV or JSON, writes JSON or CSV to stdout, and can record a run to a
//! trace file that `tpt replay` later verifies byte for byte. See
//! `docs/cli.md` and `docs/record-format.md`.

mod args;
mod commands;
mod demo;
mod error;
mod json;
mod record;
mod source;
mod table;

use crate::args::Args;
use crate::error::CliError;
use crate::source::FsSource;
use std::io::Write as _;
use std::process::ExitCode;

const USAGE: &str = "\
tpt: algorithms from the tpt-algorithms workspace on the command line

USAGE:
    tpt <command> [options] [--format json|csv] [--record <trace.json>]
    tpt demo <crate> [--format json|csv]
    tpt replay <trace.json>

COMMANDS:
    soc          Battery state of charge (EKF)           --input <file|-> [cell options]
    fraud        Isolation-forest scores                 --input <file|-> [--train <file>]
    fraud explain  Per-feature contributions for a row   --input <file|-> [--row N] [--replace mean|median]
    orderbook    Replay limit-order events               --input <file|->
    route        Capacitated vehicle routing             --input <file|-> --capacity <n> [--depot x,y]
    swarm avoid  Velocity-obstacle avoidance             --input <file|->
    swarm path   Grid shortest path                      --input <file|-> --from x,y --to x,y [--diagonal]
    consensus    Seeded consensus simulation             [--n N] [--seed S] [--ticks T] [--crash ID@TICK]
    demo         Run a crate on bundled data: soc, fraud, orderbook, route, swarm, consensus
    replay       Re-run a trace and verify the output is byte-identical

Inputs are CSV (first line is a header) or JSON (array of objects), detected
from the first character. Use - to read stdin. Run `docs/cli.md` for every option.
";

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match run(argv) {
        Ok(text) => {
            let mut stdout = std::io::stdout().lock();
            if stdout
                .write_all(text.as_bytes())
                .and_then(|()| stdout.flush())
                .is_err()
            {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("tpt: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}

/// Runs one invocation and returns the text for stdout.
fn run(argv: Vec<String>) -> Result<String, CliError> {
    match argv.first().map(String::as_str) {
        None | Some("help" | "--help" | "-h") => return Ok(USAGE.to_string()),
        Some("replay") => {
            let positionals = Args::new(argv[1..].to_vec()).positionals()?;
            return match positionals.as_slice() {
                [trace] => record::replay(trace).map(|summary| format!("{summary}\n")),
                _ => Err(CliError::Usage(
                    "usage: tpt replay <trace.json>".to_string(),
                )),
            };
        }
        Some(_) => {}
    }

    let mut args = Args::new(argv);
    let record_path = args.take_value("--record")?;
    let tokens = args.into_tokens();
    if tokens.iter().any(|t| t == "--help" || t == "-h") {
        return Ok(USAGE.to_string());
    }

    match record_path {
        None => commands::run(tokens, &mut FsSource::default()),
        Some(path) => {
            let mut source = FsSource::default();
            let output = commands::run(tokens.clone(), &mut source)?;
            record::write_trace(&path, &tokens, &source.captured, &output)?;
            Ok(output)
        }
    }
}
