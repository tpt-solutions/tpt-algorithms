// SPDX-License-Identifier: MIT OR Apache-2.0
//! Subcommand dispatch and output rendering.
//!
//! Every subcommand returns a [`Report`]: a JSON value for `--format json`
//! and, when the output is tabular, a CSV-shaped value for `--format csv`.
//! The rendered text is the exact stdout, which is what record and replay
//! compare.

pub mod consensus;
pub mod fraud;
pub mod orderbook;
pub mod route;
pub mod soc;
pub mod swarm;

use crate::args::Args;
use crate::demo;
use crate::error::CliError;
use crate::json::{self, Json};
use crate::source::Source;
use crate::table::{json_to_csv, Table};

/// Output format selected with `--format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Pretty-printed JSON (default).
    Json,
    /// CSV. Only available for tabular output.
    Csv,
}

/// What a subcommand produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Full result, rendered for `--format json`.
    pub json: Json,
    /// Tabular view rendered for `--format csv`. `None` when the output has
    /// no tabular form.
    pub csv: Option<Json>,
}

/// Reads `--format json|csv` (default `json`).
pub fn take_format(args: &mut Args) -> Result<Format, CliError> {
    match args.take_value("--format")?.as_deref() {
        None | Some("json") => Ok(Format::Json),
        Some("csv") => Ok(Format::Csv),
        Some(other) => Err(CliError::Usage(format!(
            "unknown format {other:?}; use json or csv"
        ))),
    }
}

/// Renders a report in the requested format.
pub fn render(report: &Report, format: Format) -> Result<String, CliError> {
    match format {
        Format::Json => Ok(json::to_pretty(&report.json)),
        Format::Csv => match &report.csv {
            Some(rows) => json_to_csv(rows),
            None => Err(CliError::Usage(
                "this command's output is not tabular; use --format json".to_string(),
            )),
        },
    }
}

/// Parses the input named by `--input` into a table, using `source`.
pub fn read_table(
    path: &str,
    source: &mut dyn Source,
    has_header: bool,
) -> Result<Table, CliError> {
    let text = source.read(path)?;
    Table::parse(&text, has_header)
}

/// Runs one invocation. `tokens` is the command line after the binary name.
/// Returns the text to print on stdout.
pub fn run(tokens: Vec<String>, source: &mut dyn Source) -> Result<String, CliError> {
    let mut tokens = tokens;
    if tokens.is_empty() {
        return Err(CliError::Usage(
            "no command given; try tpt --help".to_string(),
        ));
    }
    let name = tokens.remove(0);
    if name == "demo" {
        return demo::run(tokens);
    }
    let mut args = Args::new(tokens);
    let format = take_format(&mut args)?;
    let report = match name.as_str() {
        "soc" => soc::run(args, source)?,
        "fraud" => fraud::run(args, source)?,
        "orderbook" => orderbook::run(args, source)?,
        "route" => route::run(args, source)?,
        "swarm" => swarm::run(args, source)?,
        "consensus" => consensus::run(args)?,
        other => {
            return Err(CliError::Usage(format!(
                "unknown command {other:?}; expected soc, fraud, orderbook, route, swarm, consensus, demo, or replay"
            )))
        }
    };
    render(&report, format)
}
