# tpt-algorithms — Project TODO

Status legend: `[x]` done · `[~]` partial (see note) · `[ ]` not done (blocked, external, or deliberately skipped; see note). Last updated 2026-11-23 (commit after 40f5831).

Owner: TPT Solutions · License: MIT OR Apache-2.0 (own code) · Dependencies: no Apache-2.0-only, no copyleft

## Progress

| Phase | Area | Status |
|---|---|---|
| 0 | Decisions & prerequisites | [~] |
| 1 | Licensing & legal | [x] |
| 2 | Workspace scaffolding | [x] |
| 3 | Dependency governance | [~] |
| 4 | CI/CD | [~] |
| 5 | Release automation | [~] |
| 6 | Testing & validation framework | [~] |
| 7 | Internal crates | [~] |
| 8 | tpt-algo-soc | [~] |
| 9 | tpt-algo-fraud | [~] |
| 10 | tpt-algo-orderbook | [~] |
| 11 | tpt-algo-routing | [~] |
| 12 | tpt-algo-swarm | [~] |
| 13 | tpt-algo-consensus | [~] |
| 14 | Formal verification & contracts | [~] |
| 15 | Documentation site | [~] |
| 16 | Python / WASM bindings (optional) | [~] |
| 17 | v0.1 launch | [~] |
| 18 | Future crates | [~] |

---

## Phase 0 — Decisions & prerequisites
- [x] Confirm `tpt-algo-*` names are free on crates.io (routing, fraud, orderbook, swarm, soc, consensus, math-core, geometry, synth-data) — _index lookup 2026-11-23: none taken; see docs/decisions.md_
- [ ] Reserve crate names (publish 0.0.0 placeholders if needed) — _blocked: needs maintainer approval (outward-facing)_
- [ ] Create GitHub org/repo `tpt-algorithms` under TPT Solutions — _blocked: external: GitHub org not created_
- [ ] Check "TPT" / "TPT Solutions" naming for trademark conflicts — _blocked: external: legal review_
- [x] Decide MSRV policy (stable minus 2) and record it — _1.98 (stable minus 2); docs/decisions.md D3_
- [x] Confirm edition 2021
- [ ] Create crates.io account/API token for publishing (store as GitHub secret) — _blocked: external_
- [ ] Decide on a maintainer contact email for crate metadata — _blocked: open decision D6_

## Phase 1 — Licensing & legal
- [x] Add `LICENSE-MIT` (copyright TPT Solutions)
- [x] Add `LICENSE-APACHE` (Apache-2.0 text) — _official text_
- [x] Set `license = "MIT OR Apache-2.0"` in workspace package metadata
- [x] Add licensing section to root README (dual-license + contribution clause)
- [x] Decide whether a `NOTICE` file is needed — _not needed; docs/licensing.md_
- [x] Decide SPDX header policy for source files (`// SPDX-License-Identifier: MIT OR Apache-2.0`) — _docs/licensing.md_
- [x] Write clean-room policy (`docs/clean-room.md`): implement from papers/specs only, never from GPL/proprietary source — _docs/clean-room.md_
- [x] Create provenance log template (per algorithm: papers, specs, datasets, who implemented, date) — _docs/provenance/TEMPLATE.md_
- [x] Document dependency license rule (MIT/BSD/ISC/Zlib/Unicode or MIT OR Apache-2.0; no Apache-only) — _docs/dependencies.md_

## Phase 2 — Workspace scaffolding
- [x] Root `Cargo.toml` (workspace, resolver 2, members, `[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]`)
- [x] `rust-toolchain.toml`
- [x] `rustfmt.toml` and `clippy.toml`
- [x] Lints: deny `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` in non-test code — _workspace lints; tests exempt via clippy.toml_
- [x] `.gitignore` (including dataset cache dir) — _already covered_
- [x] Directory layout: `crates/`, `internal/`, `benches/`, `docs/`, `scripts/` — _benches/ added with a Criterion crate_
- [x] Crate template (Cargo.toml, `#![no_std]`, `std`/`alloc`/`serde` features, README, error type with `thiserror`) — _docs/crate-template.md_
- [x] Mark internal crates `publish = false`
- [x] `git init` and first commit — _repo already had commit cada79c_

