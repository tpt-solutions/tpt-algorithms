// SPDX-License-Identifier: MIT OR Apache-2.0
//! Limit order book wrapper: events in as JSON, trades and statistics out.
//!
//! Event JSON (an array, applied in order):
//!
//! ```json
//! [
//!   {"type": "add", "id": 1, "side": "bid", "price": 99, "qty": 10},
//!   {"type": "add", "id": 2, "side": "ask", "price": 101, "qty": 5},
//!   {"type": "modify", "id": 1, "qty": 8, "price": null},
//!   {"type": "cancel", "id": 2},
//!   {"type": "execute", "id": 3, "side": "ask", "qty": 4},
//!   {"type": "execute_order", "id": 1, "qty": 2}
//! ]
//! ```
//!
//! A batch is atomic: if any event is rejected, the book is left unchanged.

use crate::error::{invalid_input, BindingError, ErrorKind, Result};
use crate::json::{counter, int, opt_int, opt_uint, parse, string, uint};
use serde_json::{json, Value};
use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side, Trade};

/// A limit order book. Cloning is cheap enough for the atomic batch check.
#[derive(Debug, Clone, Default)]
pub struct Book {
    inner: OrderBook,
}

impl Book {
    /// An empty book.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies a JSON array of events. Returns `{"trades": [...], "stats": {...}}`;
    /// `stats.traded` counts traded quantity, not trades.
    /// where `trades` lists only the trades produced by this batch.
    ///
    /// # Errors
    /// Malformed JSON, an unknown event type, or any event the book rejects.
    /// On error the book is unchanged.
    pub fn apply_json(&mut self, events: &str) -> Result<String> {
        let doc = parse(events)?;
        let list = doc
            .as_array()
            .ok_or_else(|| invalid_input("events must be a JSON array"))?;

        let mut trial = self.inner.clone();
        let start = trial.trades().len();
        for (index, raw) in list.iter().enumerate() {
            let event = parse_event(raw)
                .map_err(|e| BindingError::new(e.kind, format!("event {index}: {}", e.message)))?;
            trial.apply(event).map_err(|e| {
                BindingError::new(book_error_kind(&e), format!("event {index}: {e}"))
            })?;
        }
        let trades: Vec<Value> = trial.trades()[start..].iter().map(trade_json).collect();
        let stats = trial.stats();
        self.inner = trial;

        Ok(json!({
            "trades": trades,
            "stats": {
                "accepted": counter(stats.accepted),
                "traded": counter(stats.traded),
                "cancelled": counter(stats.cancelled),
                "expired": counter(stats.expired),
            },
        })
        .to_string())
    }

    /// Current top of book, depth and sequence number, as JSON.
    #[must_use]
    pub fn snapshot_json(&self) -> String {
        json!({
            "best_bid": self.inner.best_bid().map(|p| p.0),
            "best_ask": self.inner.best_ask().map(|p| p.0),
            "mid": self.inner.mid(),
            "spread": self.inner.spread(),
            "order_count": self.inner.order_count(),
            "resting_quantity": counter(self.inner.resting_quantity()),
            "sequence": self.inner.sequence(),
        })
        .to_string()
    }
}

fn side(v: &Value) -> Result<Side> {
    match string(v, "side")? {
        "bid" => Ok(Side::Bid),
        "ask" => Ok(Side::Ask),
        other => Err(invalid_input(format!(
            "side must be 'bid' or 'ask', got '{other}'"
        ))),
    }
}

fn parse_event(v: &Value) -> Result<Event> {
    let kind = string(v, "type")?;
    match kind {
        "add" => Ok(Event::Add {
            id: uint(v, "id")?,
            side: side(v)?,
            price: Price(int(v, "price")?),
            qty: uint(v, "qty")?,
        }),
        "cancel" => Ok(Event::Cancel { id: uint(v, "id")? }),
        "modify" => {
            let new_qty = opt_uint(v, "qty")?;
            let new_price = opt_int(v, "price")?.map(Price);
            Ok(Event::Modify {
                id: uint(v, "id")?,
                new_qty,
                new_price,
            })
        }
        "execute" => Ok(Event::Execute {
            id: uint(v, "id")?,
            side: side(v)?,
            qty: uint(v, "qty")?,
        }),
        "execute_order" => Ok(Event::ExecuteOrder {
            id: uint(v, "id")?,
            qty: uint(v, "qty")?,
        }),
        other => Err(invalid_input(format!("unknown event type '{other}'"))),
    }
}

fn book_error_kind(e: &OrderBookError) -> ErrorKind {
    match e {
        OrderBookError::NoTrades => ErrorKind::Infeasible,
        _ => ErrorKind::InvalidInput,
    }
}

fn side_name(s: Side) -> &'static str {
    match s {
        Side::Bid => "bid",
        Side::Ask => "ask",
    }
}

fn trade_json(t: &Trade) -> Value {
    json!({
        "seq": t.seq,
        "price": t.price.0,
        "qty": t.qty,
        "aggressor_id": t.aggressor_id,
        "resting_id": t.resting_id,
        "aggressor_side": side_name(t.aggressor_side),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn batch_produces_trade_and_stats() {
        let mut book = Book::new();
        let out = book
            .apply_json(
                r#"[{"type":"add","id":1,"side":"bid","price":99,"qty":10},
                    {"type":"add","id":2,"side":"ask","price":101,"qty":5},
                    {"type":"add","id":3,"side":"bid","price":101,"qty":5}]"#,
            )
            .unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["trades"].as_array().unwrap().len(), 1);
        assert_eq!(v["stats"]["traded"], 5);
    }

    #[test]
    fn failed_batch_leaves_book_unchanged() {
        let mut book = Book::new();
        book.apply_json(r#"[{"type":"add","id":1,"side":"bid","price":99,"qty":10}]"#)
            .unwrap();
        let before = book.snapshot_json();
        let err = book
            .apply_json(r#"[{"type":"cancel","id":1},{"type":"cancel","id":42}]"#)
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(err.message.starts_with("event 1"));
        assert_eq!(book.snapshot_json(), before);
    }

    #[test]
    fn unknown_type_is_rejected() {
        let mut book = Book::new();
        let err = book.apply_json(r#"[{"type":"nope"}]"#).unwrap_err();
        assert!(err.message.contains("unknown event type"));
    }
}
