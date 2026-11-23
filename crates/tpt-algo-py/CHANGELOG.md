# Changelog

All notable changes to the `tpt-algo` Python bindings are documented in this file.

This crate is not published from this repository yet (`publish = false`). The
PyPI package is built by the manual `bindings-publish` workflow.

## [Unreleased]

### Added

- `tpt_algo.IsolationForest`: `fit` (static), `dimension`, `score` and
  `score_samples`.
- `tpt_algo.Ekf`: `predict`, `update`, `soc` and `estimate`.
- Errors raise `ValueError`.
- Unit tests in `tests/` (`python -m unittest discover -s tests -v`).
