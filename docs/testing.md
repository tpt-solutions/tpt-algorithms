# Testing and validation

Written for: contributors writing tests for an algorithm crate.

## Tiers

| Tier | Where | Runs in CI | Purpose |
|---|---|---|---|
| Unit | `#[cfg(test)] mod tests` in `src/` | yes | Individual functions and edge cases. |
| Known-answer | unit tests with hand-computed values | yes | Small instances with exact expected output. |
| Property (proptest) | `#[cfg(test)]` with `proptest!` | yes | Invariants: bounds, conservation, determinism. |
| Synthetic validation | `tests/*.rs` (not ignored) | yes | Seeded synthetic scenarios with pass/fail criteria. |
| Dataset validation | `tests/validation.rs`, `#[ignore]` | no (manual) | Public datasets. Needs `TPT_DATASET_DIR`. |

Run the dataset tier explicitly:

```sh
TPT_DATASET_DIR=/path/to/datasets cargo test -p tpt-algo-<name> -- --ignored
```

## Determinism

- Every stochastic routine takes an explicit seed.
- `tpt-algo-math-core::rng::SplitMix64` is the shared generator for internal crates. Public crates
  keep a private copy (see [decisions.md](decisions.md), D5).
- Tests must not depend on wall-clock time or thread scheduling.

## Tolerances

Use `tpt-algo-math-core::tolerance` (internal) for relative and absolute comparisons. Choose the
tolerance from the source's stated bound, not from what the code happens to produce.

## proptest conventions

- Default config (256 cases). Keep strategies small enough that a full run stays under a few
  seconds per crate.
- Name properties after the invariant, for example `book_is_never_crossed`.
- Store regressions in `proptest-regressions/` and commit them.

## Fixtures

Synthetic fixtures come from `tpt-algo-synth-data` (dev-dependency only). No third-party data is
committed to the repository.
