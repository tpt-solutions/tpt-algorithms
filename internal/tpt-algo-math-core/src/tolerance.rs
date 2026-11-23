// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tolerance helpers used by validation tests.
//!
//! Validation against published ground truth (for example "VWAP within 0.01 %")
//! goes through these functions so that every crate reports error the same way.

/// Relative error `|actual - expected| / |expected|`.
///
/// When `expected == 0.0` the absolute error is returned instead, so the result is
/// finite for finite inputs.
pub fn relative_error(actual: f64, expected: f64) -> f64 {
    let abs_err = (actual - expected).abs();
    if expected == 0.0 {
        abs_err
    } else {
        abs_err / expected.abs()
    }
}

/// True when `actual` is within relative tolerance `tol` of `expected`.
/// Non-finite inputs return `false`.
pub fn within_relative(actual: f64, expected: f64, tol: f64) -> bool {
    if !(actual.is_finite() && expected.is_finite() && tol.is_finite()) {
        return false;
    }
    relative_error(actual, expected) <= tol
}

/// True when `|actual - expected| <= tol`. Non-finite inputs return `false`.
pub fn within_absolute(actual: f64, expected: f64, tol: f64) -> bool {
    actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_error_basic() {
        assert!((relative_error(1.1, 1.0) - 0.1).abs() < 1e-12);
        assert_eq!(relative_error(0.5, 0.0), 0.5);
    }

    #[test]
    fn within_checks_reject_non_finite() {
        assert!(!within_relative(f64::NAN, 1.0, 0.1));
        assert!(!within_absolute(1.0, f64::INFINITY, 0.1));
        assert!(within_relative(1.0001, 1.0, 0.001));
    }
}
