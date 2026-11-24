# Changelog

All notable changes to `tpt-algo-routing` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

## [0.0.2](https://github.com/tpt-solutions/tpt-algorithms/compare/tpt-algo-routing-v0.0.1...tpt-algo-routing-v0.0.2) - 2026-11-24

### Other

- Merge branch 'master' of https://github.com/tpt-solutions/tpt-algorithms
- update READMEs, CHANGELOGs and dev tooling for phases 19-23

## [0.0.1](https://github.com/tpt-solutions/tpt-algorithms/compare/tpt-algo-routing-v0.0.0...tpt-algo-routing-v0.0.1) - 2026-11-23

### Other

- release

## [0.0.0](https://github.com/tpt-solutions/tpt-algorithms/releases/tag/tpt-algo-routing-v0.0.0) - 2026-11-23

### Added

- *(routing)* add feasibility contract and doctests

### Other

- *(routing)* crates.io metadata, README rewrite and CHANGELOG
- pin LF line endings and add docs.rs metadata for public crates
- Add tpt-algo-routing crate

### Added

- Capacitated vehicle routing model: `Problem` (single depot, vehicle capacity,
  optional vehicle limit, optional depot due time), `Customer` (position, demand,
  service time, optional time window) and `TimeWindow`.
- `Solution` with `cost` and `vehicles_used`, and `Problem` validation returning
  `RoutingError`.
- Clarke and Wright savings construction (`clarke_wright_savings`).
- Local search with 2-opt, relocate and swap moves (`local_search`), with a
  deterministic move order and a `Config` iteration limit.
- `solve`, which runs savings construction followed by local search.
- Time-window feasibility: early arrival waits, late arrival is infeasible.
- `serde` feature for `Customer`, `TimeWindow` and `Config`.
- `basic` example.
- docs.rs metadata (all features, `docsrs` cfg).

### Documentation

- README "Benchmarks" section listing the Criterion benchmarks in
  `benches/benches/routing.rs`, with the scheduled bench workflow as the
  source of published numbers.
