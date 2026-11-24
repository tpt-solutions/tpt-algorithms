// SPDX-License-Identifier: MIT OR Apache-2.0
//! Trace recording and replay.
//!
//! A trace is a JSON document with the command line, every input text the
//! run read, the exact stdout, and a digest of that stdout. Replay re-runs
//! the command from the trace alone and requires byte-identical output. The
//! format is described in `docs/record-format.md`.

use crate::commands;
use crate::error::CliError;
use crate::json::{self, Json};
use crate::source::MapSource;
use std::collections::BTreeMap;

/// Value of the `format` field in every trace.
pub const FORMAT: &str = "tpt-trace";
/// Value of the `version` field in every trace this build writes or reads.
pub const VERSION: i128 = 1;

/// FNV-1a, 64-bit. Used as an integrity digest, not as a security measure.
pub fn digest(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

/// Writes a trace file for one run.
pub fn write_trace(
    path: &str,
    command: &[String],
    inputs: &BTreeMap<String, String>,
    output: &str,
) -> Result<(), CliError> {
    let doc = Json::obj([
        ("format", FORMAT.into()),
        ("version", Json::Int(VERSION)),
        (
            "command",
            Json::Array(command.iter().map(|s| Json::from(s.as_str())).collect()),
        ),
        (
            "inputs",
            Json::Object(
                inputs
                    .iter()
                    .map(|(k, v)| (k.clone(), Json::from(v.as_str())))
                    .collect(),
            ),
        ),
        ("output", output.into()),
        ("digest", digest(output.as_bytes()).into()),
    ]);
    std::fs::write(path, json::to_pretty(&doc))
        .map_err(|e| CliError::Input(format!("cannot write trace {path}: {e}")))
}

/// Replays the trace at `path`. Returns a one-line summary on success and a
/// [`CliError::Mismatch`] if the output does not reproduce.
pub fn replay(path: &str) -> Result<String, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::Input(format!("cannot read trace {path}: {e}")))?;
    let doc = json::parse(&text)?;

    if doc.get("format").and_then(Json::as_str) != Some(FORMAT) {
        return Err(CliError::Input(format!("{path} is not a tpt trace")));
    }
    if doc.get("version").and_then(Json::as_int) != Some(VERSION) {
        return Err(CliError::Input(format!(
            "{path} has an unsupported trace version"
        )));
    }
    let command = doc
        .get("command")
        .and_then(Json::as_array)
        .ok_or_else(|| CliError::Input("trace has no command".to_string()))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| CliError::Input("trace command must be strings".to_string()))
        })
        .collect::<Result<Vec<String>, _>>()?;
    let mut inputs = BTreeMap::new();
    if let Some(Json::Object(pairs)) = doc.get("inputs") {
        for (k, v) in pairs {
            let text = v
                .as_str()
                .ok_or_else(|| CliError::Input("trace inputs must be strings".to_string()))?;
            inputs.insert(k.clone(), text.to_string());
        }
    }
    let recorded = doc
        .get("output")
        .and_then(Json::as_str)
        .ok_or_else(|| CliError::Input("trace has no output".to_string()))?;
    let recorded_digest = doc
        .get("digest")
        .and_then(Json::as_str)
        .ok_or_else(|| CliError::Input("trace has no digest".to_string()))?;

    let stored = digest(recorded.as_bytes());
    if stored != recorded_digest {
        return Err(CliError::Mismatch(format!(
            "trace digest {recorded_digest} does not match its recorded output ({stored}); the trace was edited"
        )));
    }

    let mut source = MapSource::new(inputs);
    let rerun = commands::run(command, &mut source)?;
    if rerun != recorded {
        return Err(CliError::Mismatch(format!(
            "replay mismatch: output differs from the trace at {}",
            describe_difference(recorded, &rerun)
        )));
    }
    Ok(format!(
        "replay verified: {recorded_digest}, {} bytes identical",
        recorded.len()
    ))
}

/// Describes where two outputs first differ, as a line and column.
fn describe_difference(expected: &str, actual: &str) -> String {
    let index = expected
        .bytes()
        .zip(actual.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| expected.len().min(actual.len()));
    let prefix = &expected.as_bytes()[..index.min(expected.len())];
    let line = prefix.iter().filter(|&&b| b == b'\n').count() + 1;
    let column = prefix.iter().rev().take_while(|&&b| b != b'\n').count() + 1;
    format!("line {line}, column {column} (byte {index})")
}
