// SPDX-License-Identifier: MIT OR Apache-2.0
//! Isolation trees and forest.

use alloc::vec::Vec;

use crate::rng::SplitMix64;
use crate::{average_path_length, FraudError, IsolationForestConfig};

#[derive(Debug, Clone)]
enum Node {
    /// External node holding the number of training points that reached it.
    Leaf { size: usize },
    Split {
        feature: usize,
        threshold: f64,
        left: usize,
        right: usize,
    },
}

#[derive(Debug, Clone)]
struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    fn build(rows: &[&[f64]], idx: &mut [usize], max_depth: usize, rng: &mut SplitMix64) -> Self {
        let mut nodes = Vec::new();
        grow(rows, idx, 0, max_depth, rng, &mut nodes);
        Self { nodes }
    }

    fn path_length(&self, x: &[f64]) -> f64 {
        let mut at = 0usize;
        let mut depth = 0usize;
        loop {
            match self.nodes.get(at) {
                Some(Node::Split {
                    feature,
                    threshold,
                    left,
                    right,
                }) => {
                    at = if x.get(*feature).copied().unwrap_or(0.0) < *threshold {
                        *left
                    } else {
                        *right
                    };
                    depth += 1;
                }
                Some(Node::Leaf { size }) => {
                    return depth as f64 + average_path_length(*size);
                }
                None => return depth as f64,
            }
        }
    }
}

