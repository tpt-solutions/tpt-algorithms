// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grid path validation against Moving AI benchmark files.
//!
//! The loader reads `.map` (`type octile`, `height`, `width`, `map`, then one
//! row of cells per line) and `.scen` (`version 1`, then one tab-separated
//! line per case: bucket, map, width, height, start x/y, goal x/y, optimal
//! length). In `.map` files `.` and `G` are passable and every other glyph is
//! an obstacle.
//!
//! The hand-made fixtures under `tests/data/` are small maps whose optimal
//! lengths were derived by hand (octile costs: 1 per orthogonal step, sqrt(2)
//! per diagonal step, no diagonal moves past a blocked corner). They are not
//! taken from the Moving AI published sets. The full published benchmark test
//! is `#[ignore]`d and needs `TPT_DATASET_DIR` (see `scripts/fetch-dataset.sh`
//! and `docs/datasets.md`).
#![cfg(feature = "std")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use tpt_algo_swarm::{find_path, Connectivity, Grid, SwarmError};

/// Tolerance for comparing sqrt(2)-based lengths with the published values
/// (which are printed to nine decimals).
const LENGTH_TOLERANCE: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq)]
struct MapFile {
    width: usize,
    height: usize,
    blocked: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq)]
struct Scenario {
    map: String,
    start: (usize, usize),
    goal: (usize, usize),
    optimal: f64,
}

/// Parse a Moving AI `.map` file.
fn parse_map(text: &str) -> Result<MapFile, String> {
    let mut lines = text.lines();
    let mut width = None;
    let mut height = None;
    let mut saw_type = false;
    for line in lines.by_ref() {
        let line = line.trim();
        if line == "map" {
            break;
        }
        let mut parts = line.split_whitespace();
        match (parts.next(), parts.next()) {
            (Some("type"), Some("octile")) => saw_type = true,
            (Some("type"), other) => return Err(format!("unsupported type {other:?}")),
            (Some("height"), Some(v)) => {
                height = Some(v.parse::<usize>().map_err(|e| format!("height: {e}"))?)
            }
            (Some("width"), Some(v)) => {
                width = Some(v.parse::<usize>().map_err(|e| format!("width: {e}"))?)
            }
            _ => return Err(format!("unexpected header line {line:?}")),
        }
    }
    if !saw_type {
        return Err("missing 'type octile' header".to_owned());
    }
    let width = width.ok_or("missing width")?;
    let height = height.ok_or("missing height")?;
    let rows: Vec<&str> = lines.map(str::trim_end).filter(|l| !l.is_empty()).collect();
    if rows.len() != height {
        return Err(format!("expected {height} map rows, found {}", rows.len()));
    }
    let mut blocked = Vec::with_capacity(width * height);
    for row in &rows {
        if row.chars().count() != width {
            return Err(format!(
                "row has {} cells, expected {width}",
                row.chars().count()
            ));
        }
        blocked.extend(row.chars().map(|c| !matches!(c, '.' | 'G')));
    }
    Ok(MapFile {
        width,
        height,
        blocked,
    })
}

/// Parse a Moving AI `.scen` file.
fn parse_scen(text: &str) -> Result<Vec<Scenario>, String> {
    let mut lines = text.lines();
    let header = lines.next().ok_or("empty scenario file")?;
    if header.trim() != "version 1" {
        return Err(format!("unsupported scenario header {header:?}"));
    }
    let mut out = Vec::new();
    for (n, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 9 {
            return Err(format!(
                "case {}: expected 9 fields, got {}",
                n + 2,
                f.len()
            ));
        }
        let num = |s: &str| -> Result<usize, String> {
            s.parse().map_err(|e| format!("case {}: {s:?}: {e}", n + 2))
        };
        let optimal: f64 = f[8]
            .parse()
            .map_err(|e| format!("case {}: optimal: {e}", n + 2))?;
        if !optimal.is_finite() || optimal < 0.0 {
            return Err(format!(
                "case {}: optimal length must be finite and >= 0",
                n + 2
            ));
        }
        out.push(Scenario {
            map: f[1].to_owned(),
            start: (num(f[4])?, num(f[5])?),
            goal: (num(f[6])?, num(f[7])?),
            optimal,
        });
    }
    Ok(out)
}

fn to_grid(map: &MapFile) -> Grid {
    Grid::from_cells(map.width, map.height, map.blocked.clone()).unwrap()
}

/// Octile length of a cell path: 1 per orthogonal step, sqrt(2) per diagonal.
/// Also checks that the path is contiguous and avoids blocked cells.
fn octile_length(map: &MapFile, cells: &[(usize, usize)]) -> Result<f64, String> {
    let mut len = 0.0;
    for w in cells.windows(2) {
        let (a, b) = (w[0], w[1]);
        let dx = a.0.abs_diff(b.0);
        let dy = a.1.abs_diff(b.1);
        match (dx, dy) {
            (1, 0) | (0, 1) => len += 1.0,
            (1, 1) => len += core::f64::consts::SQRT_2,
            _ => return Err(format!("non-adjacent step {a:?} -> {b:?}")),
        }
    }
    for &(x, y) in cells {
        if map.blocked[y * map.width + x] {
            return Err(format!("path enters blocked cell ({x}, {y})"));
        }
    }
    Ok(len)
}

