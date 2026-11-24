# Changelog

All notable changes to the `tpt-algo-wasm` WebAssembly bindings are documented in
this file.

This crate is not published from this repository yet (`publish = false`). The npm
package is built by the manual `bindings-publish` workflow.

## [Unreleased]

### Added
- Browser demo (`docs/demo`, built with `just demo`): Isolation Forest scoring and an EKF state-of-charge run.

- `IsolationForest.fit(data, dim, options)` with an options object (`n_trees`,
  `subsample_size`, `seed`) whose defaults match Python. Also `n_trees`,
  `mean_path_length`; `average_path_length(n)` is a free function.
- `Ekf` takes an options object (`capacity_ah`, `r0`, `r1`, `c1`, `soc0`,
  `soc_variance`, plus optional noise fields with Python's defaults). Adds
  `covariance`, `model`, `ocv_voltage` and `terminal_voltage`.
- `OrderBook`, `solve_routes`, `avoid`, `find_path` and `simulate_consensus`,
  which take and return JSON strings.
- Errors are thrown as `Error` objects with a `kind` property.
- Tests in `tests/` (`wasm-pack test --node`), including the shared parity
  fixtures.

### Changed

- `IsolationForest.fit` and `Ekf` take options objects instead of positional
  arguments. The seed is a number, not a `BigInt`.
