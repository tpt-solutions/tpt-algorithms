# tpt-algorithms — Project TODO

Status legend: `[x]` done · `[~]` partial (see note) · `[ ]` not done (blocked, external, or deliberately skipped; see note). Last updated 2026-11-24 (Phases 19-23 added).

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
| 19 | Stub and quiet-fallback cleanup | [~] |
| 20 | Bindings for all six crates | [x] |
| 21 | `tpt` CLI | [x] |
| 22 | Per-crate benchmarks | [~] |
| 23 | Dev tooling and adoption docs | [~] |

---

## Phase 0 — Decisions & prerequisites
- [x] Confirm `tpt-algo-*` names are free on crates.io (routing, fraud, orderbook, swarm, soc, consensus, math-core, geometry, synth-data) — _index lookup 2026-11-23: none taken; see docs/decisions.md_
- [ ] Reserve crate names (publish 0.0.0 placeholders if needed) — _blocked: needs maintainer approval (outward-facing)_
- [ ] Create GitHub org/repo `tpt-algorithms` under TPT Solutions — _blocked: external: GitHub org not created. Refreshed 2026-11-24: the repo already exists as `origin` (github.com/tpt-solutions/tpt-algorithms); the org/owner question is still external_
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
- [~] proptest conventions and shared strategies — _partial: conventions in docs/testing.md; generic strategies in math-core `strategies` (feature `proptest`), used by soc only; domain strategies stay per crate_
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
- [~] Apply contracts to physical/logical limits (SoC range, quantities, capacities) — _partial: debug_assert in soc (covariance PSD on predict/update), orderbook (quantity conservation per event), routing (feasibility after each move); public crates cannot depend on unpublished math-core, so they use debug_assert! directly; soc range/quantity contracts not yet covered by a dedicated test_
- [~] Spike an optional verifier (kani) on one critical function; record result — _partial: three harnesses on geometry `constant_accel_position` in `kinematics.rs` (cfg(kani)); CI-only job `.github/workflows/kani.yml` (Kani 0.68.0, one matrix job per harness); not yet run, result to be recorded in docs/formal-verification.md_
- [x] Document the verification approach (prusti not adopted: unmaintained) — _docs/formal-verification.md_

## Phase 15 — Documentation
- [x] Root README (purpose, crate list, licensing, status)
- [x] Per-crate README audit (consistent structure, examples compile) — _section headings aligned across the six public crates; README usage snippets mirrored as crate-level doctests in soc, routing, swarm (fraud already had one)_
- [x] docs.rs metadata (features, `docsrs` cfg) — _all-features and docsrs cfg set for each public crate_
- [x] Doc tests for every public API — _every public type, function and method in the six public crates has a runnable example (215 doctests across the six crates); checked with cargo test (default, no-default, all-features); the private `Sealed` trait is excluded_
- [x] mdBook or docs site: overview, design principles, validation methodology — _book.toml + docs/SUMMARY.md; `mdbook build` clean_
- [x] Contributing guide for dependencies and clean-room rules (brief) — _CONTRIBUTING.md_

## Phase 16 — Python / WASM bindings (optional)
- [x] Decide go/no-go and which crates — _go for v0.1 (D7 reversed); soc and fraud, via internal `tpt-algo-bindings-core`_
- [x] License-check pyo3 / maturin / wasm-bindgen trees against deny.toml — _all three pass; `target-lexicon` 0.13.6 (Apache-2.0 WITH LLVM-exception, build-only) patched with tpt-solutions/target-lexicon-mit at 297f442a; deny.toml allows that one git source; see docs/bindings.md_
- [x] Separate binding crates (unpublished or separately named), feature-gated — _`crates/tpt-algo-py` (pyo3, feature `extension-module`), `crates/tpt-algo-wasm`; both excluded from the root workspace_
- [x] Build and test in CI — _`.github/workflows/bindings.yml`: wasm tests in Node, Python wheel build and unittest on Ubuntu and Windows; not yet run on GitHub_
- [x] Publish (PyPI / npm) workflow — _`.github/workflows/bindings-publish.yml`, manual only, publishes nothing by default; not run; needs `pypi` and `npm` environments and an NPM_TOKEN secret_

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

