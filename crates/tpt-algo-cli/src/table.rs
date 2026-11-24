// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tabular input (CSV or JSON) and tabular output (CSV).
//!
//! Input is read into a [`Table`] of strings. Typed access happens through
//! the accessors, which report the row and column on failure. JSON numbers
//! keep their textual form so that the same parsing path serves both formats.

use crate::error::CliError;
use crate::json::{self, format_float, Json};

/// A rectangular table of cell text with named columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    /// Parses input text, detecting JSON by a leading `[` or `{`, and CSV
    /// otherwise. CSV without a header is allowed when `has_header` is false;
    /// its columns are then named `c0`, `c1`, and so on. JSON input is always
    /// headerless when it is an array of arrays.
    pub fn parse(text: &str, has_header: bool) -> Result<Table, CliError> {
        let trimmed = text.trim_start();
        if trimmed.starts_with('[') || trimmed.starts_with('{') {
            Self::from_json(&json::parse(text)?)
        } else {
            Self::from_csv(text, has_header)
        }
    }

    /// Builds a table from CSV text.
    pub fn from_csv(text: &str, has_header: bool) -> Result<Table, CliError> {
        let records = parse_csv_records(text)?;
        let mut iter = records.into_iter();
        let columns = if has_header {
            match iter.next() {
                Some(header) => header,
                None => return Err(CliError::Input("CSV input is empty".to_string())),
            }
        } else {
            Vec::new()
        };
        let rows: Vec<Vec<String>> = iter.collect();
        let width = if has_header {
            columns.len()
        } else {
            rows.first().map_or(0, Vec::len)
        };
        let columns = if has_header {
            columns
        } else {
            (0..width).map(|i| format!("c{i}")).collect()
        };
        for (i, row) in rows.iter().enumerate() {
            if row.len() != width {
                return Err(CliError::Input(format!(
                    "CSV row {} has {} fields, expected {width}",
                    i + 1,
                    row.len()
                )));
            }
        }
        if columns.is_empty() {
            return Err(CliError::Input("CSV input has no columns".to_string()));
        }
        Ok(Table { columns, rows })
    }

    /// Builds a table from a JSON array of objects (columns are the union of
    /// keys in first-seen order) or a JSON array of arrays.
    pub fn from_json(value: &Json) -> Result<Table, CliError> {
        let items = value
            .as_array()
            .ok_or_else(|| CliError::Input("JSON input must be an array".to_string()))?;
        if items.is_empty() {
            return Err(CliError::Input("JSON input is empty".to_string()));
        }
        if items.iter().all(|v| matches!(v, Json::Array(_))) {
            let rows: Vec<Vec<String>> = items
                .iter()
                .enumerate()
                .map(|(i, v)| cells_of(v.as_array().unwrap_or(&[]), i))
                .collect::<Result<_, _>>()?;
            let width = rows[0].len();
            if rows.iter().any(|r| r.len() != width) {
                return Err(CliError::Input(
                    "JSON arrays have unequal lengths".to_string(),
                ));
            }
            let columns = (0..width).map(|i| format!("c{i}")).collect();
            return Ok(Table { columns, rows });
        }
        let mut columns: Vec<String> = Vec::new();
        for item in items {
            let Json::Object(pairs) = item else {
                return Err(CliError::Input(
                    "JSON input must be an array of objects or of arrays".to_string(),
                ));
            };
            for (k, _) in pairs {
                if !columns.contains(k) {
                    columns.push(k.clone());
                }
            }
        }
        let mut rows = Vec::with_capacity(items.len());
        for (i, item) in items.iter().enumerate() {
            let mut row = Vec::with_capacity(columns.len());
            for col in &columns {
                let cell = match item.get(col) {
                    None | Some(Json::Null) => String::new(),
                    Some(Json::Bool(b)) => b.to_string(),
                    Some(Json::Int(n)) => n.to_string(),
                    Some(Json::Float(f)) => format_float(*f),
                    Some(Json::Str(s)) => s.clone(),
                    Some(_) => {
                        return Err(CliError::Input(format!(
                            "JSON object {i} field {col:?} is nested; only scalars are allowed"
                        )))
                    }
                };
                row.push(cell);
            }
            rows.push(row);
        }
        Ok(Table { columns, rows })
    }

    /// Number of data rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Column names.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Index of a named column, if present.
    pub fn column(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c == name)
    }

    /// Raw cell text, or `None` when the column is absent or the cell is empty.
    pub fn text(&self, row: usize, name: &str) -> Option<&str> {
        let col = self.column(name)?;
        let s = self.rows.get(row)?.get(col)?.trim();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// Returns the column index or a usage-style input error naming it.
    pub fn require(&self, name: &str) -> Result<usize, CliError> {
        self.column(name)
            .ok_or_else(|| CliError::Input(format!("missing required column {name:?}")))
    }

    /// Required finite number in `row` under column `name`.
    pub fn num(&self, row: usize, name: &str) -> Result<f64, CliError> {
        self.require(name)?;
        self.num_opt(row, name)?
            .ok_or_else(|| CliError::Input(format!("row {}: column {name:?} is empty", row + 1)))
    }

    /// Optional finite number: `None` when the column is absent or the cell is empty.
    pub fn num_opt(&self, row: usize, name: &str) -> Result<Option<f64>, CliError> {
        let Some(raw) = self.text(row, name) else {
            return Ok(None);
        };
        let v: f64 = raw.parse().map_err(|_| {
            CliError::Input(format!(
                "row {}: column {name:?} is not a number: {raw:?}",
                row + 1
            ))
        })?;
        if !v.is_finite() {
            return Err(CliError::Input(format!(
                "row {}: column {name:?} is not finite",
                row + 1
            )));
        }
        Ok(Some(v))
    }

    /// Every cell as a finite number, row by row.
    pub fn numeric_matrix(&self) -> Result<Vec<Vec<f64>>, CliError> {
        (0..self.len())
            .map(|r| {
                self.columns
                    .iter()
                    .map(|c| self.num(r, c))
                    .collect::<Result<Vec<f64>, _>>()
            })
            .collect()
    }
}

