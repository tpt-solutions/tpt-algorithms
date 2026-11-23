# tpt-algo-wasm

WebAssembly bindings (wasm-bindgen) for `tpt-algo-fraud` (Isolation Forest
anomaly scoring) and `tpt-algo-soc` (battery state-of-charge Extended Kalman
Filter).

> **Not published from this repository yet.** `publish = false`. The npm package
> is produced by the manual `bindings-publish` workflow, which needs maintainer
> approval. See [docs/bindings.md](../../docs/bindings.md).

## Classes

| Class | Purpose |
|-------|---------|
| `IsolationForest` | `fit`, `dimension`, `score` and `score_samples`. |
| `Ekf` | `new`, `predict`, `update`, `soc` and `estimate`. |

## Data layout

Data crosses the boundary as flat `Float64Array`s. Rows are concatenated, with
`dim` values per row. Errors become JavaScript `Error` objects.

The Rust `u64` seed maps to a JavaScript `BigInt`.

## Build and test

```sh
cd crates/tpt-algo-wasm
wasm-pack build --target web
wasm-pack test --node
```

The crate is excluded from the root workspace, so `cargo test --workspace` never
targets `wasm32`. It has its own lockfile.

## Usage

```js
import init, { IsolationForest, Ekf } from "./pkg/tpt_algo_wasm.js";

await init();

// Isolation Forest: rows of width 2, concatenated.
const data = new Float64Array([0, 0, 1, 0, 2, 0, 50, 50]);
const forest = IsolationForest.fit(data, 2, 100, 256, 7n);
const scores = forest.score_samples(data); // a Float64Array, values in [0, 1]

// State-of-charge EKF. The OCV table is [soc0, volts0, soc1, volts1, ...], 2 to 8 pairs.
const ekf = new Ekf(
  new Float64Array([0.0, 3.0, 0.5, 3.7, 1.0, 4.2]),
  2.0,    // capacity_ah
  0.05,   // r0
  0.02,   // r1
  1000.0, // c1
  0.8,    // soc0
  0.05,   // soc_variance
  1e-4,   // v1_variance
  1e-9,   // process_soc
  1e-6,   // process_v1
  1e-4,   // measurement
);
ekf.predict(1.0, 1.0);  // discharge current (A), dt (s)
ekf.update(3.9, 1.0);   // terminal voltage (V), current (A)
console.log(ekf.soc());
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
