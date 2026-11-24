# Changelog

All notable changes to `tpt-algo-soc` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

## [0.0.2](https://github.com/tpt-solutions/tpt-algorithms/compare/tpt-algo-soc-v0.0.1...tpt-algo-soc-v0.0.2) - 2026-11-24

### Fixed

- remove silent fallbacks and stubs from library code

### Other

- Merge branch 'master' of https://github.com/tpt-solutions/tpt-algorithms
- update READMEs, CHANGELOGs and dev tooling for phases 19-23
- add validation suites and fixtures for soc, swarm and consensus

## [0.0.1](https://github.com/tpt-solutions/tpt-algorithms/compare/tpt-algo-soc-v0.0.0...tpt-algo-soc-v0.0.1) - 2026-11-23

### Other

- release

## [0.0.0](https://github.com/tpt-solutions/tpt-algorithms/releases/tag/tpt-algo-soc-v0.0.0) - 2026-11-23

### Added

- *(soc)* add state and covariance contracts and doctests
- *(math-core)* add shared proptest strategies

### Other

- *(soc)* crates.io metadata, README rewrite and CHANGELOG
- pin LF line endings and add docs.rs metadata for public crates
- Add tpt-algo-soc crate

### Added

- First-order Thevenin (RC equivalent-circuit) cell model: `Model` built from
  `CellParams` (capacity, `R0`, `R1`, `C1`) and an open-circuit-voltage table
  `OcvCurve`.
- Extended Kalman Filter over the state `[SoC, V1]`: `Filter::predict`,
  `Filter::update` and `Filter::estimate`, returning an `Estimate`.
- Type-state lifecycle: a `Filter<Uninitialized, N>` must be turned into a
  `Filter<Initialized, N>` with `initialize` before prediction or update.
- `NoiseConfig` for process and measurement noise.
- `SocError` for invalid parameters, tables and filter inputs.
- `no_std` support without an allocator; the OCV table is a const-generic array.
- `serde` feature for `CellParams`, `NoiseConfig` and `Estimate`.
- `discharge` example.
- docs.rs metadata (all features, `docsrs` cfg).

### Documentation

- README "Benchmarks" section listing the Criterion benchmarks in
  `benches/benches/soc.rs`, with the scheduled bench workflow as the
  source of published numbers.
