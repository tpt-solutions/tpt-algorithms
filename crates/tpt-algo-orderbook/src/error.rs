// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error type.

/// Errors returned by order book operations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OrderBookError {
    /// An order quantity of zero was supplied.
    #[error("order quantity must be positive")]
    ZeroQuantity,
    /// The order id is already resting in the book.
    #[error("duplicate order id {0}")]
    DuplicateOrderId(u64),
    /// The order id is not resting in the book.
    #[error("unknown order id {0}")]
    UnknownOrderId(u64),
    /// A modify event changed neither price nor quantity.
    #[error("modify must change price or quantity")]
    EmptyModify,
    /// A window or count argument was invalid (zero or inverted).
    #[error("invalid window")]
    InvalidWindow,
    /// No trades fall in the requested window.
    #[error("no trades in window")]
    NoTrades,
    /// A quantity exceeds the resting quantity of the order.
    #[error("quantity exceeds resting quantity of order {0}")]
    QuantityTooLarge(u64),
}
