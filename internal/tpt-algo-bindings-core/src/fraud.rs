// SPDX-License-Identifier: MIT OR Apache-2.0
//! Isolation Forest wrapper.

use crate::error::{BindingError, ErrorKind, Result};
use tpt_algo_fraud::{FraudError, IsolationForest, IsolationForestConfig};

/// Default hyper-parameters, read from the Rust crate so bindings cannot drift.
#[must_use]
pub fn forest_defaults() -> IsolationForestConfig {
    IsolationForestConfig::default()
}

/// Average path length `c(n)` of an unsuccessful search in a binary tree of
/// `n` external nodes (Liu, Ting and Zhou 2008).
#[must_use]
pub fn average_path_length(n: usize) -> f64 {
    tpt_algo_fraud::average_path_length(n)
}

/// A trained Isolation Forest.
pub struct Forest {
    inner: IsolationForest,
}

impl Forest {
    /// Trains a forest on `data` (rows of equal length).
    ///
    /// # Errors
    /// Invalid configuration or data (empty, ragged, non-finite).
    pub fn fit(
        data: &[Vec<f64>],
        n_trees: usize,
        subsample_size: usize,
        seed: u64,
    ) -> Result<Self> {
        let cfg = IsolationForestConfig {
            n_trees,
            subsample_size,
            seed,
        };
        let inner = IsolationForest::fit(&cfg, data).map_err(fraud_error)?;
        Ok(Self { inner })
    }

    /// Number of features per row.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    /// Number of trees in the forest.
    #[must_use]
    pub fn n_trees(&self) -> usize {
        self.inner.n_trees()
    }

    /// Anomaly score of one row, in `[0, 1]`.
    ///
    /// # Errors
    /// Wrong row length or non-finite value.
    pub fn score(&self, row: &[f64]) -> Result<f64> {
        self.inner.score(row).map_err(fraud_error)
    }

    /// Mean path length of one row over all trees.
    ///
    /// # Errors
    /// Wrong row length or non-finite value.
    pub fn mean_path_length(&self, row: &[f64]) -> Result<f64> {
        self.inner.mean_path_length(row).map_err(fraud_error)
    }

    /// Anomaly scores of many rows, in order.
    ///
    /// # Errors
    /// Any row with the wrong length or a non-finite value.
    pub fn score_samples(&self, data: &[Vec<f64>]) -> Result<Vec<f64>> {
        self.inner.score_samples(data).map_err(fraud_error)
    }
}

fn fraud_error(e: FraudError) -> BindingError {
    let kind = match e {
        FraudError::InvalidTreeCount | FraudError::InvalidSubsampleSize(_) => {
            ErrorKind::InvalidConfig
        }
        _ => ErrorKind::InvalidInput,
    };
    BindingError::new(kind, e.to_string())
}
