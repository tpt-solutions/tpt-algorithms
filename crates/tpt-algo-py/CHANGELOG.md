# Changelog

All notable changes to the `tpt-algo` Python bindings are documented in this file.

This crate is not published from this repository yet (`publish = false`). The
PyPI package is built by the manual `bindings-publish` workflow.

## [Unreleased]

### Added

- `IsolationForest`: `fit` (static, with `n_trees`, `subsample_size` and `seed`
  keyword arguments), `dimension`, `n_trees`, `score`, `score_samples` and
  `mean_path_length`. `average_path_length(n)` is a module function.
- `Ekf`: `predict`, `update`, `soc`, `estimate`, `covariance`, `model`,
  `ocv_voltage` and `terminal_voltage`.
- `OrderBook` (atomic event batches), `solve_routes`, `avoid`, `find_path` and
  `simulate_consensus`.
- `TptAlgoError`, a `ValueError` subclass with a `kind` attribute.
- `tpt_algo.__version__`, `py.typed` and a `.pyi` stub. The native extension is
  `tpt_algo._native`.
- Unit tests in `tests/` and the shared parity fixtures (`tests/test_parity.py`).

### Changed

- Errors that were plain `ValueError` are now `TptAlgoError`, which still
  subclasses `ValueError`.
