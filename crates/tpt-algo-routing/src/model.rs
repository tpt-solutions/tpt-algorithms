// SPDX-License-Identifier: MIT OR Apache-2.0
//! Problem model: depot, homogeneous capacitated vehicles, customers with
//! optional time windows.

use alloc::vec;
use alloc::vec::Vec;

use crate::RoutingError;

/// Numerical tolerance used in feasibility and improvement checks.
pub(crate) const EPS: f64 = 1e-9;

/// A closed time window `[earliest, latest]`.
///
/// A vehicle may arrive before `earliest` and wait; arriving after `latest`
/// is infeasible.
///
/// # Example
///
/// ```
/// use tpt_algo_routing::{Customer, TimeWindow};
///
/// let w = TimeWindow { earliest: 10.0, latest: 20.0 };
/// let c = Customer::new(1.0, 0.0, 1.0).with_window(w.earliest, w.latest, 2.0);
/// assert_eq!(c.window, Some(w));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeWindow {
    /// Earliest time service may start.
    pub earliest: f64,
    /// Latest time service may start.
    pub latest: f64,
}

/// A customer to be served exactly once.
///
/// # Example
///
/// ```
/// use tpt_algo_routing::Customer;
///
/// // A customer at (3, 4) with demand 2, a time window [0, 50] and 5 units of service.
/// let c = Customer::new(3.0, 4.0, 2.0).with_window(0.0, 50.0, 5.0);
/// assert_eq!(c.demand, 2.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Customer {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Demand (non-negative, at most the vehicle capacity).
    pub demand: f64,
    /// Optional service time window.
    pub window: Option<TimeWindow>,
    /// Service duration (non-negative).
    pub service_time: f64,
}

impl Customer {
    /// Customer without a time window and with zero service time.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_routing::Customer;
    ///
    /// let c = Customer::new(3.0, 4.0, 2.0);
    /// assert_eq!(c.service_time, 0.0);
    /// assert!(c.window.is_none());
    /// ```
    #[must_use]
    pub fn new(x: f64, y: f64, demand: f64) -> Self {
        Self {
            x,
            y,
            demand,
            window: None,
            service_time: 0.0,
        }
    }

    /// Adds a time window and service time.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_routing::Customer;
    ///
    /// let c = Customer::new(0.0, 0.0, 1.0).with_window(5.0, 15.0, 3.0);
    /// assert_eq!(c.service_time, 3.0);
    /// assert_eq!(c.window.map(|w| w.latest), Some(15.0));
    /// ```
    #[must_use]
    pub fn with_window(mut self, earliest: f64, latest: f64, service_time: f64) -> Self {
        self.window = Some(TimeWindow { earliest, latest });
        self.service_time = service_time;
        self
    }
}

/// A capacitated vehicle routing problem with a single depot.
///
/// Distances are Euclidean; travel time equals distance. Vehicles leave the
/// depot at time 0.
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
/// assert_eq!(problem.capacity(), 10.0);
/// assert_eq!(problem.customers().len(), 4);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    depot: (f64, f64),
    capacity: f64,
    max_vehicles: Option<usize>,
    depot_due: f64,
    customers: Vec<Customer>,
}

impl Problem {
    /// Builds and validates a problem.
    ///
    /// # Errors
    /// [`RoutingError::InvalidInput`] for non-finite values, non-positive
    /// capacity, negative demand/service time or inverted windows;
    /// [`RoutingError::InfeasibleCustomer`] if a customer alone violates
    /// capacity or its window.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_routing::{Customer, Problem};
    ///
    /// let problem = Problem::new((0.0, 0.0), 10.0, vec![Customer::new(1.0, 0.0, 5.0)]).unwrap();
    /// assert_eq!(problem.depot(), (0.0, 0.0));
    /// ```
    pub fn new(
        depot: (f64, f64),
        capacity: f64,
        customers: Vec<Customer>,
    ) -> Result<Self, RoutingError> {
        let p = Self {
            depot,
            capacity,
            max_vehicles: None,
            depot_due: f64::INFINITY,
            customers,
        };
        p.validate()?;
        Ok(p)
    }

