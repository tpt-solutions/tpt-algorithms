// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small helpers for reading JSON documents. The routing, order book, swarm and
//! consensus APIs take JSON, so both bindings share one parser and one set of
//! error messages.

use crate::error::{invalid_input, Result};
use serde_json::Value;

/// Parses a JSON document.
pub(crate) fn parse(text: &str) -> Result<Value> {
    serde_json::from_str(text).map_err(|e| invalid_input(format!("invalid JSON: {e}")))
}

/// Field of an object, treating `null` as absent.
pub(crate) fn field<'a>(obj: &'a Value, key: &str) -> Option<&'a Value> {
    obj.get(key).filter(|v| !v.is_null())
}

/// Required field.
pub(crate) fn required<'a>(obj: &'a Value, key: &str) -> Result<&'a Value> {
    field(obj, key).ok_or_else(|| invalid_input(format!("missing field '{key}'")))
}

/// Required finite-or-not number field (the algorithms check finiteness).
pub(crate) fn float(obj: &Value, key: &str) -> Result<f64> {
    required(obj, key)?
        .as_f64()
        .ok_or_else(|| invalid_input(format!("'{key}' must be a number")))
}

/// Optional number field with a default.
pub(crate) fn float_or(obj: &Value, key: &str, default: f64) -> Result<f64> {
    match field(obj, key) {
        None => Ok(default),
        Some(v) => v
            .as_f64()
            .ok_or_else(|| invalid_input(format!("'{key}' must be a number"))),
    }
}

/// Required unsigned integer field.
pub(crate) fn uint(obj: &Value, key: &str) -> Result<u64> {
    required(obj, key)?
        .as_u64()
        .ok_or_else(|| invalid_input(format!("'{key}' must be a non-negative integer")))
}

/// Optional unsigned integer field.
pub(crate) fn opt_uint(obj: &Value, key: &str) -> Result<Option<u64>> {
    match field(obj, key) {
        None => Ok(None),
        Some(v) => v
            .as_u64()
            .map(Some)
            .ok_or_else(|| invalid_input(format!("'{key}' must be a non-negative integer"))),
    }
}

/// Optional unsigned integer field with a default.
pub(crate) fn uint_or(obj: &Value, key: &str, default: u64) -> Result<u64> {
    Ok(opt_uint(obj, key)?.unwrap_or(default))
}

/// Optional `usize` field with a default.
pub(crate) fn size_or(obj: &Value, key: &str, default: usize) -> Result<usize> {
    let v = uint_or(obj, key, default as u64)?;
    usize::try_from(v).map_err(|_| invalid_input(format!("'{key}' is too large")))
}

/// Required signed integer field.
pub(crate) fn int(obj: &Value, key: &str) -> Result<i64> {
    required(obj, key)?
        .as_i64()
        .ok_or_else(|| invalid_input(format!("'{key}' must be an integer")))
}

/// Optional signed integer field.
pub(crate) fn opt_int(obj: &Value, key: &str) -> Result<Option<i64>> {
    match field(obj, key) {
        None => Ok(None),
        Some(v) => v
            .as_i64()
            .map(Some)
            .ok_or_else(|| invalid_input(format!("'{key}' must be an integer"))),
    }
}

/// Required string field.
pub(crate) fn string<'a>(obj: &'a Value, key: &str) -> Result<&'a str> {
    required(obj, key)?
        .as_str()
        .ok_or_else(|| invalid_input(format!("'{key}' must be a string")))
}

/// Required array field.
pub(crate) fn array<'a>(obj: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    required(obj, key)?
        .as_array()
        .ok_or_else(|| invalid_input(format!("'{key}' must be an array")))
}

/// An array of exactly two numbers, such as `[x, y]`.
pub(crate) fn pair(v: &Value, what: &str) -> Result<[f64; 2]> {
    let items = v
        .as_array()
        .filter(|a| a.len() == 2)
        .ok_or_else(|| invalid_input(format!("'{what}' must be an array of two numbers")))?;
    let x = items[0]
        .as_f64()
        .ok_or_else(|| invalid_input(format!("'{what}' must hold numbers")))?;
    let y = items[1]
        .as_f64()
        .ok_or_else(|| invalid_input(format!("'{what}' must hold numbers")))?;
    Ok([x, y])
}

/// Widens a `u128` counter for JSON. Counters beyond `u64` are written as
/// decimal strings so no value is silently truncated.
pub(crate) fn counter(n: u128) -> Value {
    match u64::try_from(n) {
        Ok(v) => Value::from(v),
        Err(_) => Value::from(n.to_string()),
    }
}
