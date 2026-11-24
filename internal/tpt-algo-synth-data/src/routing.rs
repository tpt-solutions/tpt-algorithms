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
/// Demands are `u32` by construction: a draw that does not fit (impossible for
/// `max_demand <= u32::MAX`) yields `None` rather than wrapping.
pub fn cvrp_uniform(seed: u64, n: usize, side: f64, max_demand: u32) -> Option<CvrpInstance> {
    if max_demand == 0 || !side.is_finite() || side <= 0.0 {
        return None;
    }
    let mut rng = SplitMix64::new(seed);
    let mut customers = Vec::with_capacity(n);
    for _ in 0..n {
        let x = rng.uniform(0.0, side);
        let y = rng.uniform(0.0, side);
        // max_demand > 0 was checked above, so the draw is Some; the
        // conversions cannot fail for a draw below max_demand, but are checked anyway.
        let draw = rng.next_below(u64::from(max_demand))?;
        let demand = u32::try_from(draw).ok()?.checked_add(1)?;
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn cvrp_is_reproducible_and_feasible() {
        let a = cvrp_uniform(5, 30, 100.0, 10);
        assert_eq!(a, cvrp_uniform(5, 30, 100.0, 10));
        assert!(a.is_some(), "valid parameters must produce an instance");
        let inst = a.unwrap();
        assert_eq!(inst.customers.len(), 30);
        assert!(inst
            .customers
            .iter()
            .all(|c| c.demand >= 1 && c.demand <= inst.capacity));
    }

    #[test]
    fn demands_cover_the_full_range() {
        let inst = cvrp_uniform(8, 2_000, 10.0, 4).unwrap();
        for d in 1..=4u32 {
            assert!(
                inst.customers.iter().any(|c| c.demand == d),
                "demand {d} never drawn"
            );
        }
        assert!(inst.customers.iter().all(|c| (1..=4).contains(&c.demand)));
    }

    #[test]
    fn invalid_parameters_give_none() {
        assert!(cvrp_uniform(1, 5, 10.0, 0).is_none());
        assert!(cvrp_uniform(1, 5, f64::NAN, 3).is_none());
        assert!(cvrp_uniform(1, 5, 0.0, 3).is_none());
        assert!(cvrp_uniform(1, 5, f64::INFINITY, 3).is_none());
    }

    #[test]
    fn max_demand_of_one_gives_unit_demands() {
        let inst = cvrp_uniform(4, 20, 5.0, 1).unwrap();
        assert!(inst.customers.iter().all(|c| c.demand == 1));
        assert_eq!(inst.capacity, 1);
    }

    #[test]
    fn point_cloud_shape() {
        assert_eq!(point_cloud(2, 4, 3, 1.0).len(), 12);
        assert!(point_cloud(2, 4, 0, 1.0).is_empty());
    }
}
