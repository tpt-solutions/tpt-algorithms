// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type.

/// Errors returned by this crate.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum FraudError {
    /// The training data contained no rows.
    #[error("training data is empty")]
    EmptyData,
    /// A row had no features.
    #[error("rows must have at least one feature")]
    ZeroDimension,
    /// The number of trees was zero.
    #[error("number of trees must be at least 1")]
    InvalidTreeCount,
    /// The subsample size was below 2.
    #[error("subsample size must be at least 2, got {0}")]
    InvalidSubsampleSize(usize),
    /// A row had a different number of features than expected.
    #[error("row {row} has {found} features, expected {expected}")]
    DimensionMismatch {
        /// Index of the offending row.
        row: usize,
        /// Expected feature count.
        expected: usize,
        /// Feature count found.
        found: usize,
    },
    /// A value was NaN or infinite.
    #[error("non-finite value at row {row}, column {column}")]
    NonFinite {
        /// Row index.
        row: usize,
        /// Column index.
        column: usize,
    },
}