## Phase 3 — Dependency governance
- [x] `deny.toml` using current cargo-deny schema (allow: MIT, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-DFS-2016, Unicode-3.0) — _cargo-deny 0.20 schema v2_
- [x] Handle own workspace crates in cargo-deny (they are MIT OR Apache-2.0; use `private`/`exceptions`/clarify as needed)
- [x] Verify Apache-only crates are rejected and `MIT OR Apache-2.0` crates pass (test with a throwaway dependency) — _probed in a throwaway workspace: Apache-only rejected, MIT OR Apache-2.0 accepted_
- [x] Ban GPL/AGPL/LGPL/SSPL/EUPL/CC-BY-NC — _by allow-list; deny passes_
- [x] Advisory and duplicate-version (bans) sections configured
- [x] Audit license trees: nalgebra, ndarray, libm, ordered-float, thiserror, proptest, criterion, serde — _nalgebra/ndarray not used; all current deps pass deny_
- [x] Replace or avoid any dependency that fails (alternative crate or implement in math-core) — _nothing failed_
- [x] Set up `cargo audit` (and RustSec DB refresh) — _audit clean; weekly workflow_
- [x] Document the process for adding a new dependency — _docs/dependencies.md_

## Phase 4 — CI/CD (GitHub Actions)
- [x] Workflow: `cargo fmt --check` — _ci.yml (not yet run on GitHub)_
- [x] Workflow: `cargo clippy --workspace --all-targets -- -D warnings` — _ci.yml_
- [x] Workflow: `cargo test --workspace` — _ci.yml_
- [x] Workflow: `cargo deny check` — _ci.yml_
- [x] Workflow: `cargo audit` (also on a schedule) — _ci.yml + audit-schedule.yml_
- [x] Job: `no_std` build check (e.g. `thumbv7em-none-eabihf` target) — _ci.yml_
- [x] Job: feature matrix (`--no-default-features`, `alloc`, `std`, `serde`, all) — _ci.yml_
- [x] Job: MSRV build — _ci.yml; no local 1.98 toolchain to run it_
- [x] Job: `cargo doc` with warnings denied — _ci.yml_
- [x] Job: benchmark compile check (benches build, not run) — _ci.yml_
- [x] Caching (Swatinem/rust-cache or equivalent) — _ci.yml_
- [x] Dependabot/Renovate for dependency and Actions updates — _Dependabot_
- [ ] Branch protection requiring all checks — _blocked: external: GitHub settings_

## Phase 5 — Release automation
- [x] `release-plz.toml` (workspace config, changelog settings, skip internal crates)
- [x] release-plz GitHub workflow (single release PR, publish on merge)
- [ ] Add crates.io token secret — _blocked: external_
- [x] Conventional-commit guidance for changelogs — _docs/release.md_
- [ ] Dry-run release on a placeholder crate — _blocked: external_
- [x] Define versioning policy (0.x until validated, semver afterwards) — _docs/decisions.md D9_

## Phase 6 — Testing & validation framework
- [~] proptest conventions and shared strategies — _partial: conventions in docs/testing.md; shared strategies not written_
- [x] Criterion bench harness in `benches/` — _benches/ crate; primitives only_
- [x] Dataset fetch tool (`scripts/` or xtask): download on demand, verify SHA-256, cache in gitignored dir — _scripts/fetch-dataset.sh; digest check tested locally_
- [~] Record each dataset's source URL, license/terms, and checksum — _partial: URLs and terms in docs/datasets.md; SHA-256 TBD until fetched_
- [x] Tolerance/assertion helpers (relative error, RMSE)
- [x] Small synthetic/derived fixtures for CI (no third-party data committed)
- [x] Separate "validation" test target (feature or `#[ignore]`) for full-dataset runs
- [x] Deterministic RNG convention (seeded, no_std-friendly)

