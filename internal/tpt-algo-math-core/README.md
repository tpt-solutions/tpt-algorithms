# tpt-algo-math-core

Shared numerics for the `tpt-algorithms` workspace.

> **Internal crate.** `publish = false`. It is never released to crates.io, and
> the public algorithm crates do not depend on it (see `docs/decisions.md`, D5).

## Contents

| Module | Provides |
|--------|----------|
| `stats` | `mean`, `population_variance`, `sample_variance`, `rmse`, and `quantile` (needs `alloc`). |
| `tolerance` | `relative_error`, `within_relative` and `within_absolute` for validation tests. |
| `fixed` | `Fixed`, a Q32.32 fixed-point number with checked `from_int`, `from_f64`, `to_f64`, `checked_add`, `checked_sub`, `checked_mul` and `checked_div`. |
| `rng` | `SplitMix64`: `next_u64`, `next_f64`, `uniform` and `next_below`. Seeded and deterministic. |
| `error` | `MathError`, the shared error type. |
| `contract` | `requires!` and `ensures!` debug-assertion helpers. |
| `strategies` | Shared proptest strategies (feature `proptest`). |

## Features

- `std` (default): implies `alloc`.
- `alloc`: enables `stats::quantile` and other allocating helpers.
- `serde`: serde support for public data types.
- `proptest`: the `strategies` module. Test-only, and needs `std`.

Build without `std` to check `no_std` compatibility:

```sh
cargo build -p tpt-algo-math-core --no-default-features
```

## Usage

```rust
use tpt_algo_math_core::{rng::SplitMix64, stats, tolerance};

let mut rng = SplitMix64::new(42);
let xs: Vec<f64> = (0..1000).map(|_| rng.next_f64()).collect();
let m = stats::mean(&xs).unwrap();
assert!(tolerance::within_absolute(m, 0.5, 0.05));
```

## Who uses it

- `tpt-algo-synth-data` depends on it for the random number generator and
  statistics.
- `tpt-algo-soc` uses the `proptest` feature in its dev-dependencies.

## Testing

```sh
cargo test -p tpt-algo-math-core
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
