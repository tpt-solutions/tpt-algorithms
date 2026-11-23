// SPDX-License-Identifier: MIT OR Apache-2.0
//! Velocity obstacles and the avoidance step.

use crate::{SwarmError, Vec2};

/// A disc-shaped agent.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{Agent, Vec2};
///
/// // Two agents heading straight at each other.
/// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(1.0, 0.0),
///     radius: 0.5, preferred_velocity: Vec2::new(1.0, 0.0), max_speed: 1.0 };
/// let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(-1.0, 0.0), ..a };
/// assert!(a.validate().is_ok() && b.validate().is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Agent {
    /// Centre position.
    pub position: Vec2,
    /// Current velocity.
    pub velocity: Vec2,
    /// Disc radius (> 0).
    pub radius: f64,
    /// Velocity the agent would like to have.
    pub preferred_velocity: Vec2,
    /// Speed limit (> 0).
    pub max_speed: f64,
}

impl Agent {
    /// Validates that all fields are finite and radius / max speed positive.
    ///
    /// # Errors
    /// [`SwarmError::InvalidAgent`] on violation.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, SwarmError, Vec2};
    ///
    /// let bad = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(0.0, 0.0),
    ///     radius: 0.0, preferred_velocity: Vec2::new(0.0, 0.0), max_speed: 1.0 };
    /// assert!(matches!(bad.validate(), Err(SwarmError::InvalidAgent(_))));
    /// ```
    pub fn validate(&self) -> Result<(), SwarmError> {
        if !(self.position.is_finite()
            && self.velocity.is_finite()
            && self.preferred_velocity.is_finite()
            && self.radius.is_finite()
            && self.max_speed.is_finite())
        {
            return Err(SwarmError::InvalidAgent("non-finite value"));
        }
        if self.radius <= 0.0 {
            return Err(SwarmError::InvalidAgent("radius must be positive"));
        }
        if self.max_speed <= 0.0 {
            return Err(SwarmError::InvalidAgent("max_speed must be positive"));
        }
        Ok(())
    }
}

/// Which agent takes responsibility for avoiding a collision.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{AvoidanceConfig, Responsibility};
///
/// // Full: the neighbour is assumed to keep its velocity. Reciprocal: both agents share the work.
/// let cfg = AvoidanceConfig { responsibility: Responsibility::Full, ..Default::default() };
/// assert_eq!(cfg.responsibility, Responsibility::Full);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Responsibility {
    /// Classic VO (Fiorini & Shiller): the neighbour is assumed to keep its
    /// velocity, so the agent avoids the whole obstacle itself.
    Full,
    /// Reciprocal VO (van den Berg et al. 2008): the obstacle apex is shifted
    /// to the average of both velocities, so each agent takes half the effort.
    /// This is the sampled RVO variant, not ORCA's half-plane linear program.
    Reciprocal,
}

/// Parameters of the avoidance step.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::AvoidanceConfig;
///
/// let cfg = AvoidanceConfig::default();
/// assert!(cfg.horizon > 0.0);
/// assert!(cfg.rings >= 1 && cfg.angles >= 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AvoidanceConfig {
    /// Time horizon tau (> 0): collisions later than this are ignored.
    pub horizon: f64,
    /// Extra clearance added to the sum of radii (>= 0).
    pub safety_margin: f64,
    /// Number of speed rings in the candidate grid (>= 1).
    pub rings: usize,
    /// Number of directions per ring (>= 1).
    pub angles: usize,
    /// Responsibility model.
    pub responsibility: Responsibility,
}

impl Default for AvoidanceConfig {
    fn default() -> Self {
        Self {
            horizon: 4.0,
            safety_margin: 0.05,
            rings: 5,
            angles: 32,
            responsibility: Responsibility::Reciprocal,
        }
    }
}

impl AvoidanceConfig {
    fn validate(&self) -> Result<(), SwarmError> {
        if !(self.horizon.is_finite() && self.horizon > 0.0) {
            return Err(SwarmError::InvalidConfig("horizon must be positive"));
        }
        if !(self.safety_margin.is_finite() && self.safety_margin >= 0.0) {
            return Err(SwarmError::InvalidConfig("safety_margin must be >= 0"));
        }
        if self.rings == 0 || self.angles == 0 {
            return Err(SwarmError::InvalidConfig("rings and angles must be >= 1"));
        }
        Ok(())
    }
}

