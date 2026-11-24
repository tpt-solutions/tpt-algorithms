// SPDX-License-Identifier: MIT OR Apache-2.0
//! Criterion benchmarks for `tpt-algo-soc` (extended Kalman filter).

#![allow(missing_docs)]
// criterion_group! generates an undocumented entry point.
// Bench setup aborts on invalid fixtures; a panic here is the desired failure mode.
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve};
use tpt_algo_synth_data::series::noisy_ramp;

const STEPS: [usize; 2] = [500, 2_000];

fn model() -> Model<5> {
    let ocv = OcvCurve::new([(0.0, 3.0), (0.2, 3.5), (0.5, 3.7), (0.8, 3.95), (1.0, 4.2)])
        .expect("valid OCV curve");
    let params = CellParams {
        capacity_ah: 2.0,
        r0: 0.05,
        r1: 0.02,
        c1: 1000.0,
    };
    Model::new(params, ocv).expect("valid cell model")
}

fn noise() -> NoiseConfig {
    NoiseConfig {
        process_soc: 1e-9,
        process_v1: 1e-6,
        measurement: 1e-4,
    }
}

/// Simulates a 1 s-step discharge with a noisy current ramp and returns `(current, voltage)` pairs.
fn trace(model: &Model<5>, steps: usize) -> Vec<(f64, f64)> {
    let currents = noisy_ramp(7, steps, 1.0, 1.5, 0.01);
    let (mut soc, mut v1) = (0.9, 0.0);
    currents
        .into_iter()
        .map(|i| {
            let (next_soc, next_v1) = model.step(soc, v1, i, 1.0).expect("valid step");
            soc = next_soc;
            v1 = next_v1;
            (i, model.terminal_voltage(soc, v1, i))
        })
        .collect()
}

fn bench_ekf(c: &mut Criterion) {
    let model = model();
    let mut group = c.benchmark_group("soc_ekf");
    for &steps in &STEPS {
        let samples = trace(&model, steps);
        group.throughput(Throughput::Elements(steps as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(steps),
            &samples,
            |b, samples| {
                b.iter(|| {
                    let mut filter = Filter::new(model.clone(), noise())
                        .expect("valid noise")
                        .initialize(0.5, 0.1, 1e-4)
                        .expect("valid initial state");
                    for &(current, voltage) in samples {
                        filter.predict(current, 1.0).expect("predict");
                        filter.update(voltage, current).expect("update");
                    }
                    black_box(filter.estimate().soc)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_ekf);
criterion_main!(benches);
