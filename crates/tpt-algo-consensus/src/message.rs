// SPDX-License-Identifier: MIT OR Apache-2.0
//! Protocol messages.
//!
//! Messages carry no signatures. The sender identity is supplied by the
//! transport (in this crate, the simulator) and is trusted.

use crate::block::{Block, BlockId, QuorumCert};
use crate::types::{ReplicaId, View};

/// A vote for `block` proposed in `view`. The voter is the message sender.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Recipient {
    /// Every other replica.
    All,
    /// A single replica.
    One(ReplicaId),
}

/// A message a replica asks the transport to send.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Outgoing {
    /// Destination.
    pub to: Recipient,
    /// Payload.
    pub message: Message,
}
