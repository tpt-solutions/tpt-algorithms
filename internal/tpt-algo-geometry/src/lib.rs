// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared 2D/3D geometry and kinematics helpers (internal, unpublished).
//!
//! `no_std` by default. Square roots come from `libm`, so results do not depend on
//! the platform's math library.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod error;
pub mod kinematics;
pub mod shapes2d;
pub mod vec;

pub use error::GeometryError;
pub use vec::{Vec2, Vec3};