## Phase 19 — Stub and quiet-fallback cleanup
- [x] soc NASA validation test passes vacuously: implement CSV loader, OCV/R0/R1/C1 fit, replay and RMSE assertion (`crates/tpt-algo-soc/tests/validation.rs`) — _note: offline synthetic known-model test (`synthetic_known_model_fit_and_replay_recover_truth`) exercises loader, fit, replay and RMSE; NASA test stays `#[ignore]`d and needs `TPT_DATASET_DIR/nasa_discharge.csv` (`scripts/fetch-dataset.sh`). RMSE bound 0.03 is a placeholder until the cited paper bound is recorded._
- [x] swarm dataset test is a placeholder (dummy 1x1 grid, always panics): write Moving AI `.map`/`.scen` loader and compare with published optimal lengths — _note: loader and check in `tests/validation.rs`; hand-made fixtures in `tests/data/` (`hand_made_fixtures_match_hand_derived_optimal_lengths`); published-set test stays `#[ignore]`d (needs `TPT_DATASET_DIR`). Fixture optima are hand-derived, not Moving AI values._
- [x] consensus validation reads a `scenarios.txt` that does not exist: write and commit the hand-derived HotStuff traces — _note: `tests/data/hotstuff_traces.txt` with derivation in header; `hand_derived_hotstuff_traces` runs without dataset and passes._
- [x] fraud `forest.rs`: replace `unwrap_or(0.0)` and partial-depth return with `FraudError` (or documented `debug_assert!`) — _note: new `FraudError::CorruptTree` variant (public enum, additive); tests `path_length_errors_on_short_input_instead_of_defaulting`, `path_length_errors_on_dangling_node`._
- [x] consensus `types.rs` (`unwrap_or(u16::MAX)`, `unwrap_or(0)`) and `replica.rs` (`pop_front().unwrap_or(0)`): return errors or use `Option<u64>` for empty payload — _note: `Block::payload` is now `Option<u64>` (public field type change; empty block = `None`, hashed with a presence tag). `SignerSet::len` and `leader` no longer fall back; `empty_payload_is_distinct_from_command_zero` added._
- [x] fraud `rng.rs`: remove modulo bias (rejection sampling); review `cfg_attr(allow(dead_code))` — _note: `below` rejects draws in the biased tail (`rejection_boundaries`, `below_stays_in_range_and_covers_all_values`); the rng module is now `cfg(feature = "alloc")` gated instead of allowing dead code._
- [x] synth-data `order_flow.rs` and `routing.rs`: remove silent `unwrap_or(0)` and unchecked `as u32` — _note: `routing::cvrp_uniform` uses `?` with checked conversion; `order_flow` documents i64 saturation of offsets (`huge_offset_saturates_instead_of_wrapping`). `limit_orders` signature unchanged (benches and orderbook tests call it)._
- [~] Add `Default` for soc `NoiseConfig`; make binding defaults read the Rust defaults — _note: `Default` added (`src/filter.rs`, doctest). Binding part not done here: owned by another agent._
- [x] Dedicated test for soc range and quantity contracts — _note: `crates/tpt-algo-soc/tests/contracts.rs` checks SoC in [0,1], covariance diagonal and PSD after every call, including extreme inputs and a proptest._
- [ ] Run the three Kani harnesses and record the result in docs/formal-verification.md — _blocked: Kani runs in CI only (not installed locally; no Windows support). Run the `kani` workflow on GitHub, then record the result._
- [ ] Fix SHA-256 TBDs in docs/datasets.md once datasets are fetched — _blocked: needs dataset downloads (`scripts/fetch-dataset.sh`) and their checksums; not available offline._
- [x] Doc fixes: README "no PRs" vs CONTRIBUTING; add bindings, clean-room, crate-template, datasets to docs/SUMMARY.md; unify actions/checkout version; add `tpt-algo-bindings-core` to release-plz `release = false`; refresh stale Phase 0 items — _note: README now says PRs are welcome (matching CONTRIBUTING; an issue-first note is included, so this is a policy choice the maintainer should confirm). SUMMARY: clean-room, crate-template and datasets were already listed; bindings added. actions/checkout is @v7 everywhere. release-plz entry added. Phase 0: only the repo-exists note refreshed; blocked checkboxes left as they were._

