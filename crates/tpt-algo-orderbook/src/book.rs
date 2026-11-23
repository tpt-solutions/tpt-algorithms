// SPDX-License-Identifier: MIT OR Apache-2.0
//! The matching engine.

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;

use crate::{Event, OrderBookError, Price, Side, Trade};

#[derive(Debug, Clone, Copy)]
struct Resting {
    id: u64,
    qty: u64,
}

/// Cumulative quantity accounting; see the crate docs for the identity.
///
/// # Example
///
/// ```
/// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
/// # fn main() -> Result<(), OrderBookError> {
/// let mut book = OrderBook::new();
/// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
/// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
/// let s = book.stats();
/// // Every accepted unit is resting, traded (on both sides), cancelled or expired.
/// assert_eq!(s.accepted, book.resting_quantity() + 2 * s.traded + s.cancelled + s.expired);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// Total quantity accepted (limit, market, executed-order and size increases).
    pub accepted: u128,
    /// Total traded quantity (each trade counted once).
    pub traded: u128,
    /// Total quantity cancelled (including size reductions).
    pub cancelled: u128,
    /// Market-order quantity that found no liquidity.
    pub expired: u128,
}

/// Error from [`OrderBook::replay`]: which event failed and why.
///
/// # Example
///
/// ```
/// use tpt_algo_orderbook::{Event, OrderBook, Price, Side};
///
/// let mut book = OrderBook::new();
/// let events = [
///     Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 },
///     Event::Cancel { id: 42 }, // never added
/// ];
/// let err = book.replay(events).unwrap_err();
/// assert_eq!(err.index, 1);
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("event {index} failed: {source}")]
pub struct ReplayError {
    /// Zero-based index of the failing event.
    pub index: usize,
    /// Underlying error.
    pub source: OrderBookError,
}

/// A limit order book with price-time priority matching.
///
/// # Example
///
/// ```
/// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
/// # fn main() -> Result<(), OrderBookError> {
/// let mut book = OrderBook::new();
/// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
/// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
/// assert_eq!(book.best_bid(), Some(Price(99)));
/// assert_eq!(book.best_ask(), Some(Price(101)));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct OrderBook {
    bids: BTreeMap<Price, VecDeque<Resting>>,
    asks: BTreeMap<Price, VecDeque<Resting>>,
    index: BTreeMap<u64, (Side, Price)>,
    trades: Vec<Trade>,
    seq: u64,
    stats: Stats,
}

