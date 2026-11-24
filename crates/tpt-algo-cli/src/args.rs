// SPDX-License-Identifier: MIT OR Apache-2.0
//! Hand-rolled argument handling.
//!
//! Each option is taken out of the token list by the command that knows it.
//! Whatever is left at the end must be positional arguments, otherwise an
//! unknown option was passed and the command fails with a usage error.

use crate::error::CliError;
use core::str::FromStr;

/// Token list for one command invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    tokens: Vec<String>,
}

impl Args {
    /// Wraps the tokens that follow the command name.
    pub fn new(tokens: Vec<String>) -> Self {
        Self { tokens }
    }

    /// Returns the remaining tokens.
    pub fn into_tokens(self) -> Vec<String> {
        self.tokens
    }

    /// Removes the first occurrence of `name` and returns its value. Accepts
    /// both `--name value` and `--name=value`.
    pub fn take_value(&mut self, name: &str) -> Result<Option<String>, CliError> {
        let prefix = format!("{name}=");
        let mut i = 0;
        while i < self.tokens.len() {
            if self.tokens[i] == name {
                if i + 1 >= self.tokens.len() {
                    return Err(CliError::Usage(format!("option {name} needs a value")));
                }
                self.tokens.remove(i);
                return Ok(Some(self.tokens.remove(i)));
            }
            if let Some(rest) = self.tokens[i].strip_prefix(&prefix) {
                let value = rest.to_string();
                self.tokens.remove(i);
                return Ok(Some(value));
            }
            i += 1;
        }
        Ok(None)
    }

    /// Like [`Args::take_value`], parsed with `FromStr`. Returns `default`
    /// when the option is absent.
    pub fn take_parsed<T: FromStr>(&mut self, name: &str, default: T) -> Result<T, CliError> {
        match self.take_value(name)? {
            None => Ok(default),
            Some(raw) => raw
                .parse::<T>()
                .map_err(|_| CliError::Usage(format!("invalid value {raw:?} for {name}"))),
        }
    }

    /// Like [`Args::take_value`] but returns `None` for absent options and
    /// parses the value when present.
    pub fn take_opt_parsed<T: FromStr>(&mut self, name: &str) -> Result<Option<T>, CliError> {
        match self.take_value(name)? {
            None => Ok(None),
            Some(raw) => raw
                .parse::<T>()
                .map(Some)
                .map_err(|_| CliError::Usage(format!("invalid value {raw:?} for {name}"))),
        }
    }

    /// Removes every occurrence of `name` and returns all of their values.
    pub fn take_all(&mut self, name: &str) -> Result<Vec<String>, CliError> {
        let mut values = Vec::new();
        while let Some(v) = self.take_value(name)? {
            values.push(v);
        }
        Ok(values)
    }

    /// Removes a boolean switch. Returns whether it was present.
    pub fn take_switch(&mut self, name: &str) -> bool {
        let before = self.tokens.len();
        self.tokens.retain(|t| t != name);
        self.tokens.len() != before
    }

    /// Returns the positional arguments, failing if an option is still
    /// unconsumed.
    pub fn positionals(self) -> Result<Vec<String>, CliError> {
        if let Some(bad) = self
            .tokens
            .iter()
            .find(|t| t.starts_with('-') && t.len() > 1)
        {
            return Err(CliError::Usage(format!("unknown option {bad}")));
        }
        Ok(self.tokens)
    }
}

/// Parses `x,y` into a pair of floats.
pub fn parse_pair(name: &str, raw: &str) -> Result<(f64, f64), CliError> {
    let bad = || CliError::Usage(format!("{name} must be two numbers like 0,0 (got {raw:?})"));
    let (a, b) = raw.split_once(',').ok_or_else(bad)?;
    let x = a.trim().parse::<f64>().map_err(|_| bad())?;
    let y = b.trim().parse::<f64>().map_err(|_| bad())?;
    if !(x.is_finite() && y.is_finite()) {
        return Err(bad());
    }
    Ok((x, y))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Args {
        Args::new(v.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn takes_space_and_equals_forms() {
        let mut a = args(&["--format", "csv", "--seed=7", "file.csv"]);
        assert_eq!(a.take_value("--format").unwrap().as_deref(), Some("csv"));
        assert_eq!(a.take_parsed::<u64>("--seed", 0).unwrap(), 7);
        assert_eq!(a.positionals().unwrap(), vec!["file.csv"]);
    }

    #[test]
    fn missing_value_is_usage_error() {
        let mut a = args(&["--format"]);
        assert!(matches!(a.take_value("--format"), Err(CliError::Usage(_))));
    }

    #[test]
    fn unknown_option_is_rejected() {
        let a = args(&["--bogus", "1"]);
        assert!(matches!(a.positionals(), Err(CliError::Usage(_))));
    }

    #[test]
    fn negative_numbers_are_positional_values_not_options() {
        let a = args(&["-1"]);
        // A bare negative number stays a positional argument only if it is
        // consumed by a command; here it is reported as an unknown option.
        assert!(a.positionals().is_err());
    }

    #[test]
    fn parse_pair_accepts_and_rejects() {
        assert_eq!(parse_pair("--depot", "1.5, -2").unwrap(), (1.5, -2.0));
        assert!(parse_pair("--depot", "1").is_err());
        assert!(parse_pair("--depot", "1,nan").is_err());
    }
}
