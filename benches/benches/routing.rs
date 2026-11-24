// SPDX-License-Identifier: MIT OR Apache-2.0
//! Criterion benchmarks for `tpt-algo-routing` (Clarke-Wright savings and full solve).

#![allow(missing_docs)]
// criterion_group! generates an undocumented entry point.
// Bench setup aborts on invalid fixtures; a panic here is the desired failure mode.
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tpt_algo_routing::{clarke_wright_savings, solve, Config, Customer, Problem};
use tpt_algo_synth_data::routing::cvrp_uniform;

const SIZES: [usize; 3] = [50, 200, 500];

/// A feasible uniform CVRP instance with `n` customers on a 100 x 100 square.
fn problem(n: usize) -> Problem {
    let instance = cvrp_uniform(42, n, 100.0, 20).expect("valid CVRP parameters");
    let customers = instance
        .customers
        .iter()
        .map(|c| Customer::new(c.x, c.y, f64::from(c.demand)))
        .collect();
    Problem::new(instance.depot, f64::from(instance.capacity), customers)
        .expect("generated instance is feasible")
}

fn bench_savings(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_clarke_wright");
    for &n in &SIZES {
        let p = problem(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &p, |b, p| {
            b.iter(|| black_box(clarke_wright_savings(black_box(p))));
        });
    }
    group.finish();
}

fn bench_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_solve");
    group.sample_size(10);
    let config = Config::default();
    for &n in &SIZES {
        let p = problem(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &p, |b, p| {
            b.iter(|| black_box(solve(black_box(p), &config)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_savings, bench_solve);
criterion_main!(benches);
