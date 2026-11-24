// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type for the command-line front end and its exit codes.

use core::fmt;

/// Failure modes of the `tpt` binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// Bad command line: unknown command, flag or value. Exit code 2.
    Usage(String),
    /// Input could not be read or parsed, or an algorithm rejected its input.
    /// Exit code 1.
    Input(String),
    /// A trace did not verify on replay. Exit code 3.
    Mismatch(String),
}

impl CliError {
    /// Process exit code for this error.
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Usage(_) => 2,
            CliError::Input(_) => 1,
            CliError::Mismatch(_) => 3,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(m) | CliError::Input(m) | CliError::Mismatch(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for CliError {}

/// Wraps an error from an algorithm crate as an input error (exit code 1).
pub fn algo(e: impl fmt::Display) -> CliError {
    CliError::Input(e.to_string())
}
