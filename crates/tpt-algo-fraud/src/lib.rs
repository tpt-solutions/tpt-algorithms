// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anomaly scoring with seeded Isolation Forests.
//!
//! Implements the Isolation Forest of Liu, Ting and Zhou (ICDM 2008; extended
//! in ACM TKDD 2012). Each tree is grown on a random subsample by repeatedly
//! choosing a random feature and a uniform random split value between the
//! feature's minimum and maximum in the node. Anomalies are isolated in fewer
//! splits, so a short average path length means a high anomaly score.
//!
//! Input is a slice of rows, where each row is anything that implements
//! `AsRef<[f64]>` (for example `Vec<f64>` or `[f64; N]`). All rows must have
//! the same number of features and every value must be finite.
//!
//! Scores lie in `[0, 1]`; values near 1 are anomalous, values well below 0.5
//! are normal. Identical seed, configuration and data give identical scores.
//!
//! The forest needs the `alloc` feature; the configuration, error type and
//! [`average_path_length`] are available in `no_std` without it.
//!
//! ```
//! # #[cfg(feature = "alloc")] {
//! use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};
//!
//! let mut data: Vec<Vec<f64>> = (0..200)
//!     .map(|i| vec![f64::from(i % 10) * 0.1, f64::from(i % 7) * 0.1])
//!     .collect();
//! data.push(vec![50.0, 50.0]);
//! let config = IsolationForestConfig::default().with_seed(7);
//! let forest = IsolationForest::fit(&config, &data).unwrap();
//! let scores = forest.score_samples(&data).unwrap();
//! assert!(scores[200] > scores[0]);
//! # }
//! ```
//!
//! Serde support (feature `serde`) covers [`IsolationForestConfig`] only.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod config;
mod error;
#[cfg(feature = "alloc")]
mod forest;
#[cfg(feature = "alloc")]
mod rng;

pub use config::IsolationForestConfig;
pub use error::FraudError;
#[cfg(feature = "alloc")]
pub use forest::IsolationForest;

/// Euler-Mascheroni constant used in the harmonic-number approximation.
const EULER_GAMMA: f64 = core::f64::consts::EULER_GAMMA;

/// Average path length `c(n)` of an unsuccessful search in a binary search
/// tree built on `n` points (Liu et al. 2008, Eq. 1):
/// `c(n) = 2 H(n-1) - 2 (n-1) / n`, with `H(i) ~ ln(i) + gamma`.
///
/// `c(0) = c(1) = 0` and `c(2) = 1`.
///
/// # Example
///
/// ```
/// use tpt_algo_fraud::average_path_length;
///
/// // c(1) = 0 and c(2) = 1, from the Isolation Forest paper.
/// assert_eq!(average_path_length(1), 0.0);
/// assert!((average_path_length(2) - 1.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn average_path_length(n: usize) -> f64 {
    match n {
        0 | 1 => 0.0,
        2 => 1.0,
        _ => {
            let nf = n as f64;
            let h = libm::log(nf - 1.0) + EULER_GAMMA;
            2.0 * h - 2.0 * (nf - 1.0) / nf
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_known_values() {
        assert_eq!(average_path_length(0), 0.0);
        assert_eq!(average_path_length(1), 0.0);
        assert_eq!(average_path_length(2), 1.0);
        // c(256) ~ 10.2448 from the formula with H(255) ~ ln(255) + gamma.
        assert!((average_path_length(256) - 10.2448).abs() < 1e-3);
        assert!(average_path_length(3) > 1.0);
    }
}
