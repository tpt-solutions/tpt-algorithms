// SPDX-License-Identifier: MIT OR Apache-2.0
//! Synchronous multi-agent simulation.

use alloc::vec::Vec;

use crate::{avoid_among, Agent, AvoidanceConfig, SwarmError, Vec2};

/// A set of agents with goals, advanced in lock-step.
///
/// Each step every agent heads for its goal at up to `max_speed` (slowing so
/// it does not overshoot), then replaces that preferred velocity by the
/// result of [`avoid_among`] computed from the state at the start of the step.
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
    #[must_use]
    pub fn all_at_goals(&self, tol: f64) -> bool {
        self.agents
            .iter()
            .zip(&self.goals)
            .all(|(a, g)| (*g - a.position).norm() <= tol)
    }

    /// Minimum over pairs of `distance - (r_i + r_j)`; negative means
    /// overlap. `None` with fewer than two agents.
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
