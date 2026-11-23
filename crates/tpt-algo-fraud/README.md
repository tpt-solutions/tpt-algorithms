# tpt-algo-fraud

Anomaly scoring with seeded Isolation Forests (Liu, Ting, Zhou, ICDM 2008;
ACM TKDD 2012). Scores are in `[0, 1]`; higher means more anomalous.

**Status:** implemented; pre-release.

## Features

- `std` (default): implies `alloc`.
- `alloc`: the `IsolationForest` type (needs `Vec`).
- `serde`: `Serialize`/`Deserialize` for `IsolationForestConfig` only.

The crate is `no_std` without `std`. Randomness is a private seeded SplitMix64,
so identical seed, configuration and data give identical scores.

## Usage

```rust
use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};

let data: Vec<Vec<f64>> = (0..100).map(|i| vec![f64::from(i), 0.0]).collect();
let cfg = IsolationForestConfig::default().with_seed(7); // 100 trees, psi 256
let forest = IsolationForest::fit(&cfg, &data).unwrap();
let scores = forest.score_samples(&data).unwrap();
```

## Validation

- Known-answer test for `c(n)`; proptest invariants (scores in `[0, 1]`,
  determinism by seed).
- Seeded synthetic cluster plus outliers with ROC AUC > 0.9 (runs in `cargo test`).
- Credit-card fraud dataset check in `tests/validation.rs` is `#[ignore]`d and
  has not been run; set `TPT_DATASET_DIR`.

## Licence

MIT OR Apache-2.0
