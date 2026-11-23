# tpt-algo-consensus

Deterministic Byzantine fault tolerant (BFT) consensus in the HotStuff family
(chained HotStuff), with a seeded network simulator for testing. Integer
arithmetic only (no floating point), `no_std`, no `unsafe`.

Clean-room implementation from the published description in Yin, Malkhi,
Reiter, Gueta, Abraham, *HotStuff: BFT Consensus with Linearity and
Responsiveness*, PODC 2019 (background: Castro and Liskov, *Practical
Byzantine Fault Tolerance*, OSDI 1999).

## Status

Pre-release (0.0.0). Research and testing quality, **not production ready**.

Implemented:

- `Config`: `n >= 4`, `f = (n - 1) / 3`, quorum `n - f` (`2f + 1` for `n = 3f + 1`).
- Blocks, quorum certificates (signer bitset), round-robin leaders (`v mod n`).
- `Replica`: chained HotStuff vote rule (`safeNode`), lock rule, three-chain
  commit rule over consecutive views, pacemaker with exponential back-off,
  `NewView` view change with `(f + 1)` view synchronisation, block fetching.
- Type-state `Round<Prepare | PreCommit | Commit | Decide>`: illegal phase
  transitions do not compile; only `Round<Decide>` reaches the commit path.
- Runtime-checked network entry `Replica::handle` returning `ConsensusError`.
- `sim`: deterministic simulator (SplitMix64-seeded delays, reordering, drops,
  global stabilisation time, crash faults, equivocating Byzantine leaders that
  also double-vote and inflate view numbers).

## Limitations

- **No cryptography.** Messages and quorum certificates are not signed or
  verified. The sender identity given to `Replica::handle` is trusted and QC
  signer sets are taken at face value. The simulator authenticates senders
  in place of signatures, and its Byzantine replicas cannot forge votes. A
  real deployment needs authenticated channels and signature or
  threshold-signature verification of every QC. **Do not treat this crate as
  secure against real adversaries.**
- Block ids are a 64-bit non-cryptographic digest (not collision resistant
  against an adversary).
- No persistence, crash recovery, checkpointing, state transfer beyond
  single-block fetches, or pruning of old blocks (memory grows with the chain).
- View change broadcasts `NewView` to all replicas (quadratic in failed views,
  since there are no threshold signatures); the happy path is linear.
- Round-robin leaders only (no leader reputation).
- Safety and liveness are tested, not formally verified.

## Feature flags

- `std` (default): implies `alloc`.
- `alloc`: enables `Replica` and the `sim` module. Without it the crate
  provides the core types, QC validation and the type-state `Round`, and builds
  for bare-metal targets (checked on `thumbv7em-none-eabihf`).
- `serde`: `Serialize`/`Deserialize` for ids, views, signer sets, blocks, QCs
  and messages.

## Usage

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

To drive replicas from your own transport, call `Replica::start`, then
`Replica::handle` for each received message and `Replica::tick` periodically,
and send the returned `Outgoing` messages. See `examples/simulate.rs`.

## Validation status

Run under `cargo test`:

- Known-answer scenario: four replicas, no faults, unit delays, 20 ticks; the
  exact committed block sequence (ids, views, proposers, QC signer sets) and
  per-replica log lengths are asserted.
- Scenarios with crash faults (from the start and mid-run), an equivocating
  leader, two faulty replicas out of seven, and a lossy network before GST.
- Proptest safety invariant (64 cases by default; 3000 cases were also run
  locally): with up to `f` crashing or equivocating replicas under random
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

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
