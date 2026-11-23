// SPDX-License-Identifier: MIT OR Apache-2.0
//! Two agents on crossing courses plus an A* query.
#![cfg_attr(not(feature = "alloc"), allow(unused))]

#[cfg(feature = "alloc")]
use tpt_algo_swarm::{
    find_path, Agent, AvoidanceConfig, Connectivity, Grid, Simulation, SwarmError, Vec2,
};

#[cfg(feature = "alloc")]
fn disc(p: Vec2) -> Agent {
    Agent {
        position: p,
        velocity: Vec2::ZERO,
        radius: 0.5,
        preferred_velocity: Vec2::ZERO,
        max_speed: 1.0,
    }
}

#[cfg(feature = "alloc")]
fn main() -> Result<(), SwarmError> {
    let mut sim = Simulation::new(
        vec![disc(Vec2::new(-5.0, 0.0)), disc(Vec2::new(0.0, -5.0))],
        vec![Vec2::new(5.0, 0.0), Vec2::new(0.0, 5.0)],
    )?;
    let cfg = AvoidanceConfig::default();
    let mut steps = 0;
    while !sim.all_at_goals(0.1) && steps < 1000 {
        sim.step(0.1, &cfg)?;
        steps += 1;
    }
    println!(
        "reached goals after {steps} steps, min clearance {:?}",
        sim.min_clearance()
    );

    let mut grid = Grid::new(5, 5)?;
    for y in 0..4 {
        grid.set_blocked(2, y, true)?;
    }
    let path = find_path(&grid, (0, 0), (4, 0), Connectivity::Eight)?;
    println!("path cost {} via {:?}", path.cost, path.cells);
    Ok(())
}

#[cfg(not(feature = "alloc"))]
fn main() {}
