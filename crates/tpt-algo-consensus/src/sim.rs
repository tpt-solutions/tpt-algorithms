// SPDX-License-Identifier: MIT OR Apache-2.0
//! Deterministic discrete-time network simulator (requires `alloc`).
//!
//! The simulator owns `n` [`Replica`]s and a message queue ordered by
//! `(delivery tick, sequence number)`. All randomness (delays, drops) comes
//! from a private SplitMix64 generator seeded by [`SimConfig::seed`], so a run
//! is a pure function of its configuration.
//!
//! Network model: before the global stabilisation time (`gst`) messages may
//! be dropped (per-mille probability) and delayed by `1..=pre_gst_max_delay`
//! ticks; from `gst` on nothing is dropped and delays are
//! `1..=post_gst_max_delay`. Random delays also reorder messages.
//!
//! Fault model ([`FaultKind`]):
//! * `Crash { at_tick }`: the replica stops processing and sending at that tick.
//! * `Equivocate`: a Byzantine replica that sends two conflicting proposals
//!   (to alternating halves of the group) whenever it leads, votes for every
//!   proposal it receives regardless of its lock (double voting), and
//!   broadcasts inflated `NewView` view numbers. It cannot forge other
//!   replicas' votes: the simulator authenticates senders, which stands in
//!   for signatures.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::block::{mix64, Block};
use crate::error::ConsensusError;
use crate::message::{Message, NewView, Outgoing, Recipient, Vote};
use crate::replica::Replica;
use crate::types::{Config, ReplicaId, View};

/// Payload bits flipped by an equivocating leader to build its second block.
const EQUIVOCATION_MASK: u64 = 0x5A5A_5A5A_5A5A_5A5A;
/// View offset used by equivocating replicas in inflated `NewView` messages.
const INFLATED_VIEW_OFFSET: u64 = 1000;
/// Maximum number of honest-sender rejections kept for diagnostics.
const MAX_RECORDED_REJECTIONS: usize = 16;

/// Seeded SplitMix64 generator (private to the simulator).
#[derive(Debug, Clone)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix64(self.state)
    }

    /// Value in `0..bound` (0 if `bound == 0`). The modulo bias is irrelevant
    /// for scheduling purposes.
    fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            0
        } else {
            self.next_u64() % bound
        }
    }
}

/// Fault injected into one replica.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::sim::FaultKind;
///
/// // A crashed replica stops at tick 100; an equivocating one sends conflicting votes.
/// let crash = FaultKind::Crash { at_tick: 100 };
/// assert_ne!(crash, FaultKind::Equivocate);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    /// Stops at `at_tick` (0 = never starts).
    Crash {
        /// First tick at which the replica is down.
        at_tick: u64,
    },
    /// Byzantine equivocation (see module docs).
    Equivocate,
}

/// Fault assignment.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::{sim::{FaultKind, FaultSpec}, ReplicaId};
///
/// let f = FaultSpec { replica: ReplicaId(3), kind: FaultKind::Equivocate };
/// assert_eq!(f.replica, ReplicaId(3));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaultSpec {
    /// Faulty replica.
    pub replica: ReplicaId,
    /// Kind of fault.
    pub kind: FaultKind,
}

/// Simulation parameters. All quantities are integers (ticks, per-mille).
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::sim::SimConfig;
///
/// let cfg = SimConfig::new(4, 7);
/// assert_eq!(cfg.n, 4);
/// assert_eq!(cfg.seed, 7);
/// assert!(cfg.faults.is_empty()); // honest run by default
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimConfig {
    /// Number of replicas.
    pub n: u16,
    /// Scheduler seed.
    pub seed: u64,
    /// Base view timeout in ticks.
    pub base_timeout: u64,
    /// Cap on the timeout back-off exponent.
    pub max_backoff_shift: u32,
    /// Global stabilisation time (tick).
    pub gst: u64,
    /// Drop probability before `gst`, in per-mille (0..=1000).
    pub pre_gst_drop_per_mille: u16,
    /// Maximum delay before `gst` (>= 1).
    pub pre_gst_max_delay: u64,
    /// Maximum delay from `gst` on (>= 1).
    pub post_gst_max_delay: u64,
    /// Faulty replicas.
    pub faults: Vec<FaultSpec>,
}

