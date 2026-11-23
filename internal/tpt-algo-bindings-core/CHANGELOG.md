# Changelog

All notable changes to `tpt-algo-bindings-core` are documented in this file.

This crate is internal (`publish = false`) and is never released, so entries
describe workspace changes only.

## [Unreleased]

### Added

- `Forest`: Isolation Forest wrapper with `fit`, `dimension`, `score` and
  `score_samples`, returning `String` errors.
- `CellSpec` and `Ekf`: state-of-charge EKF wrapper that dispatches at runtime
  over OCV table lengths 2 to 8 (`MIN_OCV_POINTS`, `MAX_OCV_POINTS`).
