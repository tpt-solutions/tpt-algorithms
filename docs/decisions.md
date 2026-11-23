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
| D7 | Python / WASM bindings (Phase 16) | **No-go for v0.1.** Deferred to a post-v0.1 milestone. | Keeps v0.1 scope to Rust crates. Revisit once the core crates pass their validation. |
| D8 | Dependency update bot | Dependabot (`.github/dependabot.yml`) | Native to GitHub; no extra app. |
| D9 | Versioning | 0.x until validated; semver afterwards | Spec §8.2 / todo Phase 5. |
| D10 | Formal verification | Contract helpers (debug assertions) for all crates; kani spike on one function (see [formal-verification.md](formal-verification.md)); prusti not adopted | prusti is unmaintained. |

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
