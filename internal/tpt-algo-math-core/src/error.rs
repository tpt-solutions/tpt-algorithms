// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type shared by the math primitives.

use thiserror::Error;

/// Errors returned by `tpt-algo-math-core` operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MathError {
    /// An operation that needs at least one (or two) samples received fewer.
    #[error("input is empty or too short for this operation")]
    InsufficientData,
    /// Two slices that must have equal length did not.
    #[error("length mismatch: {left} vs {right}")]
    LengthMismatch {
        /// Length of the left operand.
        left: usize,
        /// Length of the right operand.
        right: usize,
    },
    /// A non-finite value (NaN or infinity) was supplied where a finite one is required.
    #[error("non-finite value encountered")]
    NonFinite,
    /// A quantile was outside `[0, 1]`.
    #[error("quantile must be within [0, 1]")]
    InvalidQuantile,
    /// Fixed-point arithmetic overflowed or divided by zero.
    #[error("fixed-point overflow or division by zero")]
    Overflow,
}
