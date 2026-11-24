# tpt-algo-benches

Cross-crate [Criterion](https://crates.io/crates/criterion) benchmark suite for the
`tpt-algorithms` workspace.

> **Not published.** `publish = false`. The suite measures performance on the
> machine that runs it. Results are not portable between machines, so compare
> runs only on the same hardware and toolchain.

## What it measures

| Bench target | Benchmarks | Measures |
|--------------|------------|----------|
| `primitives` | `splitmix64_next_u64` | Throughput of the seeded `SplitMix64` generator in `tpt-algo-math-core`. |
| `primitives` | `sample_variance_10k` | `sample_variance` over 10,000 values, from `tpt-algo-math-core::stats`. |
| `soc` | `soc_ekf/500`, `soc_ekf/2000` | Extended Kalman filter predict and update over a simulated discharge. |
| `fraud` | `fraud_fit/100_trees_5k_rows`, `fraud_score/score_samples_5k_rows` | Isolation forest fit and batch scoring. |
| `orderbook` | `orderbook_apply/10000`, `orderbook_apply/100000` | Applying a mixed add and crossing-market event stream. |
| `routing` | `routing_clarke_wright/{50,200,500}`, `routing_solve/{50,200,500}` | Clarke-Wright savings and full solve at three instance sizes. |
| `swarm` | `swarm_simulation/{10,50}`, `swarm_find_path_128x128` | Velocity-obstacle avoidance steps and grid A* on a walled grid. |
| `consensus` | `consensus_simulate/{4,7}` | Seeded, deterministic replica simulation for 600 ticks. |

Workloads come from `tpt-algo-synth-data` (seeded, so every run uses the same
input). The `consensus` target does not use synth-data: the simulator generates its
own seeded workload.

The per-crate READMEs list the same benchmarks in a "Benchmarks" section with the
published numbers from the scheduled run.

## Running

```sh
cargo bench -p tpt-algo-benches
```

To run one target, or one benchmark by name filter:

```sh
cargo bench -p tpt-algo-benches --bench routing
cargo bench -p tpt-algo-benches -- sample_variance
```

For a quick, low-precision check of one group:

```sh
cargo bench -p tpt-algo-benches --bench soc -- --quick soc_ekf
```

Criterion writes its reports to `target/criterion/`.

## Scheduled CI run

`.github/workflows/bench.yml` runs the full suite weekly (Sunday 03:00 UTC) and on
manual dispatch. It runs on a GitHub-hosted `ubuntu-latest` runner and uploads
`target/criterion/` as the `criterion-<sha>` artifact, kept for 90 days. The
per-crate README tables are filled from those artifacts.

## Layout

```text
benches/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
└── benches/
    ├── primitives.rs   # shared internal primitives
    ├── soc.rs
    ├── fraud.rs
    ├── orderbook.rs
    ├── routing.rs
    ├── swarm.rs
    └── consensus.rs
```

Each target sets `harness = false`, because Criterion supplies the `main`
function.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
