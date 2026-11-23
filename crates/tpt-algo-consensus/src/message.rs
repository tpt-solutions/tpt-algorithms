// SPDX-License-Identifier: MIT OR Apache-2.0
//! Protocol messages.
//!
//! Messages carry no signatures. The sender identity is supplied by the
//! transport (in this crate, the simulator) and is trusted.

use crate::block::{Block, BlockId, QuorumCert};
use crate::types::{ReplicaId, View};

/// A vote for `block` proposed in `view`. The voter is the message sender.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::{BlockId, Vote, View};
///
/// let v = Vote { view: View(3), block: BlockId(9) };
/// assert_eq!(v.view, View(3));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vote {
    /// View of the voted block.
    pub view: View,
    /// Voted block.
    pub block: BlockId,
}

/// Sent when a replica enters `view` without a fresh certificate (after a
/// timeout or a view synchronisation jump).
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::{NewView, QuorumCert, View};
///
/// // Sent on timeout: the highest certificate seen, and the last vote cast, if any.
/// let nv = NewView { view: View(4), high_qc: QuorumCert::genesis(), last_vote: None };
/// assert!(nv.last_vote.is_none());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewView {
    /// View the sender has entered.
    pub view: View,
    /// Highest quorum certificate known to the sender.
    pub high_qc: QuorumCert,
    /// The sender's most recent vote, so that a certificate can still be
    /// formed if the leader that should have collected it is faulty.
    pub last_vote: Option<Vote>,
}

/// A protocol message.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::*;
///
/// // A quorum certificate for `b` signed by replicas 0, 1 and 2 (a quorum when n = 4).
/// fn cert(b: &Block) -> QuorumCert {
///     let mut signers = SignerSet::new();
///     for i in 0..3u16 {
///         signers.insert(ReplicaId(i)).unwrap();
///     }
///     QuorumCert { view: b.view, block: b.id, signers }
/// }
/// use tpt_algo_consensus::{Block, Message};
///
/// let b = Block::genesis();
/// let m = Message::Proposal(b.clone());
/// assert!(matches!(m, Message::Proposal(_)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Message {
    /// A leader's proposal for its view.
    Proposal(Block),
    /// A vote, sent to the leader of the next view.
    Vote(Vote),
    /// View-change notification.
    NewView(NewView),
    /// Request for a block the sender is missing.
    FetchBlock(BlockId),
    /// Response to [`Message::FetchBlock`].
    BlockData(Block),
}

/// Destination of an outgoing message.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::{Recipient, ReplicaId};
///
/// assert_eq!(Recipient::One(ReplicaId(2)), Recipient::One(ReplicaId(2)));
/// assert_ne!(Recipient::All, Recipient::One(ReplicaId(0)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Recipient {
    /// Every other replica.
    All,
    /// A single replica.
    One(ReplicaId),
}

/// A message a replica asks the transport to send.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::{Message, Outgoing, Recipient, BlockId};
///
/// let out = Outgoing { to: Recipient::All, message: Message::FetchBlock(BlockId(5)) };
/// assert_eq!(out.to, Recipient::All);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Outgoing {
    /// Destination.
    pub to: Recipient,
    /// Payload.
    pub message: Message,
}
