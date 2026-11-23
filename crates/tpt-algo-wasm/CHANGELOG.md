# Changelog

All notable changes to the `tpt-algo-wasm` WebAssembly bindings are documented in
this file.

This crate is not published from this repository yet (`publish = false`). The npm
package is built by the manual `bindings-publish` workflow.

## [Unreleased]

### Added

- `IsolationForest`: `fit`, `dimension`, `score` and `score_samples`, on flat
  `Float64Array` data.
- `Ekf`: `new`, `predict`, `update`, `soc` and `estimate`.
- Errors become JavaScript `Error` objects.
- Node-based tests via `wasm-pack test --node`.
