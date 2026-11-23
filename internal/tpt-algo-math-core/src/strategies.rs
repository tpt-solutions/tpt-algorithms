// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared proptest strategies for generic numeric inputs (feature `proptest`).
//!
//! Domain-specific generators (orders, problems, simulations) stay in their
//! own crates; this module holds only the building blocks every crate needs.
//! All ranges are finite, so properties never see NaN or infinity unless a
//! test asks for it explicitly.

use proptest::prelude::*;

/// A finite value in `[0, 1]`, for SoC-like fractions and probabilities.
pub fn unit_interval() -> impl Strategy<Value = f64> {
    0.0f64..=1.0
}

/// A finite, strictly positive value up to `max` (exclusive of 0).
///
/// # Panics
/// If `max` is not finite and greater than `f64::MIN_POSITIVE`.
pub fn positive_finite(max: f64) -> impl Strategy<Value = f64> {
    assert!(
        max.is_finite() && max > f64::MIN_POSITIVE,
        "max must be finite and > 0"
    );
    f64::MIN_POSITIVE..=max
}

/// A finite, non-negative value up to `max`.
///
/// # Panics
/// If `max` is not finite and non-negative.
pub fn non_negative_finite(max: f64) -> impl Strategy<Value = f64> {
    assert!(max.is_finite() && max >= 0.0, "max must be finite and >= 0");
    0.0f64..=max
}

/// A seed for the deterministic generators in this workspace.
pub fn seed() -> impl Strategy<Value = u64> {
    any::<u64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn unit_interval_is_bounded(x in unit_interval()) {
            prop_assert!((0.0..=1.0).contains(&x));
        }

        #[test]
        fn positive_is_strictly_positive(x in positive_finite(10.0)) {
            prop_assert!(x > 0.0 && x <= 10.0 && x.is_finite());
        }

        #[test]
        fn non_negative_is_bounded(x in non_negative_finite(5.0)) {
            prop_assert!((0.0..=5.0).contains(&x));
        }
    }
}
