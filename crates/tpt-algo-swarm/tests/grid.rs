// SPDX-License-Identifier: MIT OR Apache-2.0
//! A* tests.
#![cfg(feature = "alloc")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use tpt_algo_swarm::{find_path, Connectivity, Grid, SwarmError};

fn wall_grid() -> Grid {
    // 5x5 with a vertical wall at x=2 leaving a gap at y=4.
    let mut g = Grid::new(5, 5).unwrap();
    for y in 0..4 {
        g.set_blocked(2, y, true).unwrap();
    }
    g
}

#[test]
fn straight_line() {
    let g = Grid::new(6, 1).unwrap();
    let p = find_path(&g, (0, 0), (5, 0), Connectivity::Four).unwrap();
    assert_eq!(p.cells.len(), 6);
    assert!((p.cost - 5.0).abs() < 1e-12);
}

#[test]
fn start_equals_goal() {
    let g = Grid::new(3, 3).unwrap();
    let p = find_path(&g, (1, 1), (1, 1), Connectivity::Eight).unwrap();
    assert_eq!(p.cells, vec![(1, 1)]);
    assert_eq!(p.cost, 0.0);
}

#[test]
fn detours_around_wall() {
    let g = wall_grid();
    let p4 = find_path(&g, (0, 0), (4, 0), Connectivity::Four).unwrap();
    assert!((p4.cost - 12.0).abs() < 1e-12); // 4 down, 4 across, 4 up
    let p8 = find_path(&g, (0, 0), (4, 0), Connectivity::Eight).unwrap();
    assert!(p8.cost < p4.cost);
    assert!(p8.cells.iter().all(|&(x, y)| !g.is_blocked(x, y).unwrap()));
}

#[test]
fn diagonal_known_cost() {
    let g = Grid::new(4, 4).unwrap();
    let p = find_path(&g, (0, 0), (3, 3), Connectivity::Eight).unwrap();
    assert!((p.cost - 4.2).abs() < 1e-12);
}

#[test]
fn no_corner_cutting() {
    let mut g = Grid::new(2, 2).unwrap();
    g.set_blocked(1, 0, true).unwrap();
    g.set_blocked(0, 1, true).unwrap();
    assert_eq!(
        find_path(&g, (0, 0), (1, 1), Connectivity::Eight),
        Err(SwarmError::Unreachable)
    );
}

#[test]
fn errors() {
    let mut g = wall_grid();
    assert_eq!(
        find_path(&g, (9, 0), (0, 0), Connectivity::Four),
        Err(SwarmError::OutOfBounds { x: 9, y: 0 })
    );
    assert_eq!(
        find_path(&g, (2, 0), (0, 0), Connectivity::Four),
        Err(SwarmError::StartBlocked)
    );
    assert_eq!(
        find_path(&g, (0, 0), (2, 1), Connectivity::Four),
        Err(SwarmError::GoalBlocked)
    );
    g.set_blocked(2, 4, true).unwrap();
    assert_eq!(
        find_path(&g, (0, 0), (4, 0), Connectivity::Eight),
        Err(SwarmError::Unreachable)
    );
    assert!(Grid::new(0, 3).is_err());
    assert!(Grid::from_cells(2, 2, vec![false; 3]).is_err());
}

#[test]
fn deterministic() {
    let g = Grid::new(8, 8).unwrap();
    let a = find_path(&g, (0, 0), (7, 7), Connectivity::Four).unwrap();
    let b = find_path(&g, (0, 0), (7, 7), Connectivity::Four).unwrap();
    assert_eq!(a, b);
}

proptest! {
    /// On random grids, any returned path is valid, connected, unblocked and
    /// not cheaper than the octile/Manhattan lower bound.
    #[test]
    fn path_validity(
        cells in proptest::collection::vec(proptest::bool::weighted(0.25), 100),
        sx in 0usize..10, sy in 0usize..10, gx in 0usize..10, gy in 0usize..10,
        eight in any::<bool>(),
    ) {
        let g = Grid::from_cells(10, 10, cells).unwrap();
        let conn = if eight { Connectivity::Eight } else { Connectivity::Four };
        if let Ok(p) = find_path(&g, (sx, sy), (gx, gy), conn) {
            prop_assert_eq!(p.cells[0], (sx, sy));
            prop_assert_eq!(*p.cells.last().unwrap(), (gx, gy));
            for w in p.cells.windows(2) {
                let dx = w[0].0.abs_diff(w[1].0);
                let dy = w[0].1.abs_diff(w[1].1);
                prop_assert!(dx <= 1 && dy <= 1 && dx + dy >= 1);
                if !eight { prop_assert!(dx + dy == 1); }
            }
            for &(x, y) in &p.cells {
                prop_assert!(!g.is_blocked(x, y).unwrap());
            }
            let dx = sx.abs_diff(gx) as f64;
            let dy = sy.abs_diff(gy) as f64;
            let lb = if eight { dx.max(dy) } else { dx + dy };
            prop_assert!(p.cost + 1e-9 >= lb);
        }
    }
}