    /// Limits the number of vehicles (routes).
    ///
    /// # Errors
    /// [`RoutingError::InvalidInput`] if `max` is zero while customers exist.
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
    /// let limited = problem.with_max_vehicles(2)?;
    /// assert_eq!(limited.max_vehicles(), Some(2));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_max_vehicles(mut self, max: usize) -> Result<Self, RoutingError> {
        if max == 0 && !self.customers.is_empty() {
            return Err(RoutingError::InvalidInput("max_vehicles must be positive"));
        }
        self.max_vehicles = Some(max);
        Ok(self)
    }

    /// Sets the latest time a vehicle may return to the depot.
    ///
    /// # Errors
    /// Same validation as [`Problem::new`] is re-run.
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
    /// // Every route must return to the depot by time 30.
    /// let timed = problem.with_depot_due(30.0)?;
    /// assert_eq!(timed.depot_due(), 30.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_depot_due(mut self, due: f64) -> Result<Self, RoutingError> {
        if due.is_nan() || due < 0.0 {
            return Err(RoutingError::InvalidInput("depot_due must be >= 0"));
        }
        self.depot_due = due;
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> Result<(), RoutingError> {
        if !(self.depot.0.is_finite() && self.depot.1.is_finite()) {
            return Err(RoutingError::InvalidInput("depot coordinates not finite"));
        }
        if !(self.capacity.is_finite() && self.capacity > 0.0) {
            return Err(RoutingError::InvalidInput("capacity must be positive"));
        }
        for c in &self.customers {
            if !(c.x.is_finite() && c.y.is_finite() && c.demand.is_finite()) {
                return Err(RoutingError::InvalidInput("customer value not finite"));
            }
            if c.demand < 0.0 {
                return Err(RoutingError::InvalidInput("negative demand"));
            }
            if !(c.service_time.is_finite() && c.service_time >= 0.0) {
                return Err(RoutingError::InvalidInput("invalid service time"));
            }
            if let Some(w) = c.window {
                if w.earliest.is_nan() || w.latest.is_nan() || w.earliest > w.latest {
                    return Err(RoutingError::InvalidInput("invalid time window"));
                }
            }
        }
        let m = Matrix::new(self);
        for i in 0..self.customers.len() {
            if !m.feasible(self, &[i]) {
                return Err(RoutingError::InfeasibleCustomer { customer: i });
            }
        }
        Ok(())
    }

    /// Depot coordinates.
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
    /// assert_eq!(problem.depot(), (0.0, 0.0));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn depot(&self) -> (f64, f64) {
        self.depot
    }

    /// Vehicle capacity.
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
    /// assert_eq!(problem.capacity(), 10.0);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn capacity(&self) -> f64 {
        self.capacity
    }

    /// Maximum number of vehicles, if limited.
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
    /// assert_eq!(problem.max_vehicles(), None); // unlimited by default
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn max_vehicles(&self) -> Option<usize> {
        self.max_vehicles
    }

    /// Latest depot return time (infinite by default).
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
    /// assert!(problem.depot_due().is_infinite()); // no deadline by default
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn depot_due(&self) -> f64 {
        self.depot_due
    }

    /// The customers.
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
    /// assert_eq!(problem.customers()[0].x, 1.0);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn customers(&self) -> &[Customer] {
        &self.customers
    }
}

/// Precomputed Euclidean distance matrix; node 0 is the depot, customer `i`
/// is node `i + 1`.
pub(crate) struct Matrix {
    n: usize,
    d: Vec<f64>,
}

