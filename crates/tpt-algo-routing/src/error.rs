// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type for the routing crate.

/// Errors returned by problem construction, solvers and solution validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RoutingError {
    /// A problem parameter is not finite, negative, or otherwise malformed.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    /// A single customer cannot be served by any vehicle (demand above
    /// capacity, or time window unreachable from the depot).
    #[error("customer {customer} cannot be served by any single-customer route")]
    InfeasibleCustomer {
        /// Zero-based customer index.
        customer: usize,
    },
    /// The constructed solution needs more vehicles than allowed.
    #[error("solution needs {required} vehicles but only {available} are available")]
    TooManyVehicles {
        /// Vehicles required by the solution found.
        required: usize,
        /// Vehicles available.
        available: usize,
    },
    /// A solution failed validation against its problem.
    #[error("invalid solution: {0}")]
    InvalidSolution(&'static str),
}
