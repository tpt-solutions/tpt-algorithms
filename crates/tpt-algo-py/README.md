# tpt-algo (Python bindings)

Python bindings for `tpt-algo-fraud` (Isolation Forest anomaly scoring) and
`tpt-algo-soc` (battery state-of-charge Extended Kalman Filter). The module is
named `tpt_algo`, and errors raise `ValueError`.

> **Not published yet.** `publish = false` in the crate. The PyPI package
> `tpt-algo` is produced by the manual `bindings-publish` workflow, which needs
> maintainer approval. See [docs/bindings.md](../../docs/bindings.md).

## Classes

| Class | Purpose |
|-------|---------|
| `tpt_algo.IsolationForest` | `fit` (static), `dimension`, `score` and `score_samples`. |
| `tpt_algo.Ekf` | `predict`, `update`, `soc` and `estimate`. |

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
print(scores[-1] > scores[0])         # the outlier scores higher

ekf = tpt_algo.Ekf(
    [(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)],  # OCV table: (soc, volts) pairs, 2 to 8
    capacity_ah=2.0, r0=0.05, r1=0.02, c1=1000.0,
    soc0=0.8, soc_variance=0.05, v1_variance=1e-4,
    process_soc=1e-9, process_v1=1e-6, measurement=1e-4,
)
ekf.predict(1.0, 1.0)  # discharge current (A), dt (s)
ekf.update(3.9, 1.0)   # terminal voltage (V), current (A)
print(ekf.soc())
```

Current is positive on discharge, voltage is in volts, time is in seconds, and
capacity is in ampere-hours.

## Testing

```sh
cd crates/tpt-algo-py
python -m unittest discover -s tests -v
```

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
