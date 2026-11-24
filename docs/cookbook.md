# Cookbook and choose-your-crate

Written for: someone who has a problem and wants to know which crate solves it, and how to run
the first example.

Every command below runs from the repository root. The crates are not published yet, so use a
git or path dependency in your own project (see the [README quickstart](https://github.com/tpt-solutions/tpt-algorithms#quickstart)).

## Choose your crate

| If you need to... | Crate | Start with | `tpt demo` | Maturity (from the crate README) |
|---|---|---|---|---|
| Estimate battery state of charge from current and voltage logs | `tpt-algo-soc` | `cargo run -p tpt-algo-soc --example discharge` | `soc` | Implemented; NASA dataset validation pending |
| Score transactions or rows for anomalies, unsupervised | `tpt-algo-fraud` | `cargo run -p tpt-algo-fraud --example basic` | `fraud` | Implemented; dataset validation pending |
| Apply a stream of order events and inspect trades, spread and VWAP | `tpt-algo-orderbook` | `cargo run -p tpt-algo-orderbook --example basic` | `orderbook` | Implemented; LOBSTER validation pending |
| Solve a capacitated vehicle routing problem | `tpt-algo-routing` | `cargo run -p tpt-algo-routing --example basic` | `route` | Implemented; benchmark validation pending |
| Keep agents apart while they move to goals, or find a grid path | `tpt-algo-swarm` | `cargo run -p tpt-algo-swarm --example crossing` | `swarm` | Implemented; scenario validation |
| Study a BFT (HotStuff-family) protocol under faults, with a seeded simulator | `tpt-algo-consensus` | `cargo run -p tpt-algo-consensus --example simulate` | `consensus` | Implemented; signatures not verified |

Not in this table: `tpt-algo-math-core`, `tpt-algo-geometry`, `tpt-algo-synth-data` and
`tpt-algo-bindings-core`. They are internal and unpublished. `synth-data` is where the seeded
inputs for tests and benchmarks come from.

If you work in Python or JavaScript, the bindings cover fraud (Isolation Forest) and soc (EKF) so
far. See [bindings.md](bindings.md).

## Recipes

### Battery state of charge from a discharge log

Use `tpt-algo-soc`. Build a `Model` from cell parameters and an open-circuit-voltage table, then
run the extended Kalman filter: `predict` with the current and time step, `update` with the
measured terminal voltage and the current.

```rust
use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve};

let ocv = OcvCurve::new([(0.0, 3.0), (0.2, 3.5), (0.5, 3.7), (0.8, 3.95), (1.0, 4.2)])?;
let model = Model::new(CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 }, ocv)?;
let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
let mut filter = Filter::new(model, noise)?.initialize(0.5, 0.1, 1e-4)?;
filter.predict(1.0, 1.0)?;          // current (A, positive on discharge), dt (s)
filter.update(3.9, 1.0)?;           // terminal voltage (V), current (A)
println!("soc = {}", filter.estimate().soc);
```

The example runs a simulated discharge from a wrong initial guess and prints the estimate
converging on the true state of charge.

### Unusual transactions, without labels

Use `tpt-algo-fraud`. Fit an isolation forest on feature rows, then score them. Scores are in
`[0, 1]`; higher means more anomalous. Seed the configuration so results are reproducible.

```rust
use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};

let config = IsolationForestConfig::default().with_trees(100).with_seed(2024);
let forest = IsolationForest::fit(&config, &rows)?;   // rows: Vec<Vec<f64>>, one row per record
let scores = forest.score_samples(&rows)?;
```

From the bindings: `tpt_algo.IsolationForest.fit(rows, n_trees=100, subsample_size=256, seed=7)`
in Python.

### Replaying a limit order book

Use `tpt-algo-orderbook`. Feed `Event`s to `OrderBook::apply`: `Add`, `Cancel`, `Modify`,
`Execute` (a market order) and `ExecuteOrder`. Read the results from the book: `best_bid`,
`best_ask`, `spread`, `mid`, `trades()`, `stats()` and `vwap_last(n)`. Prices are integer ticks
(`Price(i64)`), so there is no floating-point drift in matching.

```rust
use tpt_algo_orderbook::{Event, OrderBook, Price, Side};

let mut book = OrderBook::new();
book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(9_990), qty: 100 })?;
book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(10_010), qty: 80 })?;
book.apply(Event::Add { id: 3, side: Side::Bid, price: Price(10_010), qty: 100 })?;
println!("trades = {:?}", book.trades());
```

### Vehicle routing with capacities

Use `tpt-algo-routing`. Describe customers with a location and demand, build a `Problem` with a
depot and vehicle capacity, then call `solve`. `clarke_wright_savings` gives the constructive
solution on its own; `local_search` improves a given one.

```rust
use tpt_algo_routing::{solve, Config, Customer, Problem};

let customers = vec![
    Customer::new(1.0, 0.0, 5.0),
    Customer::new(2.0, 0.0, 5.0),
    Customer::new(-1.0, 0.0, 5.0),
    Customer::new(-2.0, 0.0, 5.0),
];
let problem = Problem::new((0.0, 0.0), 10.0, customers)?;
let solution = solve(&problem, &Config::default())?;
```

Time windows and a maximum number of vehicles are optional builders on `Customer` and `Problem`.

### Agents avoiding each other, and grid paths

Use `tpt-algo-swarm`. For moving agents, build a `Simulation` from `Agent`s (position, velocity,
radius, preferred velocity, maximum speed) and goals, then call `step(dt, &AvoidanceConfig)`. For a
single agent on a grid, `Grid` plus `find_path` with `Connectivity::Four` or `Connectivity::Eight`.

### A seeded consensus run

Use `tpt-algo-consensus`, through its deterministic simulator. `SimConfig::new(n, seed)` sets the
replica count and the seed; add faults, a global stabilisation time and message loss as needed.
`Simulator::run(ticks)` advances the simulation and `report()` returns the committed logs. There is
no live networking. Signatures are not verified in the current release; see the crate README.

```rust
use tpt_algo_consensus::sim::{SimConfig, Simulator};

let mut sim = Simulator::new(SimConfig::new(4, 2024))?;
sim.run(600);
let report = sim.report();
```

## Running the demos without writing code

The `tpt` binary runs each crate on bundled synthetic data:

```sh
cargo run -p tpt-algo-cli -- demo soc
cargo run -p tpt-algo-cli -- demo route --format csv
```

Demo names are `soc`, `fraud`, `orderbook`, `route`, `swarm` and `consensus`.

## Benchmarks

Each crate README has a Benchmarks section that lists its Criterion benchmarks. Their numbers
come from the scheduled bench workflow, not from a developer machine. See
`benches/README.md` in the repository.
