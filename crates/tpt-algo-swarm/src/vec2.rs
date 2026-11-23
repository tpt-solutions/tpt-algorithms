// SPDX-License-Identifier: MIT OR Apache-2.0
//! Minimal 2-D vector.

use core::ops::{Add, Mul, Neg, Sub};

/// A 2-D vector of `f64`.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::Vec2;
///
/// let v = Vec2::new(3.0, 4.0) + Vec2::new(1.0, -4.0) * 2.0;
/// assert_eq!(v, Vec2::new(5.0, -4.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec2 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
}

impl Vec2 {
    /// The zero vector.
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    /// Creates a vector.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Vec2;
    ///
    /// const ORIGIN: Vec2 = Vec2::new(0.0, 0.0);
    /// assert_eq!(ORIGIN.norm(), 0.0);
    /// ```
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Dot product.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Vec2;
    ///
    /// assert_eq!(Vec2::new(1.0, 2.0).dot(Vec2::new(3.0, 4.0)), 11.0);
    /// ```
    #[must_use]
    pub fn dot(self, o: Vec2) -> f64 {
        self.x * o.x + self.y * o.y
    }

    /// Squared Euclidean norm.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Vec2;
    ///
    /// assert_eq!(Vec2::new(3.0, 4.0).norm_sq(), 25.0);
    /// ```
    #[must_use]
    pub fn norm_sq(self) -> f64 {
        self.dot(self)
    }

    /// Euclidean norm.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Vec2;
    ///
    /// assert_eq!(Vec2::new(3.0, 4.0).norm(), 5.0);
    /// ```
    #[must_use]
    pub fn norm(self) -> f64 {
        libm::sqrt(self.norm_sq())
    }

    /// Whether both components are finite.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Vec2;
    ///
    /// assert!(Vec2::new(1.0, 2.0).is_finite());
    /// assert!(!Vec2::new(f64::NAN, 0.0).is_finite());
    /// ```
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Scales the vector down to length `max` if it is longer.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Vec2;
    ///
    /// let v = Vec2::new(3.0, 4.0).clamp_norm(2.5);
    /// assert!((v.norm() - 2.5).abs() < 1e-12);
    /// assert_eq!(Vec2::new(1.0, 0.0).clamp_norm(2.5), Vec2::new(1.0, 0.0)); // short vectors unchanged
    /// ```
    #[must_use]
    pub fn clamp_norm(self, max: f64) -> Vec2 {
        let n = self.norm();
        if n > max && n > 0.0 {
            self * (max / n)
        } else {
            self
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f64) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}
