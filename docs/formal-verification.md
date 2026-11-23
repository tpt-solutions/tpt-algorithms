# Formal verification and contracts

Written for: maintainers and reviewers of safety-relevant code.

## Approach

1. **Type-state** makes invalid protocol or lifecycle states unrepresentable. Used in
   `tpt-algo-soc` (filter lifecycle) and `tpt-algo-consensus` (protocol phases).
2. **Contracts** state the invariants a function relies on or guarantees. They are checked in debug and
   test builds and compiled out of release builds.
3. **Bounded model checking** (Kani) proves properties of one function, within stated bounds. It runs in
   CI only; see below.

prusti is not adopted because it is unmaintained (decision D10 in [decisions.md](decisions.md)).

## Contracts

`tpt_algo_math_core::contract` (internal crate) provides `requires!` and `ensures!` macros that expand to
`debug_assert!`. Public crates cannot depend on that internal crate (decision D5), so they use `debug_assert!`
directly with the same message format (`precondition failed: ...` / `postcondition failed: ...`).

Contracts in place:

- `tpt-algo-soc` (`filter.rs`): state of charge in `[0, 1]` on entry to `predict`; covariance diagonal
  non-negative before and after `predict` and after `update`.
- `tpt-algo-orderbook` (`book.rs`): after every event, `accepted == resting + 2 * traded + cancelled +
  expired`.
- `tpt-algo-routing` (`local_search.rs`): after every applied move, every route respects capacity and time
  windows.

Not yet covered: fraud scores in `[0, 1]`, and consensus quorum `2f + 1` for `n = 3f + 1` (both are
checked by tests, not by contracts).

## Kani (CI only)

Kani is run only in CI (`.github/workflows/kani.yml`), never locally. It is heavy: it downloads CBMC and
symbolically executes each harness. The job is set up to run only when it is useful:

- It runs when `internal/tpt-algo-geometry/**` or the workflow changes, weekly, or on manual dispatch.
- Each harness is its own matrix job, so the cheap ones are not held up by the arithmetic one.
- Kani is pinned to 0.68.0 and the action to v1.1, so results change only when the code does.

Harnesses live in `internal/tpt-algo-geometry/src/kinematics.rs` under `#[cfg(kani)]`. They cover
`constant_accel_position`:

| Harness | Property |
|---|---|
| `verification::negative_time_is_rejected` | For any positions, velocities and acceleration, `t < 0` gives `Err(NegativeTime)`. |
| `verification::non_finite_input_is_rejected` | With `t >= 0` and any non-finite position, velocity or acceleration, the result is `Err(NonFinite)`. |
| `verification::bounded_inputs_give_finite_position` | For inputs bounded by 1000 in magnitude and `t` in `[0, 1000]`, the result is `Ok` and finite. |

The third property needs its bound. Without one, `x0 + v0 t + a t^2 / 2` can overflow to infinity while the
function still returns `Ok`, so "always finite" is false. The bound is a precondition of the harness.

**Status:** the harnesses are written but have not yet run; the first run is the CI matrix on the first
pull request or dispatch that touches the geometry crate. Record the result here once it has run. If a
harness fails, record the counterexample and either fix the code or tighten the stated precondition.

The first CI run also confirms that `--harness verification::<name>` selects the harness. If Kani reports
that no harness matched, change the matrix entries to the bare function names.
