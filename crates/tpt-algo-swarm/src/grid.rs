// SPDX-License-Identifier: MIT OR Apache-2.0
//! Occupancy grid and A* search (Hart, Nilsson & Raphael, 1968).

use alloc::collections::BinaryHeap;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;

use crate::SwarmError;

/// Neighbourhood used by the search.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{find_path, Connectivity, Grid};
///
/// let grid = Grid::new(3, 3).unwrap();
/// // Four-connected moves are axis-aligned only; eight-connected allows diagonals.
/// let p4 = find_path(&grid, (0, 0), (2, 2), Connectivity::Four).unwrap();
/// let p8 = find_path(&grid, (0, 0), (2, 2), Connectivity::Eight).unwrap();
/// assert!(p8.cost < p4.cost);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connectivity {
    /// Moves along the four axes (cost 1 each; Manhattan heuristic).
    Four,
    /// Axes plus diagonals (diagonal cost ~1.4; octile heuristic). Diagonal
    /// moves are not allowed to cut a blocked corner.
    Eight,
}

/// A rectangular occupancy grid; cell `(x, y)` is stored at `y * width + x`.
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::Grid;
///
/// let mut grid = Grid::new(3, 2).unwrap();
/// grid.set_blocked(1, 0, true).unwrap();
/// assert!(grid.is_blocked(1, 0).unwrap());
/// assert_eq!((grid.width(), grid.height()), (3, 2));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    blocked: Vec<bool>,
}

impl Grid {
    /// Creates an empty (all free) grid.
    ///
    /// # Errors
    /// [`SwarmError::InvalidGrid`] if a dimension is zero or too large.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Grid;
    ///
    /// let grid = Grid::new(4, 4).unwrap();
    /// assert!(!grid.is_blocked(0, 0).unwrap()); // open by default
    /// ```
    pub fn new(width: usize, height: usize) -> Result<Self, SwarmError> {
        if width == 0 || height == 0 {
            return Err(SwarmError::InvalidGrid("dimensions must be non-zero"));
        }
        let n = width
            .checked_mul(height)
            .ok_or(SwarmError::InvalidGrid("dimensions overflow"))?;
        Ok(Self {
            width,
            height,
            blocked: vec![false; n],
        })
    }

    /// Creates a grid from row-major blocked flags.
    ///
    /// # Errors
    /// [`SwarmError::InvalidGrid`] if `blocked.len() != width * height`.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Grid;
    ///
    /// // Row-major cells: index = y * width + x. Cell (1, 0) is blocked.
    /// let grid = Grid::from_cells(2, 2, vec![false, true, false, false]).unwrap();
    /// assert!(grid.is_blocked(1, 0).unwrap());
    /// assert!(!grid.is_blocked(0, 0).unwrap());
    /// ```
    pub fn from_cells(width: usize, height: usize, blocked: Vec<bool>) -> Result<Self, SwarmError> {
        let g = Self::new(width, height)?;
        if blocked.len() != g.blocked.len() {
            return Err(SwarmError::InvalidGrid("cell count mismatch"));
        }
        Ok(Self { blocked, ..g })
    }

    /// Grid width.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Grid;
    ///
    /// assert_eq!(Grid::new(5, 2).unwrap().width(), 5);
    /// ```
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Grid height.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Grid;
    ///
    /// assert_eq!(Grid::new(5, 2).unwrap().height(), 2);
    /// ```
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Sets a cell blocked or free.
    ///
    /// # Errors
    /// [`SwarmError::OutOfBounds`].
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::Grid;
    ///
    /// let mut grid = Grid::new(3, 3).unwrap();
    /// grid.set_blocked(1, 1, true).unwrap();
    /// grid.set_blocked(1, 1, false).unwrap(); // and unblock it again
    /// assert!(!grid.is_blocked(1, 1).unwrap());
    /// ```
    pub fn set_blocked(&mut self, x: usize, y: usize, blocked: bool) -> Result<(), SwarmError> {
        let i = self.index(x, y)?;
        self.blocked[i] = blocked;
        Ok(())
    }

    /// Whether a cell is blocked.
    ///
    /// # Errors
    /// [`SwarmError::OutOfBounds`].
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_algo_swarm::{Grid, SwarmError};
    ///
    /// let grid = Grid::new(3, 3).unwrap();
    /// assert!(matches!(grid.is_blocked(3, 0), Err(SwarmError::OutOfBounds { .. })));
    /// ```
    pub fn is_blocked(&self, x: usize, y: usize) -> Result<bool, SwarmError> {
        Ok(self.blocked[self.index(x, y)?])
    }

    fn index(&self, x: usize, y: usize) -> Result<usize, SwarmError> {
        if x >= self.width || y >= self.height {
            return Err(SwarmError::OutOfBounds { x, y });
        }
        Ok(y * self.width + x)
    }
}

