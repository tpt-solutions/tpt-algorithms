// SPDX-License-Identifier: MIT OR Apache-2.0
//! Criterion benchmarks for `tpt-algo-swarm` (velocity-obstacle simulation and grid A*).

#![allow(missing_docs)]
// criterion_group! generates an undocumented entry point.
// Bench setup aborts on invalid fixtures; a panic here is the desired failure mode.
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use tpt_algo_swarm::{find_path, Agent, AvoidanceConfig, Connectivity, Grid, Simulation, Vec2};
use tpt_algo_synth_data::routing::point_cloud;

const AGENT_COUNTS: [usize; 2] = [10, 50];
const STEPS: usize = 100;
const DT: f64 = 0.1;

/// Agents start at a seeded random cloud and head for a second seeded cloud.
fn scenario(n: usize) -> Simulation {
    let starts = point_cloud(5, n, 2, 40.0);
    let goals: Vec<Vec2> = point_cloud(6, n, 2, 40.0)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|g| Vec2::new(g[0], g[1]))
        .collect();
    let agents: Vec<Agent> = starts
        .as_chunks::<2>()
        .0
        .iter()
        .zip(&goals)
        .map(|(p, goal)| {
            let position = Vec2::new(p[0], p[1]);
            Agent {
                position,
                velocity: Vec2::ZERO,
                radius: 0.5,
                preferred_velocity: (*goal - position).clamp_norm(1.0),
                max_speed: 1.0,
            }
        })
        .collect();
    Simulation::new(agents, goals).expect("valid scenario")
}

fn bench_simulation(c: &mut Criterion) {
    let cfg = AvoidanceConfig::default();
    let mut group = c.benchmark_group("swarm_simulation");
    group.sample_size(20);
    for &n in &AGENT_COUNTS {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || scenario(n),
                |mut sim| {
                    for _ in 0..STEPS {
                        sim.step(DT, &cfg).expect("step");
                    }
                    black_box(sim.min_clearance())
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_find_path(c: &mut Criterion) {
    // 128 x 128 grid with a wall down column 64, open only at the top row.
    let mut grid = Grid::new(128, 128).expect("valid grid");
    for y in 1..128 {
        grid.set_blocked(64, y, true).expect("in bounds");
    }
    c.bench_function("swarm_find_path_128x128", |b| {
        b.iter(|| {
            black_box(find_path(
                black_box(&grid),
                (0, 127),
                (127, 127),
                Connectivity::Eight,
            ))
        });
    });
}

criterion_group!(benches, bench_simulation, bench_find_path);
criterion_main!(benches);
