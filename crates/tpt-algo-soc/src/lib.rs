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
//! # Example
//!
//! ```
//! use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};
//!
//! # fn main() -> Result<(), SocError> {
//! let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)])?;
//! let params = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
//! let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
//! let mut f = Filter::new(Model::new(params, ocv)?, noise)?.initialize(0.8, 0.05, 1e-4)?;
//! f.predict(1.0, 1.0)?; // discharge current (A), dt (s)
//! f.update(3.9, 1.0)?; // measured terminal voltage (V), current (A)
//! let _soc = f.estimate().soc;
//! # Ok(())
//! # }
//! ```
//!
#![cfg_attr(not(feature = "std"), no_std)]

mod error;
mod filter;
mod model;

pub use error::SocError;
pub use filter::{Estimate, Filter, Initialized, NoiseConfig, Uninitialized};
pub use model::{CellParams, Model, OcvCurve};
