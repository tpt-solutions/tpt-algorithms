// SPDX-License-Identifier: MIT OR Apache-2.0
//! Forest configuration.

use crate::FraudError;

/// Isolation Forest hyper-parameters.
///
/// Defaults follow the paper: 100 trees, subsample size 256.
///
/// # Example
///
/// ```
/// use tpt_algo_fraud::IsolationForestConfig;
///
/// // Paper defaults: 100 trees, subsample 256, seed 0.
/// let cfg = IsolationForestConfig { n_trees: 50, subsample_size: 128, seed: 42 };
/// assert!(cfg.validate().is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IsolationForestConfig {
    /// Number of trees (at least 1).
    pub n_trees: usize,
    /// Subsample size per tree (at least 2). Capped at the number of rows.
    pub subsample_size: usize,
    /// Seed for the internal pseudo-random generator.
    pub seed: u64,
}

impl Default for IsolationForestConfig {
    fn default() -> Self {
        Self {
            n_trees: 100,
            subsample_size: 256,
            seed: 0,
        }
    }
}

impl IsolationForestConfig {
    /// Sets the number of trees.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_fraud::IsolationForestConfig;
    ///
    /// let cfg = IsolationForestConfig::default().with_trees(10);
    /// assert_eq!(cfg.n_trees, 10);
    /// ```
    #[must_use]
    pub fn with_trees(mut self, n_trees: usize) -> Self {
        self.n_trees = n_trees;
        self
    }

    /// Sets the subsample size.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_fraud::IsolationForestConfig;
    ///
    /// let cfg = IsolationForestConfig::default().with_subsample_size(64);
    /// assert_eq!(cfg.subsample_size, 64);
    /// ```
    #[must_use]
    pub fn with_subsample_size(mut self, subsample_size: usize) -> Self {
        self.subsample_size = subsample_size;
        self
    }

    /// Sets the seed.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_fraud::IsolationForestConfig;
    ///
    /// let cfg = IsolationForestConfig::default().with_seed(7);
    /// assert_eq!(cfg.seed, 7);
    /// ```
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Checks the hyper-parameters.
    ///
    /// # Errors
    /// [`FraudError::InvalidTreeCount`] or [`FraudError::InvalidSubsampleSize`].
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_fraud::{FraudError, IsolationForestConfig};
    ///
    /// assert!(IsolationForestConfig::default().validate().is_ok());
    /// let no_trees = IsolationForestConfig::default().with_trees(0);
    /// assert_eq!(no_trees.validate(), Err(FraudError::InvalidTreeCount));
    /// ```
    pub fn validate(&self) -> Result<(), FraudError> {
        if self.n_trees == 0 {
            return Err(FraudError::InvalidTreeCount);
        }
        if self.subsample_size < 2 {
            return Err(FraudError::InvalidSubsampleSize(self.subsample_size));
        }
        Ok(())
    }
}