impl SimConfig {
    /// A synchronous, fault-free network with unit delays.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_consensus::sim::SimConfig;
    ///
    /// let cfg = SimConfig::new(7, 1);
    /// assert_eq!(cfg.n, 7);
    /// ```
    #[must_use]
    pub fn new(n: u16, seed: u64) -> Self {
        Self {
            n,
            seed,
            base_timeout: Config::DEFAULT_BASE_TIMEOUT,
            max_backoff_shift: Config::DEFAULT_MAX_BACKOFF_SHIFT,
            gst: 0,
            pre_gst_drop_per_mille: 0,
            pre_gst_max_delay: 1,
            post_gst_max_delay: 1,
            faults: Vec::new(),
        }
    }
}

/// Counters collected during a run.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tpt_algo_consensus::sim::Simulator;
/// use tpt_algo_consensus::sim::SimConfig;
///
/// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
/// sim.run(200);
/// // A clean honest run never detects a safety violation.
/// assert_eq!(sim.stats().safety_violations, 0);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SimStats {
    /// Messages delivered to live replicas.
    pub delivered: u64,
    /// Messages dropped by the network.
    pub dropped: u64,
    /// Conflicting proposal pairs sent by equivocating leaders.
    pub equivocations: u64,
    /// Messages from honest senders rejected by a receiver (expected 0).
    pub rejected_from_honest: u64,
    /// Messages from faulty senders rejected by a receiver.
    pub rejected_from_faulty: u64,
    /// `ConflictingCommit` errors raised by honest replicas (expected 0).
    pub safety_violations: u64,
}

/// Outcome of a run.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tpt_algo_consensus::sim::{SimConfig, Simulator};
///
/// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
/// sim.run(200);
/// let report = sim.report();
/// assert_eq!(report.honest_logs.len(), 4);
/// assert_eq!(report.now, 201);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimReport {
    /// Committed log of every replica without an injected fault.
    pub honest_logs: Vec<(ReplicaId, Vec<Block>)>,
    /// Counters.
    pub stats: SimStats,
    /// Highest view reached by a non-faulty replica.
    pub max_honest_view: View,
    /// Current tick.
    pub now: u64,
}

#[derive(Debug, Clone, Copy)]
struct InFlight {
    from: ReplicaId,
    to: ReplicaId,
    message: Message,
}

/// Deterministic simulator of a replica group.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use tpt_algo_consensus::sim::{check_prefix_consistency, SimConfig, Simulator};
///
/// let mut sim = Simulator::new(SimConfig::new(4, 3))?;
/// sim.run(300);
/// assert!(check_prefix_consistency(&sim.report().honest_logs).is_ok());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Simulator {
    config: SimConfig,
    consensus: Config,
    replicas: Vec<Replica>,
    faults: Vec<Option<FaultKind>>,
    queue: BTreeMap<(u64, u64), InFlight>,
    seq: u64,
    now: u64,
    rng: SplitMix64,
    stats: SimStats,
    honest_rejections: Vec<(ReplicaId, ReplicaId, ConsensusError)>,
}

