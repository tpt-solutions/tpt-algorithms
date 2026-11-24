// SPDX-License-Identifier: MIT OR Apache-2.0
//! Where input text comes from.
//!
//! Commands read their inputs through [`Source`]. A normal run reads files
//! (or stdin for `-`) and, when recording, keeps a copy of everything it read.
//! A replay or a demo supplies the same inputs from memory, so the re-run
//! never touches the filesystem.

use crate::error::CliError;
use std::collections::BTreeMap;
use std::io::Read as _;

/// Supplies named input texts to a command.
pub trait Source {
    /// Returns the full text of the input named `path` (`-` means stdin).
    fn read(&mut self, path: &str) -> Result<String, CliError>;
}

/// Reads from the filesystem and stdin, and remembers every input it returns.
#[derive(Debug, Default)]
pub struct FsSource {
    /// Inputs read so far, keyed by the path as given on the command line.
    pub captured: BTreeMap<String, String>,
}

impl Source for FsSource {
    fn read(&mut self, path: &str) -> Result<String, CliError> {
        let text = if path == "-" {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| CliError::Input(format!("cannot read stdin: {e}")))?;
            buf
        } else {
            std::fs::read_to_string(path)
                .map_err(|e| CliError::Input(format!("cannot read {path}: {e}")))?
        };
        self.captured.insert(path.to_string(), text.clone());
        Ok(text)
    }
}

/// Serves inputs from memory. Used for replay and for bundled demo data.
#[derive(Debug, Default)]
pub struct MapSource {
    files: BTreeMap<String, String>,
}

impl MapSource {
    /// Creates a source over the given path-to-text map.
    pub fn new(files: BTreeMap<String, String>) -> Self {
        Self { files }
    }

    /// Creates a source from `(path, text)` pairs.
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        Self {
            files: pairs
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }
}

impl Source for MapSource {
    fn read(&mut self, path: &str) -> Result<String, CliError> {
        self.files.get(path).cloned().ok_or_else(|| {
            CliError::Input(format!("input {path:?} is not available in this trace"))
        })
    }
}
