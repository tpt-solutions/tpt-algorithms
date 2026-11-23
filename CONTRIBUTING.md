# Contributing

Thanks for helping. Please read these first:

- [docs/clean-room.md](docs/clean-room.md): algorithms are implemented from papers and specs only.
- [docs/dependencies.md](docs/dependencies.md): how dependencies are accepted (license rule).
- [docs/testing.md](docs/testing.md): test tiers and validation conventions.
- [docs/crate-template.md](docs/crate-template.md): layout for a new crate.

## Before you open a PR

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

CI runs these plus the no_std build, the feature matrix, the MSRV build and `cargo doc` with
warnings denied.

## Rules

- No `unwrap`, `expect` or `panic!` in non-test code.
- No `unsafe`.
- Every `.rs` file starts with `// SPDX-License-Identifier: MIT OR Apache-2.0`.
- Randomness takes an explicit seed.
- Do not commit datasets. Use `scripts/fetch-dataset.sh` and see [docs/datasets.md](docs/datasets.md).
- Commit messages follow Conventional Commits (see [docs/release.md](docs/release.md)).

## Licence of contributions

Contributions are dual-licensed under MIT OR Apache-2.0, as described in
[docs/licensing.md](docs/licensing.md).
