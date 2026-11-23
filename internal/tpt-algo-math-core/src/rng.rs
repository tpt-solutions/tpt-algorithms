// SPDX-License-Identifier: MIT OR Apache-2.0
//! Deterministic, seeded pseudo-random number generation.
//!
//! Convention for the whole workspace: every stochastic routine takes an explicit
//! seed and produces identical output for identical seeds on every platform. This
//! generator is SplitMix64 (Steele, Lea, Flood 2014). It is not cryptographically
//! secure and must not be used for security purposes.

/// SplitMix64 generator. `no_std`, no allocation, no global state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a generator from a seed. Any seed (including zero) is valid.
    pub const fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    /// Returns the next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns a uniform `f64` in `[0, 1)` with 53 bits of precision.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Returns a uniform `f64` in `[low, high)`.
    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.next_f64()
    }

    /// Returns a uniform integer in `[0, bound)` using rejection sampling (unbiased).
    /// Returns `None` when `bound == 0`.
    pub fn next_below(&mut self, bound: u64) -> Option<u64> {
        if bound == 0 {
            return None;
        }
        let zone = u64::MAX - (u64::MAX % bound);
        loop {
            let v = self.next_u64();
            if v < zone {
                return Some(v % bound);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        assert_ne!(SplitMix64::new(1).next_u64(), SplitMix64::new(2).next_u64());
    }

    #[test]
    fn known_first_output_for_seed_zero() {
        // Reference value for SplitMix64 with seed 0.
        assert_eq!(SplitMix64::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn next_f64_in_unit_interval() {
        let mut r = SplitMix64::new(7);
        for _ in 0..10_000 {
            let x = r.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn next_below_handles_zero_bound() {
        let mut r = SplitMix64::new(7);
        assert_eq!(r.next_below(0), None);
        assert!(r.next_below(10).is_some_and(|v| v < 10));
    }
}
