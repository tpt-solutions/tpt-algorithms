// SPDX-License-Identifier: MIT OR Apache-2.0
//! Deterministic synthetic data generators (internal, unpublished).
//!
//! Every generator takes an explicit seed and is reproducible: the same seed and
//! parameters give identical output on all platforms. Used to build CI fixtures so no
//! third-party data is committed to the repository.
//!
//! Requires `alloc` (the generators return `Vec`s).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod gaussian;
pub mod order_flow;
pub mod routing;
pub mod series;
pub mod transactions;
