# tpt-algo-swarm

Velocity-obstacle collision avoidance for disc-shaped agents, and A* path
planning on occupancy grids. The crate is pure `f64`, deterministic, and
`no_std` compatible. It uses no randomness.

Use it to steer a group of robots or simulated agents around each other while
they move toward goals, or to plan grid paths around obstacles. The avoidance
step is local: each agent chooses a velocity from the candidates that keep it
clear of neighbours.

## Status

Pre-release (`0.0.0`). Implements truncated-cone velocity obstacles (VO), a
sampled reciprocal velocity obstacle (RVO) avoidance step, a lock-step
`Simulation`, and grid A*. ORCA (the half-plane linear-programming method) is not
implemented. The crate is not yet published to crates.io; depend on it by path or
git until the first release. See the [changelog](CHANGELOG.md).

## Features

| Feature | Default | Enables |
|---------|---------|---------|
| `std`   | yes     | Implies `alloc`. Uses the standard library. |
| `alloc` | via `std` | `Simulation`, `Grid`, `GridPath`, `Connectivity` and `find_path`. |
| `serde` | no      | `Serialize` / `Deserialize` for `Vec2`, `Agent`, `AvoidanceConfig` and `Responsibility`. |

Without `alloc`, `VelocityObstacle`, `avoid` and `avoid_among` remain available.
Elementary functions come from `libm` in `no_std` builds.

## Installation

```toml
[dependencies]
tpt-algo-swarm = { path = "../tpt-algo-swarm" }
# tpt-algo-swarm = { path = "...", default-features = false }   # no alloc
```

The minimum supported Rust version is `1.98`, matching the workspace.

## Quick start

Avoid one other agent for a single step:

```rust
use tpt_algo_swarm::{avoid, Agent, AvoidanceConfig, Vec2};

let a = Agent {
    position: Vec2::new(0.0, 0.0),
    velocity: Vec2::new(1.0, 0.0),
    radius: 0.5,
    preferred_velocity: Vec2::new(1.0, 0.0),
    max_speed: 1.0,
};
let b = Agent { position: Vec2::new(4.0, 0.0), velocity: Vec2::new(-1.0, 0.0), ..a };
let v = avoid(&a, &[b], &AvoidanceConfig::default()).unwrap();
assert!(v.norm() <= 1.0 + 1e-12);
```

`avoid` returns the admissible velocity closest to `preferred_velocity`.

## Multi-agent simulation

`Simulation` steps all agents together. Each agent avoids the others and moves
toward its goal. Build the agents and goals as in the quick start, one goal per
agent:

```rust
use tpt_algo_swarm::{Agent, AvoidanceConfig, Simulation, Vec2};

fn run(agents: Vec<Agent>, goals: Vec<Vec2>) -> Result<bool, tpt_algo_swarm::SwarmError> {
    let mut sim = Simulation::new(agents, goals)?;
    let cfg = AvoidanceConfig::default();
    for _ in 0..200 {
        sim.step(0.1, &cfg)?; // time step in seconds
    }
    Ok(sim.all_at_goals(0.05))
}
```

## Grid path planning

`Grid` is an occupancy grid. `find_path` runs A* between two cells, using 4- or
8-way movement (`Connectivity::Four` or `Connectivity::Eight`):

```rust
use tpt_algo_swarm::{find_path, Connectivity, Grid};

let mut grid = Grid::new(10, 10).unwrap();
grid.set_blocked(5, 4, true).unwrap(); // a wall cell
let path = find_path(&grid, (0, 0), (9, 9), Connectivity::Eight).unwrap();
```

Use `Grid::from_cells` to build a grid from a row-major vector of blocked flags.

## API overview

| Item | Purpose |
|------|---------|
| `Vec2` | 2D vector with `new`, `norm`, `norm_sq`, `dot` and `clamp_norm`. |
| `Agent` | Position, velocity, radius, preferred velocity and maximum speed. |
| `AvoidanceConfig` | Horizon, safety margin, sampling resolution (`rings`, `angles`) and `responsibility`. |
| `VelocityObstacle` | Truncated-cone velocity obstacle between two agents (`new`, `time_to_collision`). |
| `avoid`, `avoid_among` | Choose the admissible velocity closest to the preferred one. |
| `Responsibility::Full` | Classic VO: the neighbour is assumed to keep its velocity, so the agent avoids all of it. |
| `Responsibility::Reciprocal` | Sampled RVO: each agent takes half the avoidance effort. |
| `Simulation` | Lock-step multi-agent stepping (`new`, `step`, `all_at_goals`, `min_clearance`). |
| `Grid` | Occupancy grid (`new`, `from_cells`, `set_blocked`, `is_blocked`). |
| `find_path`, `GridPath`, `Connectivity` | A* search on a grid. |
| `SwarmError` | Returned for invalid agents, grids and configuration. |

## Examples

```sh
cargo run -p tpt-algo-swarm --example crossing
```

## Validation status

Known-answer unit tests, proptest invariants, and seeded synthetic scenarios run
under `cargo test`:

- Circle swaps and crossings: minimum clearance stays non-negative, and all agents
  reach their goals within a step budget.
- Grid A*: straight-line and start-equals-goal cases, detours around walls,
  a known diagonal cost, no corner cutting, error cases, determinism, and a
  proptest path-validity invariant.

Dataset-based checks live in `tests/validation.rs`. They are `#[ignore]`d, need
`TPT_DATASET_DIR`, and are not yet implemented.

## Limitations

- No guarantees for dense or adversarial crowds. Sampling-based avoidance can
  deadlock, for example when agents face each other head-on in a tight space.
- Avoidance is local and reactive. Agents do not plan around each other, so a
  global path can still be blocked.
- ORCA is not implemented.
- Grid A* plans on a static occupancy grid. It does not handle moving obstacles.

## References

- P. Fiorini, Z. Shiller, *Motion Planning in Dynamic Environments Using Velocity
  Obstacles*, International Journal of Robotics Research, 1998.
- J. van den Berg, M. Lin, D. Manocha, *Reciprocal Velocity Obstacles for
  Real-Time Multi-Agent Navigation*, ICRA 2008.
- J. van den Berg, S. J. Guy, M. Lin, D. Manocha, *Reciprocal n-Body Collision
  Avoidance*, ISRR 2011 (ORCA).
- P. E. Hart, N. J. Nilsson, B. Raphael, *A Formal Basis for the Heuristic
  Determination of Minimum Cost Paths*, IEEE Transactions on Systems Science and
  Cybernetics, 1968.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
