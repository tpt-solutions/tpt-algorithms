# Formal verification and contracts

Written for: maintainers and reviewers of safety-relevant code.

## Approach

1. **Type-state** makes invalid protocol or lifecycle states unrepresentable. Used in
   `tpt-algo-soc` (filter lifecycle) and `tpt-algo-consensus` (protocol phases).
2. **Contracts** state `requires` and `ensures` conditions. They are checked in debug builds and
   compiled out in release builds, so they cost nothing in production.
3. **Machine-checked proofs** are optional and applied to one critical function at a time (see the
   spike below).

prusti is not adopted because it is unmaintained (decision D10 in [decisions.md](decisions.md)).

## Contract helpers

`tpt_algo_math_core::contract` (internal crate) provides `requires!` and `ensures!` macros that
expand to `debug_assert!` with the condition text in the message. Public crates do not depend on
the internal crate, so each public crate that needs them carries a copy of the two macros.

## Kani spike (todo Phase 14)

Status: **not run in this session.** Kani (`cargo kani`) needs its own installer and a CBMC
download, which were not available here. The candidate function is the constant-acceleration
position update in `tpt-algo-geometry::kinematics::constant_accel_position`, or the Clarke-Wright
merge feasibility check in `tpt-algo-routing`. A maintainer should install Kani, run the proof
harness, and record the result here.

## Applied contracts

Physical and logical limits to check with contracts:

- Battery state of charge stays in `[0, 1]` (`tpt-algo-soc`).
- Fraud scores stay in `[0, 1]` (`tpt-algo-fraud`).
- Order book quantities are conserved and the book is never crossed (`tpt-algo-orderbook`).
- Vehicle load does not exceed capacity (`tpt-algo-routing`).
- Consensus quorum is `2f + 1` for `n = 3f + 1` (`tpt-algo-consensus`).
