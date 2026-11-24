# tpt-algo-cli

The `tpt` command-line tool. It runs the tpt-algorithms crates on CSV or JSON files and prints JSON or CSV. It can record a run to a trace and replay the trace to check the output is byte-identical.

Pre-release (`0.0.0`). Not yet published.

## Subcommands

| Command | Crate | Purpose |
|---|---|---|
| `tpt soc` | tpt-algo-soc | Battery state of charge (EKF) |
| `tpt fraud`, `tpt fraud explain` | tpt-algo-fraud | Isolation-forest scores; per-feature contributions |
| `tpt orderbook` | tpt-algo-orderbook | Limit-order event replay |
| `tpt route` | tpt-algo-routing | Capacitated vehicle routing |
| `tpt swarm avoid`, `tpt swarm path` | tpt-algo-swarm | Velocity-obstacle avoidance; grid A* |
| `tpt consensus` | tpt-algo-consensus | Seeded consensus simulation |
| `tpt demo <crate>` | all of the above | Run on bundled data |
| `tpt replay <trace.json>` | | Verify a recorded run |

## Usage

```sh
cargo run -p tpt-algo-cli -- demo fraud
cargo run -p tpt-algo-cli -- fraud --input transactions.csv --seed 1 --format csv
cargo run -p tpt-algo-cli -- fraud --input transactions.csv --record run.json
cargo run -p tpt-algo-cli -- replay run.json
```

Full options, input shapes and exit codes are in [docs/cli.md](../../docs/cli.md). The trace format is in [docs/record-format.md](../../docs/record-format.md).

## Dependencies

No third-party crates. JSON and CSV handling is hand-written and covered by tests. The only dependencies are the six algorithm crates, by path.

## Limits

- `tpt soc` uses a fixed open-circuit-voltage curve.
- `tpt consensus` takes options only, not an input file.
- Input is read whole into memory.
- The browser demo (WASM) is not included yet. It depends on the Phase 20 bindings.

## Licence

MIT OR Apache-2.0.
