# tpt-algorithms — Project TODO

Owner: TPT Solutions · License: MIT OR Apache-2.0 (own code) · Dependencies: no Apache-2.0-only, no copyleft

## Progress

| Phase | Area | Status |
|---|---|---|
| 0 | Decisions & prerequisites | [ ] |
| 1 | Licensing & legal | [ ] |
| 2 | Workspace scaffolding | [ ] |
| 3 | Dependency governance | [ ] |
| 4 | CI/CD | [ ] |
| 5 | Release automation | [ ] |
| 6 | Testing & validation framework | [ ] |
| 7 | Internal crates | [ ] |
| 8 | tpt-algo-soc | [ ] |
| 9 | tpt-algo-fraud | [ ] |
| 10 | tpt-algo-orderbook | [ ] |
| 11 | tpt-algo-routing | [ ] |
| 12 | tpt-algo-swarm | [ ] |
| 13 | tpt-algo-consensus | [ ] |
| 14 | Formal verification & contracts | [ ] |
| 15 | Documentation site | [ ] |
| 16 | Python / WASM bindings (optional) | [ ] |
| 17 | v0.1 launch | [ ] |
| 18 | Future crates | [ ] |

---

## Phase 0 — Decisions & prerequisites
- [ ] Confirm `tpt-algo-*` names are free on crates.io (routing, fraud, orderbook, swarm, soc, consensus, math-core, geometry, synth-data)
- [ ] Reserve crate names (publish 0.0.0 placeholders if needed)
- [ ] Create GitHub org/repo `tpt-algorithms` under TPT Solutions
- [ ] Check "TPT" / "TPT Solutions" naming for trademark conflicts
- [ ] Decide MSRV policy (stable minus 2) and record it
- [ ] Confirm edition 2021
- [ ] Create crates.io account/API token for publishing (store as GitHub secret)
- [ ] Decide on a maintainer contact email for crate metadata

## Phase 1 — Licensing & legal
- [ ] Add `LICENSE-MIT` (copyright TPT Solutions)
- [ ] Add `LICENSE-APACHE` (Apache-2.0 text)
- [ ] Set `license = "MIT OR Apache-2.0"` in workspace package metadata
- [ ] Add licensing section to root README (dual-license + contribution clause)
- [ ] Decide whether a `NOTICE` file is needed
- [ ] Decide SPDX header policy for source files (`// SPDX-License-Identifier: MIT OR Apache-2.0`)
- [ ] Write clean-room policy (`docs/clean-room.md`): implement from papers/specs only, never from GPL/proprietary source
- [ ] Create provenance log template (per algorithm: papers, specs, datasets, who implemented, date)
- [ ] Document dependency license rule (MIT/BSD/ISC/Zlib/Unicode or MIT OR Apache-2.0; no Apache-only)

## Phase 2 — Workspace scaffolding
- [ ] Root `Cargo.toml` (workspace, resolver 2, members, `[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]`)
- [ ] `rust-toolchain.toml`
- [ ] `rustfmt.toml` and `clippy.toml`
- [ ] Lints: deny `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` in non-test code
- [ ] `.gitignore` (including dataset cache dir)
- [ ] Directory layout: `crates/`, `internal/`, `benches/`, `docs/`, `scripts/`
- [ ] Crate template (Cargo.toml, `#![no_std]`, `std`/`alloc`/`serde` features, README, error type with `thiserror`)
- [ ] Mark internal crates `publish = false`
- [ ] `git init` and first commit

## Phase 3 — Dependency governance
- [ ] `deny.toml` using current cargo-deny schema (allow: MIT, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-DFS-2016, Unicode-3.0)
- [ ] Handle own workspace crates in cargo-deny (they are MIT OR Apache-2.0; use `private`/`exceptions`/clarify as needed)
- [ ] Verify Apache-only crates are rejected and `MIT OR Apache-2.0` crates pass (test with a throwaway dependency)
- [ ] Ban GPL/AGPL/LGPL/SSPL/EUPL/CC-BY-NC
- [ ] Advisory and duplicate-version (bans) sections configured
- [ ] Audit license trees: nalgebra, ndarray, libm, ordered-float, thiserror, proptest, criterion, serde
- [ ] Replace or avoid any dependency that fails (alternative crate or implement in math-core)
- [ ] Set up `cargo audit` (and RustSec DB refresh)
- [ ] Document the process for adding a new dependency

