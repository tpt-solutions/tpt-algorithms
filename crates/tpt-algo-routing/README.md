# tpt-algo-routing

Vehicle routing (CVRP, optionally with time windows): Clarke & Wright savings
construction followed by 2-opt, relocate and swap local search. Pure Rust,
deterministic, `no_std` compatible (with `alloc`).

## Status

Pre-release (0.0.0). API may change.

## Feature flags

- `std` (default): implies `alloc`.
- `alloc`: problem model and solvers (without it only the error type exists).
- `serde`: serde support for plain data types (`Customer`, `TimeWindow`, `Config`).

## Usage

```rust
use tpt_algo_routing::{solve, Config, Customer, Problem};

let customers = vec![
    Customer::new(1.0, 0.0, 5.0),
    Customer::new(2.0, 0.0, 5.0),
    Customer::new(-1.0, 0.0, 5.0),
    Customer::new(-2.0, 0.0, 5.0),
];
let problem = Problem::new((0.0, 0.0), 10.0, customers).unwrap();
let solution = solve(&problem, &Config::default()).unwrap();
assert!((solution.cost(&problem) - 8.0).abs() < 1e-9);
```

## Validation status

Known-answer and property tests (coverage, capacity, time windows, monotone
local search) run under `cargo test`. Comparison against CVRPLIB / Solomon
best-known solutions is implemented in `tests/validation.rs` but is `#[ignore]`d
and has not been run (requires `TPT_DATASET_DIR`).

## Licence

MIT OR Apache-2.0
