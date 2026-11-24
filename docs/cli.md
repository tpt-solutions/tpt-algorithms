# The `tpt` command-line tool

Written for: people who want to run a tpt-algorithms crate from a shell, a script or a pipeline without writing Rust.

The `tpt` binary lives in `crates/tpt-algo-cli` (package `tpt-algo-cli`). It wraps the six algorithm crates behind one interface: CSV or JSON in, JSON or CSV out, and a record and replay facility that checks a run reproduces byte for byte.

```sh
cargo run -p tpt-algo-cli -- demo soc
cargo install --path crates/tpt-algo-cli   # installs `tpt` (not yet published)
```

## Quick start

```sh
tpt demo fraud                       # run a crate on bundled data
tpt demo route --format csv          # same, as CSV
tpt fraud --input transactions.csv --trees 200 --seed 1
tpt fraud explain --input transactions.csv --row 38
tpt replay trace.json                # verify a recorded run
```

## Global conventions

| Item | Behaviour |
|---|---|
| Input | CSV with a header row, or a JSON array of objects. The format is detected from the first character (`[` or `{` means JSON). |
| Stdin | Pass `-` as the input path to read stdin. |
| Output | Stdout only. JSON is pretty-printed with two-space indents and a trailing newline. |
| `--format json\|csv` | Default `json`. CSV is available for every subcommand and gives the tabular view of the result. |
| `--record <trace.json>` | Writes a trace of the run. See [record-format.md](record-format.md). |
| Numbers | Input values must be finite. `NaN` and infinities are rejected with an input error. |
| Option syntax | `--name value` or `--name=value`. Unknown options are a usage error. |
| Help | `tpt --help`, or `--help` after any command. |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Input could not be read or parsed, or an algorithm rejected its input. |
| `2` | Usage error: unknown command, option or value, or a missing required option. |
| `3` | `tpt replay` found a trace that does not reproduce (edited, or the code changed behaviour). |

Errors are printed to stderr as `tpt: <message>`.

## Input shapes

JSON input is an array of objects. Columns are the union of keys in first-seen order. Missing keys and `null` become empty cells. Nested arrays and objects inside a row are rejected.

```json
[{"time_s": 0, "current_a": 1.0, "voltage_v": 4.0}, {"time_s": 30, "current_a": 1.0, "voltage_v": 3.98}]
```

CSV input uses RFC 4180 quoting (`""` escapes a quote). Blank lines are skipped. Cells are trimmed, and an empty cell counts as absent where the column is optional.

Headerless grid input (used by `swarm path`) is a CSV with no header line, or a JSON array of arrays.

## Subcommands

### `tpt soc`: battery state of charge

Runs the extended Kalman filter from `tpt-algo-soc` over a discharge or charge profile.

Input columns: `time_s` (seconds, non-decreasing), `current_a` (positive on discharge), `voltage_v` (terminal voltage).

| Option | Default | Meaning |
|---|---|---|
| `--capacity-ah` | `2.0` | Cell capacity. |
| `--r0`, `--r1`, `--c1` | `0.05`, `0.02`, `1000` | Series resistance, RC-pair resistance and capacitance. |
| `--soc0` | `0.5` | Initial state of charge (0 to 1). |
| `--soc-variance`, `--v1-variance` | `0.1`, `1e-4` | Initial covariance. |
| `--process-soc`, `--process-v1`, `--measurement` | `1e-9`, `1e-6`, `1e-4` | Noise levels. |

The open-circuit-voltage curve is fixed at 11 points from 0.0 to 1.0 SoC (3.00 V to 4.20 V). A custom curve is not yet supported.

Output: `samples`, one object per input row with `row`, `time_s`, `soc`, `v1` and `soc_variance`.

```sh
tpt soc --input discharge.csv --soc0 0.95 --capacity-ah 3.0
```

### `tpt fraud`: isolation-forest anomaly scores

Fits an isolation forest and scores each row. Higher scores are more anomalous.

| Option | Default | Meaning |
|---|---|---|
| `--input` | required | Numeric columns only. Every column is a feature. |
| `--train` | the input | Training rows with the same columns. Without it, the forest is fit on the input itself. |
| `--trees` | `100` | Number of trees. |
| `--subsample` | `256` | Subsample size per tree, capped at the number of training rows. |
| `--seed` | `0` | Seed. The same seed and data always give the same scores. |

Output: `scores`, one `{row, score}` per input row (row numbers are 0-based).

### `tpt fraud explain`: why a row scored high

Explains one row by perturbation. Each feature in turn is replaced by its training mean (or median) and the score is recomputed. The contribution of a feature is the original score minus the perturbed score, so a positive value means the feature pushed the score up.

Takes the same options as `tpt fraud`, plus:

| Option | Default | Meaning |
|---|---|---|
| `--row` | `0` | 0-based row to explain. |
| `--replace` | `mean` | `mean` or `median`. |

Output: `row`, `score`, `replace`, and `contributions`, sorted by absolute contribution (largest first, ties in column order). Each entry has `feature`, `value`, `replacement` and `contribution`.

```sh
tpt fraud explain --input transactions.csv --row 38 --replace median
```

The contributions depend on the forest, so change `--seed` or `--trees` and the numbers move. Compare rows under the same settings.

### `tpt orderbook`: limit-order book replay

Applies a sequence of order events to an empty book.

Input columns: `op`, `id`, `side`, `price`, `qty`, `new_qty`, `new_price`. Only the columns an operation uses are read.