## Phase 4 — CI/CD (GitHub Actions)
- [ ] Workflow: `cargo fmt --check`
- [ ] Workflow: `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Workflow: `cargo test --workspace`
- [ ] Workflow: `cargo deny check`
- [ ] Workflow: `cargo audit` (also on a schedule)
- [ ] Job: `no_std` build check (e.g. `thumbv7em-none-eabihf` target)
- [ ] Job: feature matrix (`--no-default-features`, `alloc`, `std`, `serde`, all)
- [ ] Job: MSRV build
- [ ] Job: `cargo doc` with warnings denied
- [ ] Job: benchmark compile check (benches build, not run)
- [ ] Caching (Swatinem/rust-cache or equivalent)
- [ ] Dependabot/Renovate for dependency and Actions updates
- [ ] Branch protection requiring all checks

## Phase 5 — Release automation
- [ ] `release-plz.toml` (workspace config, changelog settings, skip internal crates)
- [ ] release-plz GitHub workflow (single release PR, publish on merge)
- [ ] Add crates.io token secret
- [ ] Conventional-commit guidance for changelogs
- [ ] Dry-run release on a placeholder crate
- [ ] Define versioning policy (0.x until validated, semver afterwards)

## Phase 6 — Testing & validation framework
- [ ] proptest conventions and shared strategies
- [ ] Criterion bench harness in `benches/`
- [ ] Dataset fetch tool (`scripts/` or xtask): download on demand, verify SHA-256, cache in gitignored dir
- [ ] Record each dataset's source URL, license/terms, and checksum
- [ ] Tolerance/assertion helpers (relative error, RMSE)
- [ ] Small synthetic/derived fixtures for CI (no third-party data committed)
- [ ] Separate "validation" test target (feature or `#[ignore]`) for full-dataset runs
- [ ] Deterministic RNG convention (seeded, no_std-friendly)

## Phase 7 — Internal crates
### tpt-algo-math-core
- [ ] Scaffold from template
- [ ] Matrix/linear-algebra wrappers (nalgebra/ndarray/libm decisions)
- [ ] Stats primitives (mean, variance, RMSE, quantiles)
- [ ] Fixed-point type for deterministic arithmetic
- [ ] Unit + property tests; `no_std` build check
### tpt-algo-geometry
- [ ] Scaffold from template
- [ ] 2D/3D vectors, transforms, kinematics helpers
- [ ] Spatial primitives (distance, intersection, bounding volumes)
- [ ] Unit + property tests; `no_std` build check
### tpt-algo-synth-data
- [ ] Scaffold from template
- [ ] Deterministic generators: time series, order flow, routing instances, point clouds, transactions
- [ ] Unit tests proving reproducibility from seed

## Phase 8 — tpt-algo-soc (Battery State-of-Charge, Kalman filters)
- [ ] Clean-room reference notes (papers, equivalent-circuit model) + provenance log entry
- [ ] API design (type-state for filter lifecycle, error type)
- [ ] Battery model (OCV curve, RC equivalent circuit)
- [ ] Extended Kalman Filter implementation
- [ ] Unit tests and proptest invariants (SoC in [0,1], covariance stays positive semi-definite)
- [ ] Validation: NASA Battery Dataset, RMSE within paper's claimed bounds
- [ ] CI fixture from synthetic discharge data
- [ ] Benchmarks
- [ ] Rustdoc, README, example
- [ ] `no_std` + feature-matrix check
- [ ] Release readiness review

## Phase 9 — tpt-algo-fraud (Anomaly scoring, Isolation Forest)
- [ ] Clean-room reference notes (Isolation Forest paper) + provenance entry
- [ ] API design and error type
- [ ] Isolation tree and forest implementation (seeded, deterministic)
- [ ] Anomaly scoring
- [ ] Unit tests and proptest invariants (scores in [0,1], determinism by seed)
- [ ] Validation against a public dataset or synthetic benchmark
- [ ] Benchmarks
- [ ] Rustdoc, README, example
- [ ] `no_std` + feature-matrix check
- [ ] Release readiness review

