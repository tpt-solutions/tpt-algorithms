// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type for geometry operations.

use thiserror::Error;

/// Errors returned by `tpt-algo-geometry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GeometryError {
    /// The operation needs a non-zero vector or non-degenerate shape.
    #[error("degenerate input (zero-length vector or collinear points)")]
    Degenerate,
    /// A NaN or infinite coordinate was supplied.
    #[error("non-finite coordinate")]
    NonFinite,
    /// A time step or duration was negative where only non-negative values make sense.
    #[error("negative time value")]
    NegativeTime,
}
