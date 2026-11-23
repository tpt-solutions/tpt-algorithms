// SPDX-License-Identifier: MIT OR Apache-2.0
//! Vehicle routing (CVRP, optionally with time windows): Clarke & Wright
//! savings construction followed by 2-opt / relocate / swap local search.
//!
//! The model is a single depot, a homogeneous fleet with capacity (and an
//! optional vehicle limit), and customers with demand, optional time window
//! `[earliest, latest]` and service time. Distances are Euclidean and travel
//! time equals distance. Arriving early means waiting; arriving after
//! `latest` is infeasible.
//!
//! # Features
//! - `std` (default): implies `alloc`.
//! - `alloc`: enables the problem model and solvers (they need `Vec`).
//! - `serde`: `Serialize`/`Deserialize` for plain data types
//!   ([`Customer`], [`TimeWindow`], [`Config`]).
//!
//! # References
//! - Clarke & Wright, "Scheduling of vehicles from a central depot to a
//!   number of delivery points", Operations Research 12(4), 1964.
//! - Croes, "A method for solving traveling-salesman problems", Operations
//!   Research 6(6), 1958 (2-opt); Lin, "Computer solutions of the traveling
//!   salesman problem", Bell System Technical Journal 44, 1965.
//! - Solomon, "Algorithms for the vehicle routing and scheduling problems
//!   with time window constraints", Operations Research 35(2), 1987.
//! - Toth & Vigo (eds.), "Vehicle Routing: Problems, Methods, and
//!   Applications", SIAM, 2nd ed., 2014.
//! # Example
//!
//! ```
//! # #[cfg(feature = "alloc")] {
//! use tpt_algo_routing::{solve, Config, Customer, Problem};
//!
//! let customers = vec![
//!     Customer::new(1.0, 0.0, 5.0),
//!     Customer::new(2.0, 0.0, 5.0),
//!     Customer::new(-1.0, 0.0, 5.0),
//!     Customer::new(-2.0, 0.0, 5.0),
//! ];
//! let problem = Problem::new((0.0, 0.0), 10.0, customers).unwrap();
//! let solution = solve(&problem, &Config::default()).unwrap();
//! assert!((solution.cost(&problem) - 8.0).abs() < 1e-9);
//! # }
//! ```
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
pub use error::RoutingError;

#[cfg(feature = "alloc")]
mod local_search;
#[cfg(feature = "alloc")]
mod model;
#[cfg(feature = "alloc")]
mod savings;

#[cfg(feature = "alloc")]
pub use local_search::{local_search, Config};
#[cfg(feature = "alloc")]
pub use model::{Customer, Problem, Solution, TimeWindow};
#[cfg(feature = "alloc")]
pub use savings::clarke_wright_savings;

/// Constructs a solution with savings and improves it with local search.
///
/// # Errors
/// Propagates errors from [`clarke_wright_savings`] and [`local_search`].
///
/// # Example
///
/// ```
/// use tpt_algo_routing::{solve, Config, Customer, Problem};
///
/// let customers = vec![Customer::new(1.0, 0.0, 5.0), Customer::new(-1.0, 0.0, 5.0)];
/// let problem = Problem::new((0.0, 0.0), 10.0, customers).unwrap();
/// let solution = solve(&problem, &Config::default()).unwrap();
/// assert!(solution.validate(&problem).is_ok());
/// assert_eq!(solution.vehicles_used(), 1); // both customers fit on one vehicle
/// ```
#[cfg(feature = "alloc")]
pub fn solve(problem: &Problem, config: &Config) -> Result<Solution, RoutingError> {
    let mut solution = clarke_wright_savings(problem)?;
    local_search(problem, &mut solution, config)?;
    Ok(solution)
}

