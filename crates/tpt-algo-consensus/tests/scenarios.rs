// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scripted known-answer scenarios and fault scenarios.
#![cfg(feature = "alloc")]

use tpt_algo_consensus::sim::{
    check_prefix_consistency, FaultKind, FaultSpec, SimConfig, Simulator,
};
use tpt_algo_consensus::{Block, ConsensusError, QuorumCert, ReplicaId, SignerSet, View};

/// The chain an honest, fault-free 4-replica group must build with unit
/// delays: one block per view, proposed by `v mod 4`, payload 0, each
/// justified by a QC for its parent. The QC for view `p` is formed by the
/// leader of `p + 1` from the first three votes it receives: its own, the
/// proposer of `p` (whose vote arrives one tick earlier) and, among the
/// remaining two votes that arrive together, the lower replica id (FIFO
/// tie-break by send order).
fn expected_chain(views: u64) -> Result<Vec<Block>, ConsensusError> {
    let mut chain = Vec::new();
    let mut parent = Block::genesis();
    for v in 1..=views {
        let justify = if parent.view == View::GENESIS {
            QuorumCert::genesis()
        } else {
            let p = parent.view.0;
            let next_leader = ((p + 1) % 4) as u16;
            let proposer = (p % 4) as u16;
            let third = (0..4u16)
                .find(|i| *i != next_leader && *i != proposer)
                .unwrap_or_default();
            let mut signers = SignerSet::new();
            for id in [next_leader, proposer, third] {
                signers.insert(ReplicaId(id))?;
            }
            QuorumCert {
                view: parent.view,
                block: parent.id,
                signers,
            }
        };
        let proposer = ReplicaId((v % 4) as u16);
        let b = Block::new(&parent, View(v), proposer, 0, justify)?;
        chain.push(b);
        parent = b;
    }
    Ok(chain)
}