/// Truncated-cone velocity obstacle VO^tau(A|B) in relative-velocity space.
///
/// A relative velocity `v = vA - vB` lies in the obstacle iff the disc of
/// radius `R = rA + rB` around B's relative position `p = pB - pA` is hit
/// within `[0, tau]` when A moves with `v`, i.e. iff
/// `min_{t in [0,tau]} |p - v t| < R`. The test is exact (not sampled): it
/// solves the quadratic `|p - v t|^2 = R^2` for the first contact time.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{Agent, Vec2};
/// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(1.0, 0.0),
///     radius: 0.5, preferred_velocity: Vec2::new(1.0, 0.0), max_speed: 1.0 };
/// let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(0.0, 0.0), ..a };
/// use tpt_algo_swarm::VelocityObstacle;
///
/// // B is 4 units ahead and A closes in at speed 1 relative to B.
/// let vo = VelocityObstacle::new(&a, &b, 10.0, 0.0);
/// assert!(vo.contains(Vec2::new(1.0, 0.0)));
/// assert!(!vo.contains(Vec2::new(0.0, 1.0)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VelocityObstacle {
    /// Relative position `pB - pA`.
    pub rel_position: Vec2,
    /// Combined radius including the margin.
    pub radius: f64,
    /// Safety margin included in `radius`.
    pub margin: f64,
    /// Time horizon.
    pub horizon: f64,
}

impl VelocityObstacle {
    /// Builds VO(A|B) with a time horizon and extra margin added to the radii.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, Vec2};
    /// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(1.0, 0.0),
    ///     radius: 0.5, preferred_velocity: Vec2::new(1.0, 0.0), max_speed: 1.0 };
    /// let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(0.0, 0.0), ..a };
    /// use tpt_algo_swarm::VelocityObstacle;
    ///
    /// // Radii add up to 1.0, so the obstacle is a disc of radius 1 around B.
    /// let vo = VelocityObstacle::new(&a, &b, 10.0, 0.0);
    /// assert_eq!(vo.radius, 1.0);
    /// ```
    #[must_use]
    pub fn new(a: &Agent, b: &Agent, horizon: f64, margin: f64) -> Self {
        Self {
            rel_position: b.position - a.position,
            radius: a.radius + b.radius + margin,
            margin,
            horizon,
        }
    }

    /// Time of first contact for relative velocity `v`, if within the horizon.
    /// If the discs already overlap, returns `Some(0.0)` for
    /// velocities that move them closer and `None` for separating ones.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, Vec2};
    /// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(1.0, 0.0),
    ///     radius: 0.5, preferred_velocity: Vec2::new(1.0, 0.0), max_speed: 1.0 };
    /// let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(0.0, 0.0), ..a };
    /// use tpt_algo_swarm::VelocityObstacle;
    ///
    /// let vo = VelocityObstacle::new(&a, &b, 10.0, 0.0);
    /// // Closing at speed 1 over distance 4 with contact radius 1: contact at t = 3.
    /// assert!((vo.time_to_collision(Vec2::new(1.0, 0.0)).unwrap() - 3.0).abs() < 1e-9);
    /// assert!(vo.time_to_collision(Vec2::new(0.0, 1.0)).is_none()); // moving sideways
    /// ```
    #[must_use]
    pub fn time_to_collision(&self, v: Vec2) -> Option<f64> {
        let p = self.rel_position;
        let mut radius = self.radius;
        let mut c = p.norm_sq() - radius * radius;
        if c < 0.0 && self.margin > 0.0 {
            // Inside the margin shell only: judge against the true radii so
            // that agents can still slide past each other.
            radius -= self.margin;
            c = p.norm_sq() - radius * radius;
        }
        if c < 0.0 {
            // Truly overlapping: only velocities that close the gap count.
            return if p.dot(v) > 0.0 { Some(0.0) } else { None };
        }
        let a = v.norm_sq();
        if a <= 1e-18 {
            return None;
        }
        let b = p.dot(v);
        let disc = b * b - a * c;
        if disc <= 0.0 || b <= 0.0 {
            return None;
        }
        let t = (b - libm::sqrt(disc)) / a;
        if t <= self.horizon {
            Some(t.max(0.0))
        } else {
            None
        }
    }

    /// Whether relative velocity `v` lies inside the obstacle.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Agent, Vec2};
    /// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(1.0, 0.0),
    ///     radius: 0.5, preferred_velocity: Vec2::new(1.0, 0.0), max_speed: 1.0 };
    /// let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(0.0, 0.0), ..a };
    /// use tpt_algo_swarm::VelocityObstacle;
    ///
    /// let vo = VelocityObstacle::new(&a, &b, 10.0, 0.0);
    /// assert!(vo.contains(Vec2::new(2.0, 0.0)));
    /// assert!(!vo.contains(Vec2::new(-1.0, 0.0)));
    /// ```
    #[must_use]
    pub fn contains(&self, v: Vec2) -> bool {
        self.time_to_collision(v).is_some()
    }
}

