// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scores a small dataset containing one obvious outlier.

#[cfg(feature = "alloc")]
use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};

#[cfg(not(feature = "alloc"))]
fn main() {}

#[cfg(feature = "alloc")]
fn main() {
    let mut data: Vec<Vec<f64>> = (0..300)
        .map(|i| vec![f64::from(i % 17) * 0.1, f64::from(i % 13) * 0.1])
        .collect();
    data.push(vec![40.0, -40.0]);

    let config = IsolationForestConfig::default().with_seed(2024);
    let forest = match IsolationForest::fit(&config, &data) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("fit failed: {e}");
            return;
        }
    };
    match forest.score_samples(&data) {
        Ok(scores) => {
            println!("typical row score: {:.3}", scores[0]);
            println!("outlier score:     {:.3}", scores[300]);
        }
        Err(e) => eprintln!("score failed: {e}"),
    }
}
