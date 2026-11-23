// SPDX-License-Identifier: MIT OR Apache-2.0
//! Dataset-backed validation (LOBSTER). Not run by default.
//!
//! LOBSTER (Huang and Polak, 2011, "LOBSTER: Limit Order Book Reconstruction
//! System") provides, per ticker/day, a `*_message_*.csv` (time, type, order id,
//! size, price in 1e-4 dollars, direction -1 sell / 1 buy) and a matching
//! `*_orderbook_*.csv` (ask price1, ask size1, bid price1, bid size1, ...).
//! Terms of use: to be checked before publishing workflow docs. Nothing is
//! downloaded by this crate; point `TPT_DATASET_DIR` at a directory you have
//! obtained under those terms.
#![cfg(feature = "alloc")]

use std::fs;
use std::path::{Path, PathBuf};

use tpt_algo_orderbook::{Event, OrderBook, Price, Side};

fn find(dir: &Path, needle: &str) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains(needle) && n.ends_with(".csv"))
        })
}

#[test]
#[ignore = "requires dataset; set TPT_DATASET_DIR"]
fn lobster_best_quotes_match_ground_truth() {
    let Ok(dir) = std::env::var("TPT_DATASET_DIR") else {
        eprintln!("TPT_DATASET_DIR is not set; skipping LOBSTER validation");
        return;
    };
    let dir = PathBuf::from(dir);
    let (Some(msg), Some(ob)) = (find(&dir, "_message_"), find(&dir, "_orderbook_")) else {
        eprintln!(
            "no LOBSTER *_message_*.csv / *_orderbook_*.csv in {}",
            dir.display()
        );
        return;
    };
    let msgs = fs::read_to_string(msg).unwrap();
    let obs = fs::read_to_string(ob).unwrap();

    let mut book = OrderBook::new();
    let (mut compared, mut mismatched) = (0usize, 0usize);
    for (m, o) in msgs.lines().zip(obs.lines()) {
        let f: Vec<&str> = m.split(',').map(str::trim).collect();
        if f.len() < 6 {
            continue;
        }
        let kind: u32 = f[1].parse().unwrap();
        let id: u64 = f[2].parse().unwrap();
        let size: u64 = f[3].parse().unwrap();
        let price: i64 = f[4].parse().unwrap();
        let side = if f[5] == "1" { Side::Bid } else { Side::Ask };
        // Orders resting before the sample starts are unknown to us: such
        // errors are ignored, which is why only a mismatch *rate* is asserted.
        let _ = match kind {
            1 => book.apply(Event::Add {
                id,
                side,
                price: Price(price),
                qty: size,
            }),
            2 => match book.order(id) {
                Some((_, _, q)) if q > size => book.apply(Event::Modify {
                    id,
                    new_qty: Some(q - size),
                    new_price: None,
                }),
                _ => book.apply(Event::Cancel { id }),
            },
            3 => book.apply(Event::Cancel { id }),
            4 => book.apply(Event::ExecuteOrder { id, qty: size }),
            _ => Ok(0),
        };
        let c: Vec<i64> = o.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        if c.len() >= 4 {
            if let (Some(a), Some(b)) = (book.best_ask(), book.best_bid()) {
                compared += 1;
                if a.0 != c[0] || b.0 != c[2] {
                    mismatched += 1;
                }
            }
        }
    }
    eprintln!("compared {compared} rows, {mismatched} mismatched");
    assert!(compared > 0);
    assert!(
        mismatched as f64 / compared as f64 <= 1e-4,
        "best quotes deviate from ground truth"
    );
}
