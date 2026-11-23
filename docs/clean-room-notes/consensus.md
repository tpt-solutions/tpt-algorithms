# Clean-room notes: tpt-algo-consensus (HotStuff family)

These notes record what was taken from published descriptions and how it was
mapped onto the implementation. No third-party source code was consulted or
copied. The descriptions below were written from memory of the papers. A
maintainer should re-check them against the papers.

## Sources

1. M. Yin, D. Malkhi, M. K. Reiter, G. Golan Gueta, I. Abraham. *HotStuff: BFT
   Consensus with Linearity and Responsiveness*. PODC 2019 (arXiv:1803.05069).
2. M. Castro, B. Liskov. *Practical Byzantine Fault Tolerance*. OSDI 1999
   (background: `n = 3f + 1`, quorum intersection, view change).
3. G. Steele, D. Lea, C. Flood. *Fast Splittable Pseudorandom Number
   Generators*. OOPSLA 2014 (SplitMix64, used for the simulator RNG and the
   non-cryptographic block digest).

## Concepts used from the papers

- Replica group of `n >= 3f + 1`; quorums of `n - f` intersect in at least
  `f + 1` replicas, so in at least one honest replica (PBFT, HotStuff).
- Basic HotStuff phases prepare / pre-commit / commit / decide, each driven by
  a quorum certificate (QC) for the previous phase.
- Chained HotStuff: phases are pipelined. Every block carries a QC (`justify`)
  for an earlier block. A QC for block `b''` acts as the prepare QC for
  `b''`, the pre-commit QC for `b' = b''.justify.node` and the commit QC for
  `b = b'.justify.node`.
- `safeNode` vote predicate: vote for a proposal if it extends the locked
  block (safety rule) or its `justify` is newer than the lock (liveness rule).
  Vote at most once per view (`vheight`).
- Lock update: on a QC for `b''`, lock on `b'` if it is newer than the lock.
- Commit rule: if `b''.parent = b'` and `b'.parent = b` (a direct three-chain),
  commit `b`. The paper fills skipped views with dummy nodes so that
  "direct parent" means "consecutive height". This implementation does not
  create dummy blocks. It requires the three blocks to be in consecutive
  views, which is the equivalent condition (the "consecutive-view three-chain"
  used in later HotStuff-family descriptions).
- Leader rotation and pacemaker: a leader per view, a view timer, and
  `NewView` messages carrying the sender's highest QC (`genericQC`). The leader
  proposes after collecting `n - f` `NewView` messages or a fresh QC.

## Design decisions not dictated by the papers

- **Quorum is `n - f`**, not literally `2f + 1`. These are equal for
  `n = 3f + 1` and keep intersection correct for other `n`.
- **View 0 is genesis.** The genesis block is implicitly certified in view 0.
  The first proposal is made in view 1 by replica `1 mod n`.
- **Every block's parent equals `justify.block`.** No dummy blocks.
- **Votes go to the next leader** (linear happy path). In addition, a
  `NewView` carries the sender's last vote, so a QC can still form when the
  leader that should collect it is faulty. Without this, round-robin with
  `n = 4` and one crashed replica can never form a consecutive three-chain.
- **A leader may propose once a quorum of distinct replicas voted in the
  previous view on any block.** This covers an equivocating leader that split
  the votes, so that no QC and no `NewView` would otherwise arrive.
- **View synchronisation**: `NewView` is broadcast to all replicas, not only
  to the next leader. A replica that sees `f + 1` replicas reporting higher
  views jumps to the `(f + 1)`-th highest, which at least one honest replica
  has reached (a Bracha-style amplification idea). Timeouts back off
  exponentially (integer shift, capped).
- **Block fetching**: an orphan block triggers a fetch of its parent from the
  sender on every receipt, so lost requests are retried.
- **Type-state `Round<P>`** models the per-block phase progression
  (`Prepare -> PreCommit -> Commit -> Decide`). Each transition is checked at
  run time against the blocks and QCs. The replica's commit path accepts only
  `Round<Decide>`.
- **No signatures.** QCs are signer bitsets. This is an explicit limitation.
- **Integer-only arithmetic** throughout (no floating point), so execution is
  deterministic across platforms. Probabilities in the simulator are
  per-mille integers.

## Not implemented

Threshold signatures, signature verification, pipelined execution of
commands, leader reputation, persistence or recovery, state transfer beyond
single blocks, and garbage collection.
