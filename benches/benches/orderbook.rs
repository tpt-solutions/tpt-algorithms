// SPDX-License-Identifier: MIT OR Apache-2.0
//! Criterion benchmarks for `tpt-algo-orderbook` (event application and matching).

#![allow(missing_docs)] // criterion_group! generates an undocumented entry point.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tpt_algo_orderbook::{Event, OrderBook, Price, Side};
use tpt_algo_synth_data::order_flow::{limit_orders, Side as FlowSide};

const SIZES: [usize; 2] = [10_000, 100_000];

/// Synthetic event stream: mostly limit adds, with every tenth event a market order that crosses the book.
fn events(n: usize) -> Vec<Event> {
    limit_orders(11, n, 10_000, 50, 100)
        .into_iter()
        .enumerate()
        .map(|(i, order)| {
            let id = i as u64 + 1;
            let side = match order.side {
                FlowSide::Bid => Side::Bid,
                FlowSide::Ask => Side::Ask,
            };
            if i % 10 == 9 {
                Event::Execute {
                    id,
                    side,
                    qty: order.qty,
                }
            } else {
                Event::Add {
                    id,
                    side,
                    price: Price(order.price_ticks),
                    qty: order.qty,
                }
            }
        })
        .collect()
}

fn bench_apply(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_apply");
    for &n in &SIZES {
        let stream = events(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &stream, |b, stream| {
            b.iter(|| {
                let mut book = OrderBook::new();
                for event in stream {
                    black_box(book.apply(*event).is_ok());
                }
                black_box(book.stats())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_apply);
criterion_main!(benches);
