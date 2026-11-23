// SPDX-License-Identifier: MIT OR Apache-2.0
//! Local search: intra-route 2-opt, relocate and inter-route swap.

use alloc::vec::Vec;

use crate::model::{Matrix, EPS};
use crate::{Problem, RoutingError, Solution};

/// Local search configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// Maximum number of improving moves applied. The search is
    /// deterministic: moves are scanned in a fixed order and the first
    /// strictly improving feasible move is applied.
    pub max_iterations: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_iterations: 100_000,
        }
    }
}

/// Improves `solution` in place with 2-opt (Croes 1958), relocate and swap
/// moves (cf. Lin 1965 and Toth & Vigo 2014, ch. 5).
///
/// Every applied move is feasible (capacity, time windows) and strictly
/// decreases the total cost, so the cost never increases. Returns the
/// number of improving moves applied. Empty routes are removed.
///
/// # Errors
/// [`RoutingError::InvalidSolution`] if `solution` is not a valid, feasible
/// solution of `problem` on entry.
pub fn local_search(
    problem: &Problem,
    solution: &mut Solution,
    config: &Config,
) -> Result<usize, RoutingError> {
    solution.validate(problem)?;
    let m = Matrix::new(problem);
    solution.routes.retain(|r| !r.is_empty());
    let mut applied = 0;
    while applied < config.max_iterations
        && (two_opt(problem, &m, &mut solution.routes)
            || relocate(problem, &m, &mut solution.routes)
            || swap(problem, &m, &mut solution.routes))
    {
        applied += 1;
        solution.routes.retain(|r| !r.is_empty());
    }
    Ok(applied)
}

#[inline]
fn prev_node(r: &[usize], i: usize) -> usize {
    if i == 0 {
        0
    } else {
        r[i - 1] + 1
    }
}

#[inline]
fn next_node(r: &[usize], i: usize) -> usize {
    r.get(i + 1).map_or(0, |&c| c + 1)
}

fn two_opt(p: &Problem, m: &Matrix, routes: &mut [Vec<usize>]) -> bool {
    for route in routes.iter_mut() {
        let len = route.len();
        for i in 0..len {
            for j in (i + 1)..len {
                let (a, b) = (prev_node(route, i), route[i] + 1);
                let (c, d) = (route[j] + 1, next_node(route, j));
                let delta = m.node(a, c) + m.node(b, d) - m.node(a, b) - m.node(c, d);
                if delta < -EPS {
                    let mut cand = route.clone();
                    cand[i..=j].reverse();
                    if m.feasible(p, &cand) {
                        *route = cand;
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn relocate(p: &Problem, m: &Matrix, routes: &mut [Vec<usize>]) -> bool {
    for a in 0..routes.len() {
        for i in 0..routes[a].len() {
            let c = routes[a][i];
            for b in 0..routes.len() {
                if a == b {
                    if relocate_within(p, m, &mut routes[a], i) {
                        return true;
                    }
                    continue;
                }
                let ra = &routes[a];
                let (pa, na) = (prev_node(ra, i), next_node(ra, i));
                let gain = m.node(pa, c + 1) + m.node(c + 1, na) - m.node(pa, na);
                let rb = &routes[b];
                for k in 0..=rb.len() {
                    let pb = if k == 0 { 0 } else { rb[k - 1] + 1 };
                    let nb = rb.get(k).map_or(0, |&x| x + 1);
                    let cost = m.node(pb, c + 1) + m.node(c + 1, nb) - m.node(pb, nb);
                    if cost - gain < -EPS {
                        let mut cand_b = rb.clone();
                        cand_b.insert(k, c);
                        let mut cand_a = ra.clone();
                        cand_a.remove(i);
                        if m.feasible(p, &cand_b) && m.feasible(p, &cand_a) {
                            routes[a] = cand_a;
                            routes[b] = cand_b;
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn relocate_within(p: &Problem, m: &Matrix, route: &mut Vec<usize>, i: usize) -> bool {
    let base = m.route_cost(route);
    let c = route[i];
    for k in 0..route.len() {
        if k == i {
            continue;
        }
        let mut cand = route.clone();
        cand.remove(i);
        cand.insert(k, c);
        if m.route_cost(&cand) < base - EPS && m.feasible(p, &cand) {
            *route = cand;
            return true;
        }
    }
    false
}

fn swap(p: &Problem, m: &Matrix, routes: &mut [Vec<usize>]) -> bool {
    for a in 0..routes.len() {
        for b in (a + 1)..routes.len() {
            let (ra, rb) = (&routes[a], &routes[b]);
            for i in 0..ra.len() {
                for j in 0..rb.len() {
                    let (ca, cb) = (ra[i] + 1, rb[j] + 1);
                    let (pa, na) = (prev_node(ra, i), next_node(ra, i));
                    let (pb, nb) = (prev_node(rb, j), next_node(rb, j));
                    let delta = m.node(pa, cb) + m.node(cb, na) - m.node(pa, ca) - m.node(ca, na)
                        + m.node(pb, ca)
                        + m.node(ca, nb)
                        - m.node(pb, cb)
                        - m.node(cb, nb);
                    if delta < -EPS {
                        let mut cand_a = ra.clone();
                        let mut cand_b = rb.clone();
                        cand_a[i] = cb - 1;
                        cand_b[j] = ca - 1;
                        if m.feasible(p, &cand_a) && m.feasible(p, &cand_b) {
                            routes[a] = cand_a;
                            routes[b] = cand_b;
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}
