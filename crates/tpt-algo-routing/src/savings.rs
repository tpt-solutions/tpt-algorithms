// SPDX-License-Identifier: MIT OR Apache-2.0
//! Clarke & Wright savings construction (parallel version).

use alloc::vec::Vec;

use crate::model::Matrix;
use crate::{Problem, RoutingError, Solution};

/// Builds a solution with the Clarke & Wright (1964) savings heuristic.
///
/// Starts with one route per customer, then merges route endpoints in order
/// of decreasing savings `s(i,j) = d(0,i) + d(0,j) - d(i,j)` whenever the
/// merged route stays capacity- and time-window-feasible. Ties are broken by
/// customer index, so the result is deterministic.
///
/// # Errors
/// [`RoutingError::TooManyVehicles`] if the vehicle limit is exceeded.
///
/// # Example
///
/// ```
/// use tpt_algo_routing::{Customer, Problem, RoutingError};
/// # fn main() -> Result<(), RoutingError> {
/// let customers = vec![
///     Customer::new(1.0, 0.0, 5.0),
///     Customer::new(2.0, 0.0, 5.0),
///     Customer::new(-1.0, 0.0, 5.0),
///     Customer::new(-2.0, 0.0, 5.0),
/// ];
/// let problem = Problem::new((0.0, 0.0), 10.0, customers)?;
/// use tpt_algo_routing::clarke_wright_savings;
///
/// let sol = clarke_wright_savings(&problem)?;
/// sol.validate(&problem)?; // every customer served once, all routes feasible
/// # Ok(())
/// # }
/// ```
pub fn clarke_wright_savings(problem: &Problem) -> Result<Solution, RoutingError> {
    let n = problem.customers().len();
    let m = Matrix::new(problem);
    let mut routes: Vec<Vec<usize>> = (0..n).map(|i| alloc::vec![i]).collect();
    let mut route_of: Vec<usize> = (0..n).collect();

    let mut savings: Vec<(f64, usize, usize)> = Vec::with_capacity(n * n.saturating_sub(1) / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            savings.push((m.depot_to(i) + m.depot_to(j) - m.cust(i, j), i, j));
        }
    }
    savings.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

    for &(_, i, j) in &savings {
        let (ri, rj) = (route_of[i], route_of[j]);
        if ri == rj {
            continue;
        }
        let merged = try_merge(problem, &m, &routes[ri], &routes[rj], i, j)
            .or_else(|| try_merge(problem, &m, &routes[rj], &routes[ri], j, i));
        if let Some(new_route) = merged {
            for &c in &new_route {
                route_of[c] = ri;
            }
            routes[ri] = new_route;
            routes[rj] = Vec::new();
        }
    }
    routes.retain(|r| !r.is_empty());

    if let Some(max) = problem.max_vehicles() {
        if routes.len() > max {
            return Err(RoutingError::TooManyVehicles {
                required: routes.len(),
                available: max,
            });
        }
    }
    Ok(Solution { routes })
}

/// Tries to join `a` then `b` so that customer `i` (an endpoint of `a`) is
/// followed directly by customer `j` (an endpoint of `b`), reversing routes
/// as needed.
fn try_merge(
    p: &Problem,
    m: &Matrix,
    a: &[usize],
    b: &[usize],
    i: usize,
    j: usize,
) -> Option<Vec<usize>> {
    for rev_a in [false, true] {
        let a_last = if rev_a { a.first() } else { a.last() };
        if a_last != Some(&i) {
            continue;
        }
        for rev_b in [false, true] {
            let b_first = if rev_b { b.last() } else { b.first() };
            if b_first != Some(&j) {
                continue;
            }
            let mut cand: Vec<usize> = Vec::with_capacity(a.len() + b.len());
            if rev_a {
                cand.extend(a.iter().rev());
            } else {
                cand.extend(a.iter());
            }
            if rev_b {
                cand.extend(b.iter().rev());
            } else {
                cand.extend(b.iter());
            }
            if m.feasible(p, &cand) {
                return Some(cand);
            }
        }
    }
    None
}
