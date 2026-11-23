// SPDX-License-Identifier: MIT OR Apache-2.0
//! Minimal CVRP example: build a problem, solve, print routes.

#[cfg(feature = "alloc")]
use tpt_algo_routing::{solve, Config, Customer, Problem, RoutingError};

#[cfg(not(feature = "alloc"))]
fn main() {}

#[cfg(feature = "alloc")]
fn main() -> Result<(), RoutingError> {
    let customers = vec![
        Customer::new(1.0, 0.0, 5.0),
        Customer::new(2.0, 0.0, 5.0),
        Customer::new(-1.0, 0.0, 5.0),
        Customer::new(-2.0, 0.0, 5.0),
    ];
    let problem = Problem::new((0.0, 0.0), 10.0, customers)?;
    let solution = solve(&problem, &Config::default())?;
    for (i, route) in solution.routes.iter().enumerate() {
        println!("route {i}: {route:?}");
    }
    println!("total cost: {:.3}", solution.cost(&problem));
    Ok(())
}
