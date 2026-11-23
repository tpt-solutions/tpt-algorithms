# Changelog

All notable changes to `tpt-algo-geometry` are documented in this file.

This crate is internal (`publish = false`) and is never released, so entries
describe workspace changes only.

## [Unreleased]

### Added

- `vec`: `Vec2` and `Vec3` with `dot`, `cross`, `norm`, `distance`, `normalized`
  and `is_finite`.
- `shapes2d`: `Aabb2`, `segment_intersection`, `point_segment_distance` and
  `circles_overlap`.
- `kinematics`: `constant_accel_position` and `time_to_collision`.
- `error`: `GeometryError`.
- Property tests for vector and shape helpers.
