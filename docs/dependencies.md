# Dependency governance

Written for: contributors adding or updating a dependency.

## Policy

See [licensing.md](licensing.md) for the license rule. In short: a dependency is acceptable only if
its license expression can be satisfied by MIT, BSD-2/3-Clause, ISC, Zlib, Unicode-DFS-2016 or
Unicode-3.0 (including `X OR Apache-2.0` where the MIT branch is elected).

## Process for adding a dependency

1. **Justify it.** Prefer `core`/`alloc`/`std`. Spec §5.3 requires that a crate be justified
   before it is added.
2. **Check the license tree.** Run `cargo deny check licenses` after adding it.
3. **Check advisories.** Run `cargo audit`.
4. **Add it to `[workspace.dependencies]`** in the root `Cargo.toml` and reference it with
   `.workspace = true` from member crates. One version per dependency across the workspace.
5. **Set `default-features = false`** unless the default features are needed and no_std-safe.
6. **Re-run the no_std build** for affected crates:
   `cargo build -p <crate> --no-default-features --target thumbv7em-none-eabihf`.
7. **Record the decision** in the PR description: what it is used for, license, and why no
   alternative was used.

## If a dependency fails the license rule

- Find an MIT or dual-licensed alternative, or
- implement the needed logic inside the crate (for example `libm` covers the math that would
  otherwise need a dependency), or
- stop and raise it with the maintainer. Do not add a copyleft or Apache-only dependency.

## Current accepted dependencies

| Crate | Used by | License |
|---|---|---|
| `libm` | math-core, geometry, synth-data, all algorithm crates | MIT OR Apache-2.0 |
| `thiserror` | all crates (error types) | MIT OR Apache-2.0 |
| `serde` | optional `serde` feature | MIT OR Apache-2.0 |
| `proptest` | dev-dependency (property tests) | MIT OR Apache-2.0 |
