// SPDX-License-Identifier: MIT OR Apache-2.0
//! Replica identifiers, views, signer sets and the group configuration.

use core::fmt;

use crate::error::ConsensusError;

/// Largest replica group supported by [`SignerSet`] and therefore by [`Config`].
pub const MAX_REPLICAS: u16 = 256;

const WORDS: usize = (MAX_REPLICAS as usize) / 64;

/// Identifier of a replica, in `0..n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplicaId(pub u16);

impl fmt::Display for ReplicaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

/// A protocol view (round) number. View 0 is reserved for the genesis block;
/// the first proposal happens in view 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct View(pub u64);

impl View {
    /// The genesis view.
    pub const GENESIS: View = View(0);

    /// The following view, saturating at `u64::MAX`.
    #[must_use]
    pub const fn next(self) -> View {
        View(self.0.saturating_add(1))
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Fixed-capacity bitset of replica ids (capacity [`MAX_REPLICAS`]), used as
/// the signer set of a quorum certificate and for vote aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignerSet {
    words: [u64; WORDS],
}

impl SignerSet {
    /// The empty set.
    #[must_use]
    pub const fn new() -> Self {
        Self { words: [0; WORDS] }
    }

    fn slot(id: ReplicaId) -> (usize, u64) {
        (usize::from(id.0) / 64, 1u64 << (id.0 % 64))
    }

    /// Inserts `id`. Returns `Ok(true)` if it was not already present.
    ///
    /// # Errors
    /// [`ConsensusError::UnknownReplica`] if `id >= MAX_REPLICAS`.
    pub fn insert(&mut self, id: ReplicaId) -> Result<bool, ConsensusError> {
        let (w, bit) = Self::slot(id);
        let word = self
            .words
            .get_mut(w)
            .ok_or(ConsensusError::UnknownReplica { id: id.0 })?;
        let fresh = *word & bit == 0;
        *word |= bit;
        Ok(fresh)
    }

    /// Whether `id` is a member.
    #[must_use]
    pub fn contains(&self, id: ReplicaId) -> bool {
        let (w, bit) = Self::slot(id);
        self.words.get(w).is_some_and(|x| x & bit != 0)
    }

    /// Number of members.
    #[must_use]
    pub fn len(&self) -> u16 {
        let total: u32 = self.words.iter().map(|w| w.count_ones()).sum();
        u16::try_from(total).unwrap_or(u16::MAX)
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|w| *w == 0)
    }

    /// Iterates members in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = ReplicaId> + '_ {
        (0..MAX_REPLICAS)
            .map(ReplicaId)
            .filter(move |id| self.contains(*id))
    }

    /// Set union.
    #[must_use]
    pub fn union(&self, other: &SignerSet) -> SignerSet {
        let mut words = self.words;
        for (w, o) in words.iter_mut().zip(other.words.iter()) {
            *w |= *o;
        }
        SignerSet { words }
    }

    /// Whether every member is `< n`.
    #[must_use]
    pub fn all_below(&self, n: u16) -> bool {
        self.iter().all(|id| id.0 < n)
    }
}

/// Validated replica-group configuration.
///
/// `f = (n - 1) / 3` faults are tolerated and the quorum is `n - f`, which
/// equals `2f + 1` when `n = 3f + 1` and keeps any two quorums intersecting in
/// at least `f + 1` replicas for other `n`. All arithmetic is integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    n: u16,
    f: u16,
    quorum: u16,
    base_timeout: u64,
    max_backoff_shift: u32,
}

impl Config {
    /// Default base view timeout, in logical ticks.
    pub const DEFAULT_BASE_TIMEOUT: u64 = 20;
    /// Default cap on the exponential back-off exponent.
    pub const DEFAULT_MAX_BACKOFF_SHIFT: u32 = 4;
    /// Largest accepted back-off exponent.
    pub const MAX_BACKOFF_SHIFT_LIMIT: u32 = 16;

    /// Creates a configuration for `n` replicas with default timeouts.
    ///
    /// # Errors
    /// [`ConsensusError::TooFewReplicas`] if `n < 4`,
    /// [`ConsensusError::TooManyReplicas`] if `n > MAX_REPLICAS`.
    pub fn new(n: u16) -> Result<Self, ConsensusError> {
        if n < 4 {
            return Err(ConsensusError::TooFewReplicas { n });
        }
        if n > MAX_REPLICAS {
            return Err(ConsensusError::TooManyReplicas {
                n,
                max: MAX_REPLICAS,
            });
        }
        let f = (n - 1) / 3;
        Ok(Self {
            n,
            f,
            quorum: n - f,
            base_timeout: Self::DEFAULT_BASE_TIMEOUT,
            max_backoff_shift: Self::DEFAULT_MAX_BACKOFF_SHIFT,
        })
    }

