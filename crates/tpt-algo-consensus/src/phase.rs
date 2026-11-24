// SPDX-License-Identifier: MIT OR Apache-2.0
//! Type-state view of the HotStuff phases for a single block.
//!
//! Basic HotStuff runs four phases per decision: *prepare*, *pre-commit*,
//! *commit* and *decide*. Chained HotStuff pipelines them: a block `b0`
//! proposed in view `v` is
//!
//! * in [`Prepare`] once proposed,
//! * in [`PreCommit`] once a quorum certificate for `b0` exists (it appears as
//!   the `justify` of the direct child `b1`),
//! * in [`Commit`] (locked) once the direct child `b1` (view `v + 1`) is
//!   certified,
//! * in [`Decide`] once the direct grandchild `b2` (view `v + 2`) is certified,
//!   i.e. a three-chain of consecutive-view certified blocks exists.
//!
//! [`Round`] carries the phase in its type, so the transitions can only be
//! taken in order and only a `Round<Decide>` exposes the committed block to
//! the replica's commit path. Skipping a phase does not compile:
//!
//! ```compile_fail
//! use tpt_algo_consensus::{Block, Config, Round};
//! let g = Block::genesis();
//! let cfg = Config::new(4).unwrap();
//! // `decide` exists only on `Round<Commit>`.
//! let _ = Round::propose(&g).decide(&g, &g.justify, &cfg);
//! ```
//!
//! Each transition is checked at run time against the actual blocks and
//! certificates and returns an error if they do not form a direct,
//! consecutive-view, certified chain.

use core::marker::PhantomData;

use crate::block::{Block, BlockId, QuorumCert};
use crate::error::ConsensusError;
use crate::types::{Config, View};

mod sealed {
    pub trait Sealed {}
}

/// Marker trait for protocol phases. Sealed: only the four markers below.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::{Commit, Phase, Prepare};
///
/// // Only the four protocol phases implement the sealed `Phase` trait.
/// fn assert_phase<P: Phase>() {}
/// assert_phase::<Prepare>();
/// assert_phase::<Commit>();
/// ```
pub trait Phase: sealed::Sealed + core::fmt::Debug + Copy {}

/// Phase marker: proposed, not yet certified.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// use tpt_algo_consensus::{Block, Config, Round};
///
/// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
/// let round = Round::propose(&b1); // Round<Prepare>
/// assert_eq!(round.block_id(), b1.id);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prepare;
/// Phase marker: certified by a quorum certificate.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// use tpt_algo_consensus::{Block, Config, Round};
///
/// let cfg = Config::new(4)?;
/// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
/// // Prepare -> PreCommit once b1 is certified.
/// let pre: Round<_> = Round::propose(&b1).certify(&cert(&b1), &cfg)?;
/// assert_eq!(pre.view(), View(1));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreCommit;
/// Phase marker: locked (two-chain of consecutive-view certified blocks).
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// use tpt_algo_consensus::{Block, Round};
///
/// let cfg = Config::new(4)?;
/// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
/// let b2 = Block::new(&b1, View(2), ReplicaId(2), Some(11), cert(&b1))?;
/// let commit = Round::propose(&b1).certify(&cert(&b1), &cfg)?.lock(&b2, &cert(&b2), &cfg)?;
/// assert_eq!(commit.block_id(), b1.id);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Commit;
/// Phase marker: decided (three-chain of consecutive-view certified blocks).
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// use tpt_algo_consensus::{Block, Round};
///
/// let cfg = Config::new(4)?;
/// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
/// let b2 = Block::new(&b1, View(2), ReplicaId(2), Some(11), cert(&b1))?;
/// let b3 = Block::new(&b2, View(3), ReplicaId(3), Some(12), cert(&b2))?;
/// let decided = Round::propose(&b1)
///     .certify(&cert(&b1), &cfg)?
///     .lock(&b2, &cert(&b2), &cfg)?
///     .decide(&b3, &cert(&b3), &cfg)?;
/// assert_eq!(decided.block_id(), b1.id); // b1 is final
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decide;

