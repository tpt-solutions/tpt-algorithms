# tpt-algo-synth-data

Deterministic synthetic data generators for the `tpt-algorithms` workspace.

Every generator takes an explicit seed and is reproducible: the same seed and
parameters give identical output on all platforms. The generators build test
fixtures, so no third-party data has to be committed to the repository.

> **Internal crate.** `publish = false`. It is never released to crates.io. The
> public algorithm crates use it as a dev-dependency only.

## Generators

| Module | Generators |
|--------|-----------|
| `gaussian` | `standard_normal`. |
| `series` | `random_walk` and `noisy_ramp`. |
| `routing` | `cvrp_uniform` (a CVRP instance with uniform customers), `point_cloud`. |
| `order_flow` | `limit_orders`, which produces limit-order `OrderEvent`s around a mid price, with integer-tick prices. |
| `transactions` | `transactions`, which produces labelled `Transaction` records for fraud tests. |

## Features

- `std` (default): implies `alloc` and `tpt-algo-math-core/std`.
- `alloc`: enables the allocating generators (they return `Vec`s).
- `serde`: serde support for the generated types.

The crate requires `alloc`, because every generator returns a `Vec`.

## Usage

```rust
use tpt_algo_synth_data::series::random_walk;

let walk = random_walk(7, 500, 0.0, 1.0); // seed, length, start, sigma
assert_eq!(walk.len(), 500);
```

## Testing

```sh
cargo test -p tpt-algo-synth-data
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
