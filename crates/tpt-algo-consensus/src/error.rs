// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type for the consensus crate.

/// Errors produced by configuration, message validation and the commit rule.
///
/// Validation errors (`NotLeader`, `InvalidQuorumCert`, `BlockIdMismatch`, ...)
/// mean a message from the network was rejected and the replica state was not
/// changed. [`ConsensusError::ConflictingCommit`] is different: it means the
/// replica detected that a commit would contradict its own committed log. It
/// should never happen while at most `f` replicas are faulty; observing it
/// indicates a bug or an assumption violation (for example more than `f`
/// faulty replicas, or forged quorum certificates).
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tpt_algo_consensus::{Config, ConsensusError};
///
/// // Validation errors carry the offending values, so callers can report them.
/// match Config::new(2) {
///     Err(ConsensusError::TooFewReplicas { .. }) => {}
///     other => panic!("unexpected: {other:?}"),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ConsensusError {
    /// Fewer than four replicas were requested (`n >= 3f + 1` with `f >= 1`).
    #[error("at least 4 replicas are required, got {n}")]
    TooFewReplicas {
        /// Requested replica count.
        n: u16,
    },
    /// More replicas than the fixed-capacity signer set supports.
    #[error("at most {max} replicas are supported, got {n}")]
    TooManyReplicas {
        /// Requested replica count.
        n: u16,
        /// Supported maximum.
        max: u16,
    },
    /// The base timeout was zero or the back-off shift was too large.
    #[error("invalid timeout configuration")]
    InvalidTimeout,
    /// A simulator or configuration parameter was out of range.
    #[error("invalid parameter: {what}")]
    InvalidParameter {
        /// Which parameter was rejected.
        what: &'static str,
    },
    /// A replica id outside `0..n` was used.
    #[error("replica {id} is not a member of the group")]
    UnknownReplica {
        /// Offending id.
        id: u16,
    },
    /// A proposal was not sent or not signed by the leader of its view.
    #[error("replica {sender} is not the leader of view {view}")]
    NotLeader {
        /// View of the proposal.
        view: u64,
        /// Claimed proposer or actual sender.
        sender: u16,
    },
    /// A quorum certificate is structurally invalid (for example view 0 for a
    /// non-genesis block, or signers outside the group).
    #[error("quorum certificate is structurally invalid")]
    InvalidQuorumCert,
    /// A quorum certificate has too few distinct signers.
    #[error("quorum certificate has {have} signers, {need} required")]
    InsufficientSigners {
        /// Distinct signers present.
        have: u16,
        /// Quorum size.
        need: u16,
    },
    /// A block id does not match the digest of the block contents.
    #[error("block id does not match block contents")]
    BlockIdMismatch,
    /// A block's parent differs from the block certified by its justify QC.
    #[error("block parent differs from the block certified by its justify QC")]
    BrokenParentLink,
    /// A block height is not its parent's height plus one.
    #[error("block height is inconsistent with its parent")]
    InvalidHeight,
    /// A block view does not exceed the view of its justify QC (or parent).
    #[error("block view must exceed the view of its justify QC")]
    NonIncreasingView,
    /// A quorum certificate does not certify the block or view expected.
    #[error("quorum certificate does not certify the expected block")]
    QcMismatch,
    /// Blocks do not form a direct chain with consecutive views.
    #[error("blocks do not form a direct consecutive-view chain")]
    NotDirectChain,
    /// A commit would conflict with the already committed log (safety violation).
    #[error("safety violation: conflicting commit at height {height}")]
    ConflictingCommit {
        /// Height at which the conflict was detected.
        height: u64,
    },
    /// An ancestor needed by the commit rule is missing from the block store.
    #[error("ancestor block missing from store")]
    MissingAncestor,
    /// An integer counter would overflow.
    #[error("arithmetic overflow")]
    Overflow,
}
