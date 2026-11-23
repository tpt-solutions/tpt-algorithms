// SPDX-License-Identifier: MIT OR Apache-2.0
//! # tpt-algo-consensus
//!
//! A deterministic, integer-only Byzantine fault tolerant (BFT) consensus
//! state machine in the HotStuff family (chained HotStuff), with a seeded
//! network simulator for testing.
//!
//! Clean-room implementation from the published description in
//! M. Yin, D. Malkhi, M. K. Reiter, G. Golan Gueta, I. Abraham,
//! *HotStuff: BFT Consensus with Linearity and Responsiveness*, PODC 2019.
//! Background: M. Castro, B. Liskov, *Practical Byzantine Fault Tolerance*,
//! OSDI 1999.
//!
//! ## Model
//!
//! * `n >= 4` replicas tolerate `f = (n - 1) / 3` Byzantine replicas; the
//!   quorum is `n - f` (`= 2f + 1` when `n = 3f + 1`). See [`Config`].
//! * Views are numbered from 0; view 0 holds the genesis block and the first
//!   proposal is made in view 1. The leader of view `v` is `v mod n`.
//! * Each [`Block`] extends a parent and carries a [`QuorumCert`] for it. A QC
//!   is a view, a block id and a [`SignerSet`] of at least `quorum` replicas.
//! * Commit rule: when blocks `b0 <- b1 <- b2` are a direct chain in
//!   consecutive views and `b2` is certified, `b0` (and its uncommitted
//!   ancestors) is committed. Committed blocks are final.
//! * Phases are tracked with the type-state [`Round`]
//!   (`Prepare -> PreCommit -> Commit -> Decide`); illegal transitions do not
//!   compile, and only a `Round<Decide>` reaches the commit path.
//! * Network input goes through the runtime-checked [`Replica::handle`].
//!
//! Only integer arithmetic is used (no floating point), so execution is
//! bit-for-bit reproducible across platforms.
//!
//! ## Limitations (read before use)
//!
//! * **No cryptography.** Messages and quorum certificates are not signed or
//!   verified. The sender identity passed to [`Replica::handle`] is trusted,
//!   and the signer set of a QC is taken at face value. A real deployment needs
//!   authenticated channels and signature (or threshold-signature)
//!   verification of every QC. This crate is NOT secure against real
//!   adversaries.
//! * Block ids are a 64-bit non-cryptographic digest; collisions are not
//!   adversarially hard.
//! * No persistence, state transfer beyond single-block fetches, checkpointing
//!   or garbage collection of old blocks; memory grows with the chain.
//! * View change broadcasts `NewView` to all replicas (quadratic in failed
//!   views) to synchronise views without threshold signatures; the happy path
//!   is linear (votes go to the next leader only).
//! * Safety and liveness are tested (scripted scenarios, property tests,
//!   seeded simulations), not formally verified.
//!
//! ## Features
//!
//! * `std` (default): implies `alloc`.
//! * `alloc`: enables [`Replica`] and the [`sim`] module. Without it only the
//!   core data types, QC validation and the type-state [`Round`] are available.
//! * `serde`: `Serialize`/`Deserialize` for ids, views, blocks, QCs and
//!   messages.
//!
//! ## Example
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use tpt_algo_consensus::sim::{check_prefix_consistency, SimConfig, Simulator};
//!
//! let mut sim = Simulator::new(SimConfig::new(4, 42)).unwrap();
//! sim.run(40);
//! let report = sim.report();
//! check_prefix_consistency(&report.honest_logs).unwrap();
//! assert!(report.honest_logs.iter().all(|(_, log)| !log.is_empty()));
//! # }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod block;
mod error;
mod message;
pub mod phase;
#[cfg(feature = "alloc")]
mod replica;
#[cfg(feature = "alloc")]
pub mod sim;
mod types;

pub use block::{Block, BlockId, QuorumCert, GENESIS_ID};
pub use error::ConsensusError;
pub use message::{Message, NewView, Outgoing, Recipient, Vote};
pub use phase::{Commit, Decide, Phase, PreCommit, Prepare, Round};
#[cfg(feature = "alloc")]
pub use replica::Replica;
pub use types::{Config, ReplicaId, SignerSet, View, MAX_REPLICAS};
