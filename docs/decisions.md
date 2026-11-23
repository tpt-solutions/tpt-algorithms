# Project decisions (Phase 0)

Written for: maintainers. Records each decision so it can be revisited with the reason.

| # | Decision | Chosen | Reason / status |
|---|---|---|---|
| D1 | Crate naming | `tpt-algo-*` prefix for all crates | Required by spec §3.2. |
| D2 | Edition | 2021 | Confirmed. |
| D3 | MSRV | **1.98** (stable minus 2 at time of writing; stable is 1.100) | Spec §4 policy "stable minus 2". Enforced by a CI MSRV job; `rust-version` set in `[workspace.package]`. |
| D4 | Default-no-std | Every crate `#![cfg_attr(not(feature = "std"), no_std)]`, features `std`, `alloc`, `serde` | Spec §4.1. |
| D5 | Internal crates may not be depended on by public crates | Public algorithm crates have no runtime dependency on `tpt-algo-math-core`, `-geometry` or `-synth-data` | Those are `publish = false`; a publishable crate cannot depend on an unpublished one. Shared code is therefore duplicated in small private helpers. Revisit if internal crates are later published. |
| D6 | Maintainer contact in crate metadata | **Open.** Not set. | Needs the maintainer's decision on which address to publish. |
| D7 | Python / WASM bindings (Phase 16) | **Go for v0.1** (reversed by the maintainer; previously no-go). Bindings are separate, unpublished crates `tpt-algo-py` (pyo3 + maturin) and `tpt-algo-wasm` (wasm-bindgen), covering `tpt-algo-soc` and `tpt-algo-fraud`. | The maintainer asked for bindings in v0.1. Binding crates depend on public crates, so they are never a dependency of a published crate. Publishing to PyPI and npm is set up as a manual-trigger workflow and is not run until the maintainer approves. |
| D8 | Dependency update bot | Dependabot (`.github/dependabot.yml`) | Native to GitHub; no extra app. |
| D9 | Versioning | 0.x until validated; semver afterwards | Spec §8.2 / todo Phase 5. |
| D11 | Kani verification host | **CI only** (Linux runner, path-filtered and scheduled, one harness per run) | Kani is heavy (CBMC download and bounded model checking). It is not run locally; the CI job is the only place it runs. |
| D10 | Formal verification | Contract helpers (debug assertions) for all crates; kani spike on one function (see [formal-verification.md](formal-verification.md)); prusti not adopted | prusti is unmaintained. |
| D12 | `target-lexicon` (pulled in by pyo3's build configuration) | Replaced via `[patch.crates-io]` with `tpt-solutions/target-lexicon-mit`, pinned to commit `297f442a`; `deny.toml` allows that git source only | Upstream is `Apache-2.0 WITH LLVM-exception`, which spec section 5.1 bans. The replacement is MIT-only, build-time only, and API-compatible for pyo3's use. |

## Items that need external action (not done in this repository)

These cannot be completed from the repository and were intentionally left open:

- Crate names: a read-only lookup of the crates.io index on 2026-11-23 found none of the nine names
  (`tpt-algo-soc`, `-fraud`, `-orderbook`, `-routing`, `-swarm`, `-consensus`, `-math-core`,
  `-geometry`, `-synth-data`). Reserving them by publishing placeholders is outward-facing and
  irreversible, so it awaits approval. Repeat the check before publishing (see [release.md](release.md)).
- Create the GitHub organisation and repository `tpt-algorithms` under TPT Solutions.
- Trademark check of "TPT" / "TPT Solutions" (legal review).
- Create a crates.io account and API token; store it as a GitHub Actions secret.
- Configure GitHub branch protection requiring all CI checks.
- Dry-run release-plz on a placeholder crate.
- v0.1 publish, tags, and announcement.
