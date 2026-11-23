# tpt-algo-bindings-core

Binding-independent wrappers around `tpt-algo-fraud` and `tpt-algo-soc`.

The Python and WebAssembly bindings both call this crate, so the logic is written
once. It uses plain Rust types and `String` errors, and it contains no FFI.

> **Internal crate.** `publish = false`. The published binding packages are
> `crates/tpt-algo-py` (PyPI) and `crates/tpt-algo-wasm` (npm). See
> [docs/bindings.md](../../docs/bindings.md).

## Contents

| Item | Purpose |
|------|---------|
| `Forest` | A trained Isolation Forest: `fit`, `dimension`, `score`, `score_samples`. |
| `CellSpec` | Battery cell and noise parameters, flattened for FFI callers. |
| `Ekf` | The battery state-of-charge EKF: `new`, `predict`, `update`, `estimate`. |
| `MIN_OCV_POINTS`, `MAX_OCV_POINTS` | The supported OCV table lengths, 2 to 8. |

The SoC filter is generic over the OCV table length `N`, which cannot cross an
FFI boundary directly. `Ekf` therefore dispatches at runtime over table lengths 2
to 8.

## Usage

```rust
use tpt_algo_bindings_core::Forest;

fn run() -> Result<(), String> {
    let data: Vec<Vec<f64>> = (0..100).map(|i| vec![f64::from(i), 0.0]).collect();
    let forest = Forest::fit(&data, 100, 256, 7)?;
    let _scores = forest.score_samples(&data)?;
    Ok(())
}
```

## Testing

```sh
cargo test -p tpt-algo-bindings-core
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
