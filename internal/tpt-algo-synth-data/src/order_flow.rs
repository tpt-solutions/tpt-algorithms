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
/// Returns an empty vector if `max_qty == 0`. Offsets larger than `i64::MAX`
/// saturate at `i64::MAX`, and price arithmetic saturates at the `i64` bounds.
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
        // The bound is `max_offset_ticks + 1 >= 1`, so the draw always succeeds.
        let draw = rng.next_below(max_offset_ticks.saturating_add(1));
        debug_assert!(draw.is_some(), "bound is at least 1");
        // Offsets above i64::MAX saturate (documented above).
        let offset = draw.map_or(0, |v| i64::try_from(v).unwrap_or(i64::MAX));
        // Bids sit at or below mid, asks at or above, so generated flow starts uncrossed.
        let price_ticks = match side {
            Side::Bid => mid_ticks.saturating_sub(offset.max(1)),
            Side::Ask => mid_ticks.saturating_add(offset.max(1)),
        };
        // `max_qty > 0` was checked above, so the draw always succeeds; the
        // result is in `1..=max_qty`.
        let qty_draw = rng.next_below(max_qty);
        debug_assert!(qty_draw.is_some(), "bound is at least 1");
        let qty = qty_draw.map_or(1, |v| v.saturating_add(1));
        out.push(OrderEvent {
            side,
            price_ticks,
            qty,
        });
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    #[test]
    fn offsets_and_quantities_stay_in_documented_ranges() {
        let a = limit_orders(9, 5_000, 1_000, 30, 7);
        for e in &a {
            let dist = (e.price_ticks - 1_000).abs();
            assert!((1..=30).contains(&dist), "offset {dist}");
            assert!((1..=7).contains(&e.qty), "qty {}", e.qty);
        }
    }

    #[test]
    fn huge_offset_saturates_instead_of_wrapping() {
        // An offset above i64::MAX must not wrap to a negative value, which would
        // put a bid above the mid.
        let a = limit_orders(3, 2_000, 0, u64::MAX, 1);
        assert!(a.iter().any(|e| e.price_ticks < -1_000_000));
        for e in &a {
            match e.side {
                Side::Bid => assert!(e.price_ticks < 0),
                Side::Ask => assert!(e.price_ticks > 0),
            }
        }
    }
}
