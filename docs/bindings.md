# Python and WebAssembly bindings

Written for: maintainers and users who want to call the algorithms from Python or JavaScript.

Decision D7 in [decisions.md](decisions.md) puts the bindings in v0.1.

## Scope

The first bindings cover two crates, chosen because their APIs take plain numbers and lists:

| Crate | Python | JavaScript (WebAssembly) |
|---|---|---|
| `tpt-algo-fraud` (Isolation Forest) | `tpt_algo.IsolationForest` | `IsolationForest` |
| `tpt-algo-soc` (battery state-of-charge EKF) | `tpt_algo.Ekf` | `Ekf` |

The other crates are not bound yet. The routing, orderbook, swarm and consensus APIs use types that need a
separate design for each language.

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
# WebAssembly: tests run in Node
cd crates/tpt-algo-wasm
wasm-pack test --node

# Python: build a wheel, install it, run the unittest suite
cd crates/tpt-algo-py
maturin build --release --out dist
python -m pip install --force-reinstall dist/*.whl
python -m unittest discover -s tests -v
```

The core crate is part of the workspace, so `cargo test -p tpt-algo-bindings-core` covers the shared logic.

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
