// SPDX-License-Identifier: MIT OR Apache-2.0
//! Public-dataset validation. Ignored by default; requires a local dataset.
//!
//! Expects `creditcard.csv` (ULB / Worldline "Credit Card Fraud Detection",
//! columns `Time,V1..V28,Amount,Class`) inside `TPT_DATASET_DIR`. Check the
//! dataset's licence terms before use; it is not redistributed here.
#![cfg(feature = "alloc")]

use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};

#[test]
#[ignore = "requires dataset; set TPT_DATASET_DIR"]
fn credit_card_fraud_auc() {
    let Ok(dir) = std::env::var("TPT_DATASET_DIR") else {
        eprintln!("TPT_DATASET_DIR is not set; skipping credit-card validation");
        return;
    };
    let path = std::path::Path::new(&dir).join("creditcard.csv");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut data: Vec<Vec<f64>> = Vec::new();
    let mut labels: Vec<bool> = Vec::new();
    for line in text.lines().skip(1) {
        let cells: Vec<f64> = line
            .split(',')
            .map(|c| c.trim_matches('"').parse::<f64>().unwrap())
            .collect();
        let (class, feats) = cells.split_last().unwrap();
        data.push(feats.to_vec());
        labels.push(*class > 0.5);
    }
    let cfg = IsolationForestConfig::default().with_seed(1);
    let forest = IsolationForest::fit(&cfg, &data).unwrap();
    let scores = forest.score_samples(&data).unwrap();
    // Rank-sum AUC via sorting (ties broken arbitrarily).
    let mut order: Vec<usize> = (0..scores.len()).collect();
    order.sort_by(|&a, &b| scores[a].total_cmp(&scores[b]));
    let (mut rank_sum, mut pos) = (0.0, 0.0);
    for (r, &i) in order.iter().enumerate() {
        if labels[i] {
            rank_sum += (r + 1) as f64;
            pos += 1.0;
        }
    }
    let neg = scores.len() as f64 - pos;
    let auc = (rank_sum - pos * (pos + 1.0) / 2.0) / (pos * neg);
    assert!(auc > 0.9, "AUC = {auc}");
}