impl Simulator {
    /// Builds the group and starts every live replica at tick 0.
    ///
    /// # Errors
    /// Configuration errors, [`ConsensusError::UnknownReplica`] for a fault on
    /// a non-member, or [`ConsensusError::InvalidParameter`] for zero delays
    /// or a drop rate above 1000 per-mille.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let mut cfg = SimConfig::new(4, 1);
    /// cfg.n = 3; // below the four-replica minimum
    /// assert!(Simulator::new(cfg).is_err());
    /// ```
    pub fn new(config: SimConfig) -> Result<Self, ConsensusError> {
        let consensus =
            Config::new(config.n)?.with_timeouts(config.base_timeout, config.max_backoff_shift)?;
        if config.pre_gst_max_delay == 0 || config.post_gst_max_delay == 0 {
            return Err(ConsensusError::InvalidParameter {
                what: "maximum delay must be at least 1 tick",
            });
        }
        if config.pre_gst_drop_per_mille > 1000 {
            return Err(ConsensusError::InvalidParameter {
                what: "drop rate must be at most 1000 per-mille",
            });
        }
        let n = usize::from(config.n);
        let mut faults = alloc::vec![None; n];
        for spec in &config.faults {
            let slot = faults
                .get_mut(usize::from(spec.replica.0))
                .ok_or(ConsensusError::UnknownReplica { id: spec.replica.0 })?;
            *slot = Some(spec.kind);
        }
        let replicas = (0..config.n)
            .map(|i| Replica::new(ReplicaId(i), consensus))
            .collect::<Result<Vec<_>, _>>()?;
        let mut sim = Self {
            rng: SplitMix64::new(config.seed),
            config,
            consensus,
            replicas,
            faults,
            queue: BTreeMap::new(),
            seq: 0,
            now: 0,
            stats: SimStats::default(),
            honest_rejections: Vec::new(),
        };
        for i in 0..sim.consensus.n() {
            let id = ReplicaId(i);
            if !sim.is_alive(id) {
                continue;
            }
            let result = match sim.replicas.get_mut(usize::from(i)) {
                Some(r) => r.start(0),
                None => continue,
            };
            sim.after_call(id, id, result);
        }
        sim.now = 1;
        Ok(sim)
    }

