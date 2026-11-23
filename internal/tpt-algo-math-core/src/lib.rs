// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared numerics for the tpt-algorithms workspace (internal, unpublished).
//!
//! - [`stats`]: mean, variance, RMSE, quantiles (quantiles need `alloc`).
//! - [`tolerance`]: relative-error helpers for validation tests.
//! - [`fixed`]: Q32.32 fixed-point arithmetic for deterministic code paths.
//! - [`rng`]: seeded SplitMix64 generator (deterministic, `no_std`).
//! - [`error`]: the shared [`MathError`] type.
//!
//! The crate is `no_std` by default when built with `--no-default-features`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod fixed;
pub mod rng;
pub mod stats;
pub mod tolerance;

pub use error::MathError;
