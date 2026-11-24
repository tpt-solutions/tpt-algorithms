# Python and WebAssembly bindings

Written for: maintainers and users who want to call the algorithms from Python or JavaScript.

Decision D7 in [decisions.md](decisions.md) puts the bindings in v0.1.

## Scope

All six algorithm crates are bound. Numeric inputs use lists (Python) or `Float64Array`s (WebAssembly). The
structured APIs use dicts and JSON.

| Crate | Python | JavaScript (WebAssembly) |
|---|---|---|
| `tpt-algo-fraud` (Isolation Forest) | `IsolationForest` (`fit`, `score`, `score_samples`, `mean_path_length`, `n_trees`), `average_path_length` | `IsolationForest.fit`, `average_path_length` |
| `tpt-algo-soc` (battery EKF) | `Ekf` (`predict`, `update`, `soc`, `estimate`, `covariance`, `model`, `ocv_voltage`, `terminal_voltage`) | `Ekf` (same methods) |
| `tpt-algo-routing` | `solve_routes(problem)` | `solve_routes(json)` |
| `tpt-algo-orderbook` | `OrderBook().apply(events)`, `snapshot()` | `OrderBook.apply(json)`, `snapshot()` |
| `tpt-algo-swarm` | `avoid(agent, neighbors, config)`, `find_path(grid)` | `avoid(json)`, `find_path(json)` |
| `tpt-algo-consensus` | `simulate_consensus(config)` (seeded simulator only, no live replicas) | `simulate_consensus(json)` |

### Conventions

- Options: Python uses keyword arguments. WebAssembly uses an options object with the same snake_case keys, and
  missing keys take the same defaults. The defaults come from the Rust crates where a `Default` exists
  (`IsolationForestConfig`). The EKF noise defaults are repeated in `tpt-algo-bindings-core` (`EKF_DEFAULTS`)
  until the SoC crate has a `Default` for `NoiseConfig`.
- Errors: Python raises `tpt_algo.TptAlgoError` (a `ValueError`). WebAssembly throws an `Error`. Both carry
  `kind`: `invalid_input`, `invalid_config`, `unsupported`, `infeasible` or `unreachable`.
- Python layout: the extension is `tpt_algo._native`. The public package `python/tpt_algo` has `__init__.py`, a
  `.pyi` stub and `py.typed`. `tpt_algo.__version__` is set there.
- WebAssembly: seeds and counts must be safe integers (below 2^53). Block ids in consensus output are hex strings.
- Routing input has optional time windows, but no service-time override.
- Order book batches are atomic.

### Parity fixtures

`internal/tpt-algo-bindings-core/fixtures/parity.json` holds the cases that every binding must reproduce.
`tests/parity.rs` (core), `crates/tpt-algo-py/tests/test_parity.py` and `crates/tpt-algo-wasm/tests/parity.rs`
each run the same cases and compare numbers within 1e-9. To regenerate expected values after an intended change,
run `PARITY_PRINT=1 cargo test -p tpt-algo-bindings-core --test parity -- --nocapture` and check the diff by hand.

## Layout

| Path | Role | Published? |
|---|---|---|
| `internal/tpt-algo-bindings-core` | All binding logic in plain Rust, including the EKF's dispatch over OCV table lengths 2 to 8 | No (`publish = false`) |
| `crates/tpt-algo-py` | pyo3 wrapper, built with maturin, module `tpt_algo` | No; PyPI package `tpt-algo` via the publish workflow |
| `crates/tpt-algo-wasm` | wasm-bindgen wrapper, built with wasm-pack | No; npm package from the publish workflow |

The two binding crates are excluded from the root workspace (`exclude` in the root `Cargo.toml`). This keeps
libpython and the `wasm32` target out of `cargo test --workspace`. Each one has its own lockfile.

## Build and test

```sh
# Shared logic and parity fixtures (part of the workspace)
cargo test -p tpt-algo-bindings-core

# WebAssembly: tests run in Node, then build the npm package for Node
cd crates/tpt-algo-wasm
wasm-pack test --node
wasm-pack build --target nodejs

# Python: build a wheel, install it into a venv, run the unittest suite
cd crates/tpt-algo-py
maturin build --release --out dist
python -m pip install --force-reinstall dist/*.whl
python -m unittest discover -s tests -v
```

The core crate is part of the workspace, so `cargo test -p tpt-algo-bindings-core` covers the shared logic.

Package metadata: `wasm-pack` writes `pkg/package.json` from `crates/tpt-algo-wasm/Cargo.toml` (description,
repository, license, keywords). The Python metadata lives in `crates/tpt-algo-py/pyproject.toml`.

CI runs both in `.github/workflows/bindings.yml`.

## Licence policy

The WebAssembly tree passes `cargo deny check licenses` with the root `deny.toml`.

The Python tree passes too, with one replacement. pyo3's build configuration depends on `target-lexicon`
0.13.6, which is licensed `Apache-2.0 WITH LLVM-exception`. The spec (section 5.1) bans Apache-2.0-only
dependencies. That crate is build-time only and does not appear in the wheel, but the policy still applies, so
`crates/tpt-algo-py/Cargo.toml` patches it with the maintainer's MIT-only drop-in
[tpt-solutions/target-lexicon-mit](https://github.com/tpt-solutions/target-lexicon-mit), pinned to commit
`297f442a451c1406252bb428702eecc5be596473`. `deny.toml` allows git sources from that one repository and no other.
The Python CI job runs `cargo deny` as well as the tests.

To move the pin, change the `rev` in `crates/tpt-algo-py/Cargo.toml` and rerun the Python job.

## Publishing

`.github/workflows/bindings-publish.yml` is manual only. With `publish = false` (the default) it builds
artefacts and publishes nothing. With `publish = true` it publishes to PyPI (trusted publishing, `pypi`
environment) and npm (`NPM_TOKEN` secret, `npm` environment). Both environments must be created by the
maintainer. Both packages pass the licence gate. Publishing is still outward-facing and irreversible, so run it only with the maintainer's approval.

Crate names and the PyPI and npm package names are not reserved yet; see the external-action list in
[decisions.md](decisions.md).