#[test]
fn known_answer_four_replicas_no_faults() {
    // Unit delays, no drops, no faults: view v is proposed at tick 2v - 2, its
    // QC forms at tick 2v, and b_k commits when QC(b_{k+2}) is seen: tick
    // 2k + 4 at the leader of view k + 3, tick 2k + 5 elsewhere.
    let mut sim = Simulator::new(SimConfig::new(4, 1)).unwrap();
    sim.run(20);
    let report = sim.report();
    let expected = expected_chain(8).unwrap();
    let lens: Vec<usize> = report.honest_logs.iter().map(|(_, l)| l.len()).collect();
    assert_eq!(lens, vec![7, 7, 7, 8]);
    for (id, log) in &report.honest_logs {
        assert_eq!(log.as_slice(), &expected[..log.len()], "replica {id}");
    }
    // Exact committed sequence: views 1..=8, proposers 1,2,3,0,1,2,3,0.
    let (_, longest) = &report.honest_logs[3];
    let views: Vec<u64> = longest.iter().map(|b| b.view.0).collect();
    let proposers: Vec<u16> = longest.iter().map(|b| b.proposer.0).collect();
    assert_eq!(views, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(proposers, vec![1, 2, 3, 0, 1, 2, 3, 0]);
    assert_eq!(report.stats.rejected_from_honest, 0);
    assert_eq!(report.stats.safety_violations, 0);
    // The leader of view 11 formed QC(10) at tick 20, proposed b11, voted for
    // it and moved to view 12.
    assert_eq!(report.max_honest_view, View(12));
}

#[test]
fn known_answer_is_reproducible_across_seeds_with_unit_delay() {
    // With unit delays the seed only affects nothing observable.
    let a = {
        let mut s = Simulator::new(SimConfig::new(4, 1)).unwrap();
        s.run(30);
        s.report()
    };
    let b = {
        let mut s = Simulator::new(SimConfig::new(4, 999)).unwrap();
        s.run(30);
        s.report()
    };
    assert_eq!(a, b);
}

#[test]
fn same_seed_same_run_under_random_delays() {
    let cfg = SimConfig {
        pre_gst_max_delay: 7,
        post_gst_max_delay: 3,
        pre_gst_drop_per_mille: 150,
        gst: 100,
        ..SimConfig::new(7, 12345)
    };
    let run = |c: SimConfig| {
        let mut s = Simulator::new(c).unwrap();
        s.run(300);
        s.report()
    };
    assert_eq!(run(cfg.clone()), run(cfg));
}

#[test]
fn crashed_replica_does_not_block_progress() {
    let cfg = SimConfig {
        faults: vec![FaultSpec {
            replica: ReplicaId(0),
            kind: FaultKind::Crash { at_tick: 0 },
        }],
        ..SimConfig::new(4, 3)
    };
    let mut sim = Simulator::new(cfg).unwrap();
    sim.run(400);
    let report = sim.report();
    check_prefix_consistency(&report.honest_logs).unwrap();
    assert_eq!(report.honest_logs.len(), 3);
    for (_, log) in &report.honest_logs {
        assert!(log.len() >= 10, "only {} commits", log.len());
        // Replica 0 is down, so it never proposes a committed block.
        assert!(log.iter().all(|b| b.proposer != ReplicaId(0)));
    }
    assert_eq!(report.stats.rejected_from_honest, 0);
}

#[test]
fn crash_in_the_middle_of_a_run() {
    let cfg = SimConfig {
        faults: vec![FaultSpec {
            replica: ReplicaId(2),
            kind: FaultKind::Crash { at_tick: 37 },
        }],
        post_gst_max_delay: 2,
        ..SimConfig::new(4, 8)
    };
    let mut sim = Simulator::new(cfg).unwrap();
    sim.run(500);
    let report = sim.report();
    check_prefix_consistency(&report.honest_logs).unwrap();
    assert!(report.honest_logs.iter().all(|(_, l)| l.len() >= 20));
}

#[test]
fn equivocating_leader_is_safe_and_live() {
    let cfg = SimConfig {
        faults: vec![FaultSpec {
            replica: ReplicaId(1),
            kind: FaultKind::Equivocate,
        }],
        post_gst_max_delay: 2,
        ..SimConfig::new(4, 21)
    };
    let mut sim = Simulator::new(cfg).unwrap();
    sim.run(600);
    let report = sim.report();
    check_prefix_consistency(&report.honest_logs).unwrap();
    assert!(report.stats.equivocations > 0);
    assert_eq!(report.stats.safety_violations, 0);
    assert_eq!(
        report.stats.rejected_from_honest,
        0,
        "{:?}",
        sim.honest_rejections()
    );
    assert!(report.honest_logs.iter().all(|(_, l)| l.len() >= 5));
}

#[test]
fn two_byzantine_out_of_seven() {
    let cfg = SimConfig {
        faults: vec![
            FaultSpec {
                replica: ReplicaId(2),
                kind: FaultKind::Equivocate,
            },
            FaultSpec {
                replica: ReplicaId(5),
                kind: FaultKind::Crash { at_tick: 0 },
            },
        ],
        post_gst_max_delay: 3,
        ..SimConfig::new(7, 77)
    };
    let mut sim = Simulator::new(cfg).unwrap();
    sim.run(1500);
    let report = sim.report();
    check_prefix_consistency(&report.honest_logs).unwrap();
    assert_eq!(report.stats.safety_violations, 0);
    assert_eq!(
        report.stats.rejected_from_honest,
        0,
        "{:?}",
        sim.honest_rejections()
    );
    assert!(report.honest_logs.iter().all(|(_, l)| l.len() >= 5));
}

#[test]
fn lossy_network_recovers_after_gst() {
    let cfg = SimConfig {
        gst: 300,
        pre_gst_drop_per_mille: 400,
        pre_gst_max_delay: 15,
        post_gst_max_delay: 2,
        ..SimConfig::new(4, 5)
    };
    let mut sim = Simulator::new(cfg).unwrap();
    sim.run(300);
    let before: Vec<usize> = sim
        .report()
        .honest_logs
        .iter()
        .map(|(_, l)| l.len())
        .collect();
    sim.run(600);
    let report = sim.report();
    check_prefix_consistency(&report.honest_logs).unwrap();
    assert!(report.stats.dropped > 0);
    for ((_, log), b) in report.honest_logs.iter().zip(before) {
        assert!(log.len() > b);
    }
}

#[test]
fn simulator_rejects_bad_parameters() {
    assert_eq!(
        Simulator::new(SimConfig::new(3, 0)).err(),
        Some(ConsensusError::TooFewReplicas { n: 3 })
    );
    let bad_delay = SimConfig {
        post_gst_max_delay: 0,
        ..SimConfig::new(4, 0)
    };
    assert!(matches!(
        Simulator::new(bad_delay).err(),
        Some(ConsensusError::InvalidParameter { .. })
    ));
    let bad_fault = SimConfig {
        faults: vec![FaultSpec {
            replica: ReplicaId(4),
            kind: FaultKind::Equivocate,
        }],
        ..SimConfig::new(4, 0)
    };
    assert_eq!(
        Simulator::new(bad_fault).err(),
        Some(ConsensusError::UnknownReplica { id: 4 })
    );
}

#[test]
fn prefix_checker_detects_conflicts() {
    let chain = expected_chain(3).unwrap();
    let g = Block::genesis();
    let fork = Block::new(&g, View(2), ReplicaId(2), 9, QuorumCert::genesis()).unwrap();
    let logs = vec![(ReplicaId(0), chain.clone()), (ReplicaId(1), vec![fork])];
    assert_eq!(
        check_prefix_consistency(&logs),
        Err(ConsensusError::ConflictingCommit { height: 1 })
    );
    let ok = vec![
        (ReplicaId(0), chain.clone()),
        (ReplicaId(1), chain[..1].to_vec()),
    ];
    check_prefix_consistency(&ok).unwrap();
}
