# Changelog

All notable changes to `tpt-algo-math-core` are documented in this file.

This crate is internal (`publish = false`) and is never released, so entries
describe workspace changes only.

## [Unreleased]

### Added

- `stats`: `mean`, `population_variance`, `sample_variance`, `rmse`, and `quantile`
  (with `alloc`).
- `tolerance`: `relative_error`, `within_relative` and `within_absolute`.
- `fixed`: `Fixed`, a Q32.32 fixed-point type with checked arithmetic.
- `rng`: seeded, `no_std` `SplitMix64`.
- `error`: `MathError`.
- `contract`: `requires!` and `ensures!` debug-assertion helpers.
- `strategies`: shared proptest strategies (feature `proptest`).
