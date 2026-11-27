// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Masked values: what a record holds in place of a confidential value.

use crate::json::{Value, json};
use crate::schema::object_schema;

/// The JSON name that makes an object a masked marker. No other object of a
/// record has it.
pub(crate) const MASKED: &str = "$masked";

/// The name of the definition of a masked marker in the published schema.
pub(crate) const DEFINITION: &str = "masked";

/// The masked marker that a record holds in place of `value`.
///
/// A phase calls this for every value of a `Secret` or `Sensitive` type, in
/// the payload of an element or in a reason. Nothing of `value` is read: the
/// marker holds nothing of it, so its content is in no record.
#[must_use]
pub fn masked<T: ?Sized>(value: &T) -> Value {
    let _ = value;
    json!({ MASKED: true })
}

/// Whether `value` is a masked marker: it stands in a record for a value
/// that was masked with [`masked`].
#[must_use]
pub fn is_masked(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.len() == 1 && object.get(MASKED) == Some(&Value::Bool(true)))
}

/// Whether `value` has, at any depth, an object that is named as a masked
/// marker and is not one: it could hold what a marker stands for.
pub(crate) fn has_malformed_marker(value: &Value) -> bool {
    match value {
        Value::Object(object) if object.contains_key(MASKED) => !is_masked(value),
        Value::Object(object) => object.values().any(has_malformed_marker),
        Value::Array(items) => items.iter().any(has_malformed_marker),
        _ => false,
    }
}

/// The JSON Schema of a masked marker.
pub(crate) fn schema() -> Value {
    object_schema(
        "A masked marker: it stands for a confidential value, and holds nothing of it. No \
         other object of a record has a field of this name.",
        &[(MASKED, json!({ "const": true }))],
    )
}
