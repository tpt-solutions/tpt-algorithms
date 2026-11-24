# Record and replay format

Written for: anyone who records a `tpt` run to keep as evidence, or who needs to check that a result still reproduces after a change.

A trace is what `tpt <command> --record <file>` writes. `tpt replay <file>` reads it and re-runs the command from the trace alone. The re-run must produce exactly the recorded bytes. This page defines the file and states what replay does and does not guarantee.

## File layout

A trace is one JSON document, pretty-printed with two-space indents. Keys are in this order:

| Key | Type | Meaning |
|---|---|---|
| `format` | string | Always `"tpt-trace"`. |
| `version` | integer | `1` for this layout. Replay rejects other values. |
| `command` | array of strings | The command line after the `tpt` name, with `--record <file>` removed. Includes the subcommand and all options, for example `["fraud", "--input", "data.csv", "--seed", "1"]`. |
| `inputs` | object | Every input text the run read, keyed by the path as it appeared in `command`. Stdin is keyed `-`. |
| `output` | string | The exact bytes the run wrote to stdout, as a JSON string. |
| `digest` | string | `fnv1a64:` followed by 16 lowercase hex digits. The FNV-1a 64-bit hash of the UTF-8 bytes of `output`. |

Example (trimmed):

```json
{
  "format": "tpt-trace",
  "version": 1,
  "command": ["fraud", "--input", "data/tx.csv", "--trees", "30", "--seed", "5"],
  "inputs": {
    "data/tx.csv": "amount,hour,risk\n10,9,0.1\n900,3,0.95\n"
  },
  "output": "{\n  \"scores\": [\n    {\n      \"row\": 0,\n ...",
  "digest": "fnv1a64:2f4e5e0c2e9ca69a"
}
```

### What is not in a trace

- No timestamps, hostnames, environment variables or working directory. A trace from one machine replays on another if the code behaves the same.
- No `--record` option. Replay does not write another trace.
- No binary version. The trace records what was run, not which build ran it. Replay checks behaviour, so a trace stays valid across builds that produce the same output.

### Inputs

Each input is stored as its full text. Files are read when the command asks for them. If a command reads the same path twice, the text from the last read is stored.

Inputs are stored in full, so a trace can be large. Keep traces of large datasets outside version control, or record a smaller slice.

## What `--record` does

1. The command runs normally. Its inputs come from files or stdin, and each read is copied into the trace.
2. If the command succeeds, its stdout is written to `<file>`, and the same bytes are printed. Stdout is unchanged by `--record`.
3. If the command fails, no trace is written, and the error is reported as usual.

## What `tpt replay` does

Replay checks in this order:

1. The file parses as JSON, has `format` equal to `"tpt-trace"` and `version` equal to `1`.
2. The digest in the file equals the digest of the recorded `output`. Failure means the output (or digest) was edited. Exit code 3.
3. The command is re-run with the recorded `command` and `inputs`. No file on disk is read. If the command needs an input that is not in the trace, replay fails with an input error (exit code 1).
4. The re-run's stdout is compared with the recorded `output`, byte for byte. Any difference is reported with its first line and column, and the exit code is 3.

On success, replay prints one line and exits 0:

```
replay verified: fnv1a64:2f4e5e0c2e9ca69a, 528 bytes identical
```

### Exit codes

| Code | Meaning |
|---|---|
| `0` | The trace reproduces exactly. |
| `1` | The file is not a readable trace, uses an unsupported version, or the command cannot run from the trace. |
| `2` | Usage error (for example, no trace path). |
| `3` | Mismatch: the digest does not match the output, or the re-run output differs. |

## What replay guarantees, and what it does not

Replay shows that, on the code that is running now, the command with these inputs produces exactly this output. It catches:

- an edited output or digest;
- an edited input, or an input that was altered after recording;
- a change in the algorithm's behaviour, after an upgrade or a code change, that alters the output.

It does not show that:

- the output is correct. It shows that it is repeatable.
- the trace came from a particular author or machine. The digest is an integrity check, not a signature. Someone who edits the output can also recompute the digest. Sign traces with an external tool if authorship matters.
- floating-point output is identical across platforms. Replay compares the text the current build prints. The algorithm crates are pure Rust, but a different target or compiler could round differently. Record and replay on the same target when byte-identical output matters.

## Why FNV-1a

FNV-1a is short, has no dependencies, and is deterministic. It detects accidental corruption and casual edits. It is not a cryptographic hash, so do not rely on it against a deliberate forger.

## Compatibility

- `version` changes only when the layout changes incompatibly. Adding a key that older readers can ignore does not change the version.
- A trace written by one release of `tpt` is expected to replay on a later release if the command's output has not changed. A change in output is reported as a mismatch, and the trace is not rewritten.