/// Runs every scenario against `map` and returns the number checked.
fn check_scenarios(map: &MapFile, scenarios: &[Scenario]) -> usize {
    let grid = to_grid(map);
    for sc in scenarios {
        let path = find_path(&grid, sc.start, sc.goal, Connectivity::Eight)
            .unwrap_or_else(|e| panic!("{}: {:?} -> {:?}: {e}", sc.map, sc.start, sc.goal));
        assert_eq!(path.cells.first(), Some(&sc.start));
        assert_eq!(path.cells.last(), Some(&sc.goal));
        let len = octile_length(map, &path.cells).unwrap_or_else(|e| panic!("{}: {e}", sc.map));
        assert!(
            (len - sc.optimal).abs() < LENGTH_TOLERANCE,
            "{}: {:?} -> {:?}: path length {len} != optimal {}",
            sc.map,
            sc.start,
            sc.goal,
            sc.optimal
        );
    }
    scenarios.len()
}

fn data_dir() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "data"]
        .iter()
        .collect()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn hand_made_fixtures_match_hand_derived_optimal_lengths() {
    let dir = data_dir();
    let mut total = 0;
    for stem in ["open_4x4", "wall_5x5"] {
        let map = parse_map(&read(&dir.join(format!("{stem}.map")))).unwrap();
        let scen = parse_scen(&read(&dir.join(format!("{stem}.scen")))).unwrap();
        assert!(!scen.is_empty());
        total += check_scenarios(&map, &scen);
    }
    assert_eq!(total, 8);
}

#[test]
fn map_loader_rejects_malformed_input() {
    assert!(parse_map("type grid\nheight 1\nwidth 1\nmap\n.\n").is_err());
    assert!(
        parse_map("height 1\nwidth 1\nmap\n.\n").is_err(),
        "missing type"
    );
    assert!(
        parse_map("type octile\nheight 2\nwidth 1\nmap\n.\n").is_err(),
        "short rows"
    );
    assert!(
        parse_map("type octile\nheight 1\nwidth 3\nmap\n..\n").is_err(),
        "narrow row"
    );
    let ok = parse_map("type octile\nheight 1\nwidth 3\nmap\n.@T\n").unwrap();
    assert_eq!(ok.blocked, vec![false, true, true]);
}

#[test]
fn scen_loader_rejects_malformed_input() {
    assert!(parse_scen("version 2\n").is_err());
    assert!(
        parse_scen("version 1\n0 m.map 4 4 0 0 3\n").is_err(),
        "short line"
    );
    assert!(parse_scen("version 1\n0 m.map 4 4 0 0 3 3 nan\n").is_err());
    let ok = parse_scen("version 1\n0\tm.map\t4\t4\t0\t0\t3\t3\t4.242640687\n").unwrap();
    assert_eq!(ok[0].goal, (3, 3));
}

#[test]
fn octile_checker_detects_bad_paths() {
    let map = parse_map("type octile\nheight 2\nwidth 2\nmap\n.@\n..\n").unwrap();
    // (0,0) -> (1,1) is a diagonal step; it is accepted only as sqrt(2).
    let len = octile_length(&map, &[(0, 0), (0, 1), (1, 1)]).unwrap();
    assert!((len - 2.0).abs() < 1e-12);
    assert!(
        octile_length(&map, &[(0, 0), (1, 0)]).is_err(),
        "blocked cell"
    );
    assert!(octile_length(&map, &[(0, 0), (1, 1), (0, 1)]).is_ok());
    assert!(
        octile_length(&map, &[(0, 1), (1, 0)]).is_err(),
        "not adjacent"
    );
}

#[test]
fn unreachable_goal_is_reported_not_faked() {
    let map = parse_map("type octile\nheight 1\nwidth 3\nmap\n.@.\n").unwrap();
    let grid = to_grid(&map);
    assert_eq!(
        find_path(&grid, (0, 0), (2, 0), Connectivity::Eight).unwrap_err(),
        SwarmError::Unreachable
    );
}

#[test]
#[ignore = "requires Moving AI benchmark; set TPT_DATASET_DIR to a folder of <name>.map and <name>.map.scen pairs"]
fn moving_ai_maps_match_published_optimal_lengths() {
    let Some(dir) = std::env::var_os("TPT_DATASET_DIR").map(PathBuf::from) else {
        panic!("TPT_DATASET_DIR is not set; fetch the benchmark with scripts/fetch-dataset.sh");
    };
    let mut scen_files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "scen"))
        .collect();
    scen_files.sort();
    assert!(
        !scen_files.is_empty(),
        "no .scen files in {}",
        dir.display()
    );
    let mut total = 0;
    for scen_path in scen_files {
        let scen = parse_scen(&read(&scen_path)).unwrap();
        let map_name = scen.first().expect("non-empty scenario file").map.clone();
        let map = parse_map(&read(&dir.join(&map_name))).unwrap();
        total += check_scenarios(&map, &scen);
    }
    assert!(total > 0);
}
