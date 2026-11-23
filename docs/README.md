# tpt-algorithms documentation

This book collects the project's policies and conventions. The crate APIs are documented on
docs.rs once published; until then, run `cargo doc --workspace --no-deps`.

Principles:

- **Permissive only.** Every crate is `MIT OR Apache-2.0`; copyleft and Apache-only
  dependencies are rejected by `cargo deny`.
- **Clean-room.** Algorithms are implemented from published papers and specifications.
- **`no_std` by default** where the domain allows it.
- **Validation first.** Each algorithm has a reproducible validation path against public data or a
  seeded synthetic benchmark.
