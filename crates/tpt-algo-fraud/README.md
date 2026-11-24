# tpt-algo-fraud

Anomaly scoring with seeded Isolation Forests. The crate implements the
algorithm of Liu, Ting and Zhou (ICDM 2008; extended in ACM TKDD 2012).
Each tree isolates points by random axis-parallel splits. Anomalies are
isolated in fewer splits, so they receive higher scores.

Scores lie in `[0, 1]`. Values near 1 are anomalous, and values well below 0.5
are normal. The same seed, configuration and data always give the same scores.

Use it to rank transactions, sensor readings or any numeric rows by how unusual
they are. It is unsupervised: no labels are needed. It does not classify fraud
on its own. Treat the score as a triage signal for a human or a downstream rule.

## Status

Pre-release (`0.0.0`). Isolation trees, the forest and scoring are implemented
and unit and property tested. Validation against a real fraud dataset has not
been run. The crate is not yet published to crates.io; depend on it by path or
git until the first release. See the [changelog](CHANGELOG.md).

## Features

| Feature | Default | Enables |
|---------|---------|---------|
| `std`   | yes     | Implies `alloc`. Uses the standard library. |
| `alloc` | via `std` | `IsolationForest` (needs `Vec`). |
| `serde` | no      | `Serialize` / `Deserialize` for `IsolationForestConfig`. |

Without `std` the crate is `no_std`. Without `alloc` it still exposes the
configuration, the error type and `average_path_length`.

## Installation

```toml
[dependencies]
tpt-algo-fraud = { path = "../tpt-algo-fraud" }
# tpt-algo-fraud = { path = "...", default-features = false }   # no_std, no forest
```

The minimum supported Rust version is `1.98`, matching the workspace.

## Quick start

```rust
use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};

let data: Vec<Vec<f64>> = (0..100).map(|i| vec![f64::from(i), 0.0]).collect();
let cfg = IsolationForestConfig::default().with_seed(7); // 100 trees, psi 256
let forest = IsolationForest::fit(&cfg, &data).unwrap();
let scores = forest.score_samples(&data).unwrap();
```

Rows can be any type that implements `AsRef<[f64]>`, such as `Vec<f64>` or
`[f64; N]`. All rows must have the same number of features, and every value
must be finite.

## Configuration

`IsolationForestConfig::default()` uses 100 trees and a subsample size (psi) of
256. Adjust it with the builder methods:

```rust
use tpt_algo_fraud::IsolationForestConfig;

let cfg = IsolationForestConfig::default()
    .with_trees(200)           // more trees: lower variance, more time and memory
    .with_subsample_size(128)  // points per tree
    .with_seed(42);            // fixes all randomness
```

## API overview

| Item | Purpose |
|------|---------|
| `IsolationForestConfig` | Tree count, subsample size and seed (`with_trees`, `with_subsample_size`, `with_seed`). |
| `IsolationForest::fit` | Grows a forest on a slice of rows. |
| `IsolationForest::score` | Scores one row. |
| `IsolationForest::score_samples` | Scores many rows. |
| `IsolationForest` accessors | `n_trees` and `dimension`. |
| `IsolationForest::mean_path_length` | Mean path length of one row across the trees. |
| `average_path_length` | The normalising constant `c(n)`, available without `alloc`. |
| `FraudError` | Returned for invalid configuration, shape or non-finite input. |

## Examples

```sh
cargo run -p tpt-algo-fraud --example basic
```

## Validation status

- Known-answer test for `c(n)`. Proptest invariants: scores lie in `[0, 1]`,
  and scoring is deterministic for a fixed seed.
- A seeded synthetic cluster with outliers reaches ROC AUC above 0.9. This runs
  in `cargo test`.
- The credit-card fraud dataset check in `tests/validation.rs` is `#[ignore]`d
  and has not been run. Set `TPT_DATASET_DIR` to run it.

## Limitations

- Isolation Forest is unsupervised. Its scores measure how unusual a row is,
  not whether it is fraud. Calibrating a decision threshold needs labelled data.
- Scores depend on the feature scaling and on which features are included.
  Scale and clean the inputs first.
- Training is not incremental. Refit the forest when the data distribution
  changes.
- Randomness is a private SplitMix64 generator. It is reproducible, but it is not
  a cryptographic generator and must not be used for security purposes.

## References

- F. T. Liu, K. M. Ting, Z.-H. Zhou, *Isolation Forest*, ICDM 2008.
- F. T. Liu, K. M. Ting, Z.-H. Zhou, *Isolation-Based Anomaly Detection*, ACM
  Transactions on Knowledge Discovery from Data, 2012.

## Benchmarks

Criterion benchmarks for this crate live in [`benches/benches/fraud.rs`](../../benches/benches/fraud.rs)
(the `tpt-algo-benches` package). Inputs come from `tpt-algo-synth-data` where the crate takes
generated data.

Numbers are produced by the scheduled bench workflow
(`.github/workflows/bench.yml`, weekly plus on demand) on a GitHub-hosted `ubuntu-latest`
runner, and uploaded as the `criterion-<sha>` artifact. They are not measured on a developer
machine and are not portable between runners.

| Benchmark | Measures | Median time |
|-----------|----------|-------------|
| `fraud_fit/100_trees_5k_rows` | Fit 100 trees on 5,000 synthetic log-amount rows | pending first scheduled run |
| `fraud_score/score_samples_5k_rows` | Score 5,000 rows with a fitted 100-tree forest | pending first scheduled run |

To run locally: `cargo bench -p tpt-algo-benches --bench fraud`. Criterion writes reports to
`target/criterion/`.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