| `op` | Columns used |
|---|---|
| `add` | `id`, `side`, `price`, `qty` |
| `cancel` | `id` |
| `modify` | `id`, and at least one of `new_qty` or `new_price` |
| `execute` | `id`, `side`, `qty` |
| `execute_order` | `id`, `qty` |

`side` is `bid` or `ask` (`buy` and `sell` are also accepted). `price` is an integer number of ticks.

Output: `events`, `trade_count`, `best_bid`, `best_ask`, `spread`, `mid`, `resting_quantity`, `order_count`, `sequence`, `stats` and `trades`. CSV gives the trades.

An event that the book rejects stops the run and reports its zero-based index, exit code 1.

### `tpt route`: capacitated vehicle routing

Builds routes from a depot with the Clarke and Wright savings heuristic followed by local search (`tpt-algo-routing`).

Input columns: `x`, `y`, `demand` (required), `earliest` and `latest` (both or neither, for a time window), and `service_time` (optional, default 0). Each row is one customer.

| Option | Default | Meaning |
|---|---|---|
| `--capacity` | required | Vehicle capacity. |
| `--depot` | `0,0` | Depot `x,y`. |
| `--max-vehicles` | none | Upper bound on the number of vehicles. |
| `--depot-due` | none | Latest time the depot may be returned to. |
| `--iterations` | `100000` | Local-search iteration limit. |

Output: `cost`, `vehicles_used`, and `routes`, a list of routes. Each route is a list of 0-based row numbers from the input, in visiting order. CSV gives one row per route with the stops joined by `;`.

### `tpt swarm`: avoidance and grid paths

`tpt swarm avoid` computes a collision-avoiding velocity for every agent against all the others, using velocity obstacles.

Input columns: `x`, `y`, `vx`, `vy`, `radius`, `pref_vx`, `pref_vy`, `max_speed`.

| Option | Default | Meaning |
|---|---|---|
| `--horizon` | `4.0` | Look-ahead time in seconds. |
| `--margin` | `0.05` | Extra clearance between agents. |
| `--responsibility` | `reciprocal` | `reciprocal` (each agent takes half the avoidance) or `full`. |

Output: `velocities`, one `{agent, vx, vy, speed}` per agent.

`tpt swarm path` finds a minimum-cost path on a grid with A* search.

```sh
tpt swarm path --input grid.csv --from 0,0 --to 9,4 --diagonal
```

The grid is headerless. Each line is a row of cells, and each cell is `0`, `.` or `false` for free, or `1`, `#` or `true` for blocked. The first column is x, and the first line is y = 0. `--from` and `--to` are `x,y`. `--diagonal` allows 8-way moves (diagonal cost 1.4); the default is 4-way.

Output: `cost`, `steps` and `cells` (a list of `[x, y]` pairs from start to goal). CSV gives `step`, `x`, `y`.

### `tpt consensus`: seeded consensus simulation

Runs the deterministic consensus simulator. There is no input file: the run is fully determined by its options and seed. This is the one subcommand that does not read CSV or JSON.

| Option | Default | Meaning |
|---|---|---|
| `--n` | `4` | Number of replicas. |
| `--seed` | `0` | Simulator seed. |
| `--ticks` | `200` | Number of simulated ticks. |
| `--gst` | `0` | Global stabilisation tick: messages are reliable from this tick on. |
| `--drop-per-mille` | `0` | Drop rate before stabilisation, in parts per thousand. |
| `--crash ID@TICK` | none | Crash replica `ID` at `TICK`. Repeatable. |
| `--equivocate ID` | none | Make replica `ID` equivocate. Repeatable. |

Output: `n`, `seed`, `ticks`, `now`, `max_honest_view`, `stats` (delivered, dropped, equivocations, rejected messages, safety violations), `prefix_consistent` (whether every honest replica's committed chain is a prefix of the longest), `consistency_error`, and `replicas` with each replica's committed block ids.

A run where `prefix_consistent` is `false` is a finding, not a crash: the command still exits 0 and reports it.

### `tpt demo <crate>`

Runs one of `soc`, `fraud`, `orderbook`, `route`, `swarm` or `consensus` on data bundled in the binary. The output is the same on every run and every machine.

```sh
tpt demo orderbook
tpt demo swarm --format csv
```

The bundled data lives in `crates/tpt-algo-cli/data/demo/`. The demo runs go through the same code path as a normal run, so a demo can be recorded and replayed like any other.

### `tpt replay <trace.json>`

Re-runs a recorded command from the trace alone and checks the output is byte-identical to the recorded output. On success it prints `replay verified: <digest>, <n> bytes identical` and exits 0. On mismatch it names the first differing line and column and exits 3.

See [record-format.md](record-format.md) for the file format and what replay checks.

## Recording a run

Add `--record <file>` to any command:

```sh
tpt fraud --input transactions.csv --seed 1 --record run.json > scores.json
tpt replay run.json
```

The trace holds the command line, every input the run read (file contents, or stdin under the key `-`), the exact stdout, and its digest. Replay does not need the original files.

## Known limits

- `tpt soc` uses a fixed OCV curve. A custom curve needs a code change.
- `tpt consensus` takes options only. A config file is not supported yet.
- Input is read whole into memory. Very large files are not streamed.
- `tpt` is not yet published. Its dependencies are path dependencies on the workspace crates.
- The browser demo (WASM) is not part of this tool yet. It depends on the Phase 20 bindings.
