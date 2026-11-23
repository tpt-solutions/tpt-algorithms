// SPDX-License-Identifier: MIT OR Apache-2.0
//! Validation against external datasets (not run by default).
#![cfg(feature = "std")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use tpt_algo_swarm::{find_path, Connectivity, Grid};

/// Placeholder for benchmark grid maps (e.g. Moving AI `.map` files) whose
/// optimal path lengths are published. Requires `TPT_DATASET_DIR`.
#[test]
#[ignore = "requires dataset; set TPT_DATASET_DIR"]
fn moving_ai_maps_match_published_optimal_lengths() {
    let dir = std::env::var("TPT_DATASET_DIR").unwrap();
    // The loader is intentionally not implemented until a dataset is vetted;
    // fail loudly rather than pass vacuously.
    let grid = Grid::new(1, 1).unwrap();
    let _ = find_path(&grid, (0, 0), (0, 0), Connectivity::Eight).unwrap();
    panic!("dataset loader not implemented (dir: {dir})");
}
