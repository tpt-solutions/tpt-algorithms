// SPDX-License-Identifier: MIT OR Apache-2.0
//! Property-based safety and seeded liveness tests with faulty replicas.
#![cfg(feature = "alloc")]

use proptest::prelude::*;
use tpt_algo_consensus::sim::{
    check_prefix_consistency, FaultKind, FaultSpec, SimConfig, Simulator,
};
use tpt_algo_consensus::{Config, ReplicaId};

/// Builds a fault list: `count` distinct replicas chosen by `picks`, each
/// either crashing at `crash_tick` or equivocating.
fn faults(n: u16, count: u16, picks: &[(u16, bool, u64)]) -> Vec<FaultSpec> {
    let mut out: Vec<FaultSpec> = Vec::new();
    for &(raw, equivocate, crash_tick) in picks {
        if out.len() >= usize::from(count) {
            break;
        }
        let replica = ReplicaId(raw % n);
        if out.iter().any(|f| f.replica == replica) {
            continue;
        }
        let kind = if equivocate {
            FaultKind::Equivocate
        } else {
            FaultKind::Crash {
                at_tick: crash_tick,
            }
        };
        out.push(FaultSpec { replica, kind });
    }
    out
}

fn arb_sim() -> impl Strategy<Value = SimConfig> {
    (
        prop_oneof![Just(4u16), Just(5u16), Just(7u16)],
        any::<u64>(),
        0u16..=2,
        proptest::collection::vec((any::<u16>(), any::<bool>(), 0u64..300), 6),
        0u64..400,
        0u16..=400,
        1u64..=12,
        1u64..=4,
    )
        .prop_map(|(n, seed, nf, picks, gst, drop, pre, post)| {
            let f = Config::new(n).map(|c| c.f()).unwrap_or(1);
            SimConfig {
                gst,
                pre_gst_drop_per_mille: drop,
                pre_gst_max_delay: pre,
                post_gst_max_delay: post,
                faults: faults(n, nf.min(f), &picks),
                ..SimConfig::new(n, seed)
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Safety: with at most f faulty replicas (crashing or equivocating, the
    /// latter also double-voting), under arbitrary seeds, delays, reordering
    /// and drops, no two honest replicas commit different blocks at the same
    /// height, and no honest replica detects a conflicting commit.
    #[test]
    fn no_two_conflicting_commits(cfg in arb_sim()) {
        let mut sim = Simulator::new(cfg).unwrap();
        sim.run(700);
        let report = sim.report();
        prop_assert!(check_prefix_consistency(&report.honest_logs).is_ok());
        prop_assert_eq!(report.stats.safety_violations, 0);
        prop_assert_eq!(report.stats.rejected_from_honest, 0,
            "honest messages rejected: {:?}", sim.honest_rejections());
    }

    /// Safety holds at every intermediate step, not just at the end: the
    /// committed logs only ever grow by appending.
    #[test]
    fn committed_logs_only_grow(cfg in arb_sim()) {
        let mut sim = Simulator::new(cfg).unwrap();
        let mut previous = sim.report().honest_logs;
        for _ in 0..30 {
            sim.run(20);
            let current = sim.report().honest_logs;
            for ((_, old), (_, new)) in previous.iter().zip(&current) {
                prop_assert!(new.len() >= old.len());
                prop_assert_eq!(&new[..old.len()], old.as_slice());
            }
            prop_assert!(check_prefix_consistency(&current).is_ok());
            previous = current;
        }
    }
}

/// Liveness: after the stabilisation time (synchronous delivery, no drops),
/// every honest replica commits a new block within a bounded number of views,
/// with up to f crashed or equivocating replicas.
///
/// The bound is empirical: over 1000 seeds of this generator the worst case
/// observed was 13 views; 24 leaves margin while still catching regressions.
#[test]
fn liveness_after_gst_seeded() {
    const VIEW_BOUND: u64 = 24;
    for seed in 0u64..48 {
        let n = [4u16, 5, 7][usize::try_from(seed % 3).unwrap()];
        let f = Config::new(n).unwrap().f();
        let picks: Vec<(u16, bool, u64)> = (0..6)
            .map(|i| {
                (
                    u16::try_from((seed * 7 + i * 3) % 64).unwrap(),
                    (seed + i) % 2 == 0,
                    (seed * 13 + i * 29) % 250,
                )
            })
            .collect();
        let gst = 200 + (seed % 5) * 40;
        let cfg = SimConfig {
            gst,
            pre_gst_drop_per_mille: 300,
            pre_gst_max_delay: 10,
            post_gst_max_delay: 2,
            faults: faults(n, f, &picks),
            ..SimConfig::new(n, seed.wrapping_mul(0x9E37_79B9))
        };
        let mut sim = Simulator::new(cfg).unwrap();
        sim.run(gst);
        let at_gst = sim.report();
        let baseline: Vec<usize> = at_gst.honest_logs.iter().map(|(_, l)| l.len()).collect();
        let mut done = false;
        for _ in 0..20_000 {
            sim.step();
            let r = sim.report();
            if r.honest_logs
                .iter()
                .zip(&baseline)
                .all(|((_, l), b)| l.len() > *b)
            {
                let views = r.max_honest_view.0 - at_gst.max_honest_view.0;
                assert!(
                    views <= VIEW_BOUND,
                    "seed {seed}: {views} views needed after GST"
                );
                done = true;
                break;
            }
        }
        assert!(done, "seed {seed}: no progress after GST");
        let r = sim.report();
        check_prefix_consistency(&r.honest_logs).unwrap();
        assert_eq!(r.stats.safety_violations, 0);
    }
}
