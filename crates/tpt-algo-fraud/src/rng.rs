// SPDX-License-Identifier: MIT OR Apache-2.0
//! Private SplitMix64 generator (Steele, Lea, Flood 2014).
#![cfg_attr(not(feature = "alloc"), allow(dead_code))]

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

    /// Uniform in `0..n` (`n > 0`); modulo bias is negligible for small `n`.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % (n as u64)) as usize
    }
}