impl sealed::Sealed for Prepare {}
impl sealed::Sealed for PreCommit {}
impl sealed::Sealed for Commit {}
impl sealed::Sealed for Decide {}
impl Phase for Prepare {}
impl Phase for PreCommit {}
impl Phase for Commit {}
impl Phase for Decide {}

/// Progress of one block through the HotStuff phases; `P` is the phase.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// use tpt_algo_consensus::{Block, Round};
///
/// let cfg = Config::new(4)?;
/// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
/// let round = Round::propose(&b1).certify(&cert(&b1), &cfg)?;
/// assert_eq!(round.height(), b1.height);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Round<P: Phase> {
    block: BlockId,
    view: View,
    height: u64,
    tip: BlockId,
    tip_view: View,
    _phase: PhantomData<P>,
}

impl<P: Phase> Round<P> {
    /// The block this round decides on.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// assert_eq!(Round::propose(&b1).block_id(), b1.id);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn block_id(&self) -> BlockId {
        self.block
    }

    /// View in which the block was proposed.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// assert_eq!(Round::propose(&b1).view(), View(1));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    /// Height of the block.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// assert_eq!(Round::propose(&b1).height(), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    /// Most recent certified block of the direct chain starting at the block.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// // The tip is the most recent block the round has seen; right after proposal, that is b1.
    /// assert_eq!(Round::propose(&b1).tip(), b1.id);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn tip(&self) -> BlockId {
        self.tip
    }

    fn into_phase<Q: Phase>(self) -> Round<Q> {
        Round {
            block: self.block,
            view: self.view,
            height: self.height,
            tip: self.tip,
            tip_view: self.tip_view,
            _phase: PhantomData,
        }
    }

    fn extend<Q: Phase>(
        self,
        child: &Block,
        child_qc: &QuorumCert,
        config: &Config,
    ) -> Result<Round<Q>, ConsensusError> {
        if child.parent != self.tip
            || child.justify.block != self.tip
            || child.justify.view != self.tip_view
            || self.tip_view.0.checked_add(1) != Some(child.view.0)
        {
            return Err(ConsensusError::NotDirectChain);
        }
        check_certifies(child_qc, child.id, child.view, config)?;
        let mut next = self.into_phase::<Q>();
        next.tip = child.id;
        next.tip_view = child.view;
        Ok(next)
    }
}

fn check_certifies(
    qc: &QuorumCert,
    block: BlockId,
    view: View,
    config: &Config,
) -> Result<(), ConsensusError> {
    if qc.block != block || qc.view != view {
        return Err(ConsensusError::QcMismatch);
    }
    qc.validate(config)
}

impl Round<Prepare> {
    /// Starts tracking a proposed block.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// let round = Round::propose(&b1);
    /// assert_eq!(round.block_id(), b1.id);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn propose(block: &Block) -> Self {
        Self {
            block: block.id,
            view: block.view,
            height: block.height,
            tip: block.id,
            tip_view: block.view,
            _phase: PhantomData,
        }
    }

    /// Prepare -> PreCommit: `qc` must be a valid certificate for the block.
    ///
    /// # Errors
    /// [`ConsensusError::QcMismatch`] or a QC validation error.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let cfg = Config::new(4)?;
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// // A certificate for a different block is rejected.
    /// let b2 = Block::new(&b1, View(2), ReplicaId(2), Some(11), cert(&b1))?;
    /// assert!(Round::propose(&b1).certify(&cert(&b2), &cfg).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn certify(
        self,
        qc: &QuorumCert,
        config: &Config,
    ) -> Result<Round<PreCommit>, ConsensusError> {
        check_certifies(qc, self.block, self.view, config)?;
        Ok(self.into_phase())
    }
}

