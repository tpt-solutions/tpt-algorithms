# Changelog

All notable changes to `tpt-algo-benches` are documented in this file.

This crate is internal (`publish = false`) and is never released, so entries
describe workspace changes only.

## [Unreleased]

### Added

- `primitives` Criterion benchmark: `SplitMix64::next_u64` and `sample_variance`
  over 10,000 values.
- Per-crate Criterion targets: `soc` (EKF), `fraud` (isolation forest fit and
  scoring), `orderbook` (event application), `routing` (Clarke-Wright and solve at
  50, 200 and 500 customers), `swarm` (avoidance steps and grid A*) and `consensus`
  (seeded replica simulation). Inputs come from `tpt-algo-synth-data`.
- `.github/workflows/bench.yml`: weekly and manually dispatched Criterion run that
  uploads `target/criterion/` as an artifact.
