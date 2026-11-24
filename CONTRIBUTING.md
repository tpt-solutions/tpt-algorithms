# Contributing

Thanks for helping. Contributions are accepted as issues only: report bugs, ask questions and propose
changes on the GitHub issue tracker. Pull requests are not accepted; maintainers make the changes.

Please read these first:

- [docs/clean-room.md](docs/clean-room.md): algorithms are implemented from papers and specs only.
- [docs/dependencies.md](docs/dependencies.md): how dependencies are accepted (license rule).
- [docs/testing.md](docs/testing.md): test tiers and validation conventions.
- [docs/crate-template.md](docs/crate-template.md): layout for a new crate.

## Checks the maintainers run

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

CI runs these plus the no_std build, the feature matrix, the MSRV build and `cargo doc` with
warnings denied.

## Developer tasks

The [justfile](justfile) wraps the common tasks: `just check`, `just ci`, `just fetch-data`,
`just bench`, `just bindings-test`, `just demo` and `just book`. Install `just` with `cargo install just` or
`winget install Casey.Just`. Recipes run through `sh`, so on Windows use Git Bash or make sure
Git for Windows' `sh` is on `PATH`.

Without `just`, from PowerShell or any shell, run the commands directly:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --no-default-features
cargo deny check
cargo bench -p tpt-algo-benches                       # bench (or add --bench <name>)
mdbook build                                          # book
```

`fetch-data` is a bash script (`scripts/fetch-dataset.sh <url> <sha256> <name>`); run it from Git
Bash. `bindings-test` needs maturin and wasm-pack; see [docs/bindings.md](docs/bindings.md).

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
