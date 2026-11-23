# tpt-algo-soc

Battery state-of-charge (SoC) estimation with an Extended Kalman Filter over a
first-order RC (Thevenin) equivalent-circuit model. Pure Rust, `no_std`, no
allocator required, no `unsafe`.

## Status

Pre-release (0.0.0). Core model and filter are implemented and unit/property
tested. Dataset validation is not yet performed.

## Feature flags

- `std` (default): implies `alloc`; uses the standard library.
- `alloc`: reserved; the crate does not currently allocate.
- `serde`: `Serialize`/`Deserialize` for `CellParams`, `NoiseConfig`, `Estimate`.

## Usage

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

See `examples/discharge.rs`.

## Validation status

Known-answer tests, proptest invariants (SoC in [0,1], covariance symmetric
PSD) and a seeded synthetic discharge test run under `cargo test`. The NASA
Battery Dataset test in `tests/validation.rs` is a skeleton marked `#[ignore]`
(set `TPT_DATASET_DIR`); no real-data results exist yet.

## Licence

MIT OR Apache-2.0.
