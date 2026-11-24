// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt orderbook`: replays limit-order events and reports trades and book state.
//!
//! Input columns: `op` (add, cancel, modify, execute, execute_order), `id`,
//! `side` (bid or ask; buy and sell are accepted), `price` (integer ticks),
//! `qty`, `new_qty`, `new_price`. Columns that an operation does not use may
//! be empty or absent.

use super::{read_table, Report};
use crate::args::Args;
use crate::error::{algo, CliError};
use crate::json::Json;
use crate::source::Source;
use crate::table::Table;
use std::str::FromStr;
use tpt_algo_orderbook::{Event, OrderBook, Price, Side, Trade};

/// Runs the subcommand. `args` holds the tokens after `orderbook`.
pub fn run(mut args: Args, source: &mut dyn Source) -> Result<Report, CliError> {
    let input = args
        .take_value("--input")?
        .ok_or_else(|| CliError::Usage("orderbook needs --input <file|->".to_string()))?;
    if !args.positionals()?.is_empty() {
        return Err(CliError::Usage(
            "orderbook takes no positional arguments".to_string(),
        ));
    }
    let table = read_table(&input, source, true)?;
    table.require("op")?;

    let mut events = Vec::with_capacity(table.len());
    for row in 0..table.len() {
        events.push(parse_event(&table, row)?);
    }
    let mut book = OrderBook::new();
    let count = events.len();
    let trade_count = book.replay(events).map_err(algo)?;

    let trades: Vec<Json> = book.trades().iter().map(trade_json).collect();
    let stats = book.stats();
    let stats_json = Json::obj([
        ("accepted", to_int(stats.accepted)),
        ("traded", to_int(stats.traded)),
        ("cancelled", to_int(stats.cancelled)),
        ("expired", to_int(stats.expired)),
    ]);
    let price_json = |p: Option<Price>| p.map_or(Json::Null, |p| Json::Int(i128::from(p.0)));
    let json = Json::obj([
        ("events", count.into()),
        ("trade_count", trade_count.into()),
        ("best_bid", price_json(book.best_bid())),
        ("best_ask", price_json(book.best_ask())),
        (
            "spread",
            book.spread()
                .map_or(Json::Null, |s| Json::Int(i128::from(s))),
        ),
        ("mid", book.mid().map_or(Json::Null, Json::Float)),
        ("resting_quantity", to_int(book.resting_quantity())),
        ("order_count", book.order_count().into()),
        ("sequence", book.sequence().into()),
        ("stats", stats_json),
        ("trades", Json::Array(trades.clone())),
    ]);
    Ok(Report {
        json,
        csv: Some(Json::Array(trades)),
    })
}

fn to_int(v: u128) -> Json {
    Json::Int(i128::try_from(v).unwrap_or(i128::MAX))
}

fn trade_json(t: &Trade) -> Json {
    Json::obj([
        ("seq", t.seq.into()),
        ("price", Json::Int(i128::from(t.price.0))),
        ("qty", t.qty.into()),
        (
            "aggressor_id",
            t.aggressor_id.map_or(Json::Null, Json::from),
        ),
        ("resting_id", t.resting_id.into()),
        ("aggressor_side", side_name(t.aggressor_side).into()),
    ])
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::Bid => "bid",
        Side::Ask => "ask",
    }
}

fn parse_side(table: &Table, row: usize) -> Result<Side, CliError> {
    match table
        .text(row, "side")
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("bid" | "buy") => Ok(Side::Bid),
        Some("ask" | "sell") => Ok(Side::Ask),
        _ => Err(CliError::Input(format!(
            "row {}: side must be bid or ask",
            row + 1
        ))),
    }
}

fn int<T: FromStr>(table: &Table, row: usize, name: &str) -> Result<T, CliError> {
    table
        .text(row, name)
        .ok_or_else(|| CliError::Input(format!("row {}: column {name:?} is empty", row + 1)))?
        .parse::<T>()
        .map_err(|_| {
            CliError::Input(format!(
                "row {}: column {name:?} is not an integer",
                row + 1
            ))
        })
}

fn opt_int<T: FromStr>(table: &Table, row: usize, name: &str) -> Result<Option<T>, CliError> {
    match table.text(row, name) {
        None => Ok(None),
        Some(_) => int(table, row, name).map(Some),
    }
}

fn parse_event(table: &Table, row: usize) -> Result<Event, CliError> {
    let op = table
        .text(row, "op")
        .ok_or_else(|| CliError::Input(format!("row {}: op is empty", row + 1)))?
        .to_ascii_lowercase();
    match op.as_str() {
        "add" => Ok(Event::Add {
            id: int(table, row, "id")?,
            side: parse_side(table, row)?,
            price: Price(int(table, row, "price")?),
            qty: int(table, row, "qty")?,
        }),
        "cancel" => Ok(Event::Cancel {
            id: int(table, row, "id")?,
        }),
        "modify" => Ok(Event::Modify {
            id: int(table, row, "id")?,
            new_qty: opt_int(table, row, "new_qty")?,
            new_price: opt_int::<i64>(table, row, "new_price")?.map(Price),
        }),
        "execute" => Ok(Event::Execute {
            id: int(table, row, "id")?,
            side: parse_side(table, row)?,
            qty: int(table, row, "qty")?,
        }),
        "execute_order" => Ok(Event::ExecuteOrder {
            id: int(table, row, "id")?,
            qty: int(table, row, "qty")?,
        }),
        other => Err(CliError::Input(format!(
            "row {}: unknown op {other:?}",
            row + 1
        ))),
    }
}
