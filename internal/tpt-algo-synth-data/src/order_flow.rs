// SPDX-License-Identifier: MIT OR Apache-2.0
//! Synthetic limit-order-book event streams.

use alloc::vec::Vec;
use tpt_algo_math_core::rng::SplitMix64;

/// Side of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Buy side (bids).
    Bid,
    /// Sell side (asks).
    Ask,
}

/// A synthetic order-flow event. Prices are integer ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderEvent {
    /// Side of the order.
    pub side: Side,
    /// Price in integer ticks.
    pub price_ticks: i64,
    /// Quantity in lots.
    pub qty: u64,
}

/// Generates `n` limit-order events around `mid_ticks`, with prices offset by up to
/// `max_offset_ticks` from the mid and quantities in `1..=max_qty`.
/// Returns an empty vector if `max_qty == 0`.
pub fn limit_orders(
    seed: u64,
    n: usize,
    mid_ticks: i64,
    max_offset_ticks: u64,
    max_qty: u64,
) -> Vec<OrderEvent> {
    if max_qty == 0 {
        return Vec::new();
    }
    let mut rng = SplitMix64::new(seed);
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let side = if rng.next_u64() & 1 == 0 {
            Side::Bid
        } else {
            Side::Ask
        };
        let offset = rng
            .next_below(max_offset_ticks.saturating_add(1))
            .unwrap_or(0) as i64;
        // Bids sit at or below mid, asks at or above, so generated flow starts uncrossed.
        let price_ticks = match side {
            Side::Bid => mid_ticks.saturating_sub(offset.max(1)),
            Side::Ask => mid_ticks.saturating_add(offset.max(1)),
        };
        let qty = rng.next_below(max_qty).unwrap_or(0).saturating_add(1);
        out.push(OrderEvent {
            side,
            price_ticks,
            qty,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_uncrossed() {
        let a = limit_orders(4, 500, 10_000, 20, 100);
        assert_eq!(a, limit_orders(4, 500, 10_000, 20, 100));
        for e in &a {
            match e.side {
                Side::Bid => assert!(e.price_ticks < 10_000),
                Side::Ask => assert!(e.price_ticks > 10_000),
            }
            assert!(e.qty >= 1 && e.qty <= 100);
        }
    }

    #[test]
    fn zero_qty_gives_empty() {
        assert!(limit_orders(1, 10, 0, 5, 0).is_empty());
    }
}
