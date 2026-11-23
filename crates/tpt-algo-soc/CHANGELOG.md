# Changelog

All notable changes to `tpt-algo-soc` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

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
