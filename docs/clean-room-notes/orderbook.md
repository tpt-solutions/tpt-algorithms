# Clean-room notes: tpt-algo-orderbook

Written from memory of published descriptions; a human should verify citations.

## Sources (descriptions only, no code consulted)

- R. Cont, S. Stoikov, R. Talreja, "A stochastic model for order book dynamics",
  Operations Research 58(3), 2010. Background: a limit order book as per-price
  queues; events are limit order arrival, cancellation, and market orders.
- M. D. Gould, M. A. Porter, S. Williams, M. McDonald, D. J. Fenn, S. D. Howison,
  "Limit order books", Quantitative Finance 13(11), 2013. Background: price-time
  priority, definitions of spread, mid price and depth.
- N. Huang and T. Polak, "LOBSTER: Limit Order Book Reconstruction System",
  2011 (dataset documentation). Message types: 1 submit, 2 partial cancel,
  3 delete, 4 execute visible, 5 execute hidden, 7 trading halt; price in
  1e-4 dollars; direction -1 sell / 1 buy.

## Algorithm as implemented

- Per side, an ordered map price -> FIFO queue. Best bid is the highest bid
  key, best ask the lowest ask key.
- Incoming limit order: match against the opposite best while prices cross,
  oldest first, at the resting price; rest the remainder at the limit.
- Market order: same, without a price limit; leftover is discarded.
- Modify: size reduction at the same price keeps queue position; price change
  or size increase re-queues at the back (common exchange convention).
- VWAP = sum(p*q)/sum(q) over selected trades; spread = ask - bid;
  mid = (ask + bid)/2.
- Conservation identity: accepted = resting + 2*traded + cancelled + expired.

No GPL/LGPL/proprietary code was read or copied.

## LOBSTER terms of use

To be checked before publishing workflow docs. No data is downloaded or
redistributed by this repository; `tests/validation.rs` only reads a local
directory named by `TPT_DATASET_DIR`.