## Phase 7 — Internal crates
### tpt-algo-math-core
- [x] Scaffold from template
- [ ] Matrix/linear-algebra wrappers (nalgebra/ndarray/libm decisions) — _todo: not started; nalgebra/ndarray not adopted_
- [x] Stats primitives (mean, variance, RMSE, quantiles)
- [x] Fixed-point type for deterministic arithmetic
- [x] Unit + property tests; `no_std` build check
### tpt-algo-geometry
- [x] Scaffold from template
- [~] 2D/3D vectors, transforms, kinematics helpers — _partial: vectors and kinematics done; no transform type_
- [x] Spatial primitives (distance, intersection, bounding volumes)
- [x] Unit + property tests; `no_std` build check
### tpt-algo-synth-data
- [x] Scaffold from template
- [x] Deterministic generators: time series, order flow, routing instances, point clouds, transactions
- [x] Unit tests proving reproducibility from seed

## Phase 8 — tpt-algo-soc (Battery State-of-Charge, Kalman filters)
- [x] Clean-room reference notes (papers, equivalent-circuit model) + provenance log entry — _docs/clean-room-notes/soc.md; citations from memory_
- [x] API design (type-state for filter lifecycle, error type)
- [x] Battery model (OCV curve, RC equivalent circuit)
- [x] Extended Kalman Filter implementation
- [x] Unit tests and proptest invariants (SoC in [0,1], covariance stays positive semi-definite)
- [~] Validation: NASA Battery Dataset, RMSE within paper's claimed bounds — _partial: ignored skeleton; not run; no real RMSE_
- [x] CI fixture from synthetic discharge data
- [ ] Benchmarks — _todo: skipped as instructed; not written_
- [x] Rustdoc, README, example
- [x] `no_std` + feature-matrix check
- [ ] Release readiness review — _todo: not attempted_

## Phase 9 — tpt-algo-fraud (Anomaly scoring, Isolation Forest)
- [x] Clean-room reference notes (Isolation Forest paper) + provenance entry — _docs/clean-room-notes/fraud.md_
- [x] API design and error type
- [x] Isolation tree and forest implementation (seeded, deterministic)
- [x] Anomaly scoring
- [x] Unit tests and proptest invariants (scores in [0,1], determinism by seed)
- [~] Validation against a public dataset or synthetic benchmark — _partial: synthetic AUC>0.9 test passes; public dataset not run_
- [ ] Benchmarks — _todo: skipped as instructed; not written_
- [x] Rustdoc, README, example
- [x] `no_std` + feature-matrix check
- [ ] Release readiness review — _todo: not attempted_

## Phase 10 — tpt-algo-orderbook (Limit Order Book simulation)
- [x] Clean-room reference notes + provenance entry — _docs/clean-room-notes/orderbook.md_
- [x] API design (price/quantity types, event types, error type)
- [x] Matching engine (price-time priority)
- [x] Event replay (add/cancel/execute/modify)
- [x] VWAP and spread calculations
- [x] Unit tests and proptest invariants (book never crossed, quantity conservation)
- [~] Validation: LOBSTER data (on-demand download), VWAP and spread within 0.01% of ground truth — _partial: ignored skeleton; not run_
- [ ] Check LOBSTER terms of use before documenting the workflow — _todo: terms not verified_
- [ ] Benchmarks (events/sec) — _todo: skipped as instructed; not written_
- [x] Rustdoc, README, example
- [x] `no_std` + feature-matrix check
- [ ] Release readiness review — _todo: not attempted_

## Phase 11 — tpt-algo-routing (VRP & last-mile optimization)
- [x] Clean-room reference notes (VRP heuristics from papers; no GPL solver source) + provenance entry — _docs/clean-room-notes/routing.md_
- [x] API design and error type
- [x] Problem model (depots, vehicles, capacities, time windows)
- [x] Construction heuristic (e.g. savings / insertion)
- [x] Local search improvement (2-opt, relocate, swap)
- [x] Unit tests and proptest invariants (all customers served once, capacity respected)
- [~] Validation: public benchmark instances (e.g. Solomon / CVRPLIB) vs best-known solutions — _partial: ignored skeleton; not run_
- [ ] Benchmarks — _todo: skipped as instructed; not written_
- [x] Rustdoc, README, example
- [x] `no_std` + feature-matrix check
- [ ] Release readiness review — _todo: not attempted_

