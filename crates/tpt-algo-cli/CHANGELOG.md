# Changelog

All notable changes to `tpt-algo-cli` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the version is `0.x`, minor releases may contain breaking changes.

## [Unreleased]

### Added

- `tpt` binary with subcommands `soc`, `fraud` (with `explain`), `orderbook`,
  `route`, `swarm` (`avoid` and `path`) and `consensus`.
- `--format json|csv` on every subcommand, and CSV or JSON input detected from
  the content.
- `tpt demo <crate>` running each crate on bundled synthetic data.
- `--record <trace.json>` and `tpt replay <trace.json>` with byte-identical
  verification and a digest. Format in `docs/record-format.md`.
- Hand-written JSON and CSV handling, with no third-party dependencies.
- Integration tests for each subcommand, the demos, explain, and record and
  replay, including detection of an edited trace.
