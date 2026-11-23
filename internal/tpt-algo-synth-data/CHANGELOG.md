# Changelog

All notable changes to `tpt-algo-synth-data` are documented in this file.

This crate is internal (`publish = false`) and is never released, so entries
describe workspace changes only.

## [Unreleased]

### Added

- `gaussian::standard_normal`.
- `series`: `random_walk` and `noisy_ramp`.
- `routing`: `cvrp_uniform` and `point_cloud`.
- `order_flow::limit_orders`, with `OrderEvent` and `Side`.
- `transactions::transactions`, with labelled `Transaction` records.
- Seeded, reproducible output on all platforms.
