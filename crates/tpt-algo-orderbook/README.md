# tpt-algo-orderbook

Limit order book simulation with price-time priority matching. The crate
processes a stream of order events, matches orders, and reports the book state
and trade statistics. It covers best bid and ask, spread, mid price, and
volume-weighted average price (VWAP).

Prices are integer ticks, and quantities and order ids are `u64`. The crate
uses no floating point for matching, so replays are exact.

Use it to simulate an exchange, replay a message feed, or test a trading
strategy against a deterministic matching engine. It is a simulator, not an
exchange: it has no networking, no persistence, and no self-trade prevention.

## Status

Pre-release (`0.0.0`). Implemented from published descriptions only; see
`docs/clean-room-notes/orderbook.md`. The crate is not yet published to
crates.io; depend on it by path or git until the first release. See the
[changelog](CHANGELOG.md).

## Features

| Feature | Default | Enables |
|---------|---------|---------|
| `std`   | yes     | Implies `alloc`. Uses the standard library. |
| `alloc` | via `std` | `OrderBook`, `Stats` and `ReplayError` (needs `BTreeMap` and `VecDeque`). |
| `serde` | no      | `Serialize` / `Deserialize` for `Price`, `Side`, `Event` and `Trade`. |

Without `alloc` the crate is `no_std`. It then exposes only the data types
(`Price`, `Side`, `Event`, `Trade`) and the error enum.

## Installation

```toml
[dependencies]
tpt-algo-orderbook = { path = "../tpt-algo-orderbook" }
# tpt-algo-orderbook = { path = "...", default-features = false }   # data types only
```

The minimum supported Rust version is `1.98`, matching the workspace.

## Quick start

```rust
use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};

fn run() -> Result<(), OrderBookError> {
    let mut book = OrderBook::new();
    book.apply(Event::Add { id: 1, side: Side::Ask, price: Price(101), qty: 5 })?;
    book.apply(Event::Execute { id: 2, side: Side::Bid, qty: 2 })?;
    assert_eq!(book.vwap_last(10)?, 101.0);
    Ok(())
}
```

## Matching rules

- A limit order that crosses the opposite best price matches first, at the
  resting order's price. The oldest resting order at a level is matched first
  (price-time priority). Any remainder rests at the limit price.
- A market order (`Event::Execute`) consumes liquidity until it fills or the
  opposite side is empty. Any unfilled remainder is discarded (expired).
- `Event::Modify`: reducing size at the same price keeps time priority. A price
  change or a size increase re-queues the order at the back, and it may match if
  the new price crosses the book.
- `Event::ExecuteOrder` fills a specific resting order, as exchange message feeds
  do. The aggressor is not part of the simulation.
- There is no self-trade prevention.

## Event types

| Event | Effect |
|-------|--------|
| `Add { id, side, price, qty }` | Submit a limit order. |
| `Cancel { id }` | Remove a resting order entirely. |
| `Modify { id, new_qty, new_price }` | Change the size and/or price of a resting order. |
| `Execute { id, side, qty }` | Submit a market order. |
| `ExecuteOrder { ... }` | Fill a specific resting order. |

`OrderBook::apply` returns the number of trades an event produced.
`OrderBook::replay` applies a whole sequence of events.

## API overview

| Item | Purpose |
|------|---------|
| `OrderBook::new`, `apply`, `replay` | Create a book and feed it events. |
| `best_bid`, `best_ask`, `spread`, `mid` | Top-of-book queries. |
| `level_qty`, `order_count`, `resting_quantity`, `order` | Depth and order-level queries. |
| `vwap_last(n)` | VWAP of the last `n` trades. |
| `vwap_window(from_seq, to_seq)` | VWAP over an inclusive event-sequence range. |
| `trades`, `clear_trades`, `sequence`, `stats` | Trade log, sequence number and counters. |
| `Stats` | `accepted`, `traded`, `cancelled` and `expired` counters. |
| `Price`, `Side`, `Event`, `Trade` | Plain data types, usable without `alloc`. |
| `OrderBookError`, `ReplayError` | Error types. |

### Quantity conservation

`Stats` tracks `accepted`, `traded`, `cancelled` and `expired`. Every trade
consumes its quantity from both an aggressor and a resting order, so at all
times:

```text
accepted == resting + 2 * traded + cancelled + expired
```

### VWAP

`VWAP = sum(price_i * qty_i) / sum(qty_i)` over the selected trades, in ticks
(`f64`).

## Examples

```sh
cargo run -p tpt-algo-orderbook --example basic
```

## Validation status

- Unit tests, proptest invariants (the book is never crossed, quantity is
  conserved, and trade prices stay within the aggressor's limit), and seeded
  synthetic order flow all pass under `cargo test`.
- LOBSTER validation is scaffolded in `tests/validation.rs`. It is `#[ignore]`d,
  needs `TPT_DATASET_DIR`, and has not been run. LOBSTER's terms of use must be
  checked before any workflow is documented.

## Limitations

- Single instrument per `OrderBook`. Use one book per symbol.
- No self-trade prevention, order types beyond those listed above (such as stop
  orders, iceberg orders or time-in-force), or auction phases.
- No networking, persistence, or recovery. Memory grows with resting orders.
- Prices are integer ticks. Converting to currency units is the caller's job.

## References

- Price-time priority is the standard matching rule of continuous double
  auctions. The implementation follows published descriptions; see
  `docs/clean-room-notes/orderbook.md` in the repository.
- LOBSTER (Limit Order Book System, the Efficient Reconstructor), for the
  planned validation data.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
