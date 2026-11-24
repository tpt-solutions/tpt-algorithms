// SPDX-License-Identifier: MIT OR Apache-2.0
//! Capacitated vehicle routing wrapper (savings construction plus local search).
//!
//! Input JSON:
//!
//! ```json
//! {
//!   "depot": [0.0, 0.0],
//!   "capacity": 10.0,
//!   "customers": [
//!     {"x": 1.0, "y": 0.0, "demand": 5.0},
//!     {"x": 0.0, "y": 2.0, "demand": 4.0, "window": [0.0, 10.0, 1.0]}
//!   ],
//!   "max_vehicles": 2,
//!   "depot_due": 100.0,
//!   "max_iterations": 100000
//! }
//! ```
//!
//! `window` is `[earliest, latest, service_time]`. Every key except `depot`,
//! `capacity` and `customers` is optional. Output JSON:
//! `{"routes": [[0, 1], [2]], "cost": 12.3, "vehicles": 2}`, with customers
//! given as zero-based indices into the input array.

use crate::error::{invalid_input, BindingError, ErrorKind, Result};
use crate::json::{array, field, float, pair, parse, size_or};
use serde_json::{json, Value};
use tpt_algo_routing::{solve, Config, Customer, Problem, RoutingError};

/// Solves a routing problem given as JSON; returns the solution as JSON.
///
/// # Errors
/// Malformed input, or a problem with no feasible solution.
pub fn solve_json(input: &str) -> Result<String> {
    let doc = parse(input)?;
    let depot = pair(
        field(&doc, "depot").ok_or_else(|| invalid_input("missing field 'depot'"))?,
        "depot",
    )?;
    let capacity = float(&doc, "capacity")?;

    let mut customers = Vec::new();
    for (i, c) in array(&doc, "customers")?.iter().enumerate() {
        let customer = customer(c)
            .map_err(|e| BindingError::new(e.kind, format!("customer {i}: {}", e.message)))?;
        customers.push(customer);
    }

    let mut problem =
        Problem::new((depot[0], depot[1]), capacity, customers).map_err(routing_error)?;
    if let Some(max) = field(&doc, "max_vehicles") {
        let max = max
            .as_u64()
            .ok_or_else(|| invalid_input("'max_vehicles' must be a non-negative integer"))?;
        let max = usize::try_from(max).map_err(|_| invalid_input("'max_vehicles' is too large"))?;
        problem = problem.with_max_vehicles(max).map_err(routing_error)?;
    }
    if field(&doc, "depot_due").is_some() {
        problem = problem
            .with_depot_due(float(&doc, "depot_due")?)
            .map_err(routing_error)?;
    }

    let config = Config {
        max_iterations: size_or(&doc, "max_iterations", Config::default().max_iterations)?,
    };
    let solution = solve(&problem, &config).map_err(routing_error)?;
    let cost = solution.cost(&problem);
    let vehicles = solution.vehicles_used();
    Ok(json!({
        "routes": solution.routes,
        "cost": cost,
        "vehicles": vehicles,
    })
    .to_string())
}

fn customer(c: &Value) -> Result<Customer> {
    let mut customer = Customer::new(float(c, "x")?, float(c, "y")?, float(c, "demand")?);
    if let Some(w) = field(c, "window") {
        let items = w
            .as_array()
            .filter(|a| a.len() == 3)
            .ok_or_else(|| invalid_input("'window' must be [earliest, latest, service_time]"))?;
        let mut vals = [0.0_f64; 3];
        for (slot, item) in vals.iter_mut().zip(items) {
            *slot = item
                .as_f64()
                .ok_or_else(|| invalid_input("'window' must hold numbers"))?;
        }
        customer = customer.with_window(vals[0], vals[1], vals[2]);
    }
    Ok(customer)
}

fn routing_error(e: RoutingError) -> BindingError {
    let kind = match e {
        RoutingError::InfeasibleCustomer { .. } | RoutingError::TooManyVehicles { .. } => {
            ErrorKind::Infeasible
        }
        _ => ErrorKind::InvalidInput,
    };
    BindingError::new(kind, e.to_string())
}
