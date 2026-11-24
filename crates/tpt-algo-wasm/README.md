# tpt-algo-wasm

WebAssembly bindings (wasm-bindgen) for the `tpt-algo` crates: Isolation Forest
anomaly scores, battery state-of-charge Extended Kalman filter, capacitated
vehicle routing, a limit order book, swarm collision avoidance and grid paths,
and a seeded consensus simulator.

> **Not published from this repository yet.** `publish = false`. The npm package
> is produced by the manual `bindings-publish` workflow, which needs maintainer
> approval. See [docs/bindings.md](../../docs/bindings.md).

## API

| Export | Purpose |
|--------|---------|
| `IsolationForest.fit(data, dim, options?)` | Train a forest. Options: `n_trees` (100), `subsample_size` (256), `seed` (0). Also `dimension`, `n_trees`, `score`, `score_samples`, `mean_path_length`. |
| `average_path_length(n)` | Average path length c(n). |
| `new Ekf(ocv, options)` | EKF. Options: `capacity_ah`, `r0`, `r1`, `c1`, `soc0`, `soc_variance` (required); `v1_variance` (1e-4), `process_soc` (1e-9), `process_v1` (1e-6), `measurement` (1e-4). Also `predict`, `update`, `soc`, `estimate()`, `covariance()`, `model()`, `ocv_voltage`, `terminal_voltage`. |
| `new OrderBook()` | `apply(json)` returns JSON `{trades, stats}`; `snapshot()` returns JSON. |
| `solve_routes(json)`, `avoid(json)`, `find_path(json)`, `simulate_consensus(json)` | JSON in, JSON out. |

Option names match the Python keyword arguments. Missing options take the same
defaults as Python. Any option that is present must be a number, and integer
options must be non-negative integers.

## Data layout

Numeric arrays cross the boundary as flat `Float64Array`s. Rows are concatenated,
with `dim` values per row. Seeds and counts must stay within the safe integer
range of JavaScript numbers (2^53 - 1), so pass them as numbers, not `BigInt`.

Errors are thrown as `Error` objects with a `kind` property:
`invalid_input`, `invalid_config`, `unsupported`, `infeasible` or `unreachable`.

## Build and test

```sh
cd crates/tpt-algo-wasm
wasm-pack build --target nodejs    # or --target web
wasm-pack test --node
```

The crate is excluded from the root workspace, so `cargo test --workspace` never
targets `wasm32`. It has its own lockfile.

`tests/parity.rs` runs the fixtures shared with the Rust core and the Python
bindings (`internal/tpt-algo-bindings-core/fixtures/parity.json`).

## Usage

```js
const m = require("./pkg/tpt_algo_wasm.js"); // or an ES import of the web build

// Isolation Forest: rows of width 2, concatenated.
const data = new Float64Array([0, 0, 1, 0, 2, 0, 50, 50]);
const forest = m.IsolationForest.fit(data, 2, { n_trees: 100, subsample_size: 256, seed: 7 });
const scores = forest.score_samples(data); // Float64Array, values in [0, 1]

// State-of-charge EKF. The OCV table is [soc0, volts0, soc1, volts1, ...].
const ekf = new m.Ekf(new Float64Array([0.0, 3.0, 0.5, 3.7, 1.0, 4.2]), {
  capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0,
  soc0: 0.8, soc_variance: 0.05,
});
ekf.predict(1.0, 1.0); // discharge current (A), dt (s)
ekf.update(3.9, 1.0);  // terminal voltage (V), current (A)
console.log(ekf.soc);

// Routing takes and returns JSON.
const sol = JSON.parse(m.solve_routes(JSON.stringify({
  depot: [0, 0], capacity: 10,
  customers: [{ x: 1, y: 0, demand: 5 }, { x: -1, y: 0, demand: 5 }],
})));
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
