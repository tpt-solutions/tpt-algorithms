# Browser demo

Two of the algorithm crates, compiled to WebAssembly, run in your browser:

- **Anomaly scoring** with an Isolation Forest (`tpt-algo-fraud`). Train on seeded synthetic data and click
  the plot to score any point.
- **Battery state of charge** with an Extended Kalman filter (`tpt-algo-soc`). Watch the filter converge
  from a wrong starting guess on a simulated discharge.

Nothing leaves the page: the data is generated in the browser from a seed.

[Open the browser demo](demo/index.html)

## Running it locally

The demo loads a WebAssembly module, so it needs to be served over HTTP rather than opened as a file.
Build the module, then serve `docs/demo`:

```sh
just demo                      # wasm-pack build into docs/demo/pkg (generated, not committed)
cd docs/demo && python -m http.server 8000
```

Without `just`, run the build directly:

```sh
wasm-pack build crates/tpt-algo-wasm --target web --out-dir ../../docs/demo/pkg --release
```

Then open <http://localhost:8000>. `just book` copies `docs/demo/` into the built book, so
`demo/index.html` works from the published book as well, once `docs/demo/pkg/` has been built.
