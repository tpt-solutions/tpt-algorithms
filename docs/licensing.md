# Licensing

Written for: maintainers and contributors who need to know what license applies to what.

## Project license

All first-party code is dual-licensed under **MIT OR Apache-2.0**, at the licensee's option:

- [LICENSE-MIT](../LICENSE-MIT)
- [LICENSE-APACHE](../LICENSE-APACHE)

Copyright is held by TPT Solutions. The `license = "MIT OR Apache-2.0"` field is set once in
`[workspace.package]` and inherited by every crate.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project is licensed under MIT OR Apache-2.0, without any additional terms or conditions.

## NOTICE file

Not required. Apache-2.0 requires a NOTICE only when redistributing Apache-licensed code that
carries one. Every third-party dependency is consumed as a crate (not vendored), and the project
has no vendored Apache-2.0 code. Revisit if any source is copied from an Apache-2.0 project.

## SPDX headers

Every `.rs` source file begins with:

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
```

Markdown and TOML files do not carry headers; the root license files cover them.

## Dependency license rule

Summary (full process in [dependencies.md](dependencies.md)):

- Allowed: MIT, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-DFS-2016, Unicode-3.0.
- Allowed by election: `MIT OR Apache-2.0` (the MIT branch is taken).
- Banned: Apache-2.0-only, GPL, AGPL, LGPL, SSPL, EUPL, CC-BY-NC, and unlicensed crates.
- Enforced by `cargo deny check` using [deny.toml](../deny.toml) in CI.

## Data licensing

Public datasets (NASA Battery, LOBSTER, Solomon/CVRPLIB, and others) are never committed to the
repository. They are fetched on demand and their terms are recorded in [datasets.md](datasets.md).
