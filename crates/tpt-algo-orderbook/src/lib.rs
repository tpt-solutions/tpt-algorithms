// SPDX-License-Identifier: MIT OR Apache-2.0
//! Limit order book simulation with price-time priority matching.
//!
//! The crate provides:
//!
//! * plain data types ([`Price`], [`Side`], [`Event`], [`Trade`]) that need no
//!   allocator, and
//! * with the `alloc` feature, an `OrderBook` matching engine, event replay,
//!   best bid/ask, spread, mid price and VWAP.
//!
//! Prices are integer ticks, quantities and order ids are `u64`.
//!
//! # Matching rules
//!
//! * Limit orders that cross the opposite best price are matched first, at the
//!   resting order's price, oldest resting order first within a level
//!   (price-time priority). Any remainder rests at the limit price.
//! * Market orders ([`Event::Execute`]) consume liquidity until filled or the
//!   opposite side is empty; an unfilled remainder is discarded ("expired").
//! * [`Event::Modify`]: reducing size at the same price keeps time priority;
//!   a price change or a size increase re-queues the order at the back (and may
//!   match if the new price crosses).
//! * [`Event::ExecuteOrder`] fills a specific resting order, as in exchange
//!   message feeds where the aggressor is not part of the simulation.
//! * There is no self-trade prevention.
//!
//! # Quantity conservation
//!
//! `OrderBook::stats` tracks `accepted`, `traded`, `cancelled` and `expired`.
//! Every trade consumes its quantity from both an aggressor and a resting
//! order, so at all times
//! `accepted == resting + 2 * traded + cancelled + expired`.
//!
//! # VWAP definition
//!
//! `VWAP = sum(price_i * qty_i) / sum(qty_i)` over the selected trades, in
//! ticks (`f64`). `OrderBook::vwap_last` uses the last `n` trades;
//! `OrderBook::vwap_window` uses trades whose event sequence number lies in
//! an inclusive range.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use tpt_algo_orderbook::{Event, OrderBook, Price, Side};
//!
//! let mut book = OrderBook::new();
//! book.apply(Event::Add { id: 1, side: Side::Ask, price: Price(101), qty: 5 })?;
//! book.apply(Event::Add { id: 2, side: Side::Bid, price: Price(99), qty: 5 })?;
//! assert_eq!(book.spread(), Some(2));
//! let n = book.apply(Event::Execute { id: 3, side: Side::Bid, qty: 2 })?;
//! assert_eq!(n, 1);
//! assert_eq!(book.vwap_last(10)?, 101.0);
//! # }
//! # Ok::<(), tpt_algo_orderbook::OrderBookError>(())
//! ```
//!
//! # References
//!
//! * R. Cont, S. Stoikov, R. Talreja, "A stochastic model for order book
//!   dynamics", Operations Research 58(3), 2010.
//! * M. Gould et al., "Limit order books", Quantitative Finance 13(11), 2013.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod book;
mod error;
mod types;

#[cfg(feature = "alloc")]
pub use book::{OrderBook, ReplayError, Stats};
pub use error::OrderBookError;
pub use types::{Event, Price, Side, Trade};