impl Matrix {
    pub(crate) fn new(p: &Problem) -> Self {
        let n = p.customers.len() + 1;
        let mut pts = Vec::with_capacity(n);
        pts.push(p.depot);
        pts.extend(p.customers.iter().map(|c| (c.x, c.y)));
        let mut d = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                let dx = pts[i].0 - pts[j].0;
                let dy = pts[i].1 - pts[j].1;
                d[i * n + j] = libm::sqrt(dx * dx + dy * dy);
            }
        }
        Self { n, d }
    }

    /// Distance between nodes (0 = depot, customer `c` = `c + 1`).
    #[inline]
    pub(crate) fn node(&self, a: usize, b: usize) -> f64 {
        self.d[a * self.n + b]
    }

    /// Distance between two customers.
    #[inline]
    pub(crate) fn cust(&self, a: usize, b: usize) -> f64 {
        self.node(a + 1, b + 1)
    }

    /// Distance depot to customer.
    #[inline]
    pub(crate) fn depot_to(&self, c: usize) -> f64 {
        self.node(0, c + 1)
    }

    /// Cost of a route starting and ending at the depot.
    pub(crate) fn route_cost(&self, route: &[usize]) -> f64 {
        let (Some(&first), Some(&last)) = (route.first(), route.last()) else {
            return 0.0;
        };
        let mut cost = self.depot_to(first) + self.depot_to(last);
        for w in route.windows(2) {
            cost += self.cust(w[0], w[1]);
        }
        cost
    }

    /// Capacity and time-window feasibility of a route.
    pub(crate) fn feasible(&self, p: &Problem, route: &[usize]) -> bool {
        let mut load = 0.0;
        let mut t = 0.0;
        let mut prev = 0usize;
        for &c in route {
            let cu = &p.customers[c];
            load += cu.demand;
            t += self.node(prev, c + 1);
            if let Some(w) = cu.window {
                if t > w.latest + EPS {
                    return false;
                }
                if t < w.earliest {
                    t = w.earliest;
                }
            }
            t += cu.service_time;
            prev = c + 1;
        }
        if load > p.capacity + EPS {
            return false;
        }
        if route.is_empty() {
            return true;
        }
        t += self.node(prev, 0);
        t <= p.depot_due + EPS
    }
}

/// A set of routes; each route lists customer indices in visiting order.
/// The depot is implicit at both ends.
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
/// use tpt_algo_routing::Solution;
///
/// // Two routes of two customers each, by customer index.
/// let sol = Solution { routes: vec![vec![0, 1], vec![2, 3]] };
/// sol.validate(&problem)?;
/// assert_eq!(sol.vehicles_used(), 2);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Solution {
    /// The routes.
    pub routes: Vec<Vec<usize>>,
}

impl Solution {
    /// Total Euclidean length of all routes.
    ///
    /// Customer indices must be in range; call [`Solution::validate`] first
    /// for solutions of unknown origin.
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
    /// use tpt_algo_routing::Solution;
    ///
    /// let sol = Solution { routes: vec![vec![0, 1], vec![2, 3]] };
    /// assert!((sol.cost(&problem) - 8.0).abs() < 1e-9);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn cost(&self, problem: &Problem) -> f64 {
        let m = Matrix::new(problem);
        self.routes.iter().map(|r| m.route_cost(r)).sum()
    }

    /// Number of non-empty routes.
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
    /// use tpt_algo_routing::Solution;
    ///
    /// let sol = Solution { routes: vec![vec![0, 1, 2, 3]] };
    /// assert_eq!(sol.vehicles_used(), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn vehicles_used(&self) -> usize {
        self.routes.iter().filter(|r| !r.is_empty()).count()
    }

    /// Checks that every customer is served exactly once and that every
    /// route respects capacity, time windows and the vehicle limit.
    ///
    /// # Errors
    /// [`RoutingError::InvalidSolution`] or [`RoutingError::TooManyVehicles`].
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
    /// use tpt_algo_routing::{RoutingError, Solution};
    ///
    /// // Customer 0 served twice, customer 3 never: not a valid solution.
    /// let bad = Solution { routes: vec![vec![0, 0, 1, 2]] };
    /// assert!(matches!(bad.validate(&problem), Err(RoutingError::InvalidSolution(_))));
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate(&self, problem: &Problem) -> Result<(), RoutingError> {
        let n = problem.customers.len();
        let mut seen = vec![false; n];
        for &c in self.routes.iter().flatten() {
            let slot = seen
                .get_mut(c)
                .ok_or(RoutingError::InvalidSolution("customer index out of range"))?;
            if *slot {
                return Err(RoutingError::InvalidSolution("customer served twice"));
            }
            *slot = true;
        }
        if seen.iter().any(|s| !s) {
            return Err(RoutingError::InvalidSolution("customer not served"));
        }
        let m = Matrix::new(problem);
        if self.routes.iter().any(|r| !m.feasible(problem, r)) {
            return Err(RoutingError::InvalidSolution(
                "route violates capacity or time window",
            ));
        }
        if let Some(max) = problem.max_vehicles {
            let used = self.vehicles_used();
            if used > max {
                return Err(RoutingError::TooManyVehicles {
                    required: used,
                    available: max,
                });
            }
        }
        Ok(())
    }
}
