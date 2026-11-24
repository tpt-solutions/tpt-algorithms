# Changelog

All notable changes to `tpt-algo-bindings-core` are documented in this file.

This crate is internal (`publish = false`) and is never released, so entries
describe workspace changes only.

## [Unreleased]

### Added

- `fraud`: `Forest` gained `n_trees` and `mean_path_length`. `average_path_length`
  and `forest_defaults` are new.
- `soc`: `Ekf` gained `covariance`, `cell_params`, `ocv_voltage` and
  `terminal_voltage`. `EKF_DEFAULTS` holds the binding defaults.
- `routing`, `orderbook`, `swarm` and `consensus`: JSON entry points over the
  routing, order book, swarm and consensus crates. The order book is a stateful
  `Book` whose batches are atomic.
- `error`: `BindingError` with an `ErrorKind` (`invalid_input`, `invalid_config`,
  `unsupported`, `infeasible`, `unreachable`).
- `fixtures/parity.json` and `tests/parity.rs`: cases shared with the Python and
  WebAssembly test suites.

### Changed

- Errors are `BindingError` rather than `String`.
- The consensus report writes block ids as 16-digit hex strings, because a `u64`
  does not fit a JavaScript number.