    /// Consensus configuration in use.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let sim = Simulator::new(SimConfig::new(4, 1))?;
    /// assert_eq!(sim.consensus_config().n(), 4);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn consensus_config(&self) -> &Config {
        &self.consensus
    }

    /// Current tick (the next tick [`Simulator::step`] will execute).
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
    /// assert_eq!(sim.now(), 1); // the first tick is 1
    /// sim.step();
    /// assert_eq!(sim.now(), 2);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn now(&self) -> u64 {
        self.now
    }

    /// Counters so far.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
    /// sim.run(50);
    /// assert!(sim.stats().delivered > 0);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn stats(&self) -> &SimStats {
        &self.stats
    }

    /// Up to 16 rejections of honest-sender messages `(from, to, error)`, for
    /// diagnostics. Expected to be empty.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
    /// sim.run(100);
    /// // An honest run rejects no honest message.
    /// assert!(sim.honest_rejections().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn honest_rejections(&self) -> &[(ReplicaId, ReplicaId, ConsensusError)] {
        &self.honest_rejections
    }

    /// A replica by id.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{sim::{SimConfig, Simulator}, ReplicaId};
    ///
    /// let sim = Simulator::new(SimConfig::new(4, 1))?;
    /// assert!(sim.replica(ReplicaId(2)).is_some());
    /// assert!(sim.replica(ReplicaId(9)).is_none());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn replica(&self, id: ReplicaId) -> Option<&Replica> {
        self.replicas.get(usize::from(id.0))
    }

    /// Whether `id` has no injected fault.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::{sim::{FaultKind, FaultSpec, SimConfig, Simulator}, ReplicaId};
    ///
    /// let mut cfg = SimConfig::new(4, 1);
    /// cfg.faults = vec![FaultSpec { replica: ReplicaId(3), kind: FaultKind::Crash { at_tick: 0 } }];
    /// let sim = Simulator::new(cfg)?;
    /// assert!(!sim.is_honest(ReplicaId(3)));
    /// assert!(sim.is_honest(ReplicaId(0)));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_honest(&self, id: ReplicaId) -> bool {
        matches!(self.faults.get(usize::from(id.0)), Some(None))
    }

    fn fault(&self, id: ReplicaId) -> Option<FaultKind> {
        self.faults.get(usize::from(id.0)).copied().flatten()
    }

    fn is_alive(&self, id: ReplicaId) -> bool {
        match self.fault(id) {
            Some(FaultKind::Crash { at_tick }) => self.now < at_tick,
            _ => true,
        }
    }

    fn is_equivocator(&self, id: ReplicaId) -> bool {
        self.fault(id) == Some(FaultKind::Equivocate)
    }

    /// Executes one tick: delivers due messages, then ticks every live replica.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
    /// sim.step(); // one tick: deliver due messages, then fire due timers
    /// assert_eq!(sim.now(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn step(&mut self) {
        let now = self.now;
        let due: Vec<(u64, u64)> = self
            .queue
            .range(..=(now, u64::MAX))
            .map(|(k, _)| *k)
            .collect();
        for key in due {
            if let Some(msg) = self.queue.remove(&key) {
                self.deliver(msg);
            }
        }
        for i in 0..self.consensus.n() {
            let id = ReplicaId(i);
            if !self.is_alive(id) {
                continue;
            }
            let result = match self.replicas.get_mut(usize::from(i)) {
                Some(r) => r.tick(now),
                None => continue,
            };
            self.after_call(id, id, result);
        }
        self.now = now.saturating_add(1);
    }

    /// Executes `ticks` ticks.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
    /// sim.run(120);
    /// assert_eq!(sim.now(), 121);
    /// # Ok(())
    /// # }
    /// ```
    pub fn run(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.step();
        }
    }

    /// Snapshot of the committed logs and counters.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use tpt_algo_consensus::sim::{SimConfig, Simulator};
    ///
    /// let mut sim = Simulator::new(SimConfig::new(4, 1))?;
    /// sim.run(100);
    /// let report = sim.report();
    /// assert!(report.max_honest_view.0 > 0); // views advanced
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn report(&self) -> SimReport {
        let honest: Vec<&Replica> = self
            .replicas
            .iter()
            .filter(|r| self.is_honest(r.id()))
            .collect();
        SimReport {
            honest_logs: honest
                .iter()
                .map(|r| (r.id(), r.committed().to_vec()))
                .collect(),
            stats: self.stats.clone(),
            max_honest_view: honest
                .iter()
                .map(|r| r.current_view())
                .max()
                .unwrap_or(View::GENESIS),
            now: self.now,
        }
    }

    fn deliver(&mut self, msg: InFlight) {
        if !self.is_alive(msg.to) {
            return;
        }
        self.stats.delivered = self.stats.delivered.saturating_add(1);
        let now = self.now;
        let result = match self.replicas.get_mut(usize::from(msg.to.0)) {
            Some(r) => r.handle(now, msg.from, &msg.message),
            None => return,
        };
        self.after_call(msg.from, msg.to, result);
        // An equivocating replica also votes for every proposal it sees.
        if self.is_equivocator(msg.to) {
            if let Message::Proposal(b) = msg.message {
                if msg.from == b.proposer {
                    let leader = self.consensus.leader(b.view.next());
                    let vote = Vote {
                        view: b.view,
                        block: b.id,
                    };
                    if leader != msg.to {
                        self.enqueue(msg.to, leader, Message::Vote(vote));
                    }
                }
            }
        }
    }

    fn after_call(
        &mut self,
        from: ReplicaId,
        to: ReplicaId,
        result: Result<Vec<Outgoing>, ConsensusError>,
    ) {
        match result {
            Ok(out) => self.route(to, out),
            Err(e) => self.record_error(from, to, e),
        }
    }

    fn record_error(&mut self, from: ReplicaId, to: ReplicaId, e: ConsensusError) {
        if matches!(e, ConsensusError::ConflictingCommit { .. }) && self.is_honest(to) {
            self.stats.safety_violations = self.stats.safety_violations.saturating_add(1);
        } else if self.is_honest(from) {
            self.stats.rejected_from_honest = self.stats.rejected_from_honest.saturating_add(1);
            if self.honest_rejections.len() < MAX_RECORDED_REJECTIONS {
                self.honest_rejections.push((from, to, e));
            }
        } else {
            self.stats.rejected_from_faulty = self.stats.rejected_from_faulty.saturating_add(1);
        }
    }

    fn route(&mut self, from: ReplicaId, out: Vec<Outgoing>) {
        let equivocator = self.is_equivocator(from);
        for o in out {
            match (o.to, o.message) {
                (Recipient::All, Message::Proposal(b)) if equivocator => {
                    self.equivocate(from, b);
                }
                (Recipient::All, Message::NewView(nv)) if equivocator => {
                    self.broadcast(from, Message::NewView(nv));
                    let inflated = NewView {
                        view: View(nv.view.0.saturating_add(INFLATED_VIEW_OFFSET)),
                        ..nv
                    };
                    self.broadcast(from, Message::NewView(inflated));
                }
                (Recipient::All, m) => self.broadcast(from, m),
                (Recipient::One(to), m) => {
                    if to != from {
                        self.enqueue(from, to, m);
                    }
                }
            }
        }
    }

    fn equivocate(&mut self, from: ReplicaId, b: Block) {
        let alt = self
            .replica(from)
            .and_then(|r| r.block(b.parent).copied())
            .and_then(|parent| {
                Block::new(
                    &parent,
                    b.view,
                    b.proposer,
                    Some(
                        b.payload
                            .map_or(EQUIVOCATION_MASK, |p| p ^ EQUIVOCATION_MASK),
                    ),
                    b.justify,
                )
                .ok()
            });
        let Some(alt) = alt else {
            self.broadcast(from, Message::Proposal(b));
            return;
        };
        self.stats.equivocations = self.stats.equivocations.saturating_add(1);
        for i in 0..self.consensus.n() {
            let to = ReplicaId(i);
            if to == from {
                continue;
            }
            let chosen = if i % 2 == 0 { b } else { alt };
            self.enqueue(from, to, Message::Proposal(chosen));
        }
        let leader = self.consensus.leader(b.view.next());
        if leader != from {
            let vote = Vote {
                view: alt.view,
                block: alt.id,
            };
            self.enqueue(from, leader, Message::Vote(vote));
        }
    }

    fn broadcast(&mut self, from: ReplicaId, message: Message) {
        for i in 0..self.consensus.n() {
            let to = ReplicaId(i);
            if to != from {
                self.enqueue(from, to, message);
            }
        }
    }

    fn enqueue(&mut self, from: ReplicaId, to: ReplicaId, message: Message) {
        let pre_gst = self.now < self.config.gst;
        if pre_gst && self.rng.below(1000) < u64::from(self.config.pre_gst_drop_per_mille) {
            self.stats.dropped = self.stats.dropped.saturating_add(1);
            return;
        }
        let max_delay = if pre_gst {
            self.config.pre_gst_max_delay
        } else {
            self.config.post_gst_max_delay
        };
        let delay = 1 + self.rng.below(max_delay);
        let at = self.now.saturating_add(delay);
        self.queue
            .insert((at, self.seq), InFlight { from, to, message });
        self.seq = self.seq.wrapping_add(1);
    }
}

