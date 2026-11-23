// SPDX-License-Identifier: MIT OR Apache-2.0
//! Synchronous multi-agent simulation.

use alloc::vec::Vec;

use crate::{avoid_among, Agent, AvoidanceConfig, SwarmError, Vec2};

/// A set of agents with goals, advanced in lock-step.
///
/// Each step every agent heads for its goal at up to `max_speed` (slowing so
/// it does not overshoot), then replaces that preferred velocity by the
/// result of [`avoid_among`] computed from the state at the start of the step.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{Agent, AvoidanceConfig, Simulation, Vec2};
///
/// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(0.0, 0.0),
///     radius: 0.5, preferred_velocity: Vec2::new(0.0, 0.0), max_speed: 1.0 };
/// let mut sim = Simulation::new(vec![a], vec![Vec2::new(3.0, 0.0)]).unwrap();
/// sim.step(0.5, &AvoidanceConfig::default()).unwrap();
/// assert!(sim.agents[0].position.x > 0.0); // moved toward its goal
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Simulation {
    /// Agents (public so callers can inspect positions and velocities).
    pub agents: Vec<Agent>,
    /// One goal per agent.
    pub goals: Vec<Vec2>,
}

impl Simulation {
    /// Creates a simulation.
    ///
    /// # Errors
    /// [`SwarmError::InvalidConfig`] if lengths differ or a goal is not
    /// finite; [`SwarmError::InvalidAgent`] for a bad agent.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, Simulation, SwarmError, Vec2};
    ///
    /// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(0.0, 0.0),
    ///     radius: 0.5, preferred_velocity: Vec2::new(0.0, 0.0), max_speed: 1.0 };
    /// // One goal for two agents is rejected.
    /// assert!(matches!(
    ///     Simulation::new(vec![a], vec![Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)]),
    ///     Err(SwarmError::InvalidConfig(_))
    /// ));
    /// ```
    pub fn new(agents: Vec<Agent>, goals: Vec<Vec2>) -> Result<Self, SwarmError> {
        if agents.len() != goals.len() {
            return Err(SwarmError::InvalidConfig(
                "agents and goals must have equal length",
            ));
        }
        if goals.iter().any(|g| !g.is_finite()) {
            return Err(SwarmError::InvalidConfig("goal must be finite"));
        }
        for a in &agents {
            a.validate()?;
        }
        Ok(Self { agents, goals })
    }

    /// Advances the simulation by `dt` seconds.
    ///
    /// # Errors
    /// [`SwarmError::InvalidConfig`] if `dt` is not positive and finite, plus
    /// errors from [`avoid_among`].
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, AvoidanceConfig, Simulation, SwarmError, Vec2};
    ///
    /// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(0.0, 0.0),
    ///     radius: 0.5, preferred_velocity: Vec2::new(0.0, 0.0), max_speed: 1.0 };
    /// let mut sim = Simulation::new(vec![a], vec![Vec2::new(3.0, 0.0)]).unwrap();
    /// assert!(matches!(sim.step(0.0, &AvoidanceConfig::default()), Err(SwarmError::InvalidConfig(_))));
    /// ```
    pub fn step(&mut self, dt: f64, cfg: &AvoidanceConfig) -> Result<(), SwarmError> {
        if !(dt.is_finite() && dt > 0.0) {
            return Err(SwarmError::InvalidConfig("dt must be positive"));
        }
        for (a, g) in self.agents.iter_mut().zip(&self.goals) {
            let to_goal = *g - a.position;
            let d = to_goal.norm();
            a.preferred_velocity = if d > 1e-12 {
                to_goal * (a.max_speed.min(d / dt) / d)
            } else {
                Vec2::ZERO
            };
        }
        let mut new_vel = Vec::with_capacity(self.agents.len());
        for i in 0..self.agents.len() {
            new_vel.push(avoid_among(&self.agents, i, cfg)?);
        }
        for (a, v) in self.agents.iter_mut().zip(new_vel) {
            a.velocity = v;
            a.position = a.position + v * dt;
        }
        Ok(())
    }

    /// Whether every agent is within `tol` of its goal.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, Simulation, Vec2};
    ///
    /// let a = Agent { position: Vec2::new(3.0, 0.0), velocity: Vec2::new(0.0, 0.0),
    ///     radius: 0.5, preferred_velocity: Vec2::new(0.0, 0.0), max_speed: 1.0 };
    /// let sim = Simulation::new(vec![a], vec![Vec2::new(3.0, 0.0)]).unwrap();
    /// assert!(sim.all_at_goals(0.01));
    /// ```
    #[must_use]
    pub fn all_at_goals(&self, tol: f64) -> bool {
        self.agents
            .iter()
            .zip(&self.goals)
            .all(|(a, g)| (*g - a.position).norm() <= tol)
    }

    /// Minimum over pairs of `distance - (r_i + r_j)`; negative means
    /// overlap. `None` with fewer than two agents.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, Simulation, Vec2};
    ///
    /// let mk = |x| Agent { position: Vec2::new(x, 0.0), velocity: Vec2::new(0.0, 0.0),
    ///     radius: 0.5, preferred_velocity: Vec2::new(0.0, 0.0), max_speed: 1.0 };
    /// // Centres 4 apart, radii 0.5 each: clearance is 4 - 1 = 3.
    /// let sim = Simulation::new(vec![mk(0.0), mk(4.0)], vec![Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0)]).unwrap();
    /// assert!((sim.min_clearance().unwrap() - 3.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn min_clearance(&self) -> Option<f64> {
        let mut best: Option<f64> = None;
        for (i, a) in self.agents.iter().enumerate() {
            for b in &self.agents[i + 1..] {
                let c = (a.position - b.position).norm() - (a.radius + b.radius);
                best = Some(best.map_or(c, |m| m.min(c)));
            }
        }
        best
    }
}
