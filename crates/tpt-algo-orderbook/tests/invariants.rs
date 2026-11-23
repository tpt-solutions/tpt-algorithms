// SPDX-License-Identifier: MIT OR Apache-2.0
//! Property and synthetic-flow tests.
#![cfg(feature = "alloc")]

use proptest::prelude::*;
use tpt_algo_orderbook::{Event, OrderBook, Price, Side};
use tpt_algo_synth_data::order_flow::{limit_orders, Side as SynthSide};

fn check(book: &OrderBook) {
    if let (Some(b), Some(a)) = (book.best_bid(), book.best_ask()) {
        assert!(b < a, "crossed book: {b:?} >= {a:?}");
    }
    let s = book.stats();
    assert_eq!(
        s.accepted,
        book.resting_quantity() + 2 * s.traded + s.cancelled + s.expired
    );
}

fn side(b: bool) -> Side {
    if b {
        Side::Bid
    } else {
        Side::Ask
    }
}

#[derive(Debug, Clone)]
enum Op {
    Add(bool, i64, u64),
    Cancel(u8),
    ModQty(u8, u64),
    ModPrice(u8, i64, Option<u64>),
    Market(bool, u64),
    ExecOrder(u8, u64),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        4 => (any::<bool>(), 95i64..105, 1u64..20).prop_map(|(s, p, q)| Op::Add(s, p, q)),
        1 => any::<u8>().prop_map(Op::Cancel),
        1 => (any::<u8>(), 1u64..25).prop_map(|(i, q)| Op::ModQty(i, q)),
        1 => (any::<u8>(), 95i64..105, proptest::option::of(1u64..25)).prop_map(|(i, p, q)| Op::ModPrice(i, p, q)),
        1 => (any::<bool>(), 1u64..30).prop_map(|(s, q)| Op::Market(s, q)),
        1 => (any::<u8>(), 1u64..5).prop_map(|(i, q)| Op::ExecOrder(i, q)),
    ]
}

proptest! {
    #[test]
    fn invariants_hold(ops in proptest::collection::vec(op(), 0..300)) {
        let mut book = OrderBook::new();
        let mut next_id = 1000u64;
        for o in ops {
            // Ids 1000.. are issued sequentially; u8 picks map into that range.
            let pick = |i: u8, next: u64| 1000 + u64::from(i) % (next - 1000).max(1);
            let before = book.trades().len();
            let mut aggressor: Option<(Side, Option<Price>)> = None;
            let r = match o {
                Op::Add(s, p, q) => {
                    next_id += 1;
                    aggressor = Some((side(s), Some(Price(p))));
                    book.apply(Event::Add { id: next_id - 1, side: side(s), price: Price(p), qty: q })
                }
                Op::Cancel(i) => book.apply(Event::Cancel { id: pick(i, next_id) }),
                Op::ModQty(i, q) => {
                    book.apply(Event::Modify { id: pick(i, next_id), new_qty: Some(q), new_price: None })
                }
                Op::ModPrice(i, p, q) => {
                    if let Some((s, _, _)) = book.order(pick(i, next_id)) {
                        aggressor = Some((s, Some(Price(p))));
                    }
                    book.apply(Event::Modify { id: pick(i, next_id), new_qty: q, new_price: Some(Price(p)) })
                }
                Op::Market(s, q) => {
                    next_id += 1;
                    aggressor = Some((side(s), None));
                    book.apply(Event::Execute { id: next_id - 1, side: side(s), qty: q })
                }
                Op::ExecOrder(i, q) => book.apply(Event::ExecuteOrder { id: pick(i, next_id), qty: q }),
            };
            if r.is_ok() {
                if let Some((s, Some(Price(l)))) = aggressor {
                    for t in &book.trades()[before..] {
                        match s {
                            Side::Bid => prop_assert!(t.price.0 <= l),
                            Side::Ask => prop_assert!(t.price.0 >= l),
                        }
                    }
                }
            }
            check(&book);
        }
    }
}

#[test]
fn synthetic_flow_is_consistent_and_deterministic() {
    let run = |seed| {
        let mut book = OrderBook::new();
        for (i, e) in limit_orders(seed, 2_000, 10_000, 8, 50)
            .into_iter()
            .enumerate()
        {
            let s = match e.side {
                SynthSide::Bid => Side::Bid,
                SynthSide::Ask => Side::Ask,
            };
            book.apply(Event::Add {
                id: i as u64 + 1,
                side: s,
                price: Price(e.price_ticks),
                qty: e.qty,
            })
            .unwrap();
            check(&book);
        }
        // Sweep the book with market orders to generate trades.
        for k in 0..20u64 {
            book.apply(Event::Execute {
                id: 1_000_000 + k,
                side: side(k % 2 == 0),
                qty: 100,
            })
            .unwrap();
            check(&book);
        }
        book
    };
    let a = run(42);
    let b = run(42);
    assert_eq!(a.trades(), b.trades());
    assert!(a.vwap_last(100).is_ok());
}