## Phase 12 — tpt-algo-swarm (Velocity Obstacle & path planning)
- [x] Clean-room reference notes (VO/RVO/ORCA papers) + provenance entry — _docs/clean-room-notes/swarm.md_
- [x] API design and error type
- [x] Velocity obstacle computation
- [x] Collision-avoidance step for agents
- [x] Path planning (e.g. A*/grid or roadmap)
- [x] Unit tests and proptest invariants (no collisions in solvable scenarios)
- [x] Validation: synthetic crossing/circle scenarios with collision-free and time-to-goal criteria
- [ ] Benchmarks (agents vs step time) — _todo: skipped as instructed; not written_
- [x] Rustdoc, README, example
- [x] `no_std` + feature-matrix check
- [ ] Release readiness review — _todo: not attempted_

## Phase 13 — tpt-algo-consensus (BFT consensus, HotStuff etc.)
- [x] Clean-room reference notes (HotStuff and variants papers) + provenance entry — _docs/clean-room-notes/consensus.md_
- [x] API design (type-state for protocol phases, error type)
- [x] Fixed-point / deterministic arithmetic only (no floats in protocol logic) — _no floats in src (grep verified)_
- [x] Core state machine (views, quorum certificates, leader rotation)
- [x] Deterministic network simulator for testing
- [x] Safety/liveness tests with faulty and Byzantine nodes
- [x] Proptest invariants (no two conflicting commits)
- [x] Validation: scripted scenarios from the papers
- [ ] Benchmarks — _todo: skipped as instructed; not written_
- [x] Rustdoc, README, example
- [x] `no_std` + feature-matrix check
- [ ] Release readiness review — _todo: not attempted_

## Phase 14 — Formal verification & contracts
- [~] Review all crates for type-state opportunities (invalid states unrepresentable) — _partial: done for soc and consensus; not reviewed for others_
- [x] Contract macro/helpers: `requires`/`ensures` as `debug_assert` checks — _requires!/ensures! in tpt-algo-math-core; public crates not yet using them_
- [ ] Apply contracts to physical/logical limits (SoC range, quantities, capacities) — _todo_
- [ ] Spike an optional verifier (kani or creusot) on one critical function; record result — _blocked: Kani supports Linux/macOS only; run on Linux or WSL_
- [x] Document the verification approach (prusti not adopted: unmaintained) — _docs/formal-verification.md_

## Phase 15 — Documentation
- [x] Root README (purpose, crate list, licensing, status)
- [~] Per-crate README audit (consistent structure, examples compile) — _partial: written per crate, not audited for consistency_
- [x] docs.rs metadata (features, `docsrs` cfg) — _all-features and docsrs cfg set for each public crate_
- [~] Doc tests for every public API — _partial: some crates only_
- [x] mdBook or docs site: overview, design principles, validation methodology — _book.toml + docs/SUMMARY.md; `mdbook build` clean_
- [x] Contributing guide for dependencies and clean-room rules (brief) — _CONTRIBUTING.md_

## Phase 16 — Python / WASM bindings (optional)
- [x] Decide go/no-go and which crates — _no-go for v0.1; docs/decisions.md D7_
- [ ] License-check pyo3 / maturin / wasm-bindgen trees against deny.toml — _todo: deferred (D7)_
- [ ] Separate binding crates (unpublished or separately named), feature-gated — _todo: deferred (D7)_
- [ ] Build and test in CI — _todo: deferred (D7)_
- [ ] Publish (PyPI / npm) workflow — _todo: deferred (D7)_

## Phase 17 — v0.1 launch
- [~] Final `cargo deny`, `cargo audit`, full CI green on all crates — _partial: local deny and audit pass; CI not run on GitHub_
- [x] Verify package contents (`cargo package --list`) for each public crate — _cargo package --list checked for all six public crates; no datasets included_
- [ ] Publish order: dependencies first, then algorithm crates — _todo_
- [ ] Tag releases, check docs.rs builds — _todo_
- [ ] Announcement (README badges, post) — _todo_
- [x] Maintenance runbook (dependency updates, security advisories, release flow) — _docs/maintenance.md_

## Phase 18 — Future crates
- [x] Triage and prioritize additional domain crates — _docs/future-crates.md_
- [ ] Apply the crate template and clean-room process to each — _todo_
