// SPDX-License-Identifier: MIT OR Apache-2.0
//! The chained-HotStuff replica state machine (requires `alloc`).

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec;
use alloc::vec::Vec;

use crate::block::{Block, BlockId, QuorumCert, GENESIS_ID};
use crate::error::ConsensusError;
use crate::message::{Message, NewView, Outgoing, Recipient, Vote};
use crate::phase::{Decide, Round};
use crate::types::{Config, ReplicaId, SignerSet, View};

/// Maximum number of buffered blocks whose parent is still unknown.
const MAX_ORPHANS: usize = 256;
/// Maximum number of buffered certificates whose block is still unknown.
const MAX_PENDING_QCS: usize = 64;
/// Maximum number of distinct `(view, block)` vote tallies kept.
const MAX_VOTE_ENTRIES: usize = 4096;
/// Maximum number of distinct views with new-view tallies kept.
const MAX_NEW_VIEW_ENTRIES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    /// Entered because a certificate or proposal proved progress.
    Progress,
    /// Entered because the view timer expired.
    Timeout,
    /// Entered because `f + 1` replicas reported a higher view.
    Sync,
}

#[derive(Debug, Clone, Copy)]
struct Orphan {
    from: ReplicaId,
    block: Block,
    is_proposal: bool,
}

/// One chained-HotStuff replica.
///
/// The replica is a deterministic, event-driven state machine over logical
/// time. The transport calls [`Replica::start`] once, then
/// [`Replica::handle`] for every received message and [`Replica::tick`]
/// periodically; each call returns the messages to send.
///
/// Protocol rules (Yin et al., PODC 2019, chained HotStuff):
///
/// * **Vote rule** (`safeNode`): vote for a proposal in the current view `v`
///   only if `v` exceeds the last voted view and the block either extends the
///   locked block or carries a certificate newer than the lock.
/// * **Lock rule**: on a certificate for `b2`, lock on `b2.justify` if newer.
/// * **Commit rule**: on a certificate for `b2` with `b2 -> b1 -> b0` a direct
///   chain in consecutive views, commit `b0` and its uncommitted ancestors.
///   The check is expressed with the type-state [`Round`].
/// * **Pacemaker**: round-robin leaders, exponential back-off timeouts,
///   `NewView` broadcast on timeout, and a jump to the `(f + 1)`-th highest
///   reported view so honest replicas re-synchronise.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tpt_algo_consensus::{Config, Replica, ReplicaId};
///
/// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
/// assert_eq!(r.id(), ReplicaId(0));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Replica {
    id: ReplicaId,
    config: Config,
    started: bool,
    now: u64,
    current_view: View,
    deadline: u64,
    consecutive_timeouts: u32,
    last_voted_view: View,
    last_vote: Option<Vote>,
    proposed_view: View,
    high_qc: QuorumCert,
    locked_qc: QuorumCert,
    blocks: BTreeMap<BlockId, Block>,
    committed: Vec<Block>,
    votes: BTreeMap<(View, BlockId), SignerSet>,
    new_views: BTreeMap<View, SignerSet>,
    peer_views: Vec<View>,
    orphans: BTreeMap<BlockId, Vec<Orphan>>,
    orphan_count: usize,
    pending_qcs: Vec<QuorumCert>,
    mempool: VecDeque<u64>,
    inbox: VecDeque<Message>,
}

