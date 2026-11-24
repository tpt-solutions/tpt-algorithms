// SPDX-License-Identifier: MIT OR Apache-2.0
//! Structured error type shared by the Python and WebAssembly bindings.

use core::fmt;

/// Category of a binding error. Both bindings expose the lower-case
/// [`ErrorKind::as_str`] form as the `kind` attribute of the raised error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// A value has the wrong shape, type or range (e.g. non-finite number,
    /// dimension mismatch, unknown order id).
    InvalidInput,
    /// A parameter or option is out of range (e.g. zero trees, bad capacity).
    InvalidConfig,
    /// The input is well formed but not supported by this binding (e.g. an
    /// OCV table with 9 points).
    Unsupported,
    /// The problem has no feasible solution (e.g. a customer that cannot be
    /// served, or more vehicles needed than allowed).
    Infeasible,
    /// The goal cannot be reached (e.g. a grid path that does not exist).
    Unreachable,
}

impl ErrorKind {
    /// Stable lower-case name, used in both languages.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::InvalidConfig => "invalid_config",
            Self::Unsupported => "unsupported",
            Self::Infeasible => "infeasible",
            Self::Unreachable => "unreachable",
        }
    }
}

/// An error with a category and a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingError {
    /// Category of the failure.
    pub kind: ErrorKind,
    /// Message for the user.
    pub message: String,
}

impl BindingError {
    /// Creates an error of the given kind.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for BindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BindingError {}

/// Result type of every binding-core function.
pub type Result<T> = core::result::Result<T, BindingError>;

pub(crate) fn invalid_input(message: impl Into<String>) -> BindingError {
    BindingError::new(ErrorKind::InvalidInput, message)
}

pub(crate) fn invalid_config(message: impl Into<String>) -> BindingError {
    BindingError::new(ErrorKind::InvalidConfig, message)
}