fn cells_of(items: &[Json], row: usize) -> Result<Vec<String>, CliError> {
    items
        .iter()
        .map(|v| match v {
            Json::Null => Ok(String::new()),
            Json::Bool(b) => Ok(b.to_string()),
            Json::Int(n) => Ok(n.to_string()),
            Json::Float(f) => Ok(format_float(*f)),
            Json::Str(s) => Ok(s.clone()),
            _ => Err(CliError::Input(format!(
                "JSON array row {row} holds a nested value"
            ))),
        })
        .collect()
}

/// Splits CSV text into records. Supports quoted fields with `""` escapes,
/// CRLF or LF line endings, and skips blank lines.
fn parse_csv_records(text: &str) -> Result<Vec<Vec<String>>, CliError> {
    let mut records = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();
    let mut line_has_content = false;

    let finish_record =
        |record: &mut Vec<String>, field: &mut String, records: &mut Vec<Vec<String>>| {
            record.push(core::mem::take(field).trim().to_string());
            if !(record.len() == 1 && record[0].is_empty()) {
                records.push(core::mem::take(record));
            } else {
                record.clear();
            }
        };

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                c => field.push(c),
            }
            continue;
        }
        match c {
            '"' if field.trim().is_empty() => {
                field.clear();
                in_quotes = true;
                line_has_content = true;
            }
            ',' => {
                record.push(core::mem::take(&mut field).trim().to_string());
                line_has_content = true;
            }
            '\r' => {}
            '\n' => {
                finish_record(&mut record, &mut field, &mut records);
                line_has_content = false;
            }
            c => {
                field.push(c);
                line_has_content = true;
            }
        }
    }
    if in_quotes {
        return Err(CliError::Input(
            "CSV has an unterminated quoted field".to_string(),
        ));
    }
    if line_has_content || !record.is_empty() || !field.is_empty() {
        finish_record(&mut record, &mut field, &mut records);
    }
    Ok(records)
}

