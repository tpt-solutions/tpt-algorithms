# Release process

Written for: maintainers.

## Versioning policy (todo Phase 5, spec §8.2)

- All crates stay on `0.x` until their validation tests pass against the published references.
- After a crate reaches 1.0, follow semver.
- Internal crates (`publish = false`) are never released; they are skipped in release-plz
  (`release = false` in `release-plz.toml`).

## Commit messages

Use Conventional Commits so release-plz can generate changelogs:

- `feat(<crate>): ...` for new features (minor bump while on 0.x).
- `fix(<crate>): ...` for bug fixes (patch).
- `feat!:` or a `BREAKING CHANGE:` footer for breaking changes.
- `docs:`, `test:`, `ci:`, `chore:` do not produce changelog entries by default.

## Automated flow

1. Merging a PR to `master` triggers [release-plz](../.github/workflows/release-plz.yml), which
   opens or updates a single release PR.
2. The release PR bumps versions and updates CHANGELOG files for changed `tpt-algo-*` crates.
3. Merging the release PR publishes the changed crates to crates.io in dependency order.

## Pre-release checklist

- [ ] `cargo deny check` and `cargo audit` pass.
- [ ] `cargo package --list -p <crate>` shows only intended files (no datasets, no `.datasets/`).
- [ ] `cargo publish --dry-run -p <crate>` succeeds for each public crate.
- [ ] docs.rs metadata set and `cargo doc` has no warnings.

## Checking crate names (todo Phase 0)

Read-only check against the crates.io index:

```sh
cargo info tpt-algo-routing
```

A "could not find" result suggests the name is free. Reserving names by publishing placeholders
is outward-facing and requires maintainer approval.
