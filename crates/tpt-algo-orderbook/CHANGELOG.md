# Changelog

All notable changes to `tpt-algo-orderbook` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

## [0.0.0](https://github.com/tpt-solutions/tpt-algorithms/releases/tag/tpt-algo-orderbook-v0.0.0) - 2026-11-23

### Added

- *(orderbook)* add quantity conservation contract and doctests

### Fixed

- *(fraud,swarm,orderbook)* test imports, swarm avoidance and docs

### Other

- *(orderbook)* crates.io metadata, README rewrite and CHANGELOG
- pin LF line endings and add docs.rs metadata for public crates
- Add tpt-algo-orderbook crate

### Added

- `OrderBook` matching engine with price-time priority, applying `Event`s one at
  a time (`apply`) or replaying a stream (`replay`).
- Event types: `Add` (limit order), `Cancel`, `Modify` (size reduction keeps time
  priority, other changes re-queue), `Execute` (market order) and `ExecuteOrder`
  (fill a specific resting order).
- Market queries: `best_bid`, `best_ask`, `spread`, `mid`, `level_qty`,
  `order_count`, `resting_quantity`, `order`, `trades`, `clear_trades`.
- VWAP over the last `n` trades (`vwap_last`) or over an inclusive sequence-number
  window (`vwap_window`).
- `Stats` with `accepted`, `traded`, `cancelled` and `expired` counters, and the
  quantity-conservation identity `accepted == resting + 2 * traded + cancelled + expired`.
- Plain data types `Price` (integer ticks), `Side`, `Event`, `Trade` and
  `OrderBookError`, available without `alloc`.
- `serde` feature for `Price`, `Side`, `Event` and `Trade`.
- `basic` example.
- docs.rs metadata (all features, `docsrs` cfg).
