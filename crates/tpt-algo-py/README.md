# tpt-algo (Python bindings)

Python bindings for the `tpt-algo` crates: Isolation Forest anomaly scores,
battery state-of-charge Extended Kalman filter, capacitated vehicle routing,
a limit order book, swarm collision avoidance and grid paths, and a seeded
consensus simulator. The package ships a `py.typed` marker and a `.pyi` stub.

> **Not published yet.** `publish = false` in the crate. The PyPI package
> `tpt-algo` is produced by the manual `bindings-publish` workflow, which needs
> maintainer approval. See [docs/bindings.md](../../docs/bindings.md).

## API

| Name | Purpose |
|------|---------|
| `IsolationForest.fit(data, n_trees=100, subsample_size=256, seed=0)` | Train a forest. Also `dimension`, `n_trees`, `score`, `score_samples`, `mean_path_length`. |
| `average_path_length(n)` | Average path length c(n). |
| `Ekf(ocv, capacity_ah, r0, r1, c1, soc0, soc_variance, ...)` | EKF. `predict`, `update`, `soc`, `estimate()`, `covariance()`, `model()`, `ocv_voltage(soc)`, `terminal_voltage(soc, v1, current)`. |
| `OrderBook()` | `apply(events)` returns `{"trades", "stats"}`; `snapshot()`. |
| `solve_routes(problem)` | Savings plus local search. Returns `{"routes", "cost", "vehicles"}`. |
| `avoid(agent, neighbors=None, config=None)` | Returns `{"velocity": [vx, vy]}`. |
| `find_path(grid)` | Returns `{"cells", "cost"}`. |
| `simulate_consensus(config)` | Seeded simulator only; there are no live replicas. |
| `TptAlgoError` | Subclass of `ValueError`. `kind` is `invalid_input`, `invalid_config`, `unsupported`, `infeasible` or `unreachable`. |

The routing, order book, swarm and consensus functions take dicts (or JSON
strings) and return dicts. The input shapes are in the Rust module docs for
`tpt-algo-bindings-core`.

## Build locally

Requires Python 3.10 or later (the wheel targets the stable `abi3-py310` ABI) and
[maturin](https://www.maturin.rs/).

```sh
cd crates/tpt-algo-py
maturin build --release --out dist
python -m pip install --force-reinstall dist/*.whl
```

## Usage

```python
import tpt_algo

rows = [[float(i), 0.0] for i in range(100)] + [[50.0, 50.0]]
forest = tpt_algo.IsolationForest.fit(rows, n_trees=100, subsample_size=256, seed=7)
scores = forest.score_samples(rows)   # values in [0, 1]; higher is more anomalous

ekf = tpt_algo.Ekf(
    [(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)],  # OCV table: (soc, volts) pairs, 2 to 8
    capacity_ah=2.0, r0=0.05, r1=0.02, c1=1000.0,
    soc0=0.8, soc_variance=0.05,           # other noise options have defaults
)
ekf.predict(1.0, 1.0)  # discharge current (A), dt (s)
ekf.update(3.9, 1.0)   # terminal voltage (V), current (A)
print(ekf.soc)

book = tpt_algo.OrderBook()
out = book.apply([
    {"type": "add", "id": 1, "side": "bid", "price": 99, "qty": 10},
    {"type": "add", "id": 2, "side": "ask", "price": 101, "qty": 5},
    {"type": "execute", "id": 3, "side": "ask", "qty": 4},
])
```

Current is positive on discharge, voltage is in volts, time is in seconds, and
capacity is in ampere-hours.

## Testing

```sh
cd crates/tpt-algo-py
python -m unittest discover -s tests -v
```

`tests/test_parity.py` runs the fixtures shared with the Rust core and the
WebAssembly bindings (`internal/tpt-algo-bindings-core/fixtures/parity.json`).

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
