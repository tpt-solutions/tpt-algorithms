// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt route`: capacitated vehicle routing with optional time windows.
//!
//! Input columns: `x`, `y`, `demand` (required); `earliest`, `latest`
//! (optional, both or neither), `service_time` (optional, default 0). Each
//! data row is one customer. Routes refer to customers by their 0-based row
//! number in the input.

use super::{read_table, Report};
use crate::args::{parse_pair, Args};
use crate::error::{algo, CliError};
use crate::json::Json;
use crate::source::Source;
use tpt_algo_routing::{solve, Config, Customer, Problem, TimeWindow};

/// Runs the subcommand. `args` holds the tokens after `route`.
pub fn run(mut args: Args, source: &mut dyn Source) -> Result<Report, CliError> {
    let input = args
        .take_value("--input")?
        .ok_or_else(|| CliError::Usage("route needs --input <file|->".to_string()))?;
    let depot = match args.take_value("--depot")? {
        None => (0.0, 0.0),
        Some(raw) => parse_pair("--depot", &raw)?,
    };
    let capacity: f64 = args
        .take_opt_parsed("--capacity")?
        .ok_or_else(|| CliError::Usage("route needs --capacity <number>".to_string()))?;
    let max_vehicles: Option<usize> = args.take_opt_parsed("--max-vehicles")?;
    let depot_due: Option<f64> = args.take_opt_parsed("--depot-due")?;
    let iterations: usize = args.take_parsed("--iterations", Config::default().max_iterations)?;
    if !args.positionals()?.is_empty() {
        return Err(CliError::Usage(
            "route takes no positional arguments".to_string(),
        ));
    }

    let table = read_table(&input, source, true)?;
    let mut customers = Vec::with_capacity(table.len());
    for row in 0..table.len() {
        let window = match (
            table.num_opt(row, "earliest")?,
            table.num_opt(row, "latest")?,
        ) {
            (None, None) => None,
            (Some(earliest), Some(latest)) => Some(TimeWindow { earliest, latest }),
            _ => {
                return Err(CliError::Input(format!(
                    "row {}: earliest and latest must both be set or both empty",
                    row + 1
                )))
            }
        };
        customers.push(Customer {
            x: table.num(row, "x")?,
            y: table.num(row, "y")?,
            demand: table.num(row, "demand")?,
            window,
            service_time: table.num_opt(row, "service_time")?.unwrap_or(0.0),
        });
    }

    let mut problem = Problem::new(depot, capacity, customers).map_err(algo)?;
    if let Some(m) = max_vehicles {
        problem = problem.with_max_vehicles(m).map_err(algo)?;
    }
    if let Some(due) = depot_due {
        problem = problem.with_depot_due(due).map_err(algo)?;
    }
    let config = Config {
        max_iterations: iterations,
    };
    let solution = solve(&problem, &config).map_err(algo)?;

    let routes: Vec<Json> = solution
        .routes
        .iter()
        .map(|r| Json::Array(r.iter().map(|&c| Json::from(c)).collect()))
        .collect();
    let csv_rows: Vec<Json> = solution
        .routes
        .iter()
        .enumerate()
        .map(|(k, r)| {
            let stops: Vec<String> = r.iter().map(usize::to_string).collect();
            Json::obj([("vehicle", k.into()), ("stops", stops.join(";").into())])
        })
        .collect();
    let json = Json::obj([
        ("cost", solution.cost(&problem).into()),
        ("vehicles_used", solution.vehicles_used().into()),
        ("routes", Json::Array(routes)),
    ]);
    Ok(Report {
        json,
        csv: Some(Json::Array(csv_rows)),
    })
}
