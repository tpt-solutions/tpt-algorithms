// SPDX-License-Identifier: MIT OR Apache-2.0
//! Blocks and quorum certificates.

use crate::error::ConsensusError;
use crate::types::{Config, ReplicaId, SignerSet, View};

/// Block identifier: a 64-bit digest of the block contents.
///
/// The digest is a deterministic, non-cryptographic mixing function. It is
/// adequate for simulation and testing; it is NOT collision resistant against
/// an adversary (see the crate-level limitations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockId(pub u64);

/// Id of the genesis block.
pub const GENESIS_ID: BlockId = BlockId(0);

/// 64-bit finalizer from SplitMix64 (Steele, Lea, Flood 2014).
pub(crate) const fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const fn absorb(h: u64, v: u64) -> u64 {
    mix64(h ^ v).wrapping_add(0x9E37_79B9_7F4A_7C15)
}

/// A quorum certificate: proof that a quorum of replicas voted for `block` in
/// `view`.
///
/// Signatures are represented only by the signer set; this crate does not
/// verify cryptographic signatures (see the crate-level limitations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuorumCert {
    /// View in which the votes were cast (equals the certified block's view).
    pub view: View,
    /// Certified block.
    pub block: BlockId,
    /// Distinct voters.
    pub signers: SignerSet,
}

impl QuorumCert {
    /// The implicit certificate for the genesis block (view 0, no signers).
    #[must_use]
    pub const fn genesis() -> Self {
        Self {
            view: View::GENESIS,
            block: GENESIS_ID,
            signers: SignerSet::new(),
        }
    }

    /// Whether this certifies the genesis block.
    #[must_use]
    pub fn is_genesis(&self) -> bool {
        self.block == GENESIS_ID
    }

    /// Structural validation: the genesis QC must have view 0; any other QC
    /// must have a non-zero view and at least `quorum` distinct signers, all
    /// members of the group.
    ///
    /// # Errors
    /// [`ConsensusError::InvalidQuorumCert`] or
    /// [`ConsensusError::InsufficientSigners`].
    pub fn validate(&self, config: &Config) -> Result<(), ConsensusError> {
        if self.is_genesis() {
            return if self.view == View::GENESIS {
                Ok(())
            } else {
                Err(ConsensusError::InvalidQuorumCert)
            };
        }
        if self.view == View::GENESIS || !self.signers.all_below(config.n()) {
            return Err(ConsensusError::InvalidQuorumCert);
        }
        let have = self.signers.len();
        if have < config.quorum() {
            return Err(ConsensusError::InsufficientSigners {
                have,
                need: config.quorum(),
            });
        }
        Ok(())
    }
}

/// A block of the chained protocol. Each block extends `parent` and carries
/// `justify`, a quorum certificate for that parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    /// Digest of the remaining fields (see [`Block::compute_id`]).
    pub id: BlockId,
    /// Parent block; always equal to `justify.block`.
    pub parent: BlockId,
    /// View in which the block was proposed.
    pub view: View,
    /// Distance from genesis (genesis has height 0).
    pub height: u64,
    /// Proposer; must be the leader of `view`.
    pub proposer: ReplicaId,
    /// Opaque command carried by the block.
    pub payload: u64,
    /// Quorum certificate for the parent.
    pub justify: QuorumCert,
}

impl Block {
    /// The genesis block (view 0, height 0).
    #[must_use]
    pub const fn genesis() -> Self {
        Self {
            id: GENESIS_ID,
            parent: GENESIS_ID,
            view: View::GENESIS,
            height: 0,
            proposer: ReplicaId(0),
            payload: 0,
            justify: QuorumCert::genesis(),
        }
    }

    /// Whether this is the genesis block.
    #[must_use]
    pub fn is_genesis(&self) -> bool {
        self.id == GENESIS_ID
    }

