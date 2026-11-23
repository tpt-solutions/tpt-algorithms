// SPDX-License-Identifier: MIT OR Apache-2.0
//! Time-series generators.

use crate::gaussian::standard_normal;
use alloc::vec::Vec;
use tpt_algo_math_core::rng::SplitMix64;

/// Gaussian random walk of `n` points starting at `start`, with step standard deviation `sigma`.
/// Returns `n + 1` values including the start (or just `[start]` when `n == 0`).
pub fn random_walk(seed: u64, n: usize, start: f64, sigma: f64) -> Vec<f64> {
    let mut rng = SplitMix64::new(seed);
    let mut out = Vec::with_capacity(n + 1);
    let mut x = start;
    out.push(x);
    for _ in 0..n {
        x += sigma * standard_normal(&mut rng);
        out.push(x);
    }
    out
}

/// Discharge-like curve: a linear decay from `start` to `end` over `n` steps with
/// additive Gaussian measurement noise of standard deviation `noise_sigma`.
/// Useful as a synthetic stand-in for a battery voltage/SoC trace in CI.
pub fn noisy_ramp(seed: u64, n: usize, start: f64, end: f64, noise_sigma: f64) -> Vec<f64> {
    let mut rng = SplitMix64::new(seed);
    let denom = if n == 0 { 1.0 } else { n as f64 };
    (0..=n)
        .map(|i| {
            let base = start + (end - start) * (i as f64 / denom);
            base + noise_sigma * standard_normal(&mut rng)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reproducible_from_seed() {
        assert_eq!(random_walk(9, 50, 0.0, 1.0), random_walk(9, 50, 0.0, 1.0));
        assert_ne!(random_walk(9, 50, 0.0, 1.0), random_walk(10, 50, 0.0, 1.0));
    }

    #[test]
    fn lengths_and_start() {
        let w = random_walk(1, 10, 5.0, 0.1);
        assert_eq!(w.len(), 11);
        assert_eq!(w.first().copied(), Some(5.0));
        assert_eq!(noisy_ramp(1, 4, 1.0, 0.0, 0.0), [1.0, 0.75, 0.5, 0.25, 0.0]);
    }
}
