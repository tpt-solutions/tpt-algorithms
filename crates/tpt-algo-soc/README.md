# tpt-algo-soc

Battery state-of-charge (SoC) estimation with an Extended Kalman Filter (EKF).
The cell is modelled as a first-order Thevenin equivalent circuit: an
open-circuit voltage source, a series resistance `R0`, and one RC pair. The
filter state is `[SoC, V1]`, where `V1` is the RC-pair voltage.

The crate is pure Rust and `no_std`. It needs no allocator and contains no
`unsafe` code. The open-circuit-voltage table is a const-generic array.

Use it in a battery management system, a simulator, or a teaching example. It
estimates SoC from current and voltage measurements. It does not measure SoC
directly, and it needs a cell model that matches the cell.

## Status

Pre-release (`0.0.0`). The model and filter are implemented and unit and property
tested. Validation against a real discharge dataset has not been run. The crate
is not yet published to crates.io; depend on it by path or git until the first
release. See the [changelog](CHANGELOG.md).

## Features

| Feature | Default | Enables |
|---------|---------|---------|
| `std`   | yes     | Implies `alloc`. Uses the standard library. |
| `alloc` | via `std` | Reserved. The crate does not currently allocate. |
| `serde` | no      | `Serialize` / `Deserialize` for `CellParams`, `NoiseConfig` and `Estimate`. |

The crate is `no_std` without `std`.

## Installation

```toml
[dependencies]
tpt-algo-soc = { path = "../tpt-algo-soc" }
# tpt-algo-soc = { path = "...", default-features = false }   # no_std
```

The minimum supported Rust version is `1.98`, matching the workspace.

## Quick start

```rust
use tpt_algo_soc::{CellParams, Filter, Model, NoiseConfig, OcvCurve, SocError};

fn run() -> Result<(), SocError> {
    let ocv = OcvCurve::new([(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)])?;
    let params = CellParams { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
    let noise = NoiseConfig { process_soc: 1e-9, process_v1: 1e-6, measurement: 1e-4 };
    let mut f = Filter::new(Model::new(params, ocv)?, noise)?.initialize(0.8, 0.05, 1e-4)?;
    f.predict(1.0, 1.0)?;          // discharge current (A), dt (s)
    f.update(3.9, 1.0)?;           // measured terminal voltage (V), current (A)
    let _soc = f.estimate().soc;
    Ok(())
}
```

Sign convention: current is positive on discharge (amps), terminal voltage is in
volts, time is in seconds, and capacity is in ampere-hours.

## Lifecycle

The filter's lifecycle is encoded in the type system:

1. `Filter::new(model, noise)` returns a `Filter<Uninitialized, N>`.
2. `initialize(soc, soc_variance, v1_variance)` validates the initial state and
   variances, and returns a `Filter<Initialized, N>`.
3. Only an initialized filter has `predict` and `update`, so an uninitialized
   estimate cannot be used by mistake.

## API overview

| Item | Purpose |
|------|---------|
| `OcvCurve::new` | Open-circuit voltage against SoC table, validated on construction. |
| `CellParams` | Capacity, `R0`, `R1` and `C1`. |
| `Model::new` | Combines cell parameters and an OCV curve. |
| `NoiseConfig` | Process and measurement noise. |
| `Filter<Uninitialized, N>`, `Filter<Initialized, N>` | Type-state EKF. |
| `Filter::initialize`, `predict`, `update`, `estimate` | The EKF steps. |
| `Estimate` | The current SoC and related state. |
| `SocError` | Returned for invalid parameters, tables and filter inputs. |

## Examples

```sh
cargo run -p tpt-algo-soc --example discharge
```

## Validation status

- Known-answer tests, proptest invariants (SoC stays in `[0, 1]`, and the
  covariance stays symmetric positive semi-definite), and a seeded synthetic
  discharge test all run under `cargo test`.
- The NASA Battery Dataset test in `tests/validation.rs` is a skeleton marked
  `#[ignore]`. Set `TPT_DATASET_DIR` to run it. No real-data results exist yet.

## Limitations

- One RC pair only. Cells that need a second-order model will show larger errors.
- Noise and initial covariance values are caller-supplied and not tuned. Poor
  values give poor estimates.
- The OCV curve must match the cell chemistry and temperature. Hysteresis is not
  modelled.
- Capacity is treated as constant. Ageing and temperature effects are not modelled.

## References

- The Thevenin (RC equivalent-circuit) cell model and the EKF formulation follow
  standard battery-management literature. A model-based reference is G. L. Plett,
  *Battery Management Systems, Volume II: Equivalent-Circuit Methods*, Artech
  House, 2015.
- Dataset: NASA Prognostics Center of Excellence, Battery Data Set. Check the
  source page's terms before publishing a workflow that uses it.

## Benchmarks

Criterion benchmarks for this crate live in [`benches/benches/soc.rs`](../../benches/benches/soc.rs)
(the `tpt-algo-benches` package). Inputs come from `tpt-algo-synth-data` where the crate takes
generated data.

Numbers are produced by the scheduled bench workflow
(`.github/workflows/bench.yml`, weekly plus on demand) on a GitHub-hosted `ubuntu-latest`
runner, and uploaded as the `criterion-<sha>` artifact. They are not measured on a developer
machine and are not portable between runners.

| Benchmark | Measures | Median time |
|-----------|----------|-------------|
| `soc_ekf/500` | 500 one-second steps of the EKF (predict + update) | pending first scheduled run |
| `soc_ekf/2000` | 2,000 one-second steps of the EKF (predict + update) | pending first scheduled run |

To run locally: `cargo bench -p tpt-algo-benches --bench soc`. Criterion writes reports to
`target/criterion/`.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