/// Checks that committed logs are pairwise prefix-consistent (no two logs
/// hold different blocks at the same height) and internally well formed
/// (heights `1, 2, ...` and parent links).
///
/// # Errors
/// [`ConsensusError::ConflictingCommit`] with the first offending height.
///
/// # Example
///
/// ```
/// use tpt_algo_consensus::{sim::check_prefix_consistency, ReplicaId};
///
/// // Empty logs are trivially consistent.
/// assert!(check_prefix_consistency(&[(ReplicaId(0), Vec::new())]).is_ok());
/// ```
pub fn check_prefix_consistency(logs: &[(ReplicaId, Vec<Block>)]) -> Result<(), ConsensusError> {
    let mut canonical: Vec<Block> = Vec::new();
    for (_, log) in logs {
        let mut prev = Block::genesis();
        for (idx, b) in log.iter().enumerate() {
            let height = prev.height.saturating_add(1);
            if b.height != height || b.parent != prev.id {
                return Err(ConsensusError::ConflictingCommit { height });
            }
            match canonical.get(idx) {
                Some(c) if c.id != b.id => {
                    return Err(ConsensusError::ConflictingCommit { height });
                }
                Some(_) => {}
                None => canonical.push(*b),
            }
            prev = *b;
        }
    }
    Ok(())
}
