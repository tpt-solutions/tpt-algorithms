// SPDX-License-Identifier: MIT OR Apache-2.0
//! Runs a seeded 4-replica simulation with one equivocating (Byzantine)
//! replica and a lossy network before the stabilisation time, then prints the
//! committed logs of the honest replicas.
//!
//! `cargo run -p tpt-algo-consensus --example simulate`

#[cfg(feature = "alloc")]
fn main() -> Result<(), tpt_algo_consensus::ConsensusError> {
    use tpt_algo_consensus::sim::{
        check_prefix_consistency, FaultKind, FaultSpec, SimConfig, Simulator,
    };
    use tpt_algo_consensus::ReplicaId;

    let config = SimConfig {
        gst: 200,
        pre_gst_drop_per_mille: 200,
        pre_gst_max_delay: 8,
        post_gst_max_delay: 2,
        faults: vec![FaultSpec {
            replica: ReplicaId(1),
            kind: FaultKind::Equivocate,
        }],
        ..SimConfig::new(4, 2024)
    };
    let mut sim = Simulator::new(config)?;
    sim.run(600);
    let report = sim.report();
    check_prefix_consistency(&report.honest_logs)?;

    println!(
        "tick {} | highest honest view {}",
        report.now, report.max_honest_view
    );
    println!("{:?}", report.stats);
    for (id, log) in &report.honest_logs {
        let views: Vec<u64> = log.iter().take(12).map(|b| b.view.0).collect();
        println!("{id}: {} committed, first views {views:?}", log.len());
    }
    Ok(())
}

#[cfg(not(feature = "alloc"))]
fn main() {
    println!("this example requires the `alloc` feature");
}