/// Recursively grows a node over the points `idx`; returns its node index.
fn grow(
    rows: &[&[f64]],
    idx: &mut [usize],
    depth: usize,
    max_depth: usize,
    rng: &mut SplitMix64,
    nodes: &mut Vec<Node>,
) -> usize {
    let me = nodes.len();
    nodes.push(Node::Leaf { size: idx.len() });
    if depth >= max_depth || idx.len() <= 1 {
        return me;
    }
    let dim = rows.first().map_or(0, |r| r.len());
    // Pick a random feature; if it is constant here, scan onward cyclically.
    let start = rng.below(dim.max(1));
    let mut chosen = None;
    for off in 0..dim {
        let f = (start + off) % dim;
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &i in idx.iter() {
            let v = rows.get(i).and_then(|r| r.get(f)).copied().unwrap_or(0.0);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        if lo < hi {
            chosen = Some((f, lo, hi));
            break;
        }
    }
    let Some((feature, lo, hi)) = chosen else {
        return me;
    };
    let mut threshold = lo + rng.next_f64() * (hi - lo);
    if threshold <= lo {
        threshold = lo + (hi - lo) * 0.5;
    }
    // Partition: values < threshold first.
    let mut split = 0usize;
    for k in 0..idx.len() {
        let v = idx
            .get(k)
            .and_then(|&i| rows.get(i))
            .and_then(|r| r.get(feature))
            .copied()
            .unwrap_or(0.0);
        if v < threshold {
            idx.swap(k, split);
            split += 1;
        }
    }
    if split == 0 || split == idx.len() {
        return me;
    }
    let (l, r) = idx.split_at_mut(split);
    let left = grow(rows, l, depth + 1, max_depth, rng, nodes);
    let right = grow(rows, r, depth + 1, max_depth, rng, nodes);
    if let Some(n) = nodes.get_mut(me) {
        *n = Node::Split {
            feature,
            threshold,
            left,
            right,
        };
    }
    me
}

/// A fitted Isolation Forest.
#[derive(Debug, Clone)]
pub struct IsolationForest {
    trees: Vec<Tree>,
    dim: usize,
    /// Effective subsample size used per tree.
    psi: usize,
}

impl IsolationForest {
    /// Fits a forest on `data` (one row per sample).
    ///
    /// # Errors
    /// Invalid configuration, empty data, zero-width rows, inconsistent row
    /// lengths or non-finite values.
    pub fn fit<R: AsRef<[f64]>>(
        config: &IsolationForestConfig,
        data: &[R],
    ) -> Result<Self, FraudError> {
        config.validate()?;
        let first = data.first().ok_or(FraudError::EmptyData)?;
        let dim = first.as_ref().len();
        if dim == 0 {
            return Err(FraudError::ZeroDimension);
        }
        let rows = validate_rows(data, dim)?;
        let n = rows.len();
        let psi = config.subsample_size.min(n);
        // ceil(log2(psi)) computed exactly with integer arithmetic.
        let max_depth = (usize::BITS - (psi - 1).leading_zeros()) as usize;
        let mut rng = SplitMix64::new(config.seed);
        let mut pool: Vec<usize> = (0..n).collect();
        let mut trees = Vec::with_capacity(config.n_trees);
        for _ in 0..config.n_trees {
            // Partial Fisher-Yates: sample psi indices without replacement.
            for k in 0..psi {
                let j = k + rng.below(n - k);
                pool.swap(k, j);
            }
            let mut sample: Vec<usize> = pool.iter().take(psi).copied().collect();
            trees.push(Tree::build(&rows, &mut sample, max_depth, &mut rng));
        }
        Ok(Self { trees, dim, psi })
    }

    /// Number of features per row.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// Number of trees.
    #[must_use]
    pub fn n_trees(&self) -> usize {
        self.trees.len()
    }

    /// Expected path length of `x` averaged over the trees.
    ///
    /// # Errors
    /// Wrong dimension or non-finite value.
    pub fn mean_path_length(&self, x: &[f64]) -> Result<f64, FraudError> {
        if x.len() != self.dim {
            return Err(FraudError::DimensionMismatch {
                row: 0,
                expected: self.dim,
                found: x.len(),
            });
        }
        if let Some(column) = x.iter().position(|v| !v.is_finite()) {
            return Err(FraudError::NonFinite { row: 0, column });
        }
        let total: f64 = self.trees.iter().map(|t| t.path_length(x)).sum();
        Ok(total / self.trees.len() as f64)
    }

    /// Anomaly score `2^(-E[h(x)] / c(psi))` in `[0, 1]`; higher is more
    /// anomalous. If only a single training row was used the score is 0.5.
    ///
    /// # Errors
    /// Wrong dimension or non-finite value.
    pub fn score(&self, x: &[f64]) -> Result<f64, FraudError> {
        let e = self.mean_path_length(x)?;
        let c = average_path_length(self.psi);
        if c <= 0.0 {
            return Ok(0.5);
        }
        Ok(libm::exp2(-e / c).clamp(0.0, 1.0))
    }

    /// Scores every row of `data`.
    ///
    /// # Errors
    /// The first row with wrong dimension or non-finite value is reported
    /// with its index.
    pub fn score_samples<R: AsRef<[f64]>>(&self, data: &[R]) -> Result<Vec<f64>, FraudError> {
        data.iter()
            .enumerate()
            .map(|(row, r)| {
                self.score(r.as_ref()).map_err(|e| match e {
                    FraudError::DimensionMismatch {
                        expected, found, ..
                    } => FraudError::DimensionMismatch {
                        row,
                        expected,
                        found,
                    },
                    FraudError::NonFinite { column, .. } => FraudError::NonFinite { row, column },
                    other => other,
                })
            })
            .collect()
    }
}

fn validate_rows<R: AsRef<[f64]>>(data: &[R], dim: usize) -> Result<Vec<&[f64]>, FraudError> {
    let mut rows = Vec::with_capacity(data.len());
    for (row, r) in data.iter().enumerate() {
        let r = r.as_ref();
        if r.len() != dim {
            return Err(FraudError::DimensionMismatch {
                row,
                expected: dim,
                found: r.len(),
            });
        }
        if let Some(column) = r.iter().position(|v| !v.is_finite()) {
            return Err(FraudError::NonFinite { row, column });
        }
        rows.push(r);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn grid() -> Vec<Vec<f64>> {
        (0..100)
            .map(|i| vec![f64::from(i % 10), f64::from(i / 10)])
            .collect()
    }

    #[test]
    fn errors() {
        let cfg = IsolationForestConfig::default();
        let empty: Vec<Vec<f64>> = vec![];
        assert_eq!(
            IsolationForest::fit(&cfg, &empty).unwrap_err(),
            FraudError::EmptyData
        );
        assert_eq!(
            IsolationForest::fit(&cfg.with_trees(0), &grid()).unwrap_err(),
            FraudError::InvalidTreeCount
        );
        assert_eq!(
            IsolationForest::fit(&cfg.with_subsample_size(1), &grid()).unwrap_err(),
            FraudError::InvalidSubsampleSize(1)
        );
        assert_eq!(
            IsolationForest::fit(&cfg, &[vec![]]).unwrap_err(),
            FraudError::ZeroDimension
        );
        assert!(matches!(
            IsolationForest::fit(&cfg, &[vec![1.0, 2.0], vec![1.0]]).unwrap_err(),
            FraudError::DimensionMismatch { row: 1, .. }
        ));
        assert_eq!(
            IsolationForest::fit(&cfg, &[vec![1.0], vec![f64::NAN]]).unwrap_err(),
            FraudError::NonFinite { row: 1, column: 0 }
        );
        let f = IsolationForest::fit(&cfg, &grid()).unwrap();
        assert!(f.score(&[1.0]).is_err());
        assert!(f.score(&[1.0, f64::INFINITY]).is_err());
    }

    #[test]
    fn single_row_and_constant_data() {
        let cfg = IsolationForestConfig::default();
        let f = IsolationForest::fit(&cfg, &[vec![1.0, 2.0]]).unwrap();
        assert_eq!(f.score(&[1.0, 2.0]).unwrap(), 0.5);
        let same = vec![vec![3.0, 3.0]; 20];
        let f = IsolationForest::fit(&cfg, &same).unwrap();
        let s = f.score(&[3.0, 3.0]).unwrap();
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn outlier_scores_higher_and_accessors() {
        let cfg = IsolationForestConfig::default().with_seed(3);
        let f = IsolationForest::fit(&cfg, &grid()).unwrap();
        assert_eq!(f.dimension(), 2);
        assert_eq!(f.n_trees(), 100);
        assert!(f.score(&[100.0, 100.0]).unwrap() > f.score(&[4.5, 4.5]).unwrap());
    }

    #[test]
    fn deterministic_and_seed_sensitive() {
        let cfg = IsolationForestConfig::default().with_trees(20);
        let a = IsolationForest::fit(&cfg.with_seed(1), &grid()).unwrap();
        let b = IsolationForest::fit(&cfg.with_seed(1), &grid()).unwrap();
        let c = IsolationForest::fit(&cfg.with_seed(2), &grid()).unwrap();
        let sa = a.score_samples(&grid()).unwrap();
        assert_eq!(sa, b.score_samples(&grid()).unwrap());
        assert_ne!(sa, c.score_samples(&grid()).unwrap());
    }

    #[test]
    fn score_samples_reports_row() {
        let f = IsolationForest::fit(&IsolationForestConfig::default(), &grid()).unwrap();
        let e = f.score_samples(&[vec![1.0, 1.0], vec![1.0]]).unwrap_err();
        assert!(matches!(e, FraudError::DimensionMismatch { row: 1, .. }));
    }
}
