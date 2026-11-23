// SPDX-License-Identifier: MIT OR Apache-2.0
//! Plain data types.

/// Price in integer ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Price(pub i64);

/// Side of the book.
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
    #[must_use]
    pub fn opposite(self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

/// An order book event.
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