/// Chooses a new velocity for `agents[index]` considering all other agents.
///
/// See [`avoid`] for the selection rule.
///
/// # Errors
/// [`SwarmError::IndexOutOfRange`], plus validation errors.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{Agent, Vec2};
/// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(1.0, 0.0),
///     radius: 0.5, preferred_velocity: Vec2::new(1.0, 0.0), max_speed: 1.0 };
/// let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(0.0, 0.0), ..a };
/// use tpt_algo_swarm::{avoid_among, AvoidanceConfig};
///
/// let agents = [a, b];
/// let v = avoid_among(&agents, 0, &AvoidanceConfig::default()).unwrap();
/// assert!(v.norm() <= a.max_speed + 1e-12);
/// ```
pub fn avoid_among(
    agents: &[Agent],
    index: usize,
    cfg: &AvoidanceConfig,
) -> Result<Vec2, SwarmError> {
    let me = agents.get(index).ok_or(SwarmError::IndexOutOfRange)?;
    for a in agents {
        a.validate()?;
    }
    let others = agents
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .map(|(_, a)| a);
    avoid_iter(me, others, cfg)
}

/// Chooses a new velocity for `agent` given `neighbors`.
///
/// Candidates are the clamped preferred velocity and a deterministic polar
/// grid over the max-speed disc (`rings` x `angles`); the zero velocity is
/// added only if no grid candidate is collision free.
/// If some candidate is outside every VO, the one closest to the preferred
/// velocity is returned. Otherwise the candidate with the latest earliest
/// collision time wins (ties: closest to preferred). The result never
/// exceeds `max_speed`.
///
/// # Errors
/// [`SwarmError::InvalidAgent`] / [`SwarmError::InvalidConfig`].
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{Agent, Vec2};
/// let a = Agent { position: Vec2::new(0.0, 0.0), velocity: Vec2::new(1.0, 0.0),
///     radius: 0.5, preferred_velocity: Vec2::new(1.0, 0.0), max_speed: 1.0 };
/// let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(0.0, 0.0), ..a };
/// use tpt_algo_swarm::{avoid, AvoidanceConfig};
///
/// // Agent a considers b as a neighbour and returns its new velocity.
/// let v = avoid(&a, &[b], &AvoidanceConfig::default()).unwrap();
/// assert!(v.is_finite());
/// ```
pub fn avoid(
    agent: &Agent,
    neighbors: &[Agent],
    cfg: &AvoidanceConfig,
) -> Result<Vec2, SwarmError> {
    for n in neighbors {
        n.validate()?;
    }
    avoid_iter(agent, neighbors.iter(), cfg)
}

const HYSTERESIS: f64 = 0.5;
/// Small cost for deviating to the left of the preferred velocity (a
/// right-hand rule) so that symmetric encounters resolve consistently.
const SIDE_BIAS: f64 = 0.1;

