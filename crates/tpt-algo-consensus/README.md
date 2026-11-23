# tpt-algo-consensus

Deterministic Byzantine fault tolerant (BFT) consensus in the HotStuff family
(chained HotStuff), with a seeded network simulator for testing. The state
machine uses integer arithmetic only (no floating point), is `no_std`
compatible, and contains no `unsafe` code.

This crate is a clean-room implementation of the protocol described in Yin,
Malkhi, Reiter, Gueta and Abraham, *HotStuff: BFT Consensus with Linearity and
Responsiveness* (PODC 2019). Background reading is Castro and Liskov,
*Practical Byzantine Fault Tolerance* (OSDI 1999).

Use it to study, teach or prototype a BFT protocol, or to test a deployment
design under controlled faults. It is not a production consensus library.

## Status

Pre-release (`0.0.0`). Research and testing quality, **not production ready**.
The crate is not yet published to crates.io; depend on it by path or git until
the first release. See the [changelog](CHANGELOG.md).

Implemented:

- `Config`: `n >= 4` replicas, `f = (n - 1) / 3` tolerated faults, quorum
  `n - f` (`2f + 1` when `n = 3f + 1`).
- Blocks, quorum certificates (QCs) with a signer bitset, and round-robin
  leaders (`leader(v) = v mod n`).
- `Replica`: chained HotStuff vote rule (`safeNode`), lock rule, three-chain
  commit rule over consecutive views, pacemaker with exponential back-off,
  `NewView` view change with `f + 1` view synchronisation, and single-block
  fetching.
- Type-state `Round<Prepare | PreCommit | Commit | Decide>`: illegal phase
  transitions do not compile, and only `Round<Decide>` reaches the commit path.
- A runtime-checked network entry point, `Replica::handle`, which returns
  `ConsensusError` for malformed or invalid input.
- `sim`: a deterministic simulator with SplitMix64-seeded delays, reordering,
  drops, global stabilisation time (GST), crash faults, and equivocating
  Byzantine leaders that also double-vote and inflate view numbers.

## Features

| Feature | Default | Enables |
|---------|---------|---------|
| `std`   | yes     | Implies `alloc`. Uses the standard library. |
| `alloc` | via `std` | `Replica` and the `sim` module. Without it the core types, QC validation and the type-state `Round` remain. |
| `serde` | no      | `Serialize` / `Deserialize` for ids, views, signer sets, blocks, QCs and messages. |

Without `alloc` the crate builds for bare-metal targets. This is checked on
`thumbv7em-none-eabihf`.

## Installation

```toml
[dependencies]
tpt-algo-consensus = { path = "../tpt-algo-consensus" }             # std (default)
# tpt-algo-consensus = { path = "...", default-features = false }   # core types only
```

The minimum supported Rust version is `1.98`, matching the workspace.

## Quick start

Run a four-replica simulation with one equivocating replica and check that
honest replicas never commit conflicting blocks:

```rust
use tpt_algo_consensus::sim::{check_prefix_consistency, FaultKind, FaultSpec, SimConfig, Simulator};
use tpt_algo_consensus::{ConsensusError, ReplicaId};

fn run() -> Result<(), ConsensusError> {
    let config = SimConfig {
        faults: vec![FaultSpec { replica: ReplicaId(1), kind: FaultKind::Equivocate }],
        ..SimConfig::new(4, 7)
    };
    let mut sim = Simulator::new(config)?;
    sim.run(500);
    let report = sim.report();
    check_prefix_consistency(&report.honest_logs)?; // no conflicting commits
    Ok(())
}
```

## Using your own transport

The simulator is optional. To drive replicas from your own network layer:

1. Create a `Replica` with `Replica::new(id, config)`.
2. Call `Replica::start(now)` and send the returned `Outgoing` messages.
3. For each received message, call `Replica::handle(...)` and send its output.
4. Call `Replica::tick(now)` periodically to drive timeouts.
5. Read decided blocks from `Replica::committed()`. Use `Replica::submit` to
   queue client commands.

Note that `handle` trusts the sender identity you pass in (see
[Limitations](#limitations)).

## API overview

| Item | Purpose |
|------|---------|
| `Config`, `MAX_REPLICAS` | Replica count, fault bound and quorum. |
| `ReplicaId`, `View`, `SignerSet` | Identifiers and signer bitsets. |
| `Block`, `BlockId`, `GENESIS_ID` | Chain blocks and their 64-bit digests. |
| `QuorumCert` | A view, block id and signer set with at least a quorum of signers. |
| `Message`, `Vote`, `NewView`, `Outgoing`, `Recipient` | Protocol messages and addressed output. |
| `Replica` | The replica state machine (`new`, `start`, `handle`, `tick`, `submit`, `committed`, `block`). |
| `phase::{Prepare, PreCommit, Commit, Decide, Round}` | Type-state phase tracking. |
| `ConsensusError` | All error cases returned by the public API. |
| `sim::{Simulator, SimConfig, SimReport, FaultSpec, FaultKind}` | Seeded network simulator. |
| `sim::check_prefix_consistency` | Checks that honest logs agree on their common prefix. |

## Examples

```sh
cargo run -p tpt-algo-consensus --example simulate
```

## Validation status

Run under `cargo test`:

- Known-answer scenario: four replicas, no faults, unit delays, 20 ticks. The
  exact committed block sequence (ids, views, proposers, QC signer sets) and
  per-replica log lengths are asserted.
- Scenarios with crash faults (from the start and mid-run), an equivocating
  leader, two faulty replicas out of seven, and a lossy network before GST.
- Proptest safety invariant (64 cases by default; 3000 cases were also run
  locally). With up to `f` crashing or equivocating replicas, under random
  seeds, delays, reordering and drops, no two honest replicas commit different
  blocks at the same height, logs only grow by appending, and no honest message
  is rejected.
- Seeded liveness: after GST (synchronous delivery, no drops) every honest
  replica commits a new block within 24 views (48 seeds in the test suite;
  worst case 13 views over 1000 seeds checked locally).
- Unit tests for quorum intersection (`2q > n + f` for every supported `n`),
  lock-based vote refusal, message rejection and the type-state transitions.

There is no public dataset for BFT traces. `tests/validation.rs` is an
`#[ignore]`d hook that replays external scenario expectations from
`$TPT_DATASET_DIR/consensus/scenarios.txt`.

## Limitations

- **No cryptography.** Messages and quorum certificates are not signed or
  verified. The sender identity given to `Replica::handle` is trusted, and QC
  signer sets are taken at face value. The simulator authenticates senders in
  place of signatures, and its Byzantine replicas cannot forge votes. A real
  deployment needs authenticated channels and signature or threshold-signature
  verification of every QC. **Do not treat this crate as secure against real
  adversaries.**
- Block ids are a 64-bit non-cryptographic digest, not collision resistant
  against an adversary.
- No persistence, crash recovery, checkpointing, state transfer beyond
  single-block fetches, or pruning. Memory grows with the chain.
- The view change broadcasts `NewView` to all replicas. This is quadratic in
  the number of failed views because there are no threshold signatures. The
  happy path is linear.
- Round-robin leaders only; there is no leader reputation.
- Safety and liveness are tested, not formally verified.

## References

- M. Yin, D. Malkhi, M. K. Reiter, G. Golan Gueta, I. Abraham,
  *HotStuff: BFT Consensus with Linearity and Responsiveness*, PODC 2019.
- M. Castro, B. Liskov, *Practical Byzantine Fault Tolerance*, OSDI 1999.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
