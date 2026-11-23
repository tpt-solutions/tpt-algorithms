# tpt-algo-swarm

Velocity-obstacle collision avoidance for disc-shaped agents and A* path
planning on occupancy grids. Pure `f64`, deterministic, `no_std`-compatible.

## Status

Pre-release. Implements truncated-cone VO, a sampled reciprocal VO (RVO)
avoidance step, a lock-step `Simulation`, and grid A*. ORCA (half-plane LP) is
not implemented.

## Features

- `std` (default): implies `alloc`.
- `alloc`: `Simulation`, `Grid`, `find_path`.
- `serde`: `Serialize`/`Deserialize` for `Vec2`, `Agent`, `AvoidanceConfig`,
  `Responsibility`.

Without `alloc`, `VelocityObstacle`, `avoid` and `avoid_among` remain
available.

## Usage

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

## Validation

Known-answer unit tests, proptest invariants (circle swaps and crossings:
minimum clearance stays non-negative, all agents reach goals within a step
budget; A* path validity) and seeded synthetic scenarios run under
`cargo test`. Dataset-based checks live in `tests/validation.rs` (ignored;
need `TPT_DATASET_DIR`) and are not yet implemented. No guarantees for dense
or adversarial crowds: sampling-based avoidance can deadlock.

## Licence

MIT OR Apache-2.0
