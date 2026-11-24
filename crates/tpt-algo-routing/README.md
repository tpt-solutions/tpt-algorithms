# tpt-algo-routing

Capacitated vehicle routing (CVRP), optionally with time windows. A single depot
serves customers with a homogeneous fleet. The solver builds an initial solution
with the Clarke and Wright savings heuristic, then improves it with 2-opt,
relocate and swap local search.

The crate is pure Rust and deterministic: the same problem always gives the same
routes. It is `no_std` compatible with `alloc`.

Use it to estimate delivery routes, prototype a dispatch heuristic, or as a
baseline to compare against a stronger solver. It is a heuristic. It does not
prove optimality.

## Status

Pre-release (`0.0.0`). The API may change. The crate is not yet published to
crates.io; depend on it by path or git until the first release. See the
[changelog](CHANGELOG.md).

## Features

| Feature | Default | Enables |
|---------|---------|---------|
| `std`   | yes     | Implies `alloc`. Uses the standard library. |
| `alloc` | via `std` | The problem model and solvers (`Problem`, `Solution`, `solve`, `clarke_wright_savings`, `local_search`). Without it only `RoutingError` exists. |
| `serde` | no      | `Serialize` / `Deserialize` for `Customer`, `TimeWindow` and `Config`. |

## Installation

```toml
[dependencies]
tpt-algo-routing = { path = "../tpt-algo-routing" }
# tpt-algo-routing = { path = "...", default-features = false }   # error type only
```

The minimum supported Rust version is `1.98`, matching the workspace.

## Quick start

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

Here `Customer::new(x, y, demand)` places a customer at `(x, y)` with the given
demand, and `Problem::new(depot, capacity, customers)` sets the depot location and
the vehicle capacity.

## Adding constraints

```rust
use tpt_algo_routing::{Customer, Problem};

let customers = vec![
    Customer::new(1.0, 0.0, 5.0).with_window(0.0, 10.0, 1.0), // earliest, latest, service time
    Customer::new(-1.0, 0.0, 5.0),
];
let problem = Problem::new((0.0, 0.0), 10.0, customers)
    .unwrap()
    .with_max_vehicles(2)  // fleet limit
    .unwrap()
    .with_depot_due(50.0)  // vehicles must return by this time
    .unwrap();
```

Time windows follow these rules. Travel time equals Euclidean distance. Arriving
early means waiting until `earliest`. Arriving after `latest` is infeasible.

## API overview

| Item | Purpose |
|------|---------|
| `Problem::new`, `with_max_vehicles`, `with_depot_due` | Build and constrain a problem. |
| `Customer::new`, `with_window` | Customer location, demand, and optional time window with service time. |
| `TimeWindow` | Earliest and latest arrival times. |
| `solve(&problem, &Config)` | Savings construction followed by local search. |
| `clarke_wright_savings(&problem)` | Savings construction only. |
| `local_search` | 2-opt, relocate and swap improvement. |
| `Config { max_iterations }` | Bound on improving moves. |
| `Solution::cost`, `Solution::vehicles_used` | Quality and fleet size of a solution. |
| `Solution::validate` | Checks a solution against its problem. |
| `RoutingError` | Returned for invalid problems and infeasible inputs. |

## Examples

```sh
cargo run -p tpt-algo-routing --example basic
```

## Validation status

- Known-answer and property tests cover coverage, capacity, time windows, and
  monotone local search. They run under `cargo test`.
- Comparison against CVRPLIB and Solomon best-known solutions is implemented in
  `tests/validation.rs`. It is `#[ignore]`d, needs `TPT_DATASET_DIR`, and has not
  been run.

## Limitations

- Homogeneous fleet with a single depot. There are no heterogeneous vehicles,
  multiple depots, pickup and delivery, or route-length limits.
- Travel time equals Euclidean distance. There is no road network.
- Local search is bounded by `max_iterations`. A solution is not guaranteed to
  be optimal, or even close to it, on large or adversarial instances.

## References

- G. Clarke, J. W. Wright, *Scheduling of Vehicles from a Central Depot to a
  Number of Delivery Points*, Operations Research 12(4), 1964.
- G. Croes, *A Method for Solving Traveling-Salesman Problems*, Operations
  Research 6(6), 1958 (2-opt).
- S. Lin, *Computer Solutions of the Traveling Salesman Problem*, Bell System
  Technical Journal 44, 1965.
- M. M. Solomon, *Algorithms for the Vehicle Routing and Scheduling Problems with
  Time Window Constraints*, Operations Research 35(2), 1987.
- P. Toth, D. Vigo (eds.), *Vehicle Routing: Problems, Methods, and Applications*,
  SIAM, 2nd ed., 2014.

## Benchmarks

Criterion benchmarks for this crate live in [`benches/benches/routing.rs`](../../benches/benches/routing.rs)
(the `tpt-algo-benches` package). Inputs come from `tpt-algo-synth-data` where the crate takes
generated data.

Numbers are produced by the scheduled bench workflow
(`.github/workflows/bench.yml`, weekly plus on demand) on a GitHub-hosted `ubuntu-latest`
runner, and uploaded as the `criterion-<sha>` artifact. They are not measured on a developer
machine and are not portable between runners.

| Benchmark | Measures | Median time |
|-----------|----------|-------------|
| `routing_clarke_wright/50` | Clarke-Wright savings, 50 customers | pending first scheduled run |
| `routing_clarke_wright/200` | Clarke-Wright savings, 200 customers | pending first scheduled run |
| `routing_clarke_wright/500` | Clarke-Wright savings, 500 customers | pending first scheduled run |
| `routing_solve/50` | Full `solve` (savings + local search), 50 customers | pending first scheduled run |
| `routing_solve/200` | Full `solve` (savings + local search), 200 customers | pending first scheduled run |
| `routing_solve/500` | Full `solve` (savings + local search), 500 customers | pending first scheduled run |

To run locally: `cargo bench -p tpt-algo-benches --bench routing`. Criterion writes reports to
`target/criterion/`.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
