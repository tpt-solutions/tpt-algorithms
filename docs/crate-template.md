# Crate template

Written for: contributors adding a crate under `crates/` (public) or `internal/` (private).

## Layout

```
crates/tpt-algo-<name>/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── examples/<name>.rs
├── src/
│   ├── lib.rs      # #![cfg_attr(not(feature = "std"), no_std)] + module docs
│   ├── error.rs    # thiserror error enum
│   └── ...
├── tests/
│   └── validation.rs   # #[ignore] dataset tests (optional)
```

Clean-room notes and provenance live outside the crate, in `docs/clean-room-notes/<name>.md` and
`docs/provenance/<name>.md`.

## Cargo.toml

```toml
[package]
name = "tpt-algo-<name>"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "..."
readme = "README.md"
keywords = ["...", "..."]     # up to 5, each at most 20 characters
categories = ["algorithms"]   # slugs from https://crates.io/categories

[features]
default = ["std"]
std = ["alloc"]
alloc = []
serde = ["dep:serde"]

[dependencies]
thiserror.workspace = true
serde = { workspace = true, optional = true }

[dev-dependencies]
proptest.workspace = true

[lints]
workspace = true
```

Add `[package.metadata.docs.rs] all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`
before the first publish (Phase 15).

## lib.rs header

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
```

## Error type

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("...")]
    Variant,
}
```

No `unwrap`, `expect` or `panic!` in non-test code (enforced by workspace clippy lints).

## Checklist for a new crate

- [ ] Added to `members` in the root `Cargo.toml`.
- [ ] `publish = false` for internal crates.
- [ ] `cargo build --no-default-features` and the thumbv7em target build pass.
- [ ] Clean-room notes and provenance entry written.
- [ ] README with purpose, status, features, installation, usage snippet, API overview, validation status, limitations, references and licence.
- [ ] `CHANGELOG.md` in Keep a Changelog format, starting with an `[Unreleased]` section.
- [ ] `keywords` (up to 5) and `categories` (valid crates.io slugs) in `Cargo.toml`.