    /// Returns a copy with the given pacemaker timeouts.
    ///
    /// A view that ends by timeout doubles the next timeout, up to
    /// `base_timeout << max_backoff_shift`.
    ///
    /// # Errors
    /// [`ConsensusError::InvalidTimeout`] if `base_timeout == 0` or
    /// `max_backoff_shift > MAX_BACKOFF_SHIFT_LIMIT`.
    pub fn with_timeouts(
        self,
        base_timeout: u64,
        max_backoff_shift: u32,
    ) -> Result<Self, ConsensusError> {
        if base_timeout == 0 || max_backoff_shift > Self::MAX_BACKOFF_SHIFT_LIMIT {
            return Err(ConsensusError::InvalidTimeout);
        }
        Ok(Self {
            base_timeout,
            max_backoff_shift,
            ..self
        })
    }

    /// Number of replicas.
    #[must_use]
    pub const fn n(&self) -> u16 {
        self.n
    }

    /// Number of tolerated Byzantine replicas.
    #[must_use]
    pub const fn f(&self) -> u16 {
        self.f
    }

    /// Quorum size (`n - f`).
    #[must_use]
    pub const fn quorum(&self) -> u16 {
        self.quorum
    }

    /// Base view timeout in ticks.
    #[must_use]
    pub const fn base_timeout(&self) -> u64 {
        self.base_timeout
    }

    /// Cap on the back-off exponent.
    #[must_use]
    pub const fn max_backoff_shift(&self) -> u32 {
        self.max_backoff_shift
    }

    /// Whether `id` is a member of the group.
    #[must_use]
    pub const fn contains(&self, id: ReplicaId) -> bool {
        id.0 < self.n
    }

    /// Round-robin leader of `view`: `view mod n`.
    #[must_use]
    pub fn leader(&self, view: View) -> ReplicaId {
        let idx = view.0 % u64::from(self.n);
        ReplicaId(u16::try_from(idx).unwrap_or(0))
    }

    /// View timeout after `consecutive_timeouts` views in a row ended by timeout.
    #[must_use]
    pub fn timeout_for(&self, consecutive_timeouts: u32) -> u64 {
        let shift = consecutive_timeouts.min(self.max_backoff_shift);
        self.base_timeout.saturating_mul(1u64 << shift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_quorums() {
        assert_eq!(Config::new(3), Err(ConsensusError::TooFewReplicas { n: 3 }));
        let c = Config::new(4).unwrap();
        assert_eq!((c.f(), c.quorum()), (1, 3));
        let c = Config::new(5).unwrap();
        assert_eq!((c.f(), c.quorum()), (1, 4));
        let c = Config::new(7).unwrap();
        assert_eq!((c.f(), c.quorum()), (2, 5));
        let c = Config::new(256).unwrap();
        assert_eq!((c.f(), c.quorum()), (85, 171));
        assert!(Config::new(257).is_err());
    }

    #[test]
    fn quorum_intersection_holds_for_all_sizes() {
        for n in 4..=MAX_REPLICAS {
            let c = Config::new(n).unwrap();
            let (n, f, q) = (u32::from(n), u32::from(c.f()), u32::from(c.quorum()));
            assert!(n > 3 * f);
            // Two quorums share at least f + 1 replicas (one honest).
            assert!(2 * q > n + f);
            // The honest replicas alone can form a quorum.
            assert!(q <= n - f);
        }
    }

    #[test]
    fn leader_rotation_and_timeouts() {
        let c = Config::new(4).unwrap().with_timeouts(10, 3).unwrap();
        assert_eq!(c.leader(View(0)), ReplicaId(0));
        assert_eq!(c.leader(View(5)), ReplicaId(1));
        assert_eq!(c.timeout_for(0), 10);
        assert_eq!(c.timeout_for(2), 40);
        assert_eq!(c.timeout_for(9), 80);
        assert!(c.with_timeouts(0, 1).is_err());
        assert!(c.with_timeouts(1, 17).is_err());
    }

    #[test]
    fn signer_set_basics() {
        let mut s = SignerSet::new();
        assert!(s.is_empty());
        assert!(s.insert(ReplicaId(3)).unwrap());
        assert!(!s.insert(ReplicaId(3)).unwrap());
        assert!(s.insert(ReplicaId(255)).unwrap());
        assert!(s.insert(ReplicaId(256)).is_err());
        assert_eq!(s.len(), 2);
        assert!(s.contains(ReplicaId(255)));
        assert!(!s.all_below(4));
        assert_eq!(s.iter().next(), Some(ReplicaId(3)));
    }
}
