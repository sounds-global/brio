// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Canonical JSON: the one text form of everything Lens writes.
//!
//! The published schema and every record use it, so the same value always
//! gives the same bytes.

use crate::json::{Map, Value};

/// `value` as canonical JSON text.
///
/// The keys of every object are sorted, the layout is indented by two spaces,
/// and the text ends with a newline.
pub(crate) fn to_text(value: Value) -> String {
    format!("{:#}\n", with_sorted_keys(value))
}

/// `value` with the keys of every object in sorted order.
///
/// `serde_json` only sorts keys itself while its `preserve_order` feature is
/// off, and any crate in a build can turn that feature on.
fn with_sorted_keys(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<(String, Value)> = object.into_iter().collect();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            let sorted: Map<String, Value> = entries
                .into_iter()
                .map(|(key, value)| (key, with_sorted_keys(value)))
                .collect();
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(with_sorted_keys).collect()),
        scalar => scalar,
    }
}
