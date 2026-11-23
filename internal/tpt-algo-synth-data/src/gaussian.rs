// SPDX-License-Identifier: MIT OR Apache-2.0
//! Standard normal sampling via the Box-Muller transform.

use tpt_algo_math_core::rng::SplitMix64;

/// Draws one standard normal variate (mean 0, variance 1).
///
/// Uses only the first two uniform draws and discards the second Box-Muller output,
/// so the stream consumed per call is fixed (two `f64`s) and reproducible.
pub fn standard_normal(rng: &mut SplitMix64) -> f64 {
    // Guard against ln(0): map [0,1) to (0,1].
    let u1 = 1.0 - rng.next_f64();
    let u2 = rng.next_f64();
    libm::sqrt(-2.0 * libm::log(u1)) * libm::cos(2.0 * core::f64::consts::PI * u2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_moments_close_to_standard() {
        let mut rng = SplitMix64::new(123);
        let n = 100_000;
        let mut sum = 0.0;
        let mut sum2 = 0.0;
        for _ in 0..n {
            let x = standard_normal(&mut rng);
            sum += x;
            sum2 += x * x;
        }
        let mean = sum / n as f64;
        let var = sum2 / n as f64 - mean * mean;
        assert!(libm::fabs(mean) < 0.02, "mean {mean}");
        assert!(libm::fabs(var - 1.0) < 0.02, "var {var}");
    }
}
