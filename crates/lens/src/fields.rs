// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The JSON objects and lists that a record is made of.

use crate::json::{Map, Value};

/// The `fields` of a JSON object, each a name and its value.
pub(crate) fn map(fields: impl IntoIterator<Item = (&'static str, Value)>) -> Map<String, Value> {
    fields
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

/// The JSON object with `fields`, each a name and its value.
pub(crate) fn object(fields: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Object(map(fields))
}

/// The JSON list of `values`, each written by `to_json`.
pub(crate) fn list<T>(values: &[T], to_json: fn(&T) -> Value) -> Value {
    values.iter().map(to_json).collect()
}

/// The values written as `list`, each read by `from_json`, or `None` when
/// `list` is not a list of such values.
pub(crate) fn list_from<T>(list: &Value, from_json: fn(&Value) -> Option<T>) -> Option<Vec<T>> {
    list.as_array()?.iter().map(from_json).collect()
}