impl Round<PreCommit> {
    /// PreCommit -> Commit: `child` must directly extend the block in the
    /// next view and `child_qc` must certify `child`.
    ///
    /// # Errors
    /// [`ConsensusError::NotDirectChain`], [`ConsensusError::QcMismatch`] or a
    /// QC validation error.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let cfg = Config::new(4)?;
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// let b2 = Block::new(&b1, View(2), ReplicaId(2), Some(11), cert(&b1))?;
    /// let pre = Round::propose(&b1).certify(&cert(&b1), &cfg)?;
    /// assert!(pre.lock(&b2, &cert(&b2), &cfg).is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub fn lock(
        self,
        child: &Block,
        child_qc: &QuorumCert,
        config: &Config,
    ) -> Result<Round<Commit>, ConsensusError> {
        self.extend(child, child_qc, config)
    }
}

impl Round<Commit> {
    /// Commit -> Decide: `grandchild` must directly extend the child in the
    /// next view and `grandchild_qc` must certify it.
    ///
    /// # Errors
    /// [`ConsensusError::NotDirectChain`], [`ConsensusError::QcMismatch`] or a
    /// QC validation error.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    /// use tpt_algo_consensus::{Block, Round};
    ///
    /// let cfg = Config::new(4)?;
    /// let b1 = Block::new(&Block::genesis(), View(1), ReplicaId(1), Some(10), QuorumCert::genesis())?;
    /// let b2 = Block::new(&b1, View(2), ReplicaId(2), Some(11), cert(&b1))?;
    /// let b3 = Block::new(&b2, View(3), ReplicaId(3), Some(12), cert(&b2))?;
    /// let commit = Round::propose(&b1).certify(&cert(&b1), &cfg)?.lock(&b2, &cert(&b2), &cfg)?;
    /// assert!(commit.decide(&b3, &cert(&b3), &cfg).is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub fn decide(
        self,
        grandchild: &Block,
        grandchild_qc: &QuorumCert,
        config: &Config,
    ) -> Result<Round<Decide>, ConsensusError> {
        self.extend(grandchild, grandchild_qc, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ReplicaId, SignerSet};

    fn qc(b: &Block) -> QuorumCert {
        let mut s = SignerSet::new();
        for i in 0..3 {
            s.insert(ReplicaId(i)).unwrap();
        }
        QuorumCert {
            view: b.view,
            block: b.id,
            signers: s,
        }
    }

    #[test]
    fn three_chain_decides() {
        let cfg = Config::new(4).unwrap();
        let g = Block::genesis();
        let b0 = Block::new(&g, View(1), ReplicaId(1), None, QuorumCert::genesis()).unwrap();
        let b1 = Block::new(&b0, View(2), ReplicaId(2), None, qc(&b0)).unwrap();
        let b2 = Block::new(&b1, View(3), ReplicaId(3), None, qc(&b1)).unwrap();
        let r = Round::propose(&b0)
            .certify(&b1.justify, &cfg)
            .unwrap()
            .lock(&b1, &b2.justify, &cfg)
            .unwrap()
            .decide(&b2, &qc(&b2), &cfg)
            .unwrap();
        assert_eq!(r.block_id(), b0.id);
        assert_eq!(r.tip(), b2.id);
        assert_eq!(r.height(), 1);
    }

    #[test]
    fn gap_in_views_is_rejected() {
        let cfg = Config::new(4).unwrap();
        let g = Block::genesis();
        let b0 = Block::new(&g, View(1), ReplicaId(1), None, QuorumCert::genesis()).unwrap();
        let b1 = Block::new(&b0, View(3), ReplicaId(3), None, qc(&b0)).unwrap();
        let pre = Round::propose(&b0).certify(&qc(&b0), &cfg).unwrap();
        assert_eq!(
            pre.lock(&b1, &qc(&b1), &cfg),
            Err(ConsensusError::NotDirectChain)
        );
        let mut weak = qc(&b0);
        weak.signers = SignerSet::new();
        assert!(Round::propose(&b0).certify(&weak, &cfg).is_err());
        assert_eq!(
            Round::propose(&b0).certify(&qc(&b1), &cfg),
            Err(ConsensusError::QcMismatch)
        );
    }
}
