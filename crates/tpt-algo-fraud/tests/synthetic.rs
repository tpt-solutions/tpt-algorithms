// SPDX-License-Identifier: MIT OR Apache-2.0
//! Seeded synthetic benchmark and property tests.
#![cfg(feature = "alloc")]

use proptest::prelude::*;
use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn normal(&mut self) -> f64 {
        let u1 = self.next().max(1e-12);
        let u2 = self.next();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// ROC AUC via pairwise comparison (Mann-Whitney) with tie handling.
fn auc(scores: &[f64], labels: &[bool]) -> f64 {
    let mut pos = 0.0;
    let mut neg = 0.0;
    let mut wins = 0.0;
    for (i, &si) in scores.iter().enumerate() {
        if !labels[i] {
            neg += 1.0;
            continue;
        }
        pos += 1.0;
        for (j, &sj) in scores.iter().enumerate() {
            if labels[j] {
                continue;
            }
            if si > sj {
                wins += 1.0;
            } else if si == sj {
                wins += 0.5;
            }
        }
    }
    wins / (pos * neg)
}

#[test]
fn cluster_with_injected_outliers_auc() {
    let mut rng = Lcg(42);
    let mut data = Vec::new();
    let mut labels = Vec::new();
    for _ in 0..1000 {
        data.push((0..5).map(|_| rng.normal()).collect::<Vec<_>>());
        labels.push(false);
    }
    for _ in 0..30 {
        // Uniform scatter in a wide box, far from the unit-variance cluster.
        data.push((0..5).map(|_| (rng.next() - 0.5) * 24.0).collect());
        labels.push(true);
    }
    let cfg = IsolationForestConfig::default().with_seed(11);
    let forest = IsolationForest::fit(&cfg, &data).unwrap();
    let scores = forest.score_samples(&data).unwrap();
    let a = auc(&scores, &labels);
    assert!(a > 0.9, "AUC = {a}");
}

proptest! {
    #[test]
    fn scores_in_unit_interval_and_deterministic(
        rows in prop::collection::vec(prop::collection::vec(-1e6f64..1e6, 3), 1..60),
        seed in any::<u64>(),
        trees in 1usize..10,
        psi in 2usize..80,
    ) {
        let cfg = IsolationForestConfig::default()
            .with_seed(seed).with_trees(trees).with_subsample_size(psi);
        let a = IsolationForest::fit(&cfg, &rows).unwrap();
        let b = IsolationForest::fit(&cfg, &rows).unwrap();
        let sa = a.score_samples(&rows).unwrap();
        let sb = b.score_samples(&rows).unwrap();
        prop_assert_eq!(&sa, &sb);
        for s in sa {
            prop_assert!((0.0..=1.0).contains(&s));
        }
    }
}
