# tpt-algorithms

Permissively licensed Rust implementations of scientific, financial and systems algorithms. The
goal is MIT or Apache-2.0 alternatives to paywalled or copyleft-locked industry standards.

## Principles

- **Permissive only.** Every crate is `MIT OR Apache-2.0`. Dependencies must be MIT, BSD, ISC,
  Zlib or dual-licensed with MIT. Apache-2.0-only, GPL, LGPL, AGPL and similar are rejected by
  `cargo deny` in CI.
- **Clean-room.** Algorithms are implemented from published papers and specifications, not from
  copyleft or proprietary source. See [docs/clean-room.md](docs/clean-room.md).
- **`no_std` by default** where the domain allows it. Features: `std`, `alloc`, `serde`.
- **Validation first.** Each algorithm has a reproducible validation path against public data or a
  seeded synthetic benchmark.

## Crates

| Crate | Domain | Status |
|---|---|---|
| `tpt-algo-soc` | Battery state of charge (Extended Kalman Filter) | Implemented; dataset validation pending |
| `tpt-algo-fraud` | Anomaly scoring (Isolation Forest) | Implemented; dataset validation pending |
| `tpt-algo-orderbook` | Limit order book simulation | Implemented; LOBSTER validation pending |
| `tpt-algo-routing` | Vehicle routing (CVRP, savings + local search) | Implemented; benchmark validation pending |
| `tpt-algo-swarm` | Velocity obstacles and path planning | Implemented; scenario validation |
| `tpt-algo-consensus` | BFT consensus state machine (HotStuff family) | Implemented; signatures not verified (see its README) |

Internal, unpublished: `tpt-algo-math-core`, `tpt-algo-geometry`, `tpt-algo-synth-data`.

## Building and testing

```sh
cargo test --workspace
cargo test --workspace --no-default-features
cargo build -p tpt-algo-math-core --no-default-features --target thumbv7em-none-eabihf
```

Dataset validation tests are `#[ignore]` and need `TPT_DATASET_DIR`. See
[docs/testing.md](docs/testing.md) and [docs/datasets.md](docs/datasets.md).

## Documentation

- [docs/decisions.md](docs/decisions.md): project decisions and open items
- [docs/licensing.md](docs/licensing.md)
- [docs/release.md](docs/release.md)
- [docs/maintenance.md](docs/maintenance.md)
- [docs/formal-verification.md](docs/formal-verification.md)

## Licence

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Copyright
TPT Solutions. Contributions are licensed the same way.
