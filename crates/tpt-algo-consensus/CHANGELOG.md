# Changelog

All notable changes to `tpt-algo-consensus` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

## [0.0.0](https://github.com/tpt-solutions/tpt-algorithms/releases/tag/tpt-algo-consensus-v0.0.0) - 2026-11-23

### Added

- *(consensus)* implement chained BFT state machine and simulator

### Other

- *(consensus)* crates.io metadata, README rewrite and CHANGELOG
- *(consensus)* add runnable doctests for the public API
- pin LF line endings and add docs.rs metadata for public crates
- Add tpt-algo-consensus crate skeleton

### Added

- Chained HotStuff replica state machine (`Replica`): safe-node vote rule, lock
  rule, three-chain commit rule over consecutive views, pacemaker with
  exponential back-off, `NewView` view change with `f + 1` view synchronisation,
  and single-block fetching.
- Type-state phase tracking (`Round<Prepare | PreCommit | Commit | Decide>`) in
  the `phase` module; illegal transitions do not compile.
- Core types: `Block`, `BlockId`, `QuorumCert`, `SignerSet`, `Config`,
  `ReplicaId`, `View`, `Message`, `Outgoing`, `Recipient`, `Vote`, `NewView`.
- Runtime-checked network entry point `Replica::handle` returning `ConsensusError`.
- Deterministic network simulator (`sim` module) with seeded delays, reordering,
  drops, global stabilisation time, crash faults and equivocating Byzantine
  leaders, plus `check_prefix_consistency`.
- `no_std` support: core types build without `alloc`; the `alloc` feature enables
  `Replica` and `sim`.
- `serde` feature for ids, views, signer sets, blocks, QCs and messages.
- `simulate` example.

