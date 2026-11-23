// SPDX-License-Identifier: MIT OR Apache-2.0
//! Benchmarks for shared internal primitives. Algorithm-level benchmarks are per crate.

#![allow(missing_docs)] // criterion_group! generates an undocumented entry point.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tpt_algo_math_core::{rng::SplitMix64, stats};

fn bench_rng(c: &mut Criterion) {
    c.bench_function("splitmix64_next_u64", |b| {
        let mut rng = SplitMix64::new(1);
        b.iter(|| black_box(rng.next_u64()));
    });
}

fn bench_stats(c: &mut Criterion) {
    let data = tpt_algo_synth_data::series::random_walk(7, 10_000, 0.0, 1.0);
    c.bench_function("sample_variance_10k", |b| {
        b.iter(|| black_box(stats::sample_variance(black_box(&data))));
    });
}

criterion_group!(benches, bench_rng, bench_stats);
criterion_main!(benches);
