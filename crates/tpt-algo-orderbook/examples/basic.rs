// SPDX-License-Identifier: MIT OR Apache-2.0
//! Minimal order book walk-through using only the public API.

#[cfg(feature = "alloc")]
use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};

#[cfg(feature = "alloc")]
fn main() -> Result<(), OrderBookError> {
    let mut book = OrderBook::new();
    book.apply(Event::Add {
        id: 1,
        side: Side::Bid,
        price: Price(9_990),
        qty: 100,
    })?;
    book.apply(Event::Add {
        id: 2,
        side: Side::Ask,
        price: Price(10_010),
        qty: 80,
    })?;
    book.apply(Event::Add {
        id: 3,
        side: Side::Ask,
        price: Price(10_020),
        qty: 50,
    })?;
    println!("spread = {:?}, mid = {:?}", book.spread(), book.mid());

    book.apply(Event::Add {
        id: 4,
        side: Side::Bid,
        price: Price(10_020),
        qty: 100,
    })?;
    println!("trades = {:?}", book.trades());
    println!("vwap = {:?}", book.vwap_last(10));
    println!("stats = {:?}", book.stats());
    Ok(())
}

#[cfg(not(feature = "alloc"))]
fn main() {}
