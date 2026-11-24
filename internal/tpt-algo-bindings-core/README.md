# tpt-algo-bindings-core

Binding-independent wrappers around the `tpt-algo` crates: Isolation Forest, the
battery state-of-charge EKF, routing, the order book, swarm avoidance and grid
paths, and the consensus simulator.

The Python and WebAssembly bindings both call this crate, so the logic is written
once. Failures are a `BindingError` with an `ErrorKind`, and the structured inputs
(routing, order book, swarm, consensus) are JSON strings. The crate has no FFI.

> **Internal crate.** `publish = false`. The published binding packages are
> `crates/tpt-algo-py` (PyPI) and `crates/tpt-algo-wasm` (npm). See
> [docs/bindings.md](../../docs/bindings.md).

## Contents

| Item | Purpose |
|------|---------|
| `fraud::Forest` | Isolation Forest: `fit`, `dimension`, `n_trees`, `score`, `score_samples`, `mean_path_length`. |
| `fraud::forest_defaults`, `soc::EKF_DEFAULTS` | Default hyper-parameters the bindings use. |
| `fraud::average_path_length` | Average path length c(n). |
| `soc::Ekf`, `soc::CellSpec` | Battery EKF: `new`, `predict`, `update`, `estimate`, `covariance`, `cell_params`, `ocv_voltage`, `terminal_voltage`. |
| `routing::solve_json` | Routing: JSON problem in, JSON solution out. |
| `orderbook::Book` | Order book: `apply_json` (atomic batch), `snapshot_json`. |
| `swarm::avoid_json`, `swarm::find_path_json` | Velocity obstacles and grid shortest paths, as JSON. |
| `consensus::simulate_json` | Seeded consensus simulator, as JSON. No live replicas. |
| `ErrorKind`, `BindingError` | `invalid_input`, `invalid_config`, `unsupported`, `infeasible`, `unreachable`. |

The SoC filter is generic over the OCV table length `N`, which cannot cross an
FFI boundary directly. `Ekf` therefore dispatches at runtime over table lengths 2
to 8.

## Shared fixtures

`fixtures/parity.json` holds the cases that every binding must reproduce. The
Rust test `tests/parity.rs` runs them here. The Python and WebAssembly suites run
the same file. Set `PARITY_PRINT=1` with `cargo test -p tpt-algo-bindings-core
--test parity -- --nocapture` to print the actual output of each case.

## Usage

```rust
use tpt_algo_bindings_core::{Book, Forest};

fn run() -> Result<(), tpt_algo_bindings_core::BindingError> {
    let data: Vec<Vec<f64>> = (0..100).map(|i| vec![f64::from(i), 0.0]).collect();
    let forest = Forest::fit(&data, 100, 256, 7)?;
    let _scores = forest.score_samples(&data)?;

    let mut book = Book::new();
    let _out = book.apply_json(r#"[{"type":"add","id":1,"side":"bid","price":99,"qty":10}]"#)?;
    Ok(())
}
```

## Testing

```sh
cargo test -p tpt-algo-bindings-core
cargo clippy -p tpt-algo-bindings-core --all-targets -- -D warnings
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
