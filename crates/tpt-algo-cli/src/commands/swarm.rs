// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt swarm`: velocity-obstacle avoidance and grid path finding.
//!
//! `tpt swarm avoid --input agents.csv` computes an avoidance velocity for
//! every agent against all others. Columns: `x`, `y`, `vx`, `vy`, `radius`,
//! `pref_vx`, `pref_vy`, `max_speed`.
//!
//! `tpt swarm path --input grid.csv --from x,y --to x,y [--diagonal]` finds a
//! shortest path on a grid. The input has no header; each line is a row of
//! cells, where `0`, `.` or `false` is free and `1`, `#` or `true` is blocked.

use super::{read_table, Report};
use crate::args::{parse_pair, Args};
use crate::error::{algo, CliError};
use crate::json::Json;
use crate::source::Source;
use tpt_algo_swarm::{
    avoid_among, find_path, Agent, AvoidanceConfig, Connectivity, Grid, Responsibility, Vec2,
};

/// Runs the subcommand. `args` holds the tokens after `swarm`.
pub fn run(args: Args, source: &mut dyn Source) -> Result<Report, CliError> {
    let mut tokens = args.into_tokens();
    if tokens.is_empty() {
        return Err(CliError::Usage(
            "swarm needs an action: avoid or path".to_string(),
        ));
    }
    let action = tokens.remove(0);
    let args = Args::new(tokens);
    match action.as_str() {
        "avoid" => avoid(args, source),
        "path" => path(args, source),
        other => Err(CliError::Usage(format!(
            "unknown swarm action {other:?}; use avoid or path"
        ))),
    }
}

fn avoid(mut args: Args, source: &mut dyn Source) -> Result<Report, CliError> {
    let input = args
        .take_value("--input")?
        .ok_or_else(|| CliError::Usage("swarm avoid needs --input <file|->".to_string()))?;
    let defaults = AvoidanceConfig::default();
    let horizon = args.take_parsed("--horizon", defaults.horizon)?;
    let safety_margin = args.take_parsed("--margin", defaults.safety_margin)?;
    let responsibility = match args.take_value("--responsibility")?.as_deref() {
        None | Some("reciprocal") => Responsibility::Reciprocal,
        Some("full") => Responsibility::Full,
        Some(other) => {
            return Err(CliError::Usage(format!(
                "--responsibility must be full or reciprocal, got {other:?}"
            )))
        }
    };
    if !args.positionals()?.is_empty() {
        return Err(CliError::Usage(
            "unexpected positional argument for swarm avoid".to_string(),
        ));
    }
    let cfg = AvoidanceConfig {
        horizon,
        safety_margin,
        responsibility,
        ..defaults
    };

    let table = read_table(&input, source, true)?;
    let mut agents = Vec::with_capacity(table.len());
    for row in 0..table.len() {
        agents.push(Agent {
            position: Vec2 {
                x: table.num(row, "x")?,
                y: table.num(row, "y")?,
            },
            velocity: Vec2 {
                x: table.num(row, "vx")?,
                y: table.num(row, "vy")?,
            },
            radius: table.num(row, "radius")?,
            preferred_velocity: Vec2 {
                x: table.num(row, "pref_vx")?,
                y: table.num(row, "pref_vy")?,
            },
            max_speed: table.num(row, "max_speed")?,
        });
    }

    let mut rows = Vec::with_capacity(agents.len());
    for i in 0..agents.len() {
        let v = avoid_among(&agents, i, &cfg).map_err(algo)?;
        rows.push(Json::obj([
            ("agent", i.into()),
            ("vx", v.x.into()),
            ("vy", v.y.into()),
            ("speed", v.norm().into()),
        ]));
    }
    let rows = Json::Array(rows);
    Ok(Report {
        json: Json::obj([("velocities", rows.clone())]),
        csv: Some(rows),
    })
}

fn path(mut args: Args, source: &mut dyn Source) -> Result<Report, CliError> {
    let input = args
        .take_value("--input")?
        .ok_or_else(|| CliError::Usage("swarm path needs --input <file|->".to_string()))?;
    let from = parse_cell("--from", &args.take_value("--from")?)?;
    let to = parse_cell("--to", &args.take_value("--to")?)?;
    let conn = if args.take_switch("--diagonal") {
        Connectivity::Eight
    } else {
        Connectivity::Four
    };
    if !args.positionals()?.is_empty() {
        return Err(CliError::Usage(
            "unexpected positional argument for swarm path".to_string(),
        ));
    }

    let table = read_table(&input, source, false)?;
    let height = table.len();
    let width = table.columns().len();
    let mut blocked = Vec::with_capacity(width * height);
    for row in 0..height {
        for col in 0..width {
            let name = format!("c{col}");
            let cell = table.text(row, &name).unwrap_or("");
            blocked.push(match cell.to_ascii_lowercase().as_str() {
                "0" | "." | "false" => false,
                "1" | "#" | "true" => true,
                other => {
                    return Err(CliError::Input(format!(
                        "grid row {} column {}: cell {other:?} is not 0/1, ./#, or true/false",
                        row + 1,
                        col + 1
                    )))
                }
            });
        }
    }
    let grid = Grid::from_cells(width, height, blocked).map_err(algo)?;
    let found = find_path(&grid, from, to, conn).map_err(algo)?;

    let steps: Vec<Json> = found
        .cells
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| Json::obj([("step", i.into()), ("x", x.into()), ("y", y.into())]))
        .collect();
    let csv = Json::Array(steps.clone());
    let cells: Vec<Json> = found
        .cells
        .iter()
        .map(|&(x, y)| Json::Array(vec![x.into(), y.into()]))
        .collect();
    Ok(Report {
        json: Json::obj([
            ("cost", found.cost.into()),
            ("steps", found.cells.len().into()),
            ("cells", Json::Array(cells)),
        ]),
        csv: Some(csv),
    })
}

/// Parses `x,y` as a grid coordinate.
fn parse_cell(name: &str, raw: &Option<String>) -> Result<(usize, usize), CliError> {
    let raw = raw
        .as_deref()
        .ok_or_else(|| CliError::Usage(format!("swarm path needs {name} x,y")))?;
    let (x, y) = parse_pair(name, raw)?;
    let to_index = |v: f64| {
        if v >= 0.0 && v.fract() == 0.0 && v <= usize::MAX as f64 {
            Ok(v as usize)
        } else {
            Err(CliError::Usage(format!(
                "{name} must be whole, non-negative coordinates"
            )))
        }
    };
    Ok((to_index(x)?, to_index(y)?))
}
