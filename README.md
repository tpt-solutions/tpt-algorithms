# tpt-algorithms

[![CI](https://github.com/tpt-solutions/tpt-algorithms/actions/workflows/ci.yml/badge.svg)](https://github.com/tpt-solutions/tpt-algorithms/actions/workflows/ci.yml)
[![Benchmarks](https://github.com/tpt-solutions/tpt-algorithms/actions/workflows/bench.yml/badge.svg)](https://github.com/tpt-solutions/tpt-algorithms/actions/workflows/bench.yml)
![Licence: MIT OR Apache-2.0](https://img.shields.io/badge/licence-MIT%20OR%20Apache--2.0-blue.svg)
![MSRV: 1.98](https://img.shields.io/badge/MSRV-1.98-orange.svg)

Permissively licensed Rust implementations of scientific, financial and systems algorithms. The
goal is MIT or Apache-2.0 alternatives to paywalled or copyleft-locked industry standards.

## Principles

- **Permissive only.** Every crate is `MIT OR Apache-2.0`. Dependencies must be MIT, BSD, ISC,
  Zlib or dual-licensed with MIT. Apache-2.0-only, GPL, LGPL, AGPL and similar are rejected by
  `cargo deny` in CI.
- **Clean-room.** Algorithms are implemented from published papers and specifications, not from
  copyleft or proprietary source. See [docs/clean-room.md](docs/clean-room.md).
- **`no_std` by default** where the domain allows it. Features: `std`, `alloc`, `serde`.
- **Validation first.** Each algorithm has a reproducible validation path against public data or a
  seeded synthetic benchmark.

## Crates

| Crate | Domain | Status |
|---|---|---|
| `tpt-algo-soc` | Battery state of charge (Extended Kalman Filter) | Implemented; dataset validation pending |
| `tpt-algo-fraud` | Anomaly scoring (Isolation Forest) | Implemented; dataset validation pending |
| `tpt-algo-orderbook` | Limit order book simulation | Implemented; LOBSTER validation pending |
| `tpt-algo-routing` | Vehicle routing (CVRP, savings + local search) | Implemented; benchmark validation pending |
| `tpt-algo-swarm` | Velocity obstacles and path planning | Implemented; scenario validation |
| `tpt-algo-consensus` | BFT consensus state machine (HotStuff family) | Implemented; signatures not verified (see its README) |

Not sure which one you need? The [cookbook](docs/cookbook.md) maps common problems to crates.

Internal, unpublished: `tpt-algo-math-core`, `tpt-algo-geometry`, `tpt-algo-synth-data`,
`tpt-algo-bindings-core`.

Python (`tpt-algo-py`) and WebAssembly (`tpt-algo-wasm`) bindings wrap the crates above. They are
built separately with maturin and wasm-pack. See [docs/bindings.md](docs/bindings.md).

## Quickstart

The crates are not on crates.io or PyPI or npm yet (see [docs/release.md](docs/release.md)), so
each quickstart builds from this repository. The Python and JavaScript steps need
`pip install maturin` and `wasm-pack` first.

### Rust

```sh
cargo run -p tpt-algo-soc --example discharge
```

To use a crate in your own project, add a git dependency until the first release:

```toml
[dependencies]
tpt-algo-soc = { git = "https://github.com/tpt-solutions/tpt-algorithms" }
```

### Python

```sh
cd crates/tpt-algo-py
maturin build --release --out dist
python -m pip install --force-reinstall dist/*.whl
```

```python
import tpt_algo

rows = [[float(i), 0.0] for i in range(100)] + [[50.0, 50.0]]
forest = tpt_algo.IsolationForest.fit(rows, n_trees=100, subsample_size=256, seed=7)
print(forest.score_samples(rows)[-1])  # the outlier scores high, in [0, 1]
```

### JavaScript (WebAssembly)

```sh
cd crates/tpt-algo-wasm
wasm-pack build --target web
```

```js
import init, { IsolationForest } from "./pkg/tpt_algo_wasm.js";

await init();
const data = new Float64Array([0, 0, 1, 0, 2, 0, 50, 50]); // rows of width 2, concatenated
const forest = IsolationForest.fit(data, 2, 100, 256, 7n);
console.log(forest.score_samples(data));
```

### CLI

```sh
cargo run -p tpt-algo-cli -- demo soc      # bundled synthetic data, no files needed
cargo install --path crates/tpt-algo-cli  # installs the `tpt` binary
tpt demo fraud
```

## Building and testing

```sh
cargo test --workspace
cargo test --workspace --no-default-features
cargo build -p tpt-algo-math-core --no-default-features --target thumbv7em-none-eabihf
```

With [`just`](https://github.com/casey/just) installed, `just check` runs format and lint checks,
and `just ci` runs the Rust part of CI. See the [justfile](justfile) for all recipes.

Dataset validation tests are `#[ignore]` and need `TPT_DATASET_DIR`. See
[docs/testing.md](docs/testing.md) and [docs/datasets.md](docs/datasets.md).

Criterion benchmarks live in [benches/](benches/README.md). Each crate README has a Benchmarks
section; the numbers come from the scheduled bench workflow.

## Documentation

- [docs/cookbook.md](docs/cookbook.md): choose your crate, and recipes for common problems
- [docs/decisions.md](docs/decisions.md): project decisions and open items
- [docs/licensing.md](docs/licensing.md)
- [docs/release.md](docs/release.md)
- [docs/maintenance.md](docs/maintenance.md)
- [docs/formal-verification.md](docs/formal-verification.md)

## Contributing

Contributions are accepted as issues only. Report bugs, ask questions and propose changes on the issue
tracker; pull requests are not accepted. Read [CONTRIBUTING.md](CONTRIBUTING.md) before you start.

## Licence

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Copyright
TPT Solutions. Contributions are licensed the same way.
