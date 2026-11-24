// SPDX-License-Identifier: MIT OR Apache-2.0
//! `tpt fraud`: isolation-forest anomaly scores and per-feature explanations.
//!
//! `tpt fraud [score]` scores every input row. `tpt fraud explain` explains
//! one row by perturbation: each feature is replaced by its training mean (or
//! median) and the change in score is that feature's contribution. A positive
//! contribution means the feature pushes the score up.

use super::{read_table, Report};
use crate::args::Args;
use crate::error::{algo, CliError};
use crate::json::Json;
use crate::source::Source;
use tpt_algo_fraud::{IsolationForest, IsolationForestConfig};

/// Which statistic replaces a feature during explanation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replace {
    /// Column mean of the training data.
    Mean,
    /// Column median of the training data.
    Median,
}

impl Replace {
    fn parse(raw: &str) -> Result<Self, CliError> {
        match raw {
            "mean" => Ok(Replace::Mean),
            "median" => Ok(Replace::Median),
            other => Err(CliError::Usage(format!(
                "--replace must be mean or median, got {other:?}"
            ))),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Replace::Mean => "mean",
            Replace::Median => "median",
        }
    }
}

/// Runs the subcommand. `args` holds the tokens after `fraud`.
pub fn run(args: Args, source: &mut dyn Source) -> Result<Report, CliError> {
    let mut tokens = args.into_tokens();
    let explain = match tokens.first().map(String::as_str) {
        Some("explain") => {
            tokens.remove(0);
            true
        }
        Some("score") => {
            tokens.remove(0);
            false
        }
        _ => false,
    };
    let mut args = Args::new(tokens);
    let input = args
        .take_value("--input")?
        .ok_or_else(|| CliError::Usage("fraud needs --input <file|->".to_string()))?;
    let train_path = args.take_value("--train")?;
    let trees: usize = args.take_parsed("--trees", 100)?;
    let subsample: usize = args.take_parsed("--subsample", 256)?;
    let seed: u64 = args.take_parsed("--seed", 0)?;
    let row: usize = args.take_parsed("--row", 0)?;
    let replace = match args.take_value("--replace")? {
        None => Replace::Mean,
        Some(raw) => Replace::parse(&raw)?,
    };
    if !args.positionals()?.is_empty() {
        return Err(CliError::Usage(
            "unexpected positional argument for fraud".to_string(),
        ));
    }

    let data_table = read_table(&input, source, true)?;
    let data = data_table.numeric_matrix()?;
    let train_table = match &train_path {
        Some(path) => read_table(path, source, true)?,
        None => data_table.clone(),
    };
    if train_table.columns() != data_table.columns() {
        return Err(CliError::Input(
            "--train must have the same columns as --input".to_string(),
        ));
    }
    let train = train_table.numeric_matrix()?;

    let config = IsolationForestConfig {
        n_trees: trees,
        subsample_size: subsample.min(train.len()),
        seed,
    };
    let forest = IsolationForest::fit(&config, &train).map_err(algo)?;

    if explain {
        explain_row(&forest, &data, &train, data_table.columns(), row, replace)
    } else {
        let scores = forest.score_samples(&data).map_err(algo)?;
        let rows: Vec<Json> = scores
            .iter()
            .enumerate()
            .map(|(i, s)| Json::obj([("row", i.into()), ("score", (*s).into())]))
            .collect();
        let rows = Json::Array(rows);
        Ok(Report {
            json: Json::obj([("scores", rows.clone())]),
            csv: Some(rows),
        })
    }
}

fn explain_row(
    forest: &IsolationForest,
    data: &[Vec<f64>],
    train: &[Vec<f64>],
    columns: &[String],
    row: usize,
    replace: Replace,
) -> Result<Report, CliError> {
    let x = data
        .get(row)
        .ok_or_else(|| CliError::Usage(format!("--row {row} is out of range")))?;
    let base = forest.score(x).map_err(algo)?;
    let stats: Vec<f64> = (0..columns.len())
        .map(|j| column_stat(train, j, replace))
        .collect();

    let mut contributions: Vec<(usize, f64, f64)> = Vec::with_capacity(x.len());
    for j in 0..x.len() {
        let mut perturbed = x.clone();
        perturbed[j] = stats[j];
        let score = forest.score(&perturbed).map_err(algo)?;
        contributions.push((j, base - score, stats[j]));
    }
    // Largest magnitude first; ties keep column order.
    contributions.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()).then_with(|| a.0.cmp(&b.0)));

    let entries: Vec<Json> = contributions
        .iter()
        .map(|(j, delta, replacement)| {
            Json::obj([
                ("feature", columns[*j].as_str().into()),
                ("value", x[*j].into()),
                ("replacement", (*replacement).into()),
                ("contribution", (*delta).into()),
            ])
        })
        .collect();

    let csv_rows: Vec<Json> = entries
        .iter()
        .map(|entry| {
            let mut pairs = vec![("row".to_string(), Json::from(row))];
            pairs.push(("score".to_string(), Json::Float(base)));
            if let Json::Object(fields) = entry {
                pairs.extend(fields.iter().cloned());
            }
            Json::Object(pairs)
        })
        .collect();

    Ok(Report {
        json: Json::obj([
            ("row", row.into()),
            ("score", base.into()),
            ("replace", replace.name().into()),
            ("contributions", Json::Array(entries)),
        ]),
        csv: Some(Json::Array(csv_rows)),
    })
}

fn column_stat(train: &[Vec<f64>], col: usize, replace: Replace) -> f64 {
    let mut values: Vec<f64> = train.iter().map(|r| r[col]).collect();
    match replace {
        Replace::Mean => values.iter().sum::<f64>() / values.len() as f64,
        Replace::Median => {
            values.sort_by(f64::total_cmp);
            let n = values.len();
            if n % 2 == 1 {
                values[n / 2]
            } else {
                (values[n / 2 - 1] + values[n / 2]) / 2.0
            }
        }
    }
}