fn avoid_iter<'a, I>(agent: &Agent, others: I, cfg: &AvoidanceConfig) -> Result<Vec2, SwarmError>
where
    I: Iterator<Item = &'a Agent> + Clone,
{
    cfg.validate()?;
    agent.validate()?;
    let pref = agent.preferred_velocity.clamp_norm(agent.max_speed);

    // Earliest collision time (in own-motion time) over all neighbours.
    let ttc = |cand: Vec2| -> f64 {
        let mut best = f64::INFINITY;
        for b in others.clone() {
            let vo = VelocityObstacle::new(agent, b, cfg.horizon, cfg.safety_margin);
            let rel = match cfg.responsibility {
                Responsibility::Full => cand - b.velocity,
                // Candidate v' is in RVO iff 2v' - vA - vB is in VO.
                Responsibility::Reciprocal => cand * 2.0 - agent.velocity - b.velocity,
            };
            if let Some(t) = vo.time_to_collision(rel) {
                if t < best {
                    best = t;
                }
            }
        }
        best
    };

    let best_t = core::cell::Cell::new(ttc(pref));
    if best_t.get().is_infinite() {
        return Ok(pref);
    }
    let mut best_v = pref;
    let mut best_d = HYSTERESIS * (pref - agent.velocity).norm_sq();
    let mut consider = |cand: Vec2| {
        let t = ttc(cand);
        // Distance to the preferred velocity, plus a small hysteresis term
        // towards the current velocity that damps left/right oscillation.
        let d = (cand - pref).norm_sq()
            + HYSTERESIS * (cand - agent.velocity).norm_sq()
            + SIDE_BIAS * (pref.x * cand.y - pref.y * cand.x);
        let better = match (t.is_infinite(), best_t.get().is_infinite()) {
            (true, true) => d < best_d,
            (true, false) => true,
            (false, true) => false,
            (false, false) => t > best_t.get() || (t == best_t.get() && d < best_d),
        };
        if better {
            best_v = cand;
            best_t.set(t);
            best_d = d;
        }
    };
    for r in 1..=cfg.rings {
        let speed = agent.max_speed * (r as f64) / (cfg.rings as f64);
        for k in 0..cfg.angles {
            let th = core::f64::consts::TAU * (k as f64) / (cfg.angles as f64);
            consider(Vec2::new(speed * libm::cos(th), speed * libm::sin(th)));
        }
    }
    // Stopping is only a fallback: preferring it among free candidates makes
    // two agents blocking each other's goal path freeze in place.
    if best_t.get().is_finite() {
        consider(Vec2::ZERO);
    }
    Ok(best_v.clamp_norm(agent.max_speed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ag(x: f64, y: f64, vx: f64, vy: f64) -> Agent {
        Agent {
            position: Vec2::new(x, y),
            velocity: Vec2::new(vx, vy),
            radius: 0.5,
            preferred_velocity: Vec2::new(vx, vy),
            max_speed: 2.0,
        }
    }

    #[test]
    fn vo_known_answers() {
        let a = ag(0.0, 0.0, 1.0, 0.0);
        let b = ag(10.0, 0.0, 0.0, 0.0);
        let vo = VelocityObstacle::new(&a, &b, 20.0, 0.0);
        // Head-on at speed 1: contact when distance 10 - 1 = 9 is covered.
        let t = vo.time_to_collision(Vec2::new(1.0, 0.0)).unwrap();
        assert!((t - 9.0).abs() < 1e-12);
        // Moving away / sideways fast / stopped: no collision.
        assert!(!vo.contains(Vec2::new(-1.0, 0.0)));
        assert!(!vo.contains(Vec2::new(0.0, 1.0)));
        assert!(!vo.contains(Vec2::ZERO));
        // Too slow to arrive within the horizon.
        let short = VelocityObstacle::new(&a, &b, 5.0, 0.0);
        assert!(!short.contains(Vec2::new(1.0, 0.0)));
        // Tangent boundary: offset exactly R is not a collision.
        let tang = VelocityObstacle::new(&a, &ag(10.0, 1.0, 0.0, 0.0), 20.0, 0.0);
        assert!(!tang.contains(Vec2::new(1.0, 0.0)));
        // Overlap: approaching is immediate collision, separating is allowed.
        let ov = VelocityObstacle::new(&a, &ag(0.5, 0.0, 0.0, 0.0), 1.0, 0.0);
        assert_eq!(ov.time_to_collision(Vec2::new(1.0, 0.0)), Some(0.0));
        assert_eq!(ov.time_to_collision(Vec2::new(-1.0, 0.0)), None);
    }

    #[test]
    fn free_path_keeps_preferred() {
        let a = ag(0.0, 0.0, 1.0, 0.0);
        let far = ag(0.0, 50.0, 0.0, 0.0);
        let v = avoid(&a, &[far], &AvoidanceConfig::default()).unwrap();
        assert_eq!(v, a.preferred_velocity);
    }

    #[test]
    fn result_avoids_vo_and_respects_speed() {
        let a = ag(0.0, 0.0, 1.0, 0.0);
        let b = ag(4.0, 0.0, -1.0, 0.0);
        for resp in [Responsibility::Full, Responsibility::Reciprocal] {
            let cfg = AvoidanceConfig {
                responsibility: resp,
                ..AvoidanceConfig::default()
            };
            let v = avoid(&a, &[b], &cfg).unwrap();
            assert!(v.norm() <= a.max_speed + 1e-12);
            assert!(v != a.preferred_velocity);
            let vo = VelocityObstacle::new(&a, &b, cfg.horizon, cfg.safety_margin);
            let rel = match resp {
                Responsibility::Full => v - b.velocity,
                Responsibility::Reciprocal => v * 2.0 - a.velocity - b.velocity,
            };
            assert!(!vo.contains(rel));
        }
    }

    #[test]
    fn clamps_preferred_to_max_speed() {
        let mut a = ag(0.0, 0.0, 0.0, 0.0);
        a.preferred_velocity = Vec2::new(10.0, 0.0);
        let v = avoid(&a, &[], &AvoidanceConfig::default()).unwrap();
        assert!((v.norm() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn validation_errors() {
        let a = ag(0.0, 0.0, 0.0, 0.0);
        let bad = AvoidanceConfig {
            horizon: 0.0,
            ..AvoidanceConfig::default()
        };
        assert!(matches!(
            avoid(&a, &[], &bad),
            Err(SwarmError::InvalidConfig(_))
        ));
        let mut nan = a;
        nan.position = Vec2::new(f64::NAN, 0.0);
        assert!(matches!(
            avoid(&nan, &[], &AvoidanceConfig::default()),
            Err(SwarmError::InvalidAgent(_))
        ));
        assert_eq!(
            avoid_among(&[a], 3, &AvoidanceConfig::default()),
            Err(SwarmError::IndexOutOfRange)
        );
    }
}