#[cfg(all(test, feature = "alloc"))]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use alloc::vec;

    fn line_problem() -> Problem {
        Problem::new(
            (0.0, 0.0),
            10.0,
            vec![
                Customer::new(1.0, 0.0, 5.0),
                Customer::new(2.0, 0.0, 5.0),
                Customer::new(-1.0, 0.0, 5.0),
                Customer::new(-2.0, 0.0, 5.0),
            ],
        )
        .unwrap()
    }

    #[test]
    fn known_answer_four_customers() {
        // Optimum: east pair and west pair, each route 0->1->2->0 = 4; total 8.
        let p = line_problem();
        let s = solve(&p, &Config::default()).unwrap();
        s.validate(&p).unwrap();
        assert_eq!(s.vehicles_used(), 2);
        assert!((s.cost(&p) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn savings_alone_finds_optimum_here() {
        let p = line_problem();
        let s = clarke_wright_savings(&p).unwrap();
        assert!((s.cost(&p) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn time_windows_force_split() {
        // Visiting both on one route would arrive at t=3 at the second stop.
        let p = Problem::new(
            (0.0, 0.0),
            100.0,
            vec![
                Customer::new(1.0, 0.0, 1.0).with_window(0.0, 1.0, 0.0),
                Customer::new(-1.0, 0.0, 1.0).with_window(0.0, 2.5, 0.0),
            ],
        )
        .unwrap();
        let s = solve(&p, &Config::default()).unwrap();
        s.validate(&p).unwrap();
        assert_eq!(s.vehicles_used(), 2);
        assert!((s.cost(&p) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn waiting_is_allowed() {
        let p = Problem::new(
            (0.0, 0.0),
            100.0,
            vec![
                Customer::new(1.0, 0.0, 1.0).with_window(5.0, 6.0, 1.0),
                Customer::new(2.0, 0.0, 1.0).with_window(0.0, 8.0, 0.0),
            ],
        )
        .unwrap();
        let s = solve(&p, &Config::default()).unwrap();
        s.validate(&p).unwrap();
        assert_eq!(s.vehicles_used(), 1);
    }

    #[test]
    fn errors() {
        assert!(matches!(
            Problem::new((0.0, 0.0), 0.0, vec![]),
            Err(RoutingError::InvalidInput(_))
        ));
        assert_eq!(
            Problem::new((0.0, 0.0), 5.0, vec![Customer::new(1.0, 1.0, 6.0)]),
            Err(RoutingError::InfeasibleCustomer { customer: 0 })
        );
        assert!(matches!(
            Problem::new((0.0, 0.0), 5.0, vec![Customer::new(f64::NAN, 1.0, 1.0)]),
            Err(RoutingError::InvalidInput(_))
        ));
        let p = line_problem().with_max_vehicles(1).unwrap();
        assert_eq!(
            clarke_wright_savings(&p),
            Err(RoutingError::TooManyVehicles {
                required: 2,
                available: 1
            })
        );
    }

    #[test]
    fn empty_problem() {
        let p = Problem::new((0.0, 0.0), 1.0, vec![]).unwrap();
        let s = solve(&p, &Config::default()).unwrap();
        assert!(s.routes.is_empty());
    }

    #[test]
    fn validate_detects_problems() {
        let p = line_problem();
        let dup = Solution {
            routes: vec![vec![0, 1], vec![1, 2], vec![3]],
        };
        assert!(dup.validate(&p).is_err());
        let over = Solution {
            routes: vec![vec![0, 1, 2, 3]],
        };
        assert!(over.validate(&p).is_err());
        let mut bad = Solution {
            routes: vec![vec![0, 1, 2, 3]],
        };
        assert!(local_search(&p, &mut bad, &Config::default()).is_err());
    }

    #[test]
    fn two_opt_untangles_route() {
        let p = Problem::new(
            (0.0, 0.0),
            100.0,
            vec![
                Customer::new(1.0, 1.0, 1.0),
                Customer::new(-1.0, 1.0, 1.0),
                Customer::new(1.0, -1.0, 1.0),
                Customer::new(-1.0, -1.0, 1.0),
            ],
        )
        .unwrap();
        // Crossing tour: 0 -> 3 -> ... deliberately bad order.
        let mut s = Solution {
            routes: vec![vec![0, 3, 1, 2]],
        };
        let before = s.cost(&p);
        let n = local_search(&p, &mut s, &Config::default()).unwrap();
        assert!(n > 0);
        assert!(s.cost(&p) < before - 1e-6);
        s.validate(&p).unwrap();
    }

    #[test]
    fn iteration_limit_zero_is_noop() {
        let p = line_problem();
        let mut s = Solution {
            routes: vec![vec![0, 3], vec![1, 2]],
        };
        let orig = s.clone();
        let n = local_search(&p, &mut s, &Config { max_iterations: 0 }).unwrap();
        assert_eq!(n, 0);
        assert_eq!(s, orig);
    }
}
