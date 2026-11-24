// SPDX-License-Identifier: MIT OR Apache-2.0
//! Criterion benchmarks for `tpt-algo-consensus` (seeded, deterministic simulator only).

#![allow(missing_docs)]
// criterion_group! generates an undocumented entry point.
// Bench setup aborts on invalid fixtures; a panic here is the desired failure mode.
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tpt_algo_consensus::sim::{SimConfig, Simulator};

const REPLICAS: [u16; 2] = [4, 7];
const TICKS: u64 = 600;

fn bench_simulate(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus_simulate");
    group.throughput(Throughput::Elements(TICKS));
    for &n in &REPLICAS {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut sim = Simulator::new(SimConfig::new(n, 2024)).expect("valid config");
                sim.run(TICKS);
                black_box(sim.report())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_simulate);
criterion_main!(benches);