impl Replica {
    /// Creates replica `id` of the group described by `config`.
    ///
    /// # Errors
    /// [`ConsensusError::UnknownReplica`] if `id` is not in `0..n`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, ConsensusError, Replica, ReplicaId};
    ///
    /// // Replica ids must be members of the configured set.
    /// assert!(matches!(
    ///     Replica::new(ReplicaId(9), Config::new(4)?),
    ///     Err(ConsensusError::UnknownReplica { .. })
    /// ));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(id: ReplicaId, config: Config) -> Result<Self, ConsensusError> {
        if !config.contains(id) {
            return Err(ConsensusError::UnknownReplica { id: id.0 });
        }
        let genesis = Block::genesis();
        let mut blocks = BTreeMap::new();
        blocks.insert(genesis.id, genesis);
        Ok(Self {
            id,
            config,
            started: false,
            now: 0,
            current_view: View::GENESIS,
            deadline: 0,
            consecutive_timeouts: 0,
            last_voted_view: View::GENESIS,
            last_vote: None,
            proposed_view: View::GENESIS,
            high_qc: QuorumCert::genesis(),
            locked_qc: QuorumCert::genesis(),
            blocks,
            committed: Vec::new(),
            votes: BTreeMap::new(),
            new_views: BTreeMap::new(),
            peer_views: vec![View::GENESIS; usize::from(config.n())],
            orphans: BTreeMap::new(),
            orphan_count: 0,
            pending_qcs: Vec::new(),
            mempool: VecDeque::new(),
            inbox: VecDeque::new(),
        })
    }

    /// This replica's id.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let r = Replica::new(ReplicaId(2), Config::new(4)?)?;
    /// assert_eq!(r.id(), ReplicaId(2));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn id(&self) -> ReplicaId {
        self.id
    }

    /// Group configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert_eq!(r.config().n(), 4);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Current view.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId, View};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert_eq!(r.current_view(), View::GENESIS);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn current_view(&self) -> View {
        self.current_view
    }

    /// Highest known quorum certificate.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert!(r.high_qc().is_genesis()); // nothing certified yet
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn high_qc(&self) -> &QuorumCert {
        &self.high_qc
    }

    /// Certificate of the locked block.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert!(r.locked_qc().is_genesis());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn locked_qc(&self) -> &QuorumCert {
        &self.locked_qc
    }

    /// Highest view this replica voted in.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId, View};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert_eq!(r.last_voted_view(), View::GENESIS); // has not voted
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn last_voted_view(&self) -> View {
        self.last_voted_view
    }

    /// Committed blocks in order; entry `i` has height `i + 1`. Final.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert!(r.committed().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn committed(&self) -> &[Block] {
        &self.committed
    }

    /// Looks up a stored block.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{BlockId, Config, Replica, ReplicaId};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert!(r.block(BlockId(123)).is_none()); // unknown block
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn block(&self, id: BlockId) -> Option<&Block> {
        self.blocks.get(&id)
    }

    /// Logical time at which the current view times out, once started.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// assert_eq!(r.deadline(), None); // no timer until started
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn deadline(&self) -> Option<u64> {
        if self.started {
            Some(self.deadline)
        } else {
            None
        }
    }

    /// Queues a command to be proposed the next time this replica leads.
    /// Without queued commands the leader proposes an empty block (`payload: None`).
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let mut r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// r.submit(42); // queued as the payload of the next proposal
    /// # Ok(())
    /// # }
    /// ```
    pub fn submit(&mut self, command: u64) {
        self.mempool.push_back(command);
    }

    /// Starts the replica at logical time `now` by entering view 1.
    ///
    /// # Errors
    /// Propagates internal errors (none are expected for a fresh replica).
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let mut r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// let _out = r.start(0)?; // arms the view timer
    /// assert!(r.deadline().is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn start(&mut self, now: u64) -> Result<Vec<Outgoing>, ConsensusError> {
        let mut out = Vec::new();
        if self.started {
            return Ok(out);
        }
        self.started = true;
        self.now = now;
        self.enter_view(View(1), Entry::Progress, &mut out)?;
        self.drain(&mut out)?;
        Ok(out)
    }

    /// Runtime-checked entry point for a message from the network.
    ///
    /// `from` is the authenticated sender (trusted, see crate limitations).
    /// Malformed or illegal messages are rejected with an error and leave the
    /// replica state unchanged; stale or duplicate messages are ignored.
    ///
    /// # Errors
    /// Any validation error from [`ConsensusError`];
    /// [`ConsensusError::ConflictingCommit`] signals a detected safety
    /// violation.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Message, Replica, ReplicaId, BlockId};
    ///
    /// let mut r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// r.start(0)?;
    /// // A message from outside the configured set is rejected.
    /// assert!(r.handle(1, ReplicaId(9), &Message::FetchBlock(BlockId(1))).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn handle(
        &mut self,
        now: u64,
        from: ReplicaId,
        message: &Message,
    ) -> Result<Vec<Outgoing>, ConsensusError> {
        if !self.config.contains(from) {
            return Err(ConsensusError::UnknownReplica { id: from.0 });
        }
        let mut out = Vec::new();
        if !self.started || from == self.id {
            return Ok(out);
        }
        self.now = self.now.max(now);
        let result = self.process(from, message, &mut out);
        let drained = self.drain(&mut out);
        result?;
        drained?;
        Ok(out)
    }

    /// Advances logical time; on view timeout moves to the next view and
    /// broadcasts `NewView`.
    ///
    /// # Errors
    /// Propagates internal errors (for example a detected conflicting commit).
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{Config, Replica, ReplicaId};
    ///
    /// let mut r = Replica::new(ReplicaId(0), Config::new(4)?)?;
    /// r.start(0)?;
    /// let deadline = r.deadline().unwrap();
    /// let _out = r.tick(deadline)?; // the view timer fired: the replica moves to a new view
    /// # Ok(())
    /// # }
    /// ```
    pub fn tick(&mut self, now: u64) -> Result<Vec<Outgoing>, ConsensusError> {
        let mut out = Vec::new();
        if !self.started {
            return Ok(out);
        }
        self.now = self.now.max(now);
        if self.now >= self.deadline {
            let next = self.current_view.next();
            self.enter_view(next, Entry::Timeout, &mut out)?;
        }
        self.drain(&mut out)?;
        Ok(out)
    }

    fn send(&mut self, to: Recipient, message: Message, out: &mut Vec<Outgoing>) {
        match to {
            Recipient::One(id) if id == self.id => self.inbox.push_back(message),
            Recipient::One(_) => out.push(Outgoing { to, message }),
            Recipient::All => {
                self.inbox.push_back(message);
                out.push(Outgoing { to, message });
            }
        }
    }

    fn drain(&mut self, out: &mut Vec<Outgoing>) -> Result<(), ConsensusError> {
        while let Some(message) = self.inbox.pop_front() {
            let me = self.id;
            self.process(me, &message, out)?;
        }
        Ok(())
    }

    fn process(
        &mut self,
        from: ReplicaId,
        message: &Message,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        match *message {
            Message::Proposal(block) => {
                if block.is_genesis() {
                    return Ok(());
                }
                self.validate_block(&block)?;
                if from != block.proposer {
                    return Err(ConsensusError::NotLeader {
                        view: block.view.0,
                        sender: from.0,
                    });
                }
                self.accept_block(from, block, true, out)
            }
            Message::BlockData(block) => {
                if block.is_genesis() {
                    return Ok(());
                }
                self.validate_block(&block)?;
                self.accept_block(from, block, false, out)
            }
            Message::Vote(vote) => self.on_vote(from, vote, out),
            Message::NewView(nv) => self.on_new_view(from, nv, out),
            Message::FetchBlock(id) => {
                if let Some(block) = self.blocks.get(&id).copied() {
                    if !block.is_genesis() && from != self.id {
                        self.send(Recipient::One(from), Message::BlockData(block), out);
                    }
                }
                Ok(())
            }
        }
    }

    /// Context-free structural checks of a block received from the network.
    fn validate_block(&self, b: &Block) -> Result<(), ConsensusError> {
        if !self.config.contains(b.proposer) {
            return Err(ConsensusError::UnknownReplica { id: b.proposer.0 });
        }
        if self.config.leader(b.view) != b.proposer {
            return Err(ConsensusError::NotLeader {
                view: b.view.0,
                sender: b.proposer.0,
            });
        }
        b.justify.validate(&self.config)?;
        if b.parent != b.justify.block {
            return Err(ConsensusError::BrokenParentLink);
        }
        if b.view <= b.justify.view {
            return Err(ConsensusError::NonIncreasingView);
        }
        if b.height == 0 {
            return Err(ConsensusError::InvalidHeight);
        }
        if !b.has_valid_id() {
            return Err(ConsensusError::BlockIdMismatch);
        }
        Ok(())
    }

    fn accept_block(
        &mut self,
        from: ReplicaId,
        block: Block,
        is_proposal: bool,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        let mut work = vec![Orphan {
            from,
            block,
            is_proposal,
        }];
        let mut top = true;
        while let Some(item) = work.pop() {
            let is_top = core::mem::replace(&mut top, false);
            match self.accept_one(item, &mut work, out) {
                Ok(()) => {}
                Err(e) if is_top || matches!(e, ConsensusError::ConflictingCommit { .. }) => {
                    return Err(e)
                }
                // A previously buffered block turned out to be inconsistent
                // with its parent: drop it.
                Err(_) => {}
            }
        }
        Ok(())
    }

    fn accept_one(
        &mut self,
        item: Orphan,
        work: &mut Vec<Orphan>,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        let b = item.block;
        if !self.blocks.contains_key(&b.id) {
            let Some(parent) = self.blocks.get(&b.parent).copied() else {
                let waiters = self.orphans.entry(b.parent).or_default();
                let known = waiters.iter().any(|o| o.block.id == b.id);
                if known {
                    // Keep the proposal flag if the same block arrives again
                    // as a proposal.
                    if let Some(o) = waiters.iter_mut().find(|o| o.block.id == b.id) {
                        o.is_proposal |= item.is_proposal;
                    }
                } else if self.orphan_count < MAX_ORPHANS {
                    waiters.push(item);
                    self.orphan_count += 1;
                }
                if waiters.is_empty() {
                    self.orphans.remove(&b.parent);
                }
                // Re-request on every orphan: an earlier request or its
                // response may have been lost.
                if item.from != self.id {
                    self.send(
                        Recipient::One(item.from),
                        Message::FetchBlock(b.parent),
                        out,
                    );
                }
                return Ok(());
            };
            if parent.height.checked_add(1) != Some(b.height) {
                return Err(ConsensusError::InvalidHeight);
            }
            if parent.view != b.justify.view {
                return Err(ConsensusError::QcMismatch);
            }
            self.blocks.insert(b.id, b);
            if let Some(waiting) = self.orphans.remove(&b.id) {
                self.orphan_count = self.orphan_count.saturating_sub(waiting.len());
                work.extend(waiting);
            }
            self.process_qc(b.justify, out)?;
            self.retry_pending(b.id, out)?;
        }
        if item.is_proposal {
            self.consider_vote(&b, out)?;
        }
        Ok(())
    }

    fn add_pending_qc(&mut self, qc: QuorumCert) {
        if self.pending_qcs.len() < MAX_PENDING_QCS && !self.pending_qcs.contains(&qc) {
            self.pending_qcs.push(qc);
        }
    }

    fn retry_pending(
        &mut self,
        id: BlockId,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        if !self.pending_qcs.iter().any(|qc| qc.block == id) {
            return Ok(());
        }
        let (ready, rest): (Vec<_>, Vec<_>) = core::mem::take(&mut self.pending_qcs)
            .into_iter()
            .partition(|qc| qc.block == id);
        self.pending_qcs = rest;
        for qc in ready {
            // A malformed buffered certificate is dropped; only a detected
            // safety violation is propagated.
            if let Err(e @ ConsensusError::ConflictingCommit { .. }) = self.process_qc(qc, out) {
                return Err(e);
            }
        }
        Ok(())
    }

    /// Processes a certificate for a stored (or unknown) block: updates the
    /// high QC and the lock, applies the commit rule, and advances the view.
    fn process_qc(
        &mut self,
        qc: QuorumCert,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        if qc.is_genesis() {
            return Ok(());
        }
        let Some(b2) = self.blocks.get(&qc.block).copied() else {
            self.add_pending_qc(qc);
            return Ok(());
        };
        if b2.view != qc.view {
            return Err(ConsensusError::QcMismatch);
        }
        if qc.view > self.high_qc.view {
            self.high_qc = qc;
        }
        if b2.justify.view > self.locked_qc.view {
            self.locked_qc = b2.justify;
        }
        self.apply_commit_rule(&b2, &qc)?;
        if qc.view >= self.current_view {
            self.enter_view(qc.view.next(), Entry::Progress, out)?;
        }
        Ok(())
    }

    fn apply_commit_rule(&mut self, b2: &Block, qc: &QuorumCert) -> Result<(), ConsensusError> {
        let Some(b1) = self.blocks.get(&b2.justify.block).copied() else {
            return Ok(());
        };
        if b1.is_genesis() {
            return Ok(());
        }
        let Some(b0) = self.blocks.get(&b1.justify.block).copied() else {
            return Ok(());
        };
        if b0.is_genesis() {
            return Ok(());
        }
        let decided = Round::propose(&b0)
            .certify(&b1.justify, &self.config)
            .and_then(|r| r.lock(&b1, &b2.justify, &self.config))
            .and_then(|r| r.decide(b2, qc, &self.config));
        match decided {
            Ok(round) => self.commit(round),
            // Not a direct consecutive-view three-chain: nothing to commit.
            Err(_) => Ok(()),
        }
    }

    /// Appends the decided block and its uncommitted ancestors to the log.
    /// Only a `Round<Decide>` can reach this point.
    fn commit(&mut self, round: Round<Decide>) -> Result<(), ConsensusError> {
        let (tip_height, tip_id) = self
            .committed
            .last()
            .map_or((0, GENESIS_ID), |b| (b.height, b.id));
        let target = round.block_id();
        if round.height() <= tip_height {
            let idx = usize::try_from(round.height().saturating_sub(1))
                .map_err(|_| ConsensusError::Overflow)?;
            return match self.committed.get(idx) {
                Some(b) if b.id == target => Ok(()),
                _ => Err(ConsensusError::ConflictingCommit {
                    height: round.height(),
                }),
            };
        }
        let mut chain = Vec::new();
        let mut cursor = target;
        loop {
            let b = self
                .blocks
                .get(&cursor)
                .copied()
                .ok_or(ConsensusError::MissingAncestor)?;
            if b.height <= tip_height {
                if b.id == tip_id {
                    break;
                }
                return Err(ConsensusError::ConflictingCommit { height: b.height });
            }
            chain.push(b);
            cursor = b.parent;
        }
        chain.reverse();
        self.committed.extend(chain);
        Ok(())
    }

    fn extends(&self, block: &Block, qc: &QuorumCert) -> bool {
        let mut cursor = *block;
        loop {
            if cursor.id == qc.block {
                return true;
            }
            if cursor.view <= qc.view || cursor.is_genesis() {
                return false;
            }
            match self.blocks.get(&cursor.parent) {
                Some(p) => cursor = *p,
                None => return false,
            }
        }
    }

    fn safe_node(&self, block: &Block) -> bool {
        block.justify.view > self.locked_qc.view || self.extends(block, &self.locked_qc)
    }

    fn consider_vote(&mut self, b: &Block, out: &mut Vec<Outgoing>) -> Result<(), ConsensusError> {
        if b.view != self.current_view || b.view <= self.last_voted_view || !self.safe_node(b) {
            return Ok(());
        }
        self.last_voted_view = b.view;
        let vote = Vote {
            view: b.view,
            block: b.id,
        };
        self.last_vote = Some(vote);
        let next = b.view.next();
        let leader = self.config.leader(next);
        self.send(Recipient::One(leader), Message::Vote(vote), out);
        self.enter_view(next, Entry::Progress, out)
    }

    fn on_vote(
        &mut self,
        from: ReplicaId,
        vote: Vote,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        if vote.view == View::GENESIS || vote.view <= self.high_qc.view {
            return Ok(());
        }
        let key = (vote.view, vote.block);
        if !self.votes.contains_key(&key) && self.votes.len() >= MAX_VOTE_ENTRIES {
            return Ok(());
        }
        let set = self.votes.entry(key).or_default();
        if !set.insert(from)? {
            return Ok(());
        }
        if set.len() != self.config.quorum() {
            return self.try_propose(out);
        }
        let qc = QuorumCert {
            view: vote.view,
            block: vote.block,
            signers: *set,
        };
        if self.blocks.contains_key(&vote.block) {
            self.process_qc(qc, out)?;
        } else {
            self.add_pending_qc(qc);
            if from != self.id {
                self.send(Recipient::One(from), Message::FetchBlock(vote.block), out);
            }
        }
        self.try_propose(out)
    }

    fn on_new_view(
        &mut self,
        from: ReplicaId,
        nv: NewView,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        nv.high_qc.validate(&self.config)?;
        if !nv.high_qc.is_genesis() {
            if self.blocks.contains_key(&nv.high_qc.block) {
                self.process_qc(nv.high_qc, out)?;
            } else if nv.high_qc.view > self.high_qc.view {
                self.add_pending_qc(nv.high_qc);
                if from != self.id {
                    self.send(
                        Recipient::One(from),
                        Message::FetchBlock(nv.high_qc.block),
                        out,
                    );
                }
            }
        }
        if let Some(vote) = nv.last_vote {
            self.on_vote(from, vote, out)?;
        }
        if let Some(slot) = self.peer_views.get_mut(usize::from(from.0)) {
            if nv.view > *slot {
                *slot = nv.view;
            }
        }
        if nv.view >= self.current_view
            && (self.new_views.contains_key(&nv.view)
                || self.new_views.len() < MAX_NEW_VIEW_ENTRIES)
        {
            self.new_views.entry(nv.view).or_default().insert(from)?;
        }
        self.sync_views(out)?;
        self.try_propose(out)
    }

    /// If at least `f + 1` replicas reported views above ours, at least one
    /// honest replica is at the `(f + 1)`-th highest of them: jump there.
    fn sync_views(&mut self, out: &mut Vec<Outgoing>) -> Result<(), ConsensusError> {
        let threshold = usize::from(self.config.f()) + 1;
        let mut ahead: Vec<View> = self
            .peer_views
            .iter()
            .copied()
            .filter(|v| *v > self.current_view)
            .collect();
        if ahead.len() < threshold {
            return Ok(());
        }
        ahead.sort_unstable_by(|a, b| b.cmp(a));
        if let Some(target) = ahead.get(threshold - 1).copied() {
            self.enter_view(target, Entry::Sync, out)?;
        }
        Ok(())
    }

    fn enter_view(
        &mut self,
        view: View,
        entry: Entry,
        out: &mut Vec<Outgoing>,
    ) -> Result<(), ConsensusError> {
        if view <= self.current_view {
            return Ok(());
        }
        self.current_view = view;
        match entry {
            Entry::Progress => self.consecutive_timeouts = 0,
            Entry::Timeout => {
                self.consecutive_timeouts = self.consecutive_timeouts.saturating_add(1)
            }
            Entry::Sync => {}
        }
        self.deadline = self
            .now
            .saturating_add(self.config.timeout_for(self.consecutive_timeouts));
        let floor = self.high_qc.view;
        self.votes.retain(|(v, _), _| *v > floor);
        self.new_views.retain(|v, _| *v >= view);
        if entry != Entry::Progress {
            let nv = NewView {
                view,
                high_qc: self.high_qc,
                last_vote: self.last_vote,
            };
            self.send(Recipient::All, Message::NewView(nv), out);
        }
        self.try_propose(out)
    }

    fn try_propose(&mut self, out: &mut Vec<Outgoing>) -> Result<(), ConsensusError> {
        let view = self.current_view;
        if self.config.leader(view) != self.id || self.proposed_view >= view {
            return Ok(());
        }
        // Ready when the previous view is known to be over: a certificate for
        // it, a quorum of `NewView` messages for this view, or a quorum of
        // distinct replicas that voted in the previous view (on any block,
        // e.g. after an equivocating leader split the votes).
        let quorum = self.config.quorum();
        let prev = View(view.0.saturating_sub(1));
        let ready = self.high_qc.view.next() == view
            || self.new_views.get(&view).is_some_and(|s| s.len() >= quorum)
            || self
                .votes
                .range((prev, BlockId(0))..=(prev, BlockId(u64::MAX)))
                .fold(SignerSet::new(), |acc, (_, s)| acc.union(s))
                .len()
                >= quorum;
        if !ready {
            return Ok(());
        }
        let Some(parent) = self.blocks.get(&self.high_qc.block).copied() else {
            return Ok(());
        };
        let payload = self.mempool.pop_front();
        let block = Block::new(&parent, view, self.id, payload, self.high_qc)?;
        self.proposed_view = view;
        self.send(Recipient::All, Message::Proposal(block), out);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config::new(4).unwrap()
    }

    fn full_qc(b: &Block) -> QuorumCert {
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
    fn leader_of_view_one_proposes_on_start() {
        let mut r = Replica::new(ReplicaId(1), cfg()).unwrap();
        let out = r.start(0).unwrap();
        assert!(out
            .iter()
            .any(|o| matches!(o.message, Message::Proposal(b) if b.view == View(1))));
        // It voted for its own proposal and moved to view 2.
        assert_eq!(r.current_view(), View(2));
        assert_eq!(r.last_voted_view(), View(1));
    }

    #[test]
    fn rejects_illegal_messages() {
        let mut r = Replica::new(ReplicaId(0), cfg()).unwrap();
        r.start(0).unwrap();
        let g = Block::genesis();
        let b1 = Block::new(&g, View(1), ReplicaId(1), None, QuorumCert::genesis()).unwrap();
        // Proposal relayed by a non-leader.
        assert_eq!(
            r.handle(1, ReplicaId(2), &Message::Proposal(b1)),
            Err(ConsensusError::NotLeader { view: 1, sender: 2 })
        );
        // Proposal by the wrong proposer.
        let wrong = Block::new(&g, View(1), ReplicaId(2), None, QuorumCert::genesis()).unwrap();
        assert!(matches!(
            r.handle(1, ReplicaId(2), &Message::Proposal(wrong)),
            Err(ConsensusError::NotLeader { .. })
        ));
        // Tampered contents.
        let mut tampered = b1;
        tampered.payload = Some(5);
        assert_eq!(
            r.handle(1, ReplicaId(1), &Message::Proposal(tampered)),
            Err(ConsensusError::BlockIdMismatch)
        );
        // Unknown sender.
        assert_eq!(
            r.handle(1, ReplicaId(9), &Message::Proposal(b1)),
            Err(ConsensusError::UnknownReplica { id: 9 })
        );
        // Under-signed certificate.
        let mut weak = full_qc(&b1);
        weak.signers = SignerSet::new();
        weak.signers.insert(ReplicaId(0)).unwrap();
        let nv = NewView {
            view: View(2),
            high_qc: weak,
            last_vote: None,
        };
        assert_eq!(
            r.handle(1, ReplicaId(3), &Message::NewView(nv)),
            Err(ConsensusError::InsufficientSigners { have: 1, need: 3 })
        );
        // Nothing was voted or committed because of the rejected messages.
        assert_eq!(r.last_voted_view(), View::GENESIS);
        assert!(r.committed().is_empty());
        // The valid proposal is accepted and voted for.
        let out = r.handle(1, ReplicaId(1), &Message::Proposal(b1)).unwrap();
        assert!(out.iter().any(|o| o.to == Recipient::One(ReplicaId(2))
            && matches!(o.message, Message::Vote(v) if v.block == b1.id)));
    }

    fn qc_n(b: &Block, q: u16) -> QuorumCert {
        let mut s = SignerSet::new();
        for i in 0..q {
            s.insert(ReplicaId(i)).unwrap();
        }
        QuorumCert {
            view: b.view,
            block: b.id,
            signers: s,
        }
    }

    #[test]
    fn locked_replica_refuses_conflicting_proposal() {
        // n = 5: leaders of views 1..=4 are replicas 1..=4, quorum 4.
        let c = Config::new(5).unwrap();
        let mut r = Replica::new(ReplicaId(0), c).unwrap();
        r.start(0).unwrap();
        let g = Block::genesis();
        let b1 = Block::new(&g, View(1), ReplicaId(1), None, QuorumCert::genesis()).unwrap();
        let b2 = Block::new(&b1, View(2), ReplicaId(2), None, qc_n(&b1, 4)).unwrap();
        let b3 = Block::new(&b2, View(3), ReplicaId(3), None, qc_n(&b2, 4)).unwrap();
        r.handle(1, ReplicaId(1), &Message::Proposal(b1)).unwrap();
        r.handle(2, ReplicaId(2), &Message::Proposal(b2)).unwrap();
        r.handle(3, ReplicaId(3), &Message::Proposal(b3)).unwrap();
        // Lock is on b1 (b3 carries QC(b2) whose justify is QC(b1)).
        assert_eq!(r.locked_qc().block, b1.id);
        assert_eq!(r.current_view(), View(4));
        // A fork from genesis in view 4 with an old certificate is unsafe.
        let fork = Block::new(&g, View(4), ReplicaId(4), Some(1), QuorumCert::genesis()).unwrap();
        r.handle(4, ReplicaId(4), &Message::Proposal(fork)).unwrap();
        assert_eq!(r.last_voted_view(), View(3));
        // A proposal extending the lock in the same view is still voted for.
        let b4 = Block::new(&b3, View(4), ReplicaId(4), None, qc_n(&b3, 4)).unwrap();
        r.handle(5, ReplicaId(4), &Message::Proposal(b4)).unwrap();
        assert_eq!(r.last_voted_view(), View(4));
        // b4 carries QC(b3): b1 <- b2 <- b3 is a consecutive three-chain.
        assert_eq!(r.committed().len(), 1);
        assert_eq!(r.committed()[0].id, b1.id);
    }
}