/// A path found by [`find_path`].
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{find_path, Connectivity, Grid};
///
/// let grid = Grid::new(3, 1).unwrap();
/// let path = find_path(&grid, (0, 0), (2, 0), Connectivity::Four).unwrap();
/// assert_eq!(path.cells, vec![(0, 0), (1, 0), (2, 0)]);
/// assert_eq!(path.cost, 2.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GridPath {
    /// Cells from start to goal inclusive.
    pub cells: Vec<(usize, usize)>,
    /// Total path length (orthogonal step = 1, diagonal step = 1.4).
    pub cost: f64,
}

// Integer costs (x10) keep the search exact and deterministic.
const ORTHO: u64 = 10;
const DIAG: u64 = 14;

fn heuristic(conn: Connectivity, x: usize, y: usize, gx: usize, gy: usize) -> u64 {
    let dx = x.abs_diff(gx) as u64;
    let dy = y.abs_diff(gy) as u64;
    match conn {
        Connectivity::Four => ORTHO * (dx + dy),
        Connectivity::Eight => {
            let (lo, hi) = if dx < dy { (dx, dy) } else { (dy, dx) };
            DIAG * lo + ORTHO * (hi - lo)
        }
    }
}

/// Finds a minimum-cost path with A*.
///
/// With [`Connectivity::Eight`] diagonal cost is 1.4 (an integer-friendly
/// approximation of sqrt 2); the octile heuristic with the same constants is
/// consistent, so the result is optimal for these costs. Ties are broken by
/// larger g (deeper node first), then lower cell index, so output is
/// deterministic.
///
/// # Errors
/// [`SwarmError::OutOfBounds`], [`SwarmError::StartBlocked`],
/// [`SwarmError::GoalBlocked`], [`SwarmError::Unreachable`].
///
/// # Example
///
/// ```
/// use tpt_algo_swarm::{find_path, Connectivity, Grid, SwarmError};
///
/// let mut grid = Grid::new(3, 1).unwrap();
/// grid.set_blocked(1, 0, true).unwrap(); // a wall across the only row
/// assert!(matches!(
///     find_path(&grid, (0, 0), (2, 0), Connectivity::Four),
///     Err(SwarmError::Unreachable)
/// ));
/// ```
pub fn find_path(
    grid: &Grid,
    start: (usize, usize),
    goal: (usize, usize),
    conn: Connectivity,
) -> Result<GridPath, SwarmError> {
    let s = grid.index(start.0, start.1)?;
    let g = grid.index(goal.0, goal.1)?;
    if grid.blocked[s] {
        return Err(SwarmError::StartBlocked);
    }
    if grid.blocked[g] {
        return Err(SwarmError::GoalBlocked);
    }
    let w = grid.width;
    let n = grid.blocked.len();
    let mut best_g = vec![u64::MAX; n];
    let mut parent = vec![usize::MAX; n];
    let mut closed = vec![false; n];
    let mut heap = BinaryHeap::new();
    best_g[s] = 0;
    heap.push(Reverse((
        heuristic(conn, start.0, start.1, goal.0, goal.1),
        Reverse(0u64),
        s,
    )));

    const N4: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    const N8: [(isize, isize); 8] = [
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
        (1, 1),
        (-1, 1),
        (-1, -1),
        (1, -1),
    ];
    let dirs: &[(isize, isize)] = match conn {
        Connectivity::Four => &N4,
        Connectivity::Eight => &N8,
    };

    while let Some(Reverse((_, Reverse(cg), u))) = heap.pop() {
        if closed[u] {
            continue;
        }
        closed[u] = true;
        if u == g {
            break;
        }
        let (ux, uy) = (u % w, u / w);
        for &(dx, dy) in dirs {
            let (Some(nx), Some(ny)) = (ux.checked_add_signed(dx), uy.checked_add_signed(dy))
            else {
                continue;
            };
            if nx >= w || ny >= grid.height {
                continue;
            }
            let v = ny * w + nx;
            if grid.blocked[v] || closed[v] {
                continue;
            }
            let step = if dx != 0 && dy != 0 {
                // No corner cutting.
                if grid.blocked[uy * w + nx] || grid.blocked[ny * w + ux] {
                    continue;
                }
                DIAG
            } else {
                ORTHO
            };
            let ng = cg + step;
            if ng < best_g[v] {
                best_g[v] = ng;
                parent[v] = u;
                heap.push(Reverse((
                    ng + heuristic(conn, nx, ny, goal.0, goal.1),
                    Reverse(ng),
                    v,
                )));
            }
        }
    }

    if !closed[g] {
        return Err(SwarmError::Unreachable);
    }
    let mut cells = Vec::new();
    let mut cur = g;
    while cur != usize::MAX {
        cells.push((cur % w, cur / w));
        cur = parent[cur];
    }
    cells.reverse();
    Ok(GridPath {
        cells,
        cost: best_g[g] as f64 / ORTHO as f64,
    })
}
