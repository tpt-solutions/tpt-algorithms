# tpt-algo-geometry

Shared 2D and 3D geometry and kinematics helpers for the `tpt-algorithms`
workspace. Square roots come from `libm`, so results do not depend on the
platform's math library.

> **Internal crate.** `publish = false`. It is never released to crates.io, and
> the public algorithm crates do not depend on it (see `docs/decisions.md`, D5).

## Contents

| Module | Provides |
|--------|----------|
| `vec` | `Vec2` and `Vec3`: `dot`, `cross`, `norm`, `norm_squared`, `distance`, `normalized` (returns `Result`) and `is_finite`. |
| `shapes2d` | `Aabb2` (`from_points`, `contains`, `intersects`), `segment_intersection`, `point_segment_distance` and `circles_overlap`. |
| `kinematics` | `constant_accel_position` and `time_to_collision`. |
| `error` | `GeometryError`, returned for example when normalising a zero vector. |

## Features

- `std` (default): implies `alloc`.
- `alloc`: reserved for allocating helpers.
- `serde`: serde support for the vector and shape types.

The crate is `no_std` without `std`.

## Usage

```rust
use tpt_algo_geometry::{shapes2d::circles_overlap, Vec2};

let a = Vec2::new(0.0, 0.0);
let b = Vec2::new(3.0, 4.0);
assert_eq!(a.distance(b), 5.0);
assert!(circles_overlap(a, 2.0, b, 3.1));
```

## Testing

```sh
cargo test -p tpt-algo-geometry
```

Property tests cover the vector and shape helpers.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Licence

Licensed under either of MIT or Apache-2.0, at your option.
