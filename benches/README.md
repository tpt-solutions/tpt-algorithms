# tpt-algo-benches

Cross-crate [Criterion](https://crates.io/crates/criterion) benchmark suite for the
`tpt-algorithms` workspace.

> **Not published.** `publish = false`. The suite measures performance on the
> machine that runs it. Results are not portable between machines, so compare
> runs only on the same hardware and toolchain.

## What it measures

| Benchmark | Measures |
|-----------|----------|
| `splitmix64_next_u64` | Throughput of the seeded `SplitMix64` generator in `tpt-algo-math-core`. |
| `sample_variance_10k` | `sample_variance` over 10,000 values, from `tpt-algo-math-core::stats`. |

The suite is small on purpose. It covers shared primitives. Benchmarks for the
public algorithm crates should be added here as they are needed.

## Running

```sh
cargo bench -p tpt-algo-benches
```

To run one benchmark, pass its name as a filter:

```sh
cargo bench -p tpt-algo-benches -- sample_variance
```

Criterion writes its reports to `target/criterion/`.

## Layout

```text
benches/
├── Cargo.toml
├── README.md
└── benches/
    └── primitives.rs   # criterion_group! entry point
```

The `primitives` target sets `harness = false`, because Criterion supplies the
`main` function.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
