// SPDX-License-Identifier: MIT OR Apache-2.0
//! Synthetic capacitated vehicle routing instances and point clouds.

use alloc::vec::Vec;
use tpt_algo_math_core::rng::SplitMix64;

/// A customer location with demand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Customer {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Demand units (strictly positive).
    pub demand: u32,
}

/// A synthetic CVRP instance. Index 0 of `customers` is not the depot; the depot is `depot`.
#[derive(Debug, Clone, PartialEq)]
pub struct CvrpInstance {
    /// Depot coordinates.
    pub depot: (f64, f64),
    /// Customers.
    pub customers: Vec<Customer>,
    /// Vehicle capacity (every demand is at most this).
    pub capacity: u32,
}

/// Generates `n` customers uniformly in `[0, side]²` with demands in `1..=max_demand`.
/// The capacity is set to `max_demand`, so each customer fits in one vehicle.
/// Returns `None` if `max_demand == 0` or `side` is not finite and positive.
pub fn cvrp_uniform(seed: u64, n: usize, side: f64, max_demand: u32) -> Option<CvrpInstance> {
    if max_demand == 0 || !side.is_finite() || side <= 0.0 {
        return None;
    }
    let mut rng = SplitMix64::new(seed);
    let mut customers = Vec::with_capacity(n);
    for _ in 0..n {
        let x = rng.uniform(0.0, side);
        let y = rng.uniform(0.0, side);
        let demand = rng.next_below(u64::from(max_demand)).unwrap_or(0) as u32 + 1;
        customers.push(Customer { x, y, demand });
    }
    Some(CvrpInstance {
        depot: (side / 2.0, side / 2.0),
        customers,
        capacity: max_demand,
    })
}

/// Uniform random point cloud in `[0, side]^dim` as flat row-major `dim`-tuples.
/// Returns an empty vector if `dim == 0`.
pub fn point_cloud(seed: u64, n: usize, dim: usize, side: f64) -> Vec<f64> {
    if dim == 0 {
        return Vec::new();
    }
    let mut rng = SplitMix64::new(seed);
    (0..n * dim).map(|_| rng.uniform(0.0, side)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cvrp_is_reproducible_and_feasible() {
        let a = cvrp_uniform(5, 30, 100.0, 10);
        assert_eq!(a, cvrp_uniform(5, 30, 100.0, 10));
        let inst = a.unwrap_or(CvrpInstance {
            depot: (0.0, 0.0),
            customers: Vec::new(),
            capacity: 0,
        });
        assert_eq!(inst.customers.len(), 30);
        assert!(inst
            .customers
            .iter()
            .all(|c| c.demand >= 1 && c.demand <= inst.capacity));
    }

    #[test]
    fn invalid_parameters_give_none() {
        assert!(cvrp_uniform(1, 5, 10.0, 0).is_none());
        assert!(cvrp_uniform(1, 5, f64::NAN, 3).is_none());
    }

    #[test]
    fn point_cloud_shape() {
        assert_eq!(point_cloud(2, 4, 3, 1.0).len(), 12);
        assert!(point_cloud(2, 4, 0, 1.0).is_empty());
    }
}
