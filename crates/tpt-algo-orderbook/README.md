# tpt-algo-orderbook

Limit order book simulation: price-time priority matching, event replay
(add / cancel / modify / market execute / execute-order), best bid/ask, spread,
mid price and VWAP. Prices are integer ticks; quantities and ids are `u64`.

## Status

Pre-release (0.0.x). Implemented from published descriptions only (see
`docs/clean-room-notes/orderbook.md`).

## Feature flags

* `std` (default): implies `alloc`.
* `alloc`: enables `OrderBook` (needs `BTreeMap`/`VecDeque`). Without it the crate is
  `no_std` and exposes only the data types and the error enum.
* `serde`: `Serialize`/`Deserialize` for `Price`, `Side`, `Event`, `Trade`.

## Usage

```rust
use tpt_algo_orderbook::{Event, OrderBook, Price, Side};

let mut book = OrderBook::new();
book.apply(Event::Add { id: 1, side: Side::Ask, price: Price(101), qty: 5 })?;
book.apply(Event::Execute { id: 2, side: Side::Bid, qty: 2 })?;
assert_eq!(book.vwap_last(10)?, 101.0);
# Ok::<(), tpt_algo_orderbook::OrderBookError>(())
```

Run `cargo run -p tpt-algo-orderbook --example basic`.

## Validation status

Unit tests, proptest invariants (never crossed, quantity conservation, trade
prices within aggressor limits) and seeded synthetic flow pass. LOBSTER
validation is scaffolded in `tests/validation.rs` (ignored; set
`TPT_DATASET_DIR`) and has not been run. LOBSTER terms of use are to be checked
before any workflow is documented.

## Licence

MIT OR Apache-2.0.
