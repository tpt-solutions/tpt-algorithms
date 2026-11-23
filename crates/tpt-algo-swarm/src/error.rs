// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type.

/// Errors returned by this crate.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SwarmError {
    /// A configuration parameter is out of range.
    #[error("invalid configuration: {0}")]
    InvalidConfig(&'static str),
    /// An agent has a non-finite value, or a non-positive radius or max speed.
    #[error("invalid agent: {0}")]
    InvalidAgent(&'static str),
    /// Grid dimensions do not match the supplied cell data, or are zero.
    #[error("invalid grid: {0}")]
    InvalidGrid(&'static str),
    /// A cell lies outside the grid.
    #[error("cell ({x}, {y}) is outside the grid")]
    OutOfBounds {
        /// Column.
        x: usize,
        /// Row.
        y: usize,
    },
    /// The start cell is blocked.
    #[error("start cell is blocked")]
    StartBlocked,
    /// The goal cell is blocked.
    #[error("goal cell is blocked")]
    GoalBlocked,
    /// No path connects start and goal.
    #[error("goal is unreachable from start")]
    Unreachable,
    /// Index into the agent list is out of range.
    #[error("agent index out of range")]
    IndexOutOfRange,
}
