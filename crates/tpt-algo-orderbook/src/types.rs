// SPDX-License-Identifier: MIT OR Apache-2.0
//! Plain data types.

/// Price in integer ticks.
///
/// # Example
///
/// ```
/// use tpt_algo_orderbook::Price;
///
/// // Prices are integer ticks, so ordering is exact.
/// assert!(Price(101) > Price(99));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Price(pub i64);

/// Side of the book.
///
/// # Example
///
/// ```
/// use tpt_algo_orderbook::Side;
///
/// assert_eq!(Side::Bid.opposite(), Side::Ask);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Side {
    /// Buy side.
    Bid,
    /// Sell side.
    Ask,
}

impl Side {
    /// The opposite side.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_orderbook::Side;
    ///
    /// assert_eq!(Side::Ask.opposite(), Side::Bid);
    /// ```
    #[must_use]
    pub fn opposite(self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

/// An order book event.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use tpt_algo_orderbook::{Event, OrderBook, Price, Side};
///
/// let mut book = OrderBook::new();
/// book.apply(Event::Add { id: 1, side: Side::Ask, price: Price(101), qty: 5 }).unwrap();
/// book.apply(Event::Modify { id: 1, new_qty: Some(3), new_price: None }).unwrap(); // size reduction
/// assert_eq!(book.order(1), Some((Side::Ask, Price(101), 3)));
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Event {
    /// Submit a limit order.
    Add {
        /// Unique order id.
        id: u64,
        /// Order side.
        side: Side,
        /// Limit price.
        price: Price,
        /// Quantity (must be positive).
        qty: u64,
    },
    /// Cancel a resting order entirely.
    Cancel {
        /// Id of the resting order.
        id: u64,
    },
    /// Modify a resting order. At least one of `new_qty`, `new_price` must be
    /// given. Pure size reductions keep time priority; anything else loses it.
    Modify {
        /// Id of the resting order.
        id: u64,
        /// New total quantity (positive), if changing.
        new_qty: Option<u64>,
        /// New price, if changing.
        new_price: Option<Price>,
    },
    /// Market order: consumes liquidity on the opposite side; any unfilled
    /// remainder is discarded.
    Execute {
        /// Id recorded as the aggressor in trades.
        id: u64,
        /// Aggressor side.
        side: Side,
        /// Quantity (must be positive).
        qty: u64,
    },
    /// Fill `qty` of a specific resting order (external aggressor), as in
    /// exchange message feeds. The trade has no aggressor id.
    ExecuteOrder {
        /// Id of the resting order.
        id: u64,
        /// Executed quantity (positive, at most the resting quantity).
        qty: u64,
    },
}

/// A single trade.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use tpt_algo_orderbook::{Event, OrderBook, Price, Side};
///
/// let mut book = OrderBook::new();
/// book.apply(Event::Add { id: 1, side: Side::Bid, price: Price(99), qty: 10 }).unwrap();
/// book.apply(Event::Add { id: 2, side: Side::Ask, price: Price(101), qty: 5 }).unwrap();
/// book.apply(Event::Execute { id: 3, side: Side::Ask, qty: 4 }).unwrap();
/// let t = book.trades()[0];
/// assert_eq!(t.aggressor_side, Side::Ask);
/// assert_eq!(t.aggressor_id, Some(3));
/// assert_eq!(t.resting_id, 1);
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trade {
    /// Sequence number of the event that produced the trade (starts at 1).
    pub seq: u64,
    /// Execution price (the resting order's price).
    pub price: Price,
    /// Executed quantity.
    pub qty: u64,
    /// Aggressor order id, `None` for [`Event::ExecuteOrder`].
    pub aggressor_id: Option<u64>,
    /// Resting order id.
    pub resting_id: u64,
    /// Side of the aggressor.
    pub aggressor_side: Side,
}
