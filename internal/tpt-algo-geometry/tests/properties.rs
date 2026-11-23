// SPDX-License-Identifier: MIT OR Apache-2.0
//! Property tests for geometry invariants.

use proptest::prelude::*;
use tpt_algo_geometry::{shapes2d::point_segment_distance, Vec2};

proptest! {
    #[test]
    fn normalized_vectors_have_unit_length(x in -1e3f64..1e3, y in -1e3f64..1e3) {
        prop_assume!(x.abs() + y.abs() > 1e-6);
        let u = Vec2::new(x, y).normalized().unwrap_or_default();
        prop_assert!((u.norm() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn distance_is_symmetric_and_non_negative(
        ax in -1e3f64..1e3, ay in -1e3f64..1e3,
        bx in -1e3f64..1e3, by in -1e3f64..1e3,
    ) {
        let (a, b) = (Vec2::new(ax, ay), Vec2::new(bx, by));
        prop_assert!(a.distance(b) >= 0.0);
        prop_assert!((a.distance(b) - b.distance(a)).abs() < 1e-9);
    }

    #[test]
    fn point_to_segment_never_exceeds_endpoint_distance(
        px in -1e3f64..1e3, py in -1e3f64..1e3,
        ax in -1e3f64..1e3, ay in -1e3f64..1e3,
        bx in -1e3f64..1e3, by in -1e3f64..1e3,
    ) {
        let (p, a, b) = (Vec2::new(px, py), Vec2::new(ax, ay), Vec2::new(bx, by));
        let d = point_segment_distance(p, a, b);
        prop_assert!(d <= p.distance(a) + 1e-9);
        prop_assert!(d <= p.distance(b) + 1e-9);
    }
}
