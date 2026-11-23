# Changelog

All notable changes to `tpt-algo-fraud` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

### Added

- `IsolationForest` with `fit`, `score` and `score_samples`, following Liu, Ting
  and Zhou (ICDM 2008; ACM TKDD 2012). Scores lie in `[0, 1]`.
- `IsolationForestConfig` with builder methods for tree count, subsample size
  (psi) and seed.
- Seeded, private SplitMix64 random number generator, so identical seed,
  configuration and data give identical scores.
- `average_path_length` (`c(n)`) and `mean_path_length` helpers, available
  without `alloc`.
- `FraudError` for invalid configuration and input data.
- `serde` feature for `IsolationForestConfig`.
- `basic` example.
- docs.rs metadata (all features, `docsrs` cfg).
