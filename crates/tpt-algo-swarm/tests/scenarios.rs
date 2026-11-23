// SPDX-License-Identifier: MIT OR Apache-2.0
//! Seeded synthetic scenarios: collision-free and time-to-goal criteria.
#![cfg(feature = "alloc")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use tpt_algo_swarm::{avoid, Agent, AvoidanceConfig, Responsibility, Simulation, Vec2};

const RADIUS: f64 = 0.5;
const SPEED: f64 = 1.0;
const DT: f64 = 0.1;
const EPS: f64 = 1e-6;

fn agent(p: Vec2) -> Agent {
    Agent {
        position: p,
        velocity: Vec2::ZERO,
        radius: RADIUS,
        preferred_velocity: Vec2::ZERO,
        max_speed: SPEED,
    }
}

/// Runs to completion; returns (min clearance, steps used or None).
fn run(sim: &mut Simulation, cfg: &AvoidanceConfig, budget: usize) -> (f64, Option<usize>) {
    let mut min_c = f64::INFINITY;
    for step in 0..budget {
        if sim.all_at_goals(0.1) {
            return (min_c, Some(step));
        }
        sim.step(DT, cfg).unwrap();
        min_c = min_c.min(sim.min_clearance().unwrap());
    }
    (min_c, sim.all_at_goals(0.1).then_some(budget))
}

fn circle(n: usize, radius: f64, phase: f64) -> Simulation {
    let mut agents = Vec::new();
    let mut goals = Vec::new();
    for k in 0..n {
        let th = phase + core::f64::consts::TAU * k as f64 / n as f64;
        let p = Vec2::new(radius * th.cos(), radius * th.sin());
        agents.push(agent(p));
        goals.push(-p);
    }
    Simulation::new(agents, goals).unwrap()
}

#[test]
fn crossing_two_agents() {
    let a = agent(Vec2::new(-5.0, 0.0));
    let b = agent(Vec2::new(0.0, -5.0));
    let mut sim =
        Simulation::new(vec![a, b], vec![Vec2::new(5.0, 0.0), Vec2::new(0.0, 5.0)]).unwrap();
    let (min_c, steps) = run(&mut sim, &AvoidanceConfig::default(), 400);
    assert!(min_c >= -EPS, "overlap: clearance {min_c}");
    let steps = steps.expect("goals not reached");
    // Straight-line time is 100 steps; allow generous detour.
    assert!(steps <= 200, "too slow: {steps}");
}

#[test]
fn head_on_two_agents() {
    // Exactly symmetric head-on: classic deadlock-prone case.
    let a = agent(Vec2::new(-5.0, 0.0));
    let b = agent(Vec2::new(5.0, 0.0));
    let mut sim =
        Simulation::new(vec![a, b], vec![Vec2::new(5.0, 0.0), Vec2::new(-5.0, 0.0)]).unwrap();
    let (min_c, steps) = run(&mut sim, &AvoidanceConfig::default(), 600);
    assert!(min_c >= -EPS, "overlap: clearance {min_c}");
    assert!(steps.is_some(), "goals not reached");
}

#[test]
fn circle_swap_four_and_six() {
    for n in [3usize, 4, 6] {
        let mut sim = circle(n, 6.0, 0.1);
        let (min_c, steps) = run(&mut sim, &AvoidanceConfig::default(), 1200);
        assert!(min_c >= -EPS, "n={n}: overlap: clearance {min_c}");
        let steps = steps.unwrap_or_else(|| panic!("n={n}: goals not reached"));
        assert!(steps <= 1000, "n={n}: too slow {steps}");
    }
}

#[test]
fn full_responsibility_avoids_static_obstacle() {
    // Only the walker reacts; the blocker is a passive static obstacle, which
    // is the situation classic (non-reciprocal) VO is designed for.
    let cfg = AvoidanceConfig {
        responsibility: Responsibility::Full,
        ..AvoidanceConfig::default()
    };
    let blocker = agent(Vec2::new(0.0, 0.0));
    let mut walker = agent(Vec2::new(-5.0, 0.1));
    let goal = Vec2::new(5.0, 0.1);
    let mut min_c = f64::INFINITY;
    let mut reached = false;
    for _ in 0..600 {
        let to_goal = goal - walker.position;
        if to_goal.norm() <= 0.1 {
            reached = true;
            break;
        }
        walker.preferred_velocity = to_goal * (SPEED.min(to_goal.norm() / DT) / to_goal.norm());
        walker.velocity = avoid(&walker, &[blocker], &cfg).unwrap();
        walker.position = walker.position + walker.velocity * DT;
        min_c = min_c.min((walker.position - blocker.position).norm() - 2.0 * RADIUS);
    }
    assert!(min_c >= -EPS, "overlap: clearance {min_c}");
    assert!(reached);
}

#[test]
fn simulation_is_deterministic() {
    let mut s1 = circle(5, 6.0, 0.3);
    let mut s2 = s1.clone();
    let cfg = AvoidanceConfig::default();
    for _ in 0..50 {
        s1.step(DT, &cfg).unwrap();
        s2.step(DT, &cfg).unwrap();
    }
    assert_eq!(s1, s2);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Circle swaps of 2..=5 agents with arbitrary phase stay collision free
    /// and reach their goals.
    #[test]
    fn circle_swap_invariants(n in 2usize..=5, phase in 0.0f64..core::f64::consts::TAU) {
        let mut sim = circle(n, 6.0, phase);
        let (min_c, steps) = run(&mut sim, &AvoidanceConfig::default(), 1500);
        prop_assert!(min_c >= -EPS, "clearance {}", min_c);
        prop_assert!(steps.is_some(), "goals not reached");
    }

    /// Two agents on arbitrary crossing courses never overlap.
    #[test]
    fn crossing_invariants(angle in 0.3f64..2.8, dist in 4.0f64..8.0) {
        let a = agent(Vec2::new(-dist, 0.0));
        let b = agent(Vec2::new(dist * angle.cos(), dist * angle.sin()));
        let mut sim = Simulation::new(
            vec![a, b],
            vec![Vec2::new(dist, 0.0), Vec2::new(-dist * angle.cos(), -dist * angle.sin())],
        ).unwrap();
        let (min_c, steps) = run(&mut sim, &AvoidanceConfig::default(), 1500);
        prop_assert!(min_c >= -EPS, "clearance {}", min_c);
        prop_assert!(steps.is_some(), "goals not reached");
    }
}
