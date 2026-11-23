// SPDX-License-Identifier: MIT OR Apache-2.0
//! Q32.32 signed fixed-point arithmetic.
//!
//! Operations are integer-only and fully deterministic across platforms. All
//! arithmetic is checked: overflow and division by zero return
//! [`MathError::Overflow`] instead of panicking or wrapping.

use crate::error::MathError;

/// Number of fractional bits.
pub const FRAC_BITS: u32 = 32;
const ONE_RAW: i64 = 1i64 << FRAC_BITS;

/// A Q32.32 fixed-point number stored in an `i64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fixed(i64);

impl Fixed {
    /// Zero.
    pub const ZERO: Fixed = Fixed(0);
    /// One.
    pub const ONE: Fixed = Fixed(ONE_RAW);

    /// Builds a value from the raw Q32.32 representation.
    pub const fn from_raw(raw: i64) -> Self {
        Fixed(raw)
    }

    /// Returns the raw Q32.32 representation.
    pub const fn to_raw(self) -> i64 {
        self.0
    }

    /// Converts an integer, failing if it does not fit in the integer part.
    pub fn from_int(v: i64) -> Result<Self, MathError> {
        v.checked_mul(ONE_RAW).map(Fixed).ok_or(MathError::Overflow)
    }

    /// Converts a finite `f64`, rounding half away from zero. Fails on NaN, infinity or overflow.
    pub fn from_f64(v: f64) -> Result<Self, MathError> {
        if !v.is_finite() {
            return Err(MathError::NonFinite);
        }
        let scaled = v * ONE_RAW as f64;
        if scaled >= i64::MAX as f64 || scaled < i64::MIN as f64 {
            return Err(MathError::Overflow);
        }
        Ok(Fixed(round_half_away(scaled)))
    }

    /// Converts to `f64`.
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / ONE_RAW as f64
    }

    /// Checked addition.
    pub fn checked_add(self, rhs: Self) -> Result<Self, MathError> {
        self.0.checked_add(rhs.0).map(Fixed).ok_or(MathError::Overflow)
    }

    /// Checked subtraction.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, MathError> {
        self.0.checked_sub(rhs.0).map(Fixed).ok_or(MathError::Overflow)
    }

    /// Checked multiplication. The discarded fraction is truncated toward negative infinity.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, MathError> {
        let wide = (self.0 as i128) * (rhs.0 as i128);
        let shifted = wide >> FRAC_BITS;
        i64::try_from(shifted).map(Fixed).map_err(|_| MathError::Overflow)
    }

    /// Checked division. Fails on division by zero or overflow.
    pub fn checked_div(self, rhs: Self) -> Result<Self, MathError> {
        if rhs.0 == 0 {
            return Err(MathError::Overflow);
        }
        let wide = ((self.0 as i128) << FRAC_BITS) / (rhs.0 as i128);
        i64::try_from(wide).map(Fixed).map_err(|_| MathError::Overflow)
    }
}

fn round_half_away(x: f64) -> i64 {
    if x >= 0.0 {
        (x + 0.5) as i64
    } else {
        (x - 0.5) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_plus_one_is_two() {
        assert_eq!(Fixed::ONE.checked_add(Fixed::ONE), Fixed::from_int(2));
    }

    #[test]
    fn mul_and_div_known_values() {
        let a = Fixed::from_f64(3.5).unwrap_or(Fixed::ZERO);
        let b = Fixed::from_f64(2.0).unwrap_or(Fixed::ZERO);
        assert_eq!(a.checked_mul(b).map(Fixed::to_f64), Ok(7.0));
        assert_eq!(a.checked_div(b).map(Fixed::to_f64), Ok(1.75));
    }

    #[test]
    fn div_by_zero_is_error() {
        assert_eq!(Fixed::ONE.checked_div(Fixed::ZERO), Err(MathError::Overflow));
    }

    #[test]
    fn overflow_is_error_not_panic() {
        assert_eq!(Fixed::from_int(i64::MAX), Err(MathError::Overflow));
        assert_eq!(Fixed::from_f64(f64::NAN), Err(MathError::NonFinite));
        assert_eq!(Fixed::from_f64(1e300), Err(MathError::Overflow));
        assert_eq!(Fixed::from_raw(i64::MAX).checked_add(Fixed::ONE), Err(MathError::Overflow));
    }
}
