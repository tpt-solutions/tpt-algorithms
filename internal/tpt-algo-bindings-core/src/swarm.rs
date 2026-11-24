// SPDX-License-Identifier: MIT OR Apache-2.0
//! Collision avoidance (velocity obstacles) and grid path finding, as JSON.
//!
//! `avoid` input:
//!
//! ```json
//! {
//!   "agent": {"position": [0, 0], "velocity": [1, 0], "radius": 0.5,
//!             "preferred_velocity": [1, 0], "max_speed": 1.0},
//!   "neighbors": [{"position": [4, 0], "velocity": [0, 0], "radius": 0.5,
//!                  "preferred_velocity": [0, 0], "max_speed": 1.0}],
//!   "config": {"horizon": 4.0, "safety_margin": 0.05, "rings": 5,
//!              "angles": 32, "responsibility": "reciprocal"}
//! }
//! ```
//!
//! `neighbors` and `config` are optional; missing `config` keys take the
//! crate defaults. Output: `{"velocity": [vx, vy]}`.
//!
//! `find_path` input: `{"width", "height", "blocked": [[x, y], ...],
//! "start": [x, y], "goal": [x, y], "connectivity": 4 | 8}` (default 4).
//! Output: `{"cells": [[x, y], ...], "cost": c}`.

use crate::error::{invalid_input, BindingError, ErrorKind, Result};
use crate::json::{array, field, float, float_or, pair, parse, size_or, string, uint_or};
use serde_json::{json, Value};
use tpt_algo_swarm::{
    avoid, find_path, Agent, AvoidanceConfig, Connectivity, Grid, Responsibility, SwarmError, Vec2,
};

/// Computes a collision-free velocity for one agent, given JSON input.
///
/// # Errors
/// Malformed input or an invalid agent or configuration.
pub fn avoid_json(input: &str) -> Result<String> {
    let doc = parse(input)?;
    let subject = parse_agent(required_value(&doc, "agent")?)?;
    let neighbors = match field(&doc, "neighbors") {
        None => Vec::new(),
        Some(_) => array(&doc, "neighbors")?
            .iter()
            .map(parse_agent)
            .collect::<Result<Vec<_>>>()?,
    };
    let config = match field(&doc, "config") {
        None => AvoidanceConfig::default(),
        Some(c) => avoidance_config(c)?,
    };
    let v = avoid(&subject, &neighbors, &config).map_err(swarm_error)?;
    Ok(json!({ "velocity": [v.x, v.y] }).to_string())
}

/// Finds a shortest path on a grid, given JSON input.
///
/// # Errors
/// Malformed input, out-of-range cells, blocked endpoints, or no path.
pub fn find_path_json(input: &str) -> Result<String> {
    let doc = parse(input)?;
    let width = size_or(&doc, "width", 0)?;
    let height = size_or(&doc, "height", 0)?;
    let mut grid = Grid::new(width, height).map_err(swarm_error)?;
    if field(&doc, "blocked").is_some() {
        for cell in array(&doc, "blocked")? {
            let [x, y] = cell_index(cell, "blocked")?;
            grid.set_blocked(x, y, true).map_err(swarm_error)?;
        }
    }
    let start = cell_index(required_value(&doc, "start")?, "start")?;
    let goal = cell_index(required_value(&doc, "goal")?, "goal")?;
    let connectivity = match uint_or(&doc, "connectivity", 4)? {
        4 => Connectivity::Four,
        8 => Connectivity::Eight,
        other => {
            return Err(BindingError::new(
                ErrorKind::InvalidConfig,
                format!("connectivity must be 4 or 8, got {other}"),
            ))
        }
    };
    let path = find_path(
        &grid,
        (start[0], start[1]),
        (goal[0], goal[1]),
        connectivity,
    )
    .map_err(swarm_error)?;
    let cells: Vec<[usize; 2]> = path.cells.iter().map(|&(x, y)| [x, y]).collect();
    Ok(json!({ "cells": cells, "cost": path.cost }).to_string())
}

fn required_value<'a>(doc: &'a Value, key: &str) -> Result<&'a Value> {
    field(doc, key).ok_or_else(|| invalid_input(format!("missing field '{key}'")))
}

fn vec2(v: &Value, key: &str) -> Result<Vec2> {
    let [x, y] = pair(required_value(v, key)?, key)?;
    Ok(Vec2::new(x, y))
}

fn parse_agent(v: &Value) -> Result<Agent> {
    Ok(Agent {
        position: vec2(v, "position")?,
        velocity: vec2(v, "velocity")?,
        radius: float(v, "radius")?,
        preferred_velocity: vec2(v, "preferred_velocity")?,
        max_speed: float(v, "max_speed")?,
    })
}

fn avoidance_config(v: &Value) -> Result<AvoidanceConfig> {
    let d = AvoidanceConfig::default();
    let responsibility = match field(v, "responsibility") {
        None => d.responsibility,
        Some(_) => match string(v, "responsibility")? {
            "full" => Responsibility::Full,
            "reciprocal" => Responsibility::Reciprocal,
            other => {
                return Err(invalid_input(format!(
                    "responsibility must be 'full' or 'reciprocal', got '{other}'"
                )))
            }
        },
    };
    Ok(AvoidanceConfig {
        horizon: float_or(v, "horizon", d.horizon)?,
        safety_margin: float_or(v, "safety_margin", d.safety_margin)?,
        rings: size_or(v, "rings", d.rings)?,
        angles: size_or(v, "angles", d.angles)?,
        responsibility,
    })
}

fn cell_index(v: &Value, what: &str) -> Result<[usize; 2]> {
    let [x, y] = pair(v, what)?;
    let to_index = |n: f64| -> Result<usize> {
        if n >= 0.0 && n.fract() == 0.0 && n <= usize::MAX as f64 {
            Ok(n as usize)
        } else {
            Err(invalid_input(format!(
                "'{what}' must hold non-negative integers"
            )))
        }
    };
    Ok([to_index(x)?, to_index(y)?])
}

fn swarm_error(e: SwarmError) -> BindingError {
    let kind = match e {
        SwarmError::InvalidConfig(_) => ErrorKind::InvalidConfig,
        SwarmError::Unreachable => ErrorKind::Unreachable,
        _ => ErrorKind::InvalidInput,
    };
    BindingError::new(kind, e.to_string())
}
