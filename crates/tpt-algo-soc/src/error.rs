// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type.

/// Errors returned by model construction and filter operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SocError {
    /// Capacity must be finite and strictly positive.
    #[error("capacity must be finite and > 0")]
    InvalidCapacity,
    /// `R0` must be finite and non-negative; `R1` finite and strictly positive.
    #[error("resistance parameter invalid (R0 >= 0, R1 > 0, finite)")]
    InvalidResistance,
    /// `C1` must be finite and strictly positive.
    #[error("capacitance must be finite and > 0")]
    InvalidCapacitance,
    /// The time step must be finite and strictly positive.
    #[error("time step must be finite and > 0")]
    InvalidTimeStep,
    /// The OCV table is malformed (too few points, non-monotone SoC, SoC outside `[0,1]`).
    #[error("invalid OCV table: {0}")]
    InvalidOcvTable(&'static str),
    /// A noise variance is negative, non-finite, or the measurement variance is zero.
    #[error("invalid noise configuration")]
    InvalidNoise,
    /// Initial SoC outside `[0,1]` or initial covariance invalid.
    #[error("invalid initial state or covariance")]
    InvalidInitialState,
    /// A runtime input (current or voltage) was NaN or infinite.
    #[error("non-finite input: {0}")]
    NonFinite(&'static str),
}
