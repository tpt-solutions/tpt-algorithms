// SPDX-License-Identifier: MIT OR Apache-2.0
//! Binding-independent wrappers around the `tpt-algo` crates.
//!
//! The Python and WebAssembly bindings both call this crate, so the logic is
//! written once. Everything here uses plain Rust types, [`BindingError`] for
//! failures, and JSON strings for the structured inputs (routing, order
//! book, swarm and consensus). No FFI appears in this crate.
//!
//! - [`fraud`]: Isolation Forest (fit, scores, mean path length).
//! - [`soc`]: battery state-of-charge EKF. The filter is generic over the
//!   OCV table length `N`, which cannot cross an FFI boundary, so [`soc::Ekf`]
//!   dispatches at runtime over table lengths 2 to 8.
//! - [`routing`], [`orderbook`], [`swarm`], [`consensus`]: JSON in, JSON out.

pub mod consensus;
pub mod error;
pub mod fraud;
pub mod orderbook;
pub mod routing;
pub mod soc;
pub mod swarm;

mod json;

pub use error::{BindingError, ErrorKind, Result};
pub use fraud::{average_path_length, forest_defaults, Forest};
pub use orderbook::Book;
pub use soc::{CellSpec, Ekf, EkfDefaults, EKF_DEFAULTS, MAX_OCV_POINTS, MIN_OCV_POINTS};
