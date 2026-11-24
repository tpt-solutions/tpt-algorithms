// SPDX-License-Identifier: MIT OR Apache-2.0
//! Criterion benchmarks for `tpt-algo-fraud` (isolation forest).

#![allow(missing_docs)]
// criterion_group! generates an undocumented entry point.
// Bench setup aborts on invalid fixtures; a panic here is the desired failure mode.
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};
use tpt_algo_synth_data::transactions::transactions;

const ROWS: usize = 5_000;

/// Log-amount feature vectors from synthetic transactions with about 2% anomalies.
fn rows(seed: u64, n: usize) -> Vec<Vec<f64>> {
    transactions(seed, n, 3.0, 0.5, 0.02, 10.0)
        .into_iter()
        .map(|t| vec![t.amount.ln()])
        .collect()
}

fn config() -> IsolationForestConfig {
    IsolationForestConfig::default()
        .with_trees(100)
        .with_seed(7)
}

fn bench_fit(c: &mut Criterion) {
    let train = rows(1, ROWS);
    let mut group = c.benchmark_group("fraud_fit");
    group.throughput(Throughput::Elements(ROWS as u64));
    group.bench_function("100_trees_5k_rows", |b| {
        b.iter(|| black_box(IsolationForest::fit(&config(), black_box(&train))));
    });
    group.finish();
}

fn bench_score(c: &mut Criterion) {
    let train = rows(1, ROWS);
    let eval = rows(2, ROWS);
    let forest = IsolationForest::fit(&config(), &train).expect("fit");
    let mut group = c.benchmark_group("fraud_score");
    group.throughput(Throughput::Elements(ROWS as u64));
    group.bench_function("score_samples_5k_rows", |b| {
        b.iter(|| black_box(forest.score_samples(black_box(&eval))));
    });
    group.finish();
}

criterion_group!(benches, bench_fit, bench_score);
criterion_main!(benches);
