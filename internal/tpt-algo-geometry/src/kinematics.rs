// SPDX-License-Identifier: MIT OR Apache-2.0
//! Kinematics helpers for constant-acceleration motion and time-to-collision.

use crate::error::GeometryError;
use crate::vec::Vec2;

/// Position after `t` seconds under constant acceleration: `x0 + v0 t + a t² / 2`.
pub fn constant_accel_position(x0: Vec2, v0: Vec2, a: Vec2, t: f64) -> Result<Vec2, GeometryError> {
    if t < 0.0 {
        return Err(GeometryError::NegativeTime);
    }
    if !(x0.is_finite() && v0.is_finite() && a.is_finite() && t.is_finite()) {
        return Err(GeometryError::NonFinite);
    }
    Ok(x0 + v0 * t + a * (0.5 * t * t))
}

/// Earliest non-negative time at which two discs of combined radius `radius_sum` meet,
/// given relative position `rel_pos` (other minus self) and relative velocity `rel_vel`.
///
/// Returns `Ok(Some(t))` with `t >= 0`, or `Ok(None)` if they never come within
/// `radius_sum` (or are already separating from outside the radius).
pub fn time_to_collision(rel_pos: Vec2, rel_vel: Vec2, radius_sum: f64) -> Result<Option<f64>, GeometryError> {
    if !(rel_pos.is_finite() && rel_vel.is_finite() && radius_sum.is_finite()) {
        return Err(GeometryError::NonFinite);
    }
    if radius_sum < 0.0 {
        return Err(GeometryError::NegativeTime);
    }
    let c = rel_pos.norm_squared() - radius_sum * radius_sum;
    if c <= 0.0 {
        // Already overlapping at t = 0.
        return Ok(Some(0.0));
    }
    let a = rel_vel.norm_squared();
    let b = 2.0 * rel_pos.dot(rel_vel);
    if a == 0.0 {
        return Ok(None);
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 || b >= 0.0 {
        // No real root, or both roots are in the past (moving apart).
        return Ok(None);
    }
    let t = (-b - libm::sqrt(disc)) / (2.0 * a);
    Ok(if t >= 0.0 { Some(t) } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_under_gravity_like_accel() {
        let p = constant_accel_position(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 2.0), 2.0);
        assert_eq!(p, Ok(Vec2::new(2.0, 4.0)));
        assert_eq!(
            constant_accel_position(Vec2::default(), Vec2::default(), Vec2::default(), -1.0),
            Err(GeometryError::NegativeTime)
        );
    }

    #[test]
    fn head_on_collision_time() {
        // Gap 10 m closing at 2 m/s with combined radius 2 m: meet when gap = 2, i.e. t = 4.
        let t = time_to_collision(Vec2::new(10.0, 0.0), Vec2::new(-2.0, 0.0), 2.0);
        assert_eq!(t, Ok(Some(4.0)));
    }

    #[test]
    fn separating_or_parallel_never_collide() {
        assert_eq!(time_to_collision(Vec2::new(10.0, 0.0), Vec2::new(2.0, 0.0), 2.0), Ok(None));
        assert_eq!(time_to_collision(Vec2::new(10.0, 0.0), Vec2::new(0.0, 0.0), 2.0), Ok(None));
        assert_eq!(time_to_collision(Vec2::new(10.0, 5.0), Vec2::new(-1.0, 0.0), 2.0), Ok(None));
    }
}
