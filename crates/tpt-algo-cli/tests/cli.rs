// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end tests for the `tpt` binary: every subcommand, the demos,
//! explain, format handling, and record and replay (including detection of a
//! deliberately altered trace).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn tpt(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tpt"))
        .args(args)
        .output()
        .expect("failed to launch tpt")
}

fn code(out: &Output) -> i32 {
    out.status.code().expect("tpt was terminated by a signal")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("stderr is UTF-8")
}

/// A fresh, empty scratch directory unique to this test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tpt-cli-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

/// FNV-1a 64 in the same form the binary uses for trace digests.
fn fnv(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

const SOC_CSV: &str =
    "time_s,current_a,voltage_v\n0,1.0,4.0\n30,1.0,3.98\n60,1.0,3.97\n90,1.0,3.96\n";

const FRAUD_CSV: &str = "amount,hour,risk\n10,9,0.1\n12,10,0.2\n11,9,0.1\n13,11,0.15\n\
10.5,9,0.12\n12.5,10,0.1\n11.5,12,0.2\n900,3,0.95\n";

// ---------------------------------------------------------------- general

#[test]
fn help_lists_commands_and_exits_zero() {
    let out = tpt(&["--help"]);
    assert_eq!(code(&out), 0);
    let text = stdout(&out);
    for word in [
        "soc",
        "fraud",
        "orderbook",
        "route",
        "swarm",
        "consensus",
        "demo",
        "replay",
    ] {
        assert!(text.contains(word), "usage should mention {word}");
    }
}

#[test]
fn unknown_command_is_a_usage_error() {
    let out = tpt(&["bogus"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("unknown command"));
}

#[test]
fn unknown_option_is_a_usage_error() {
    let out = tpt(&["consensus", "--nope", "1"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("unknown option"));
}

#[test]
fn unknown_format_is_rejected() {
    let out = tpt(&["demo", "soc", "--format", "xml"]);
    assert_eq!(code(&out), 2);
}

#[test]
fn csv_format_works_for_tabular_commands() {
    // Every subcommand has a tabular view, so CSV works for all of them.
    let out = tpt(&["demo", "orderbook", "--format", "csv"]);
    assert_eq!(code(&out), 0);
    let out = tpt(&["consensus", "--format", "csv", "--ticks", "10"]);
    assert_eq!(code(&out), 0);
}

// ------------------------------------------------------------------ demos

#[test]
fn every_demo_runs_and_is_deterministic() {
    for name in ["soc", "fraud", "orderbook", "route", "swarm", "consensus"] {
        let first = tpt(&["demo", name]);
        assert_eq!(code(&first), 0, "demo {name} failed: {}", stderr(&first));
        let text = stdout(&first);
        assert!(
            text.starts_with("{\n"),
            "demo {name} should print a JSON object"
        );
        assert!(text.ends_with("}\n"));
        let second = tpt(&["demo", name]);
        assert_eq!(stdout(&second), text, "demo {name} is not deterministic");
    }
}

#[test]
fn demo_output_has_expected_shape() {
    let soc = stdout(&tpt(&["demo", "soc"]));
    assert!(soc.contains("\"samples\""));
    assert!(soc.contains("\"soc\""));
    assert!(stdout(&tpt(&["demo", "fraud"])).contains("\"scores\""));
    let book = stdout(&tpt(&["demo", "orderbook"]));
    assert!(book.contains("\"trades\"") && book.contains("\"stats\""));
    let route = stdout(&tpt(&["demo", "route"]));
    assert!(route.contains("\"vehicles_used\""));
    assert!(stdout(&tpt(&["demo", "swarm"])).contains("\"velocities\""));
    let cons = stdout(&tpt(&["demo", "consensus"]));
    assert!(cons.contains("\"prefix_consistent\": true"));
}

#[test]
fn demo_csv_has_a_header_row() {
    let cases = [
        ("soc", "time_s"),
        ("fraud", "row,score"),
        ("orderbook", "seq,price"),
        ("route", "vehicle,stops"),
        ("swarm", "agent,vx"),
        ("consensus", "replica,committed"),
    ];
    for (name, header) in cases {
        let out = tpt(&["demo", name, "--format", "csv"]);
        assert_eq!(code(&out), 0, "demo {name} csv failed: {}", stderr(&out));
        let text = stdout(&out);
        assert!(
            text.contains(header),
            "demo {name} csv should start with {header}"
        );
        assert!(
            !text.contains('{'),
            "csv output should not contain JSON braces"
        );
    }
}

#[test]
fn demo_rejects_unknown_or_missing_crate() {
    assert_eq!(code(&tpt(&["demo"])), 2);
    assert_eq!(code(&tpt(&["demo", "nothing"])), 2);
    assert_eq!(code(&tpt(&["demo", "soc", "extra"])), 2);
}

// ----------------------------------------------------------------- soc

#[test]
fn soc_reads_csv_and_emits_one_sample_per_row() {
    let dir = scratch("soc-csv");
    let input = write(&dir, "soc.csv", SOC_CSV);
    let out = tpt(&["soc", "--input", &input, "--soc0", "0.9"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert_eq!(text.matches("\"row\"").count(), 4);
}

#[test]
fn soc_accepts_json_and_matches_csv() {
    let dir = scratch("soc-json");
    let csv = write(&dir, "soc.csv", SOC_CSV);
    let json = write(
        &dir,
        "soc.json",
        r#"[{"time_s":0,"current_a":1.0,"voltage_v":4.0},
            {"time_s":30,"current_a":1.0,"voltage_v":3.98},
            {"time_s":60,"current_a":1.0,"voltage_v":3.97},
            {"time_s":90,"current_a":1.0,"voltage_v":3.96}]"#,
    );
    let from_csv = stdout(&tpt(&["soc", "--input", &csv]));
    let from_json = stdout(&tpt(&["soc", "--input", &json]));
    assert_eq!(from_csv, from_json);
}

#[test]
fn soc_csv_output_is_a_table() {
    let dir = scratch("soc-table");
    let input = write(&dir, "soc.csv", SOC_CSV);
    let out = tpt(&["soc", "--input", &input, "--format", "csv"]);
    assert_eq!(code(&out), 0);
    let text = stdout(&out);
    assert!(text.starts_with("row,time_s,soc,v1,soc_variance\n"));
    assert_eq!(text.lines().count(), 5);
}

#[test]
fn soc_missing_column_is_an_input_error() {
    let dir = scratch("soc-missing");
    let input = write(&dir, "bad.csv", "time_s,current_a\n0,1\n");
    let out = tpt(&["soc", "--input", &input]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("voltage_v"));
}

#[test]
fn soc_decreasing_time_is_rejected() {
    let dir = scratch("soc-time");
    let input = write(&dir, "t.csv", "time_s,current_a,voltage_v\n10,1,4\n5,1,4\n");
    assert_eq!(code(&tpt(&["soc", "--input", &input])), 1);
}

#[test]
fn soc_requires_input() {
    assert_eq!(code(&tpt(&["soc"])), 2);
}

// --------------------------------------------------------------- fraud

#[test]
fn fraud_score_emits_a_score_per_row() {
    let dir = scratch("fraud-score");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let out = tpt(&[
        "fraud",
        "--input",
        &input,
        "--trees",
        "50",
        "--subsample",
        "8",
        "--seed",
        "3",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out).matches("\"score\"").count(), 8);
}

#[test]
fn fraud_score_subcommand_word_is_accepted() {
    let dir = scratch("fraud-score-word");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let with_word = tpt(&["fraud", "score", "--input", &input, "--trees", "20"]);
    let without = tpt(&["fraud", "--input", &input, "--trees", "20"]);
    assert_eq!(code(&with_word), 0);
    assert_eq!(stdout(&with_word), stdout(&without));
}

#[test]
fn fraud_explain_reports_every_feature_sorted_by_magnitude() {
    let dir = scratch("fraud-explain");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let out = tpt(&[
        "fraud",
        "explain",
        "--input",
        &input,
        "--row",
        "7",
        "--trees",
        "50",
        "--subsample",
        "8",
        "--seed",
        "1",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("\"row\": 7"));
    assert_eq!(text.matches("\"feature\"").count(), 3);
    // The outlier's amount dominates the explanation, so it comes first.
    let amount = text.find("\"feature\": \"amount\"").unwrap();
    assert!(amount < text.find("\"feature\": \"hour\"").unwrap());
    assert!(amount < text.find("\"feature\": \"risk\"").unwrap());
}

#[test]
fn fraud_explain_median_and_csv() {
    let dir = scratch("fraud-explain-median");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let out = tpt(&[
        "fraud",
        "explain",
        "--input",
        &input,
        "--row",
        "7",
        "--trees",
        "50",
        "--subsample",
        "8",
        "--seed",
        "1",
        "--replace",
        "median",
        "--format",
        "csv",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("row,score,feature,value,replacement,contribution\n"));
    assert_eq!(text.lines().count(), 4);
}

#[test]
fn fraud_explain_rejects_bad_row_and_replace() {
    let dir = scratch("fraud-explain-bad");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let out = tpt(&[
        "fraud", "explain", "--input", &input, "--row", "99", "--trees", "10",
    ]);
    assert_eq!(code(&out), 2);
    let out = tpt(&["fraud", "explain", "--input", &input, "--replace", "mode"]);
    assert_eq!(code(&out), 2);
}

#[test]
fn fraud_train_file_must_match_columns() {
    let dir = scratch("fraud-train");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let train = write(&dir, "t.csv", "a,b\n1,2\n3,4\n");
    assert_eq!(
        code(&tpt(&["fraud", "--input", &input, "--train", &train])),
        1
    );
}

// ----------------------------------------------------------- orderbook

#[test]
fn orderbook_replays_events_and_reports_trades() {
    let dir = scratch("book");
    let input = write(
        &dir,
        "ev.csv",
        "op,id,side,price,qty,new_qty,new_price\nadd,1,ask,101,10,,\nadd,2,bid,101,4,,\n",
    );
    let out = tpt(&["orderbook", "--input", &input]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("\"trade_count\": 1"));
    assert!(text.contains("\"qty\": 4"));
    assert!(text.contains("\"best_ask\": 101"));
    assert!(text.contains("\"best_bid\": null"));
}

#[test]
fn orderbook_invalid_event_is_an_input_error() {
    let dir = scratch("book-bad");
    let input = write(&dir, "ev.csv", "op,id\ncancel,42\n");
    let out = tpt(&["orderbook", "--input", &input]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("event 0"));
}

#[test]
fn orderbook_rejects_unknown_op_and_side() {
    let dir = scratch("book-op");
    let input = write(&dir, "ev.csv", "op,id,side,price,qty\nhold,1,bid,1,1\n");
    assert_eq!(code(&tpt(&["orderbook", "--input", &input])), 1);
    let input = write(&dir, "ev2.csv", "op,id,side,price,qty\nadd,1,up,1,1\n");
    assert_eq!(code(&tpt(&["orderbook", "--input", &input])), 1);
}

// ------------------------------------------------------------- route

#[test]
fn route_covers_every_customer_once() {
    let out = tpt(&["demo", "route", "--format", "csv"]);
    assert_eq!(code(&out), 0);
    let mut seen: Vec<usize> = stdout(&out)
        .lines()
        .skip(1)
        .flat_map(|line| {
            let stops = line.split(',').nth(1).unwrap_or("");
            stops
                .split(';')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<usize>().unwrap())
                .collect::<Vec<_>>()
        })
        .collect();
    seen.sort_unstable();
    assert_eq!(seen, (0..8).collect::<Vec<_>>());
}

#[test]
fn route_requires_capacity_and_valid_depot() {
    let dir = scratch("route-args");
    let input = write(&dir, "c.csv", "x,y,demand\n1,1,1\n");
    assert_eq!(code(&tpt(&["route", "--input", &input])), 2);
    assert_eq!(
        code(&tpt(&[
            "route",
            "--input",
            &input,
            "--capacity",
            "5",
            "--depot",
            "0"
        ])),
        2
    );
}

#[test]
fn route_rejects_half_time_window() {
    let dir = scratch("route-window");
    let input = write(&dir, "c.csv", "x,y,demand,earliest\n1,1,1,0\n");
    let out = tpt(&["route", "--input", &input, "--capacity", "5"]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("both"));
}

#[test]
fn route_demand_over_capacity_is_an_input_error() {
    let dir = scratch("route-cap");
    let input = write(&dir, "c.csv", "x,y,demand\n1,1,9\n");
    assert_eq!(
        code(&tpt(&["route", "--input", &input, "--capacity", "5"])),
        1
    );
}

// ------------------------------------------------------------- swarm

#[test]
fn swarm_avoid_produces_bounded_velocities() {
    let out = tpt(&["demo", "swarm"]);
    assert_eq!(code(&out), 0);
    assert_eq!(stdout(&out).matches("\"agent\"").count(), 4);
    assert_eq!(stdout(&out).matches("\"speed\"").count(), 4);
}

#[test]
fn swarm_path_finds_shortest_route_around_wall() {
    let dir = scratch("grid");
    // Rows are y, columns are x. Column x=1 is a wall except in the bottom row.
    let grid = write(&dir, "g.csv", "0,1,0\n0,1,0\n0,0,0\n");
    let out = tpt(&[
        "swarm", "path", "--input", &grid, "--from", "0,0", "--to", "2,0", "--format", "csv",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("step,x,y\n"));
    // Six moves around the wall give seven cells, plus the header line.
    assert_eq!(text.lines().count(), 1 + 7);
    let json = stdout(&tpt(&[
        "swarm", "path", "--input", &grid, "--from", "0,0", "--to", "2,0",
    ]));
    assert!(json.contains("\"cost\": 6.0"));
}

#[test]
fn swarm_path_diagonal_is_shorter() {
    let dir = scratch("grid-diag");
    let grid = write(&dir, "g.csv", "0,0\n0,0\n");
    let four = stdout(&tpt(&[
        "swarm", "path", "--input", &grid, "--from", "0,0", "--to", "1,1",
    ]));
    let eight = stdout(&tpt(&[
        "swarm",
        "path",
        "--input",
        &grid,
        "--from",
        "0,0",
        "--to",
        "1,1",
        "--diagonal",
    ]));
    assert!(four.contains("\"steps\": 3"));
    assert!(eight.contains("\"steps\": 2"));
}

#[test]
fn swarm_path_unreachable_and_bad_cells_fail() {
    let dir = scratch("grid-bad");
    let blocked = write(&dir, "g.csv", "0,1,0\n");
    let out = tpt(&[
        "swarm", "path", "--input", &blocked, "--from", "0,0", "--to", "2,0",
    ]);
    assert_eq!(code(&out), 1);
    let weird = write(&dir, "w.csv", "0,x\n");
    assert_eq!(
        code(&tpt(&[
            "swarm", "path", "--input", &weird, "--from", "0,0", "--to", "1,0"
        ])),
        1
    );
    assert_eq!(
        code(&tpt(&["swarm", "path", "--input", &weird, "--from", "0,0"])),
        2
    );
    assert_eq!(code(&tpt(&["swarm"])), 2);
}

// ---------------------------------------------------------- consensus

#[test]
fn consensus_is_consistent_and_seeded() {
    let a = tpt(&["consensus", "--n", "4", "--seed", "9", "--ticks", "120"]);
    assert_eq!(code(&a), 0, "{}", stderr(&a));
    assert!(stdout(&a).contains("\"prefix_consistent\": true"));
    let b = tpt(&["consensus", "--n", "4", "--seed", "9", "--ticks", "120"]);
    assert_eq!(stdout(&a), stdout(&b));
}

#[test]
fn consensus_tolerates_one_crash() {
    let out = tpt(&[
        "consensus",
        "--n",
        "4",
        "--seed",
        "2",
        "--ticks",
        "200",
        "--crash",
        "3@10",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("\"prefix_consistent\": true"));
}

#[test]
fn consensus_bad_fault_spec_is_usage_error() {
    assert_eq!(code(&tpt(&["consensus", "--crash", "3"])), 2);
    assert_eq!(code(&tpt(&["consensus", "--equivocate", "x"])), 2);
}

// ------------------------------------------------------ record / replay

#[test]
fn record_then_replay_verifies_byte_identical_output() {
    let dir = scratch("replay-ok");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let trace = dir.join("trace.json").to_string_lossy().into_owned();
    let recorded = tpt(&[
        "fraud",
        "--input",
        &input,
        "--trees",
        "30",
        "--subsample",
        "8",
        "--seed",
        "5",
        "--record",
        &trace,
    ]);
    assert_eq!(code(&recorded), 0, "{}", stderr(&recorded));

    let text = fs::read_to_string(&trace).unwrap();
    assert!(text.contains("\"format\": \"tpt-trace\""));
    assert!(text.contains("\"version\": 1"));
    assert!(text.contains("\"digest\": \"fnv1a64:"));

    let replayed = tpt(&["replay", &trace]);
    assert_eq!(code(&replayed), 0, "{}", stderr(&replayed));
    assert!(stdout(&replayed).starts_with("replay verified: fnv1a64:"));

    // The live run and the replay agree with the file on disk being unchanged.
    let direct = tpt(&[
        "fraud",
        "--input",
        &input,
        "--trees",
        "30",
        "--subsample",
        "8",
        "--seed",
        "5",
    ]);
    assert_eq!(stdout(&direct), stdout(&recorded));
}

#[test]
fn record_and_replay_demo() {
    let dir = scratch("replay-demo");
    let trace = dir.join("demo.json").to_string_lossy().into_owned();
    assert_eq!(code(&tpt(&["demo", "route", "--record", &trace])), 0);
    let out = tpt(&["replay", &trace]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}

#[test]
fn replay_detects_altered_input() {
    let dir = scratch("replay-input");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let trace = dir.join("trace.json").to_string_lossy().into_owned();
    assert_eq!(
        code(&tpt(&[
            "fraud",
            "--input",
            &input,
            "--trees",
            "30",
            "--subsample",
            "8",
            "--seed",
            "5",
            "--record",
            &trace,
        ])),
        0
    );
    let original = fs::read_to_string(&trace).unwrap();
    // Change a value in the recorded input (escaped newlines are kept as-is).
    let tampered = original.replace("900,3,0.95", "5,3,0.95");
    assert_ne!(tampered, original, "the edit must reach the recorded input");
    fs::write(&trace, tampered).unwrap();
    let out = tpt(&["replay", &trace]);
    assert_eq!(
        code(&out),
        3,
        "stdout: {} stderr: {}",
        stdout(&out),
        stderr(&out)
    );
    assert!(stderr(&out).contains("mismatch"));
}

#[test]
fn replay_detects_altered_output_via_digest() {
    let dir = scratch("replay-digest");
    let input = write(&dir, "f.csv", FRAUD_CSV);
    let trace = dir.join("trace.json").to_string_lossy().into_owned();
    assert_eq!(
        code(&tpt(&[
            "fraud",
            "--input",
            &input,
            "--trees",
            "30",
            "--subsample",
            "8",
            "--seed",
            "5",
            "--record",
            &trace,
        ])),
        0
    );
    let original = fs::read_to_string(&trace).unwrap();
    let tampered = original.replacen("\\\"row\\\": 0,", "\\\"row\\\": 9,", 1);
    assert_ne!(tampered, original);
    fs::write(&trace, tampered).unwrap();
    let out = tpt(&["replay", &trace]);
    assert_eq!(code(&out), 3);
    assert!(stderr(&out).contains("does not match"));
}

#[test]
fn replay_detects_output_edited_with_recomputed_digest() {
    let dir = scratch("replay-recomputed");
    let trace_path = dir.join("trace.json");
    // Hand-built trace whose digest is valid for its output but whose output
    // does not match what the command produces.
    let output = "{\n  \"tampered\": true\n}\n";
    let doc = format!(
        "{{\n  \"format\": \"tpt-trace\",\n  \"version\": 1,\n  \"command\": [\"demo\", \"consensus\"],\n  \"inputs\": {{}},\n  \"output\": {},\n  \"digest\": \"{}\"\n}}\n",
        json_string(output),
        fnv(output.as_bytes())
    );
    fs::write(&trace_path, doc).unwrap();
    let out = tpt(&["replay", &trace_path.to_string_lossy()]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(stderr(&out).contains("replay mismatch"));
}

/// Encodes a string as a JSON literal for the hand-built trace above.
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[test]
fn replay_rejects_non_trace_and_missing_files() {
    let dir = scratch("replay-bad");
    let not_trace = write(&dir, "x.json", "{\"hello\": 1}");
    assert_eq!(code(&tpt(&["replay", &not_trace])), 1);
    let missing = dir.join("nope.json").to_string_lossy().into_owned();
    assert_eq!(code(&tpt(&["replay", &missing])), 1);
    assert_eq!(code(&tpt(&["replay"])), 2);
}

#[test]
fn stdin_input_is_recorded_and_replayed() {
    use std::io::Write as _;
    use std::process::Stdio;
    let dir = scratch("replay-stdin");
    let trace = dir.join("stdin.json").to_string_lossy().into_owned();
    let mut child = Command::new(env!("CARGO_BIN_EXE_tpt"))
        .args([
            "fraud",
            "--input",
            "-",
            "--trees",
            "20",
            "--subsample",
            "8",
            "--record",
            &trace,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(FRAUD_CSV.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(code(&tpt(&["replay", &trace])), 0);
}