impl OrderBook {
    /// Creates an empty book.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::OrderBook;
    ///
    /// let book = OrderBook::new();
    /// assert_eq!(book.order_count(), 0);
    /// assert!(book.best_bid().is_none());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Highest resting bid price.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.best_bid(), Some(Price(99)));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    /// Lowest resting ask price.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.best_ask(), Some(Price(101)));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Spread `ask - bid` in ticks; `None` if a side is empty or on overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.spread(), Some(2));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn spread(&self) -> Option<i64> {
        self.best_ask()?.0.checked_sub(self.best_bid()?.0)
    }

    /// Mid price in ticks; `None` if a side is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.mid(), Some(100.0));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn mid(&self) -> Option<f64> {
        Some((self.best_bid()?.0 as f64 + self.best_ask()?.0 as f64) / 2.0)
    }

    /// Total resting quantity at a price level (zero if none).
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.level_qty(Side::Bid, Price(99)), 10);
    /// assert_eq!(book.level_qty(Side::Bid, Price(98)), 0); // empty level
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn level_qty(&self, side: Side, price: Price) -> u128 {
        self.side_map(side)
            .get(&price)
            .map_or(0, |q| q.iter().map(|o| u128::from(o.qty)).sum())
    }

    /// Total resting quantity in the whole book.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.resting_quantity(), 15);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn resting_quantity(&self) -> u128 {
        self.bids
            .values()
            .chain(self.asks.values())
            .flat_map(|q| q.iter())
            .map(|o| u128::from(o.qty))
            .sum()
    }

    /// Number of resting orders.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.order_count(), 2);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn order_count(&self) -> usize {
        self.index.len()
    }

    /// Side, price and remaining quantity of a resting order.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.order(1), Some((Side::Bid, Price(99), 10)));
    /// assert_eq!(book.order(7), None);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn order(&self, id: u64) -> Option<(Side, Price, u64)> {
        let &(side, price) = self.index.get(&id)?;
        let q = self.side_map(side).get(&price)?;
        q.iter().find(|o| o.id == id).map(|o| (side, price, o.qty))
    }

    /// All trades so far, oldest first.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// book.apply(Event::Execute { id: 3, side: Side::Ask, qty: 4 })?;
    /// let trades = book.trades();
    /// assert_eq!(trades.len(), 1);
    /// assert_eq!(trades[0].price, Price(99));
    /// assert_eq!(trades[0].qty, 4);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    /// Discards the trade log (statistics are kept).
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// book.apply(Event::Execute { id: 3, side: Side::Ask, qty: 4 })?;
    /// book.clear_trades();
    /// assert!(book.trades().is_empty());
    /// assert_eq!(book.resting_quantity(), 11); // the book itself is unchanged
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_trades(&mut self) {
        self.trades.clear();
    }

    /// Cumulative quantity statistics.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// assert_eq!(book.stats().accepted, 15);
    /// assert_eq!(book.stats().traded, 0);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Number of events applied so far.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// // one sequence number per applied event
    /// assert_eq!(book.sequence(), 2);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn sequence(&self) -> u64 {
        self.seq
    }

    /// VWAP in ticks of the last `n` trades in the log.
    ///
    /// # Errors
    /// [`OrderBookError::InvalidWindow`] if `n == 0`;
    /// [`OrderBookError::NoTrades`] if the log is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// book.apply(Event::Execute { id: 3, side: Side::Ask, qty: 4 })?;
    /// assert_eq!(book.vwap_last(1)?, 99.0);
    /// assert!(book.vwap_last(0).is_err()); // a zero window is invalid
    /// # Ok(())
    /// # }
    /// ```
    pub fn vwap_last(&self, n: usize) -> Result<f64, OrderBookError> {
        if n == 0 {
            return Err(OrderBookError::InvalidWindow);
        }
        let start = self.trades.len().saturating_sub(n);
        vwap(&self.trades[start..])
    }

    /// VWAP in ticks of trades with `from_seq <= seq <= to_seq`.
    ///
    /// # Errors
    /// [`OrderBookError::InvalidWindow`] if `from_seq > to_seq`;
    /// [`OrderBookError::NoTrades`] if no trade lies in the window.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// book.apply(Event::Execute { id: 3, side: Side::Ask, qty: 4 })?; // seq 3
    /// assert_eq!(book.vwap_window(3, 3)?, 99.0);
    /// assert!(matches!(book.vwap_window(1, 2), Err(OrderBookError::NoTrades)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn vwap_window(&self, from_seq: u64, to_seq: u64) -> Result<f64, OrderBookError> {
        if from_seq > to_seq {
            return Err(OrderBookError::InvalidWindow);
        }
        let start = self.trades.partition_point(|t| t.seq < from_seq);
        let end = self.trades.partition_point(|t| t.seq <= to_seq);
        vwap(&self.trades[start..end])
    }

    /// Applies one event. Returns the number of trades it produced; they are
    /// the last that many entries of [`OrderBook::trades`]. On error the book
    /// is unchanged.
    ///
    /// # Errors
    /// See [`OrderBookError`].
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, OrderBookError, Price, Side};
    /// # fn main() -> Result<(), OrderBookError> {
    /// let mut book = OrderBook::new();
    /// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 })?;
    /// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 })?;
    /// // A bid at or above the best ask crosses the book and trades.
    /// let trades = book.apply(Event::Add { id: 4, side: Side::Bid, price: Price(101), qty: 3 })?;
    /// assert_eq!(trades, 1);
    /// assert_eq!(book.level_qty(Side::Ask, Price(101)), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn apply(&mut self, event: Event) -> Result<usize, OrderBookError> {
        let before = self.trades.len();
        match event {
            Event::Add {
                id,
                side,
                price,
                qty,
            } => self.add(id, side, price, qty)?,
            Event::Cancel { id } => self.cancel(id)?,
            Event::Modify {
                id,
                new_qty,
                new_price,
            } => self.modify(id, new_qty, new_price)?,
            Event::Execute { id, side, qty } => self.market(id, side, qty)?,
            Event::ExecuteOrder { id, qty } => self.execute_order(id, qty)?,
        }
        // Quantity conservation: every accepted unit is resting, traded (on both
        // sides), cancelled or expired. Checked after every event in debug builds.
        let s = self.stats;
        debug_assert!(
            s.accepted == self.resting_quantity() + 2 * s.traded + s.cancelled + s.expired,
            "postcondition failed: accepted == resting + 2 * traded + cancelled + expired"
        );
        Ok(self.trades.len() - before)
    }

    /// Applies events in order, stopping at the first error. Returns the
    /// total number of trades produced.
    ///
    /// # Errors
    /// [`ReplayError`] with the index of the failing event.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::{Event, OrderBook, Price, Side};
    ///
    /// let events = vec![
    ///     Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 },
    ///     Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 },
    ///     Event::Add { id: 3, side: Side::Bid, price: Price(101), qty: 5 },
    /// ];
    /// let mut book = OrderBook::new();
    /// assert_eq!(book.replay(events).unwrap(), 1); // one trade in total
    /// ```
    pub fn replay<I: IntoIterator<Item = Event>>(
        &mut self,
        events: I,
    ) -> Result<usize, ReplayError> {
        let mut total = 0;
        for (index, e) in events.into_iter().enumerate() {
            total += self
                .apply(e)
                .map_err(|source| ReplayError { index, source })?;
        }
        Ok(total)
    }

    fn side_map(&self, side: Side) -> &BTreeMap<Price, VecDeque<Resting>> {
        match side {
            Side::Bid => &self.bids,
            Side::Ask => &self.asks,
        }
    }

    fn side_map_mut(&mut self, side: Side) -> &mut BTreeMap<Price, VecDeque<Resting>> {
        match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        }
    }

    fn add(&mut self, id: u64, side: Side, price: Price, qty: u64) -> Result<(), OrderBookError> {
        if qty == 0 {
            return Err(OrderBookError::ZeroQuantity);
        }
        if self.index.contains_key(&id) {
            return Err(OrderBookError::DuplicateOrderId(id));
        }
        self.seq += 1;
        self.stats.accepted += u128::from(qty);
        self.submit_limit(id, side, price, qty);
        Ok(())
    }

    fn market(&mut self, id: u64, side: Side, qty: u64) -> Result<(), OrderBookError> {
        if qty == 0 {
            return Err(OrderBookError::ZeroQuantity);
        }
        self.seq += 1;
        self.stats.accepted += u128::from(qty);
        let rem = self.match_incoming(Some(id), side, None, qty);
        self.stats.expired += u128::from(rem);
        Ok(())
    }

    fn execute_order(&mut self, id: u64, qty: u64) -> Result<(), OrderBookError> {
        if qty == 0 {
            return Err(OrderBookError::ZeroQuantity);
        }
        let (side, price, rest) = self.order(id).ok_or(OrderBookError::UnknownOrderId(id))?;
        if qty > rest {
            return Err(OrderBookError::QuantityTooLarge(id));
        }
        self.seq += 1;
        self.stats.accepted += u128::from(qty);
        self.stats.traded += u128::from(qty);
        self.trades.push(Trade {
            seq: self.seq,
            price,
            qty,
            aggressor_id: None,
            resting_id: id,
            aggressor_side: side.opposite(),
        });
        self.reduce_resting(id, side, price, qty);
        Ok(())
    }

    fn cancel(&mut self, id: u64) -> Result<(), OrderBookError> {
        let (side, price, qty) = self.order(id).ok_or(OrderBookError::UnknownOrderId(id))?;
        self.seq += 1;
        self.remove_resting(id, side, price);
        self.stats.cancelled += u128::from(qty);
        Ok(())
    }

    fn modify(
        &mut self,
        id: u64,
        new_qty: Option<u64>,
        new_price: Option<Price>,
    ) -> Result<(), OrderBookError> {
        if new_qty.is_none() && new_price.is_none() {
            return Err(OrderBookError::EmptyModify);
        }
        if new_qty == Some(0) {
            return Err(OrderBookError::ZeroQuantity);
        }
        let (side, price, qty) = self.order(id).ok_or(OrderBookError::UnknownOrderId(id))?;
        let nq = new_qty.unwrap_or(qty);
        let np = new_price.unwrap_or(price);
        self.seq += 1;
        if np == price && nq <= qty {
            if nq < qty {
                self.reduce_resting(id, side, price, qty - nq);
                self.stats.cancelled += u128::from(qty - nq);
            }
            return Ok(());
        }
        // Loses time priority: remove and resubmit.
        self.remove_resting(id, side, price);
        if nq >= qty {
            self.stats.accepted += u128::from(nq - qty);
        } else {
            self.stats.cancelled += u128::from(qty - nq);
        }
        self.submit_limit(id, side, np, nq);
        Ok(())
    }

    /// Matches then rests the remainder. Caller has validated and counted.
    fn submit_limit(&mut self, id: u64, side: Side, price: Price, qty: u64) {
        let rem = self.match_incoming(Some(id), side, Some(price), qty);
        if rem > 0 {
            self.side_map_mut(side)
                .entry(price)
                .or_default()
                .push_back(Resting { id, qty: rem });
            self.index.insert(id, (side, price));
        }
    }

    /// Consumes opposite liquidity; returns the unfilled remainder.
    fn match_incoming(
        &mut self,
        id: Option<u64>,
        side: Side,
        limit: Option<Price>,
        mut qty: u64,
    ) -> u64 {
        while qty > 0 {
            let best = match side {
                Side::Bid => self.best_ask(),
                Side::Ask => self.best_bid(),
            };
            let Some(best) = best else { break };
            let crosses = match (side, limit) {
                (_, None) => true,
                (Side::Bid, Some(l)) => best <= l,
                (Side::Ask, Some(l)) => best >= l,
            };
            if !crosses {
                break;
            }
            let opp = side.opposite();
            let Some(front) = self
                .side_map(opp)
                .get(&best)
                .and_then(|q| q.front())
                .copied()
            else {
                break;
            };
            let fill = qty.min(front.qty);
            qty -= fill;
            self.stats.traded += u128::from(fill);
            self.trades.push(Trade {
                seq: self.seq,
                price: best,
                qty: fill,
                aggressor_id: id,
                resting_id: front.id,
                aggressor_side: side,
            });
            self.reduce_resting(front.id, opp, best, fill);
        }
        qty
    }

    /// Reduces a resting order by `by`, removing it when exhausted. Keeps its queue position.
    fn reduce_resting(&mut self, id: u64, side: Side, price: Price, by: u64) {
        let mut emptied = false;
        let mut exhausted = false;
        if let Some(q) = self.side_map_mut(side).get_mut(&price) {
            if let Some(pos) = q.iter().position(|o| o.id == id) {
                if let Some(o) = q.get_mut(pos) {
                    o.qty = o.qty.saturating_sub(by);
                    exhausted = o.qty == 0;
                }
                if exhausted {
                    q.remove(pos);
                }
            }
            emptied = q.is_empty();
        }
        if exhausted {
            self.index.remove(&id);
        }
        if emptied {
            self.side_map_mut(side).remove(&price);
        }
    }

    fn remove_resting(&mut self, id: u64, side: Side, price: Price) {
        let mut emptied = false;
        if let Some(q) = self.side_map_mut(side).get_mut(&price) {
            if let Some(pos) = q.iter().position(|o| o.id == id) {
                q.remove(pos);
            }
            emptied = q.is_empty();
        }
        if emptied {
            self.side_map_mut(side).remove(&price);
        }
        self.index.remove(&id);
    }
}

