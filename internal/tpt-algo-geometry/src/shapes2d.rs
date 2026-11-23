// SPDX-License-Identifier: MIT OR Apache-2.0
//! 2D spatial primitives: segments, bounding boxes, circles.

use crate::vec::Vec2;

/// Axis-aligned bounding box in 2D.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aabb2 {
    /// Minimum corner.
    pub min: Vec2,
    /// Maximum corner.
    pub max: Vec2,
}

impl Aabb2 {
    /// Smallest box containing all points. Returns `None` for an empty iterator.
    pub fn from_points<I: IntoIterator<Item = Vec2>>(points: I) -> Option<Self> {
        let mut it = points.into_iter();
        let first = it.next()?;
        let mut bb = Aabb2 {
            min: first,
            max: first,
        };
        for p in it {
            bb.min.x = bb.min.x.min(p.x);
            bb.min.y = bb.min.y.min(p.y);
            bb.max.x = bb.max.x.max(p.x);
            bb.max.y = bb.max.y.max(p.y);
        }
        Some(bb)
    }

    /// True when `p` lies inside or on the boundary.
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    /// True when the two boxes overlap (touching counts as overlap).
    pub fn intersects(&self, o: &Aabb2) -> bool {
        self.min.x <= o.max.x
            && o.min.x <= self.max.x
            && self.min.y <= o.max.y
            && o.min.y <= self.max.y
    }
}

/// Intersection point of segments `p1p2` and `q1q2`, if they properly cross or touch at
/// a single point. Parallel or collinear segments return `None`.
pub fn segment_intersection(p1: Vec2, p2: Vec2, q1: Vec2, q2: Vec2) -> Option<Vec2> {
    let r = p2 - p1;
    let s = q2 - q1;
    let denom = r.cross(s);
    if denom == 0.0 {
        return None;
    }
    let qp = q1 - p1;
    let t = qp.cross(s) / denom;
    let u = qp.cross(r) / denom;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(p1 + r * t)
    } else {
        None
    }
}

/// Distance from point `p` to the closed segment `ab`.
pub fn point_segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b - a;
    let len2 = ab.norm_squared();
    if len2 == 0.0 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// True when two circles (centre, radius) overlap or touch.
pub fn circles_overlap(c1: Vec2, r1: f64, c2: Vec2, r2: f64) -> bool {
    c1.distance(c2) <= r1 + r2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_from_points_and_contains() {
        let pts = [
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, -1.0),
            Vec2::new(1.0, 3.0),
        ];
        let bb = Aabb2::from_points(pts).unwrap_or(Aabb2 {
            min: Vec2::default(),
            max: Vec2::default(),
        });
        assert_eq!(bb.min, Vec2::new(0.0, -1.0));
        assert_eq!(bb.max, Vec2::new(2.0, 3.0));
        assert!(bb.contains(Vec2::new(1.0, 1.0)));
        assert!(!bb.contains(Vec2::new(3.0, 1.0)));
        assert!(Aabb2::from_points(core::iter::empty()).is_none());
    }

    #[test]
    fn crossing_segments_intersect_at_expected_point() {
        let p = segment_intersection(
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
            Vec2::new(2.0, 0.0),
        );
        assert_eq!(p, Some(Vec2::new(1.0, 1.0)));
    }

    #[test]
    fn parallel_segments_do_not_intersect() {
        let p = segment_intersection(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
        );
        assert_eq!(p, None);
    }

    #[test]
    fn point_to_segment_distance_cases() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        assert_eq!(point_segment_distance(Vec2::new(5.0, 3.0), a, b), 3.0);
        assert_eq!(point_segment_distance(Vec2::new(-4.0, 3.0), a, b), 5.0);
    }

    #[test]
    fn circles_touching_overlap() {
        assert!(circles_overlap(
            Vec2::new(0.0, 0.0),
            1.0,
            Vec2::new(2.0, 0.0),
            1.0
        ));
        assert!(!circles_overlap(
            Vec2::new(0.0, 0.0),
            1.0,
            Vec2::new(3.0, 0.0),
            1.0
        ));
    }
}
