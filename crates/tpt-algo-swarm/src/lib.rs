// SPDX-License-Identifier: MIT OR Apache-2.0
//! Velocity-obstacle collision avoidance and grid path planning.
//!
//! * [`VelocityObstacle`]: truncated-cone velocity obstacle between two disc agents.
//! * [`avoid`] / [`avoid_among`]: choose the admissible velocity closest to the
//!   preferred one using a deterministic candidate grid.
//! * `Simulation` (needs `alloc`): synchronous multi-agent stepping.
//! * `Grid` and `find_path` (need `alloc`): A* on an occupancy grid.
//!
//! Everything is `f64`; `no_std` builds use `libm`. No randomness is used.
//!
//! # References
//! Fiorini & Shiller (IJRR 1998); van den Berg, Lin & Manocha (ICRA 2008);
//! van den Berg, Guy, Lin & Manocha (ISRR 2011); Hart, Nilsson & Raphael
//! (IEEE TSSC 1968).
//! # Example
//!
//! ```
//! use tpt_algo_swarm::{avoid, Agent, AvoidanceConfig, Vec2};
//!
//! let a = Agent {
//!     position: Vec2::new(0.0, 0.0),
//!     velocity: Vec2::new(1.0, 0.0),
//!     radius: 0.5,
//!     preferred_velocity: Vec2::new(1.0, 0.0),
//!     max_speed: 1.0,
//! };
//! let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(-1.0, 0.0), ..a };
//! let v = avoid(&a, &[b], &AvoidanceConfig::default()).unwrap();
//! assert!(v.norm() <= 1.0 + 1e-12);
//! ```
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod avoid;
mod error;
#[cfg(feature = "alloc")]
mod grid;
#[cfg(feature = "alloc")]
mod sim;
mod vec2;

pub use avoid::{avoid, avoid_among, Agent, AvoidanceConfig, Responsibility, VelocityObstacle};
pub use error::SwarmError;
#[cfg(feature = "alloc")]
pub use grid::{find_path, Connectivity, Grid, GridPath};
#[cfg(feature = "alloc")]
pub use sim::Simulation;
pub use vec2::Vec2;