    /// Builds a child of `parent` proposed in `view`, justified by `justify`.
    ///
    /// # Errors
    /// [`ConsensusError::QcMismatch`] if `justify` does not certify `parent`
    /// in `parent.view`; [`ConsensusError::NonIncreasingView`] if
    /// `view <= parent.view`; [`ConsensusError::Overflow`] on height overflow.
    pub fn new(
        parent: &Block,
        view: View,
        proposer: ReplicaId,
        payload: u64,
        justify: QuorumCert,
    ) -> Result<Self, ConsensusError> {
        if justify.block != parent.id || justify.view != parent.view {
            return Err(ConsensusError::QcMismatch);
        }
        if view <= parent.view {
            return Err(ConsensusError::NonIncreasingView);
        }
        let height = parent
            .height
            .checked_add(1)
            .ok_or(ConsensusError::Overflow)?;
        let id = Self::compute_id(parent.id, view, height, proposer, payload, justify.view);
        Ok(Self {
            id,
            parent: parent.id,
            view,
            height,
            proposer,
            payload,
            justify,
        })
    }

    /// Deterministic digest of the block fields. Never returns [`GENESIS_ID`].
    #[must_use]
    pub fn compute_id(
        parent: BlockId,
        view: View,
        height: u64,
        proposer: ReplicaId,
        payload: u64,
        justify_view: View,
    ) -> BlockId {
        let mut h = 0x5450_545F_4253_5446_u64;
        h = absorb(h, parent.0);
        h = absorb(h, view.0);
        h = absorb(h, height);
        h = absorb(h, u64::from(proposer.0));
        h = absorb(h, payload);
        h = absorb(h, justify_view.0);
        BlockId(h.max(1))
    }

    /// Whether `id` matches the digest of the other fields.
    #[must_use]
    pub fn has_valid_id(&self) -> bool {
        self.id
            == Self::compute_id(
                self.parent,
                self.view,
                self.height,
                self.proposer,
                self.payload,
                self.justify.view,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qc_for(b: &Block, n: u16) -> QuorumCert {
        let mut signers = SignerSet::new();
        for i in 0..n {
            signers.insert(ReplicaId(i)).unwrap();
        }
        QuorumCert {
            view: b.view,
            block: b.id,
            signers,
        }
    }

    #[test]
    fn block_construction_and_ids() {
        let cfg = Config::new(4).unwrap();
        let g = Block::genesis();
        let b1 = Block::new(&g, View(1), ReplicaId(1), 7, QuorumCert::genesis()).unwrap();
        assert!(b1.has_valid_id());
        assert_eq!(b1.height, 1);
        assert_ne!(b1.id, GENESIS_ID);
        let again = Block::new(&g, View(1), ReplicaId(1), 7, QuorumCert::genesis()).unwrap();
        assert_eq!(again.id, b1.id);
        let other = Block::new(&g, View(1), ReplicaId(1), 8, QuorumCert::genesis()).unwrap();
        assert_ne!(other.id, b1.id);
        let qc = qc_for(&b1, 3);
        qc.validate(&cfg).unwrap();
        assert_eq!(
            qc_for(&b1, 2).validate(&cfg),
            Err(ConsensusError::InsufficientSigners { have: 2, need: 3 })
        );
        let b2 = Block::new(&b1, View(2), ReplicaId(2), 0, qc).unwrap();
        assert_eq!(b2.parent, b1.id);
        assert_eq!(
            Block::new(&b1, View(1), ReplicaId(1), 0, qc),
            Err(ConsensusError::NonIncreasingView)
        );
        assert_eq!(
            Block::new(&b2, View(3), ReplicaId(3), 0, qc),
            Err(ConsensusError::QcMismatch)
        );
        let mut tampered = b2;
        tampered.payload = 99;
        assert!(!tampered.has_valid_id());
    }

    #[test]
    fn genesis_qc_rules() {
        let cfg = Config::new(4).unwrap();
        QuorumCert::genesis().validate(&cfg).unwrap();
        let mut bad = QuorumCert::genesis();
        bad.view = View(3);
        assert_eq!(bad.validate(&cfg), Err(ConsensusError::InvalidQuorumCert));
        let mut outside = qc_for(&Block::genesis(), 3);
        outside.block = BlockId(5);
        outside.view = View(1);
        outside.signers.insert(ReplicaId(9)).unwrap();
        assert_eq!(
            outside.validate(&cfg),
            Err(ConsensusError::InvalidQuorumCert)
        );
    }
}
