# Changelog

All notable changes to `tpt-algo-swarm` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

### Added

- `VelocityObstacle`: truncated-cone velocity obstacle between two disc-shaped
  agents, with time-to-collision queries.
- `avoid` and `avoid_among`: choose the admissible velocity closest to the
  preferred one, using a deterministic candidate grid and reciprocal
  responsibility (`Responsibility`).
- `Agent`, `AvoidanceConfig` and `Vec2` (`f64`) types.
- `Simulation`: synchronous, lock-step multi-agent stepping (`alloc`).
- `Grid` occupancy grid with `Connectivity` (4- or 8-way) and `find_path`, an A*
  search returning a `GridPath` (`alloc`).
- `no_std` support; `libm` supplies the elementary functions.
- `serde` feature for `Vec2`, `Agent`, `AvoidanceConfig` and `Responsibility`.
- `crossing` example.
- docs.rs metadata (all features, `docsrs` cfg).

### Documentation

- README "Benchmarks" section listing the Criterion benchmarks in
  `benches/benches/swarm.rs`, with the scheduled bench workflow as the
  source of published numbers.
