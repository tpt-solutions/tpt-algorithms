// SPDX-License-Identifier: MIT OR Apache-2.0
//! Plain `f64` 2D and 3D vectors.

use crate::error::GeometryError;
use core::ops::{Add, Mul, Neg, Sub};

/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec2 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
}

/// A 3D vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec3 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
}

impl Vec2 {
    /// Creates a vector.
    pub const fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    /// Dot product.
    pub fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y
    }

    /// Scalar 2D cross product (z component of the 3D cross product).
    pub fn cross(self, o: Self) -> f64 {
        self.x * o.y - self.y * o.x
    }

    /// Squared Euclidean length.
    pub fn norm_squared(self) -> f64 {
        self.dot(self)
    }

    /// Euclidean length.
    pub fn norm(self) -> f64 {
        libm::sqrt(self.norm_squared())
    }

    /// Euclidean distance to `o`.
    pub fn distance(self, o: Self) -> f64 {
        (self - o).norm()
    }

    /// Unit vector in the same direction. Fails on a zero or non-finite vector.
    pub fn normalized(self) -> Result<Self, GeometryError> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        let n = self.norm();
        if n == 0.0 {
            Err(GeometryError::Degenerate)
        } else {
            Ok(self * (1.0 / n))
        }
    }

    /// Returns true when both components are finite.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Vec3 {
    /// Creates a vector.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    /// Dot product.
    pub fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Cross product.
    pub fn cross(self, o: Self) -> Self {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    /// Euclidean length.
    pub fn norm(self) -> f64 {
        libm::sqrt(self.dot(self))
    }

    /// Euclidean distance to `o`.
    pub fn distance(self, o: Self) -> f64 {
        (self - o).norm()
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

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_cross_norm_2d() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.norm(), 5.0);
        assert_eq!(a.dot(Vec2::new(1.0, 0.0)), 3.0);
        assert_eq!(Vec2::new(1.0, 0.0).cross(Vec2::new(0.0, 1.0)), 1.0);
    }

    #[test]
    fn normalize_rejects_zero() {
        assert_eq!(
            Vec2::new(0.0, 0.0).normalized(),
            Err(GeometryError::Degenerate)
        );
        assert_eq!(
            Vec2::new(f64::NAN, 0.0).normalized(),
            Err(GeometryError::NonFinite)
        );
        let u = Vec2::new(0.0, 2.0).normalized().unwrap_or_default();
        assert_eq!(u, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn cross_3d_basis() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(x.cross(y), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(Vec3::new(1.0, 2.0, 2.0).norm(), 3.0);
    }
}