/// Renders a JSON value as CSV. The value must be an array of flat objects
/// (scalar values only), or a single flat object. Columns are the union of
/// keys in first-seen order. Values that are not scalars are rejected.
pub fn json_to_csv(value: &Json) -> Result<String, CliError> {
    let rows: Vec<&Json> = match value {
        Json::Array(items) => items.iter().collect(),
        Json::Object(_) => vec![value],
        _ => {
            return Err(CliError::Input(
                "CSV output needs an array of objects".to_string(),
            ))
        }
    };
    let mut columns: Vec<String> = Vec::new();
    for row in &rows {
        let Json::Object(pairs) = row else {
            return Err(CliError::Input(
                "CSV output needs an array of objects".to_string(),
            ));
        };
        for (k, _) in pairs {
            if !columns.contains(k) {
                columns.push(k.clone());
            }
        }
    }
    if columns.is_empty() {
        return Ok(String::new());
    }
    let mut out = String::new();
    out.push_str(
        &columns
            .iter()
            .map(|c| escape_csv(c))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');
    for row in &rows {
        let mut cells = Vec::with_capacity(columns.len());
        for col in &columns {
            let cell = match row.get(col) {
                None | Some(Json::Null) => String::new(),
                Some(Json::Bool(b)) => b.to_string(),
                Some(Json::Int(n)) => n.to_string(),
                Some(Json::Float(f)) => format_float(*f),
                Some(Json::Str(s)) => s.clone(),
                Some(_) => {
                    return Err(CliError::Input(
                        "CSV output cannot hold nested values; use --format json".to_string(),
                    ))
                }
            };
            cells.push(escape_csv(&cell));
        }
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    Ok(out)
}

fn escape_csv(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn csv_with_header_quotes_and_blank_lines() {
        let t = Table::from_csv("a,b\r\n1,\"x,\"\"y\"\"\"\n\n2,\n", true).unwrap();
        assert_eq!(t.columns(), ["a", "b"]);
        assert_eq!(t.len(), 2);
        assert_eq!(t.text(0, "b"), Some("x,\"y\""));
        assert_eq!(t.num_opt(1, "b").unwrap(), None);
        assert_eq!(t.num(1, "a").unwrap(), 2.0);
    }

    #[test]
    fn csv_ragged_rows_are_rejected() {
        assert!(Table::from_csv("a,b\n1\n", true).is_err());
        assert!(Table::from_csv("a,b\n\"open", true).is_err());
    }

    #[test]
    fn headerless_csv_names_columns() {
        let t = Table::from_csv("0,1\n1,0\n", false).unwrap();
        assert_eq!(t.columns(), ["c0", "c1"]);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn json_objects_union_columns() {
        let t = Table::parse(r#"[{"a": 1, "b": "x"}, {"a": 2.5, "c": true}]"#, true).unwrap();
        assert_eq!(t.columns(), ["a", "b", "c"]);
        assert_eq!(t.num(1, "a").unwrap(), 2.5);
        assert_eq!(t.text(1, "b"), None);
    }

    #[test]
    fn json_nested_values_are_rejected() {
        assert!(Table::parse(r#"[{"a": [1]}]"#, true).is_err());
    }

    #[test]
    fn non_finite_numbers_are_rejected() {
        let t = Table::from_csv("a\ninf\n", true).unwrap();
        assert!(t.num(0, "a").is_err());
    }

    #[test]
    fn csv_output_flattens_objects() {
        let v = Json::Array(vec![
            Json::obj([("x", Json::Int(1)), ("y", Json::from("a,b"))]),
            Json::obj([("x", Json::Float(2.0)), ("z", Json::Bool(false))]),
        ]);
        assert_eq!(json_to_csv(&v).unwrap(), "x,y,z\n1,\"a,b\",\n2.0,,false\n");
        assert!(json_to_csv(&Json::Array(vec![Json::Array(vec![])])).is_err());
    }
}
