// SPDX-License-Identifier: MIT OR Apache-2.0
//! Descriptive statistics over `f64` slices.
//!
//! All functions reject non-finite inputs with [`MathError::NonFinite`] so that
//! NaN never silently propagates into downstream estimators.

use crate::error::MathError;

fn check_finite(xs: &[f64]) -> Result<(), MathError> {
    if xs.iter().all(|x| x.is_finite()) {
        Ok(())
    } else {
        Err(MathError::NonFinite)
    }
}

/// Arithmetic mean. Requires at least one finite sample.
pub fn mean(xs: &[f64]) -> Result<f64, MathError> {
    if xs.is_empty() {
        return Err(MathError::InsufficientData);
    }
    check_finite(xs)?;
    let sum: f64 = xs.iter().sum();
    Ok(sum / xs.len() as f64)
}

/// Population variance (divisor `n`). Requires at least one finite sample.
pub fn population_variance(xs: &[f64]) -> Result<f64, MathError> {
    let m = mean(xs)?;
    let ss: f64 = xs.iter().map(|x| (x - m) * (x - m)).sum();
    Ok(ss / xs.len() as f64)
}

/// Sample variance (divisor `n - 1`). Requires at least two finite samples.
pub fn sample_variance(xs: &[f64]) -> Result<f64, MathError> {
    if xs.len() < 2 {
        return Err(MathError::InsufficientData);
    }
    let m = mean(xs)?;
    let ss: f64 = xs.iter().map(|x| (x - m) * (x - m)).sum();
    Ok(ss / (xs.len() - 1) as f64)
}

/// Root-mean-square error between `estimate` and `truth`.
pub fn rmse(estimate: &[f64], truth: &[f64]) -> Result<f64, MathError> {
    if estimate.len() != truth.len() {
        return Err(MathError::LengthMismatch {
            left: estimate.len(),
            right: truth.len(),
        });
    }
    if estimate.is_empty() {
        return Err(MathError::InsufficientData);
    }
    check_finite(estimate)?;
    check_finite(truth)?;
    let ss: f64 = estimate
        .iter()
        .zip(truth.iter())
        .map(|(e, t)| (e - t) * (e - t))
        .sum();
    Ok(libm::sqrt(ss / estimate.len() as f64))
}

/// Linear-interpolation quantile (type 7, as used by NumPy's default).
///
/// `q` must be in `[0, 1]`. The input is copied and sorted internally.
#[cfg(feature = "alloc")]
pub fn quantile(xs: &[f64], q: f64) -> Result<f64, MathError> {
    use alloc::vec::Vec;
    if xs.is_empty() {
        return Err(MathError::InsufficientData);
    }
    if !(0.0..=1.0).contains(&q) {
        return Err(MathError::InvalidQuantile);
    }
    check_finite(xs)?;
    let mut sorted: Vec<f64> = xs.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    let pos = q * (n - 1) as f64;
    // `pos` is always >= 0 here; truncation equals floor for non-negative values.
    let lo = pos as usize;
    let hi = if lo + 1 < n { lo + 1 } else { lo };
    let frac = pos - lo as f64;
    let a = sorted.get(lo).copied().ok_or(MathError::InsufficientData)?;
    let b = sorted.get(hi).copied().ok_or(MathError::InsufficientData)?;
    Ok(a + (b - a) * frac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_of_known_values() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0]), Ok(2.5));
    }

    #[test]
    fn mean_rejects_empty_and_nan() {
        assert_eq!(mean(&[]), Err(MathError::InsufficientData));
        assert_eq!(mean(&[1.0, f64::NAN]), Err(MathError::NonFinite));
    }

    #[test]
    fn variance_known_values() {
        let xs = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let p = population_variance(&xs).unwrap_or(f64::NAN);
        assert!((p - 4.0).abs() < 1e-12);
        let s = sample_variance(&xs).unwrap_or(f64::NAN);
        assert!((s - 32.0 / 7.0).abs() < 1e-12);
        assert_eq!(sample_variance(&[1.0]), Err(MathError::InsufficientData));
    }

    #[test]
    fn rmse_known_value() {
        let r = rmse(&[1.0, 2.0, 3.0], &[1.0, 2.0, 5.0]).unwrap_or(f64::NAN);
        assert!((r - (4.0f64 / 3.0).sqrt()).abs() < 1e-12);
        assert_eq!(
            rmse(&[1.0], &[1.0, 2.0]),
            Err(MathError::LengthMismatch { left: 1, right: 2 })
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn quantile_interpolates() {
        let xs = [4.0, 1.0, 3.0, 2.0];
        assert_eq!(quantile(&xs, 0.0), Ok(1.0));
        assert_eq!(quantile(&xs, 1.0), Ok(4.0));
        assert_eq!(quantile(&xs, 0.5), Ok(2.5));
        assert_eq!(quantile(&xs, 1.5), Err(MathError::InvalidQuantile));
    }
}