## Phase 20 — Bindings for all six crates
- [x] Expose fraud `n_trees`, `mean_path_length`, `average_path_length` — _Python and WASM; `IsolationForest.n_trees`/`mean_path_length`, `average_path_length`; tests in `test_api.py`, `tests/api.rs`, parity cases `forest_*`, `average_path_length`_
- [x] Expose soc `covariance`, `model`, terminal voltage helpers — _`Ekf.covariance`/`model`/`ocv_voltage`/`terminal_voltage` in both bindings; parity case `ekf_explicit`_
- [x] Routing binding (JSON or flat arrays) — _`solve_routes` takes JSON (optional time windows, no service-time override); parity `routing_basic`_
- [x] Orderbook binding (events in, trades and stats out) — _`OrderBook.apply` (atomic batch) and `snapshot`; parity `orderbook_batch`_
- [x] Swarm binding (avoid, find_path) — _JSON entry points `avoid` and `find_path`; parity `swarm_avoid`, `swarm_find_path`_
- [x] Consensus binding: seeded simulator only, no live replicas — _`simulate_consensus` only; no replica handle exposed; parity `consensus_*`_
- [x] WASM options objects with defaults matching Python; structured error kinds in both — _`IsolationForest.fit` and `Ekf` take options objects with Python's keys and defaults; errors carry `kind` (`TptAlgoError.kind` / `Error.kind`). EKF noise defaults are repeated in core `EKF_DEFAULTS` until the Phase 19 `NoiseConfig` item lands_
- [x] Python `.pyi`, `py.typed`, `__version__`; include via maturin — _package `tpt_algo` (`__init__.py`, `__init__.pyi`, `py.typed`) over `tpt_algo._native`; wheel contents checked with `maturin build`_
- [x] npm `package.json` metadata in the wasm-pack publish step — _`wasm-pack build` writes description, repository, license and keywords from `crates/tpt-algo-wasm/Cargo.toml`; publish workflow not changed; crate has no LICENSE file yet (wasm-pack warns)_
- [x] Parity tests running the same fixtures through Rust, Python and WASM — _`internal/tpt-algo-bindings-core/fixtures/parity.json`, run by `tests/parity.rs` (core), `tests/test_parity.py`, `tests/parity.rs` (wasm, Node)_

## Phase 21 — `tpt` CLI (`crates/tpt-algo-cli`)
- [x] Subcommands: soc, fraud, orderbook, route, swarm, consensus (CSV/JSON in, JSON/CSV out) — _note: all six in `crates/tpt-algo-cli`; `consensus` takes options only (seed and config), not an input file; OCV curve for `soc` is fixed at 11 points; see docs/cli.md_
- [x] `tpt demo <crate>` with bundled synthetic data — _note: small hand-made datasets embedded from `crates/tpt-algo-cli/data/demo/` (not generated via synth-data); deterministic_
- [x] `tpt fraud explain`: per-feature contribution to a score (by perturbation) — _note: contribution = score minus score with the feature replaced by its training mean or median; sorted by magnitude_
- [x] `--record <trace.json>` and `tpt replay` with byte-identical verification — _note: FNV-1a digest plus re-run from recorded inputs; exit 3 on mismatch; tests cover edited input and edited output; docs/record-format.md_
- [x] Browser demo page (WASM) for fraud and soc, deployed with the mdBook — _note: `docs/demo/` (static page, `just demo` builds `docs/demo/pkg`, generated and gitignored) and `docs/demo.md` in the book. Checked in headless Edge via Playwright: both demos run with no console errors; the page must be served over HTTP. Not yet deployed anywhere, as there is no deploy workflow._
- [x] CLI docs page and record/replay format page — _note: docs/cli.md and docs/record-format.md, both listed in docs/SUMMARY.md_

## Phase 22 — Per-crate benchmarks
- [~] Criterion benches for soc, fraud, orderbook, routing (3 sizes), swarm, consensus using synth-data — _note: all six targets added in `benches/benches/` and compile (`cargo bench -p tpt-algo-benches --no-run`); clippy clean; `soc` run with `--quick` only. Routing has 3 sizes (50, 200, 500) for both savings and solve. `consensus` does not consume synth-data: the simulator generates its own seeded workload._
- [x] Scheduled CI bench run publishing numbers as an artifact — _note: `.github/workflows/bench.yml` (weekly cron plus workflow_dispatch) uploads `target/criterion/`. YAML parses; the first scheduled run has not happened yet._
- [~] Results table in each crate README — _note: "Benchmarks" sections added to the six crate READMEs with placeholder rows reading "pending first scheduled run". No timings are recorded: none were measured in a way that should be published._

## Phase 23 — Dev tooling and adoption docs
- [x] `xtask` or `justfile`: check, ci, fetch-data, bench, bindings-test, book — _note: chose `justfile` (no new dependency; xtask would be a new workspace member). `just` is not installed on this machine, so the recipes were not run through `just`; the commands were checked individually. PowerShell equivalents are in CONTRIBUTING.md._
- [x] Root README: badges, 30-second quickstart per language (Rust, Python, JS, CLI) — _note: quickstarts build from the repo because crates are not yet on crates.io, PyPI or npm. The CLI demo was run (`tpt demo soc`, `fraud`, `route --format csv`)._
- [x] Cookbook page and "choose your crate" table — _docs/cookbook.md; mdBook builds._
- [~] Update CHANGELOGs for every change above — _note: Unreleased "Documentation" entries added to the six algorithm crates and `benches/`. Source changes from Phases 19-21 (soc, fraud, consensus, py, wasm, bindings-core, synth-data, cli) are not summarised yet; those entries belong with the phase that finishes them. `tpt-algo-cli` has no CHANGELOG.md yet._
