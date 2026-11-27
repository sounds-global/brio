// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Masked markers: how a record says where a confidential value was, while
//! holding nothing of it.

use std::collections::BTreeSet;

use crate::json::{Value, json};
use crate::schema::list_schema;

/// The JSON name of the masked markers of an element or a reason.
pub(crate) const MASKED: &str = "masked";

/// The name of the definition of a masked marker in the published schema.
pub(crate) const DEFINITION: &str = "masked_marker";

/// The masked markers of one JSON value, such as the payload of an element:
/// each names a place in that value where a value was masked.
///
/// A marker is beside the value and never inside it, so a phase is free to
/// put anything in a payload: nothing there is taken for a marker.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MaskedMarkers(BTreeSet<String>);

impl MaskedMarkers {
    /// Masks what `value` holds at `place`: drops it, and marks the place.
    ///
    /// A place that `value` does not have is marked all the same, and the
    /// record writer refuses the marker.
    pub(crate) fn mask(&mut self, value: &mut Value, place: &str) {
        if let Some(masked) = value.pointer_mut(place) {
            *masked = Value::Null;
        }
        self.0.insert(place.to_owned());
    }

    /// Whether a value was masked at `place`.
    pub(crate) fn has(&self, place: &str) -> bool {
        self.0.contains(place)
    }

    /// Every place where a value was masked, in order.
    pub(crate) fn places(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }

    /// A marker that names a place where `value` holds no masked value: it
    /// has no such place, or holds something there.
    pub(crate) fn misplaced_in(&self, value: &Value) -> Option<&str> {
        self.places()
            .find(|place| value.pointer(place) != Some(&Value::Null))
    }

    /// These markers as they are written in a record.
    pub(crate) fn to_json(&self) -> Value {
        self.places().collect()
    }

    /// The markers written as `markers`, or `None` when `markers` is not a
    /// list of places.
    pub(crate) fn from_json(markers: &Value) -> Option<Self> {
        markers
            .as_array()?
            .iter()
            .map(|place| place.as_str().map(str::to_owned))
            .collect::<Option<BTreeSet<String>>>()
            .map(Self)
    }
}

/// The JSON Schema of a masked marker.
pub(crate) fn schema() -> Value {
    json!({
        "description": "A masked marker: the place, as a JSON Pointer (RFC 6901), where a \
                        confidential value was masked. The record holds null at that place and \
                        nothing of the value.",
        "type": "string",
        "format": "json-pointer"
    })
}

/// The JSON Schema of the masked markers of `what`, such as `the payload`.
pub(crate) fn list_schema_for(what: &str) -> Value {
    list_schema(
        &format!(
            "The masked markers of {what}: every place in it where a value was masked, in \
             order and once each."
        ),
        DEFINITION,
    )
}
