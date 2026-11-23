// SPDX-License-Identifier: MIT OR Apache-2.0
//! Battery state-of-charge (SoC) estimation with an Extended Kalman Filter.
//!
//! The cell is modelled as a first-order Thevenin equivalent circuit
//! (open-circuit voltage source, series resistance `R0`, one RC pair).
//! The filter state is `[SoC, V1]` where `V1` is the RC-pair voltage.
//!
//! The filter lifecycle is encoded in the type system: a
//! [`Filter<Uninitialized, N>`] must be turned into a
//! [`Filter<Initialized, N>`] with [`Filter::initialize`] (which validates its
//! inputs) before `predict` / `update` become available.
//!
//! Sign convention: current is positive on discharge (amps), terminal
//! voltage is in volts, time in seconds, capacity in ampere-hours.
//!
//! The crate is `no_std` compatible and needs no allocator; the OCV table is
//! a const-generic array.
#![cfg_attr(not(feature = "std"), no_std)]

mod error;
mod filter;
mod model;

pub use error::SocError;
pub use filter::{Estimate, Filter, Initialized, NoiseConfig, Uninitialized};
pub use model::{CellParams, Model, OcvCurve};
