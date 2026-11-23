# Changelog

All notable changes to `tpt-algo-benches` are documented in this file.

This crate is internal (`publish = false`) and is never released, so entries
describe workspace changes only.

## [Unreleased]

### Added

- `primitives` Criterion benchmark: `SplitMix64::next_u64` and `sample_variance`
  over 10,000 values.
