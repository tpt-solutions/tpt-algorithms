// SPDX-License-Identifier: MIT OR Apache-2.0
//! Property tests for math-core invariants.

use proptest::prelude::*;
use tpt_algo_math_core::{fixed::Fixed, rng::SplitMix64, stats};

proptest! {
    #[test]
    fn mean_lies_between_min_and_max(xs in prop::collection::vec(-1e6f64..1e6, 1..200)) {
        let m = stats::mean(&xs).unwrap_or(f64::NAN);
        let lo = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        prop_assert!(m >= lo - 1e-9 && m <= hi + 1e-9);
    }

    #[test]
    fn variance_is_non_negative(xs in prop::collection::vec(-1e3f64..1e3, 2..100)) {
        let v = stats::population_variance(&xs).unwrap_or(f64::NAN);
        prop_assert!(v >= 0.0);
    }

    #[test]
    fn rmse_is_symmetric_and_zero_on_identity(
        xs in prop::collection::vec(-1e3f64..1e3, 1..100),
        ys in prop::collection::vec(-1e3f64..1e3, 1..100),
    ) {
        let n = xs.len().min(ys.len());
        let (a, b) = (&xs[..n], &ys[..n]);
        let ab = stats::rmse(a, b).unwrap_or(f64::NAN);
        let ba = stats::rmse(b, a).unwrap_or(f64::NAN);
        prop_assert!((ab - ba).abs() < 1e-9);
        prop_assert_eq!(stats::rmse(a, a).unwrap_or(f64::NAN), 0.0);
    }

    #[test]
    fn fixed_point_roundtrip_error_is_bounded(v in -1e6f64..1e6) {
        let f = Fixed::from_f64(v).unwrap_or(Fixed::ZERO);
        prop_assert!((f.to_f64() - v).abs() <= 1.0 / (1u64 << 32) as f64);
    }

    #[test]
    fn rng_is_reproducible(seed in any::<u64>()) {
        let mut a = SplitMix64::new(seed);
        let mut b = SplitMix64::new(seed);
        for _ in 0..32 {
            prop_assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