## Phase 10 — tpt-algo-orderbook (Limit Order Book simulation)
- [ ] Clean-room reference notes + provenance entry
- [ ] API design (price/quantity types, event types, error type)
- [ ] Matching engine (price-time priority)
- [ ] Event replay (add/cancel/execute/modify)
- [ ] VWAP and spread calculations
- [ ] Unit tests and proptest invariants (book never crossed, quantity conservation)
- [ ] Validation: LOBSTER data (on-demand download), VWAP and spread within 0.01% of ground truth
- [ ] Check LOBSTER terms of use before documenting the workflow
- [ ] Benchmarks (events/sec)
- [ ] Rustdoc, README, example
- [ ] `no_std` + feature-matrix check
- [ ] Release readiness review

## Phase 11 — tpt-algo-routing (VRP & last-mile optimization)
- [ ] Clean-room reference notes (VRP heuristics from papers; no GPL solver source) + provenance entry
- [ ] API design and error type
- [ ] Problem model (depots, vehicles, capacities, time windows)
- [ ] Construction heuristic (e.g. savings / insertion)
- [ ] Local search improvement (2-opt, relocate, swap)
- [ ] Unit tests and proptest invariants (all customers served once, capacity respected)
- [ ] Validation: public benchmark instances (e.g. Solomon / CVRPLIB) vs best-known solutions
- [ ] Benchmarks
- [ ] Rustdoc, README, example
- [ ] `no_std` + feature-matrix check
- [ ] Release readiness review

## Phase 12 — tpt-algo-swarm (Velocity Obstacle & path planning)
- [ ] Clean-room reference notes (VO/RVO/ORCA papers) + provenance entry
- [ ] API design and error type
- [ ] Velocity obstacle computation
- [ ] Collision-avoidance step for agents
- [ ] Path planning (e.g. A*/grid or roadmap)
- [ ] Unit tests and proptest invariants (no collisions in solvable scenarios)
- [ ] Validation: synthetic crossing/circle scenarios with collision-free and time-to-goal criteria
- [ ] Benchmarks (agents vs step time)
- [ ] Rustdoc, README, example
- [ ] `no_std` + feature-matrix check
- [ ] Release readiness review

## Phase 13 — tpt-algo-consensus (BFT consensus, HotStuff etc.)
- [ ] Clean-room reference notes (HotStuff and variants papers) + provenance entry
- [ ] API design (type-state for protocol phases, error type)
- [ ] Fixed-point / deterministic arithmetic only (no floats in protocol logic)
- [ ] Core state machine (views, quorum certificates, leader rotation)
- [ ] Deterministic network simulator for testing
- [ ] Safety/liveness tests with faulty and Byzantine nodes
- [ ] Proptest invariants (no two conflicting commits)
- [ ] Validation: scripted scenarios from the papers
- [ ] Benchmarks
- [ ] Rustdoc, README, example
- [ ] `no_std` + feature-matrix check
- [ ] Release readiness review

## Phase 14 — Formal verification & contracts
- [ ] Review all crates for type-state opportunities (invalid states unrepresentable)
- [ ] Contract macro/helpers: `requires`/`ensures` as `debug_assert` checks
- [ ] Apply contracts to physical/logical limits (SoC range, quantities, capacities)
- [ ] Spike an optional verifier (kani or creusot) on one critical function; record result
- [ ] Document the verification approach (prusti not adopted: unmaintained)

## Phase 15 — Documentation
- [ ] Root README (purpose, crate list, licensing, status)
- [ ] Per-crate README audit (consistent structure, examples compile)
- [ ] docs.rs metadata (features, `docsrs` cfg)
- [ ] Doc tests for every public API
- [ ] mdBook or docs site: overview, design principles, validation methodology
- [ ] Contributing guide for dependencies and clean-room rules (brief)

## Phase 16 — Python / WASM bindings (optional)
- [ ] Decide go/no-go and which crates
- [ ] License-check pyo3 / maturin / wasm-bindgen trees against deny.toml
- [ ] Separate binding crates (unpublished or separately named), feature-gated
- [ ] Build and test in CI
- [ ] Publish (PyPI / npm) workflow

## Phase 17 — v0.1 launch
- [ ] Final `cargo deny`, `cargo audit`, full CI green on all crates
- [ ] Verify package contents (`cargo package --list`) for each public crate
- [ ] Publish order: dependencies first, then algorithm crates
- [ ] Tag releases, check docs.rs builds
- [ ] Announcement (README badges, post)
- [ ] Maintenance runbook (dependency updates, security advisories, release flow)

## Phase 18 — Future crates
- [ ] Triage and prioritize additional domain crates
- [ ] Apply the crate template and clean-room process to each
