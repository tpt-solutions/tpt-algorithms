// SPDX-License-Identifier: MIT OR Apache-2.0
//! Synthetic labelled transactions for anomaly-detection tests.

use crate::gaussian::standard_normal;
use alloc::vec::Vec;
use tpt_algo_math_core::rng::SplitMix64;

/// A transaction amount with a ground-truth anomaly label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transaction {
    /// Amount (positive).
    pub amount: f64,
    /// True if this transaction was generated as an outlier.
    pub is_anomaly: bool,
}

/// Generates `n` transactions: normal amounts are log-normal with parameters `(mu, sigma)`
/// in log-space; each transaction is an anomaly with probability `anomaly_rate`, in which
/// case its amount is multiplied by `anomaly_scale`.
pub fn transactions(
    seed: u64,
    n: usize,
    mu: f64,
    sigma: f64,
    anomaly_rate: f64,
    anomaly_scale: f64,
) -> Vec<Transaction> {
    let mut rng = SplitMix64::new(seed);
    (0..n)
        .map(|_| {
            let base = libm::exp(mu + sigma * standard_normal(&mut rng));
            let is_anomaly = rng.next_f64() < anomaly_rate;
            let amount = if is_anomaly {
                base * anomaly_scale
            } else {
                base
            };
            Transaction { amount, is_anomaly }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reproducible_and_positive() {
        let a = transactions(8, 1000, 3.0, 0.5, 0.05, 10.0);
        assert_eq!(a, transactions(8, 1000, 3.0, 0.5, 0.05, 10.0));
        assert!(a.iter().all(|t| t.amount > 0.0));
    }

    #[test]
    fn zero_anomaly_rate_has_no_labels() {
        let a = transactions(3, 500, 0.0, 1.0, 0.0, 5.0);
        assert!(a.iter().all(|t| !t.is_anomaly));
    }
}
