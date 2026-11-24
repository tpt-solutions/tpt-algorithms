// SPDX-License-Identifier: MIT OR Apache-2.0
//! Private SplitMix64 generator (Steele, Lea, Flood 2014).

pub(crate) struct SplitMix64(u64);

impl SplitMix64 {
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    pub(crate) fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `0..n` (`n > 0`), without modulo bias: draws are rejected
    /// while they fall in the final incomplete bucket of `0..2^64`.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        let n = n as u64;
        debug_assert!(n > 0, "below() requires n > 0");
        loop {
            if let Some(v) = accept(self.next_u64(), n) {
                return v;
            }
        }
    }
}

/// Maps a raw draw `x` into `0..n`, or rejects it (`None`) when `x` lies in
/// the biased low region `[0, 2^64 mod n)`. The accepted range has a size that
/// is a multiple of `n`, so every residue is equally likely.
fn accept(x: u64, n: u64) -> Option<usize> {
    // 2^64 mod n == (2^64 - n) mod n == (-n) mod n.
    let threshold = n.wrapping_neg() % n.max(1);
    if x < threshold {
        None
    } else {
        Some((x % n.max(1)) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejection_boundaries() {
        // 2^64 mod 3 == 1, so only x == 0 is rejected.
        assert_eq!(accept(0, 3), None);
        assert_eq!(accept(1, 3), Some(1));
        assert_eq!(accept(2, 3), Some(2));
        assert_eq!(accept(u64::MAX, 3), Some(0)); // 2^64 - 1 == 0 mod 3
                                                  // Powers of two never reject.
        assert_eq!(accept(0, 4), Some(0));
        assert_eq!(accept(u64::MAX, 4), Some(3));
    }

    #[test]
    fn below_stays_in_range_and_covers_all_values() {
        let mut rng = SplitMix64::new(11);
        let mut seen = [0usize; 7];
        for _ in 0..7_000 {
            let v = rng.below(7);
            assert!(v < 7);
            seen[v] += 1;
        }
        assert!(seen.iter().all(|&c| c > 800 && c < 1200), "{seen:?}");
    }

    #[test]
    fn below_one_is_always_zero() {
        let mut rng = SplitMix64::new(5);
        assert!((0..100).all(|_| rng.below(1) == 0));
    }
}