fn vwap(trades: &[Trade]) -> Result<f64, OrderBookError> {
    let mut num: i128 = 0;
    let mut den: u128 = 0;
    for t in trades {
        num += i128::from(t.price.0) * i128::from(t.qty);
        den += u128::from(t.qty);
    }
    if den == 0 {
        return Err(OrderBookError::NoTrades);
    }
    Ok(num as f64 / den as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add(b: &mut OrderBook, id: u64, side: Side, p: i64, q: u64) -> usize {
        b.apply(Event::Add {
            id,
            side,
            price: Price(p),
            qty: q,
        })
        .unwrap()
    }

    #[test]
    fn quotes_and_empty_sides() {
        let mut b = OrderBook::new();
        assert_eq!(b.spread(), None);
        assert_eq!(b.mid(), None);
        assert_eq!(b.vwap_last(3), Err(OrderBookError::NoTrades));
        add(&mut b, 1, Side::Bid, 99, 10);
        assert_eq!(b.spread(), None);
        add(&mut b, 2, Side::Ask, 102, 10);
        assert_eq!(b.spread(), Some(3));
        assert_eq!(b.mid(), Some(100.5));
    }

    #[test]
    fn price_time_priority() {
        let mut b = OrderBook::new();
        add(&mut b, 1, Side::Ask, 100, 5);
        add(&mut b, 2, Side::Ask, 100, 5);
        add(&mut b, 3, Side::Ask, 101, 5);
        let n = add(&mut b, 4, Side::Bid, 101, 12);
        assert_eq!(n, 3);
        let t = b.trades();
        assert_eq!((t[0].resting_id, t[0].price, t[0].qty), (1, Price(100), 5));
        assert_eq!((t[1].resting_id, t[1].price, t[1].qty), (2, Price(100), 5));
        assert_eq!((t[2].resting_id, t[2].price, t[2].qty), (3, Price(101), 2));
        assert_eq!(b.order(3), Some((Side::Ask, Price(101), 3)));
        assert_eq!(b.order(4), None);
    }

    #[test]
    fn remainder_rests() {
        let mut b = OrderBook::new();
        add(&mut b, 1, Side::Ask, 100, 5);
        add(&mut b, 2, Side::Bid, 100, 8);
        assert_eq!(b.best_ask(), None);
        assert_eq!(b.order(2), Some((Side::Bid, Price(100), 3)));
    }

    #[test]
    fn modify_priority() {
        let mut b = OrderBook::new();
        add(&mut b, 1, Side::Bid, 100, 10);
        add(&mut b, 2, Side::Bid, 100, 10);
        b.apply(Event::Modify {
            id: 1,
            new_qty: Some(4),
            new_price: None,
        })
        .unwrap();
        b.apply(Event::Execute {
            id: 9,
            side: Side::Ask,
            qty: 1,
        })
        .unwrap();
        assert_eq!(b.trades()[0].resting_id, 1);
        // Size up loses priority.
        b.apply(Event::Modify {
            id: 1,
            new_qty: Some(6),
            new_price: None,
        })
        .unwrap();
        b.apply(Event::Execute {
            id: 10,
            side: Side::Ask,
            qty: 1,
        })
        .unwrap();
        assert_eq!(b.trades()[1].resting_id, 2);
        // Price change that crosses matches.
        add(&mut b, 3, Side::Ask, 105, 2);
        let n = b
            .apply(Event::Modify {
                id: 3,
                new_qty: None,
                new_price: Some(Price(100)),
            })
            .unwrap();
        assert_eq!(n, 1);
        assert_eq!(b.trades()[2].aggressor_id, Some(3));
    }

    #[test]
    fn errors_leave_book_unchanged() {
        let mut b = OrderBook::new();
        add(&mut b, 1, Side::Bid, 100, 10);
        let seq = b.sequence();
        assert_eq!(
            b.apply(Event::Add {
                id: 1,
                side: Side::Ask,
                price: Price(1),
                qty: 1
            }),
            Err(OrderBookError::DuplicateOrderId(1))
        );
        assert_eq!(
            b.apply(Event::Cancel { id: 7 }),
            Err(OrderBookError::UnknownOrderId(7))
        );
        assert_eq!(
            b.apply(Event::Modify {
                id: 1,
                new_qty: None,
                new_price: None
            }),
            Err(OrderBookError::EmptyModify)
        );
        assert_eq!(
            b.apply(Event::ExecuteOrder { id: 1, qty: 11 }),
            Err(OrderBookError::QuantityTooLarge(1))
        );
        assert_eq!(
            b.apply(Event::Execute {
                id: 2,
                side: Side::Bid,
                qty: 0
            }),
            Err(OrderBookError::ZeroQuantity)
        );
        assert_eq!(b.sequence(), seq);
        assert_eq!(b.resting_quantity(), 10);
    }

    #[test]
    fn market_order_expires_remainder() {
        let mut b = OrderBook::new();
        add(&mut b, 1, Side::Ask, 100, 3);
        b.apply(Event::Execute {
            id: 2,
            side: Side::Bid,
            qty: 5,
        })
        .unwrap();
        assert_eq!(b.stats().expired, 2);
        assert_eq!(b.stats().traded, 3);
    }

    #[test]
    fn vwap_known_answer() {
        let mut b = OrderBook::new();
        add(&mut b, 1, Side::Ask, 100, 1);
        add(&mut b, 2, Side::Ask, 103, 3);
        b.apply(Event::Execute {
            id: 3,
            side: Side::Bid,
            qty: 4,
        })
        .unwrap();
        // (100*1 + 103*3) / 4 = 102.25
        assert_eq!(b.vwap_last(10).unwrap(), 102.25);
        assert_eq!(b.vwap_last(1).unwrap(), 103.0);
        assert_eq!(b.vwap_last(0), Err(OrderBookError::InvalidWindow));
        let s = b.trades()[0].seq;
        assert_eq!(b.vwap_window(s, s).unwrap(), 102.25);
        assert_eq!(b.vwap_window(s + 1, s + 5), Err(OrderBookError::NoTrades));
        assert_eq!(b.vwap_window(5, 1), Err(OrderBookError::InvalidWindow));
    }

    #[test]
    fn execute_order_and_replay() {
        let mut b = OrderBook::new();
        let ev = [
            Event::Add {
                id: 1,
                side: Side::Bid,
                price: Price(50),
                qty: 10,
            },
            Event::ExecuteOrder { id: 1, qty: 4 },
            Event::Cancel { id: 1 },
        ];
        assert_eq!(b.replay(ev), Ok(1));
        assert_eq!(b.stats().cancelled, 6);
        let err = b.replay([Event::Cancel { id: 1 }]).unwrap_err();
        assert_eq!(err.index, 0);
    }
}
