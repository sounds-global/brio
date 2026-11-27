// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Masked markers: how a record says where a confidential value was, while
//! holding nothing of it.

use std::collections::BTreeSet;
use std::fmt;

use crate::json::{Value, json};
use crate::schema::list_schema;

/// The JSON name of the masked markers of an element or a reason.
pub(crate) const MASKED: &str = "masked";

/// The name of the definition of a masked marker in the published schema.
pub(crate) const DEFINITION: &str = "masked_marker";

/// The part of an element that a masked marker is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskedPart {
    /// The payload of the element.
    Payload,
    /// The justification of one of the reasons of the element.
    Reason,
}

impl fmt::Display for MaskedPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Payload => "its payload",
            Self::Reason => "a reason",
        })
    }
}

/// The masked markers of one JSON value, such as the payload of an element:
/// each names a place in that value where a value was masked.
///
/// A marker is beside the value and never inside it, so a phase is free to
/// put anything in a payload: nothing there is taken for a marker.
///
/// A place is a path of names and list positions from the outside in. A
/// marker is that path as a JSON Pointer (RFC 6901), which Lens writes
/// itself, so one place has one marker.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MaskedMarkers {
    /// The markers, each the one JSON Pointer of its place. No marker is
    /// inside another.
    markers: BTreeSet<String>,
    /// The first place a phase asked to mask where there was no value.
    without_a_value: Option<String>,
}

impl MaskedMarkers {
    /// Masks what `value` holds at `place`: drops it, and marks the place.
    ///
    /// What is inside a masked value is masked with it, under one marker. A
    /// place where `value` holds nothing is remembered, and the record
    /// writer refuses it.
    pub(crate) fn mask(&mut self, value: &mut Value, place: &[&str]) {
        let marker = pointer_of(place);
        if self.covers(&marker) {
            return;
        }
        match find_mut(value, place) {
            Some(masked) if !masked.is_null() => {
                *masked = Value::Null;
                self.markers.retain(|inside| !is_inside(inside, &marker));
                self.markers.insert(marker);
            }
            _ => {
                self.without_a_value.get_or_insert(marker);
            }
        }
    }

    /// Whether the value at `place` was masked, alone or with a value it is
    /// inside.
    pub(crate) fn has(&self, place: &[&str]) -> bool {
        self.covers(&pointer_of(place))
    }

    /// Whether a marker is at `pointer` or around it.
    fn covers(&self, pointer: &str) -> bool {
        self.markers
            .iter()
            .any(|marker| marker == pointer || is_inside(pointer, marker))
    }

    /// Every masked marker, in the order of their text.
    pub(crate) fn markers(&self) -> impl Iterator<Item = &str> {
        self.markers.iter().map(String::as_str)
    }

    /// A marker that names a place where no value of `value` was masked:
    /// there was nothing to mask, there is no such place, or a value is
    /// there.
    pub(crate) fn misplaced_in(&self, value: &Value) -> Option<&str> {
        let holds_null = |marker: &&str| {
            places_of(marker).is_some_and(|place| find(value, &place) == Some(&Value::Null))
        };
        self.without_a_value
            .as_deref()
            .or_else(|| self.markers().find(|marker| !holds_null(marker)))
    }

    /// These markers as they are written in a record.
    pub(crate) fn to_json(&self) -> Value {
        self.markers().collect()
    }

    /// The markers written as `markers`, or `None` when `markers` is not a
    /// list of JSON Pointers.
    pub(crate) fn from_json(markers: &Value) -> Option<Self> {
        let markers = markers
            .as_array()?
            .iter()
            .map(|marker| {
                let marker = marker.as_str()?;
                places_of(marker).map(|_| marker.to_owned())
            })
            .collect::<Option<BTreeSet<String>>>()?;
        Some(Self {
            markers,
            without_a_value: None,
        })
    }
}

/// The JSON Pointer of `place`, a path of names and list positions.
fn pointer_of(place: &[&str]) -> String {
    place
        .iter()
        .map(|step| format!("/{}", step.replace('~', "~0").replace('/', "~1")))
        .collect()
}

/// The place that the JSON Pointer `pointer` names, or `None` when
/// `pointer` is not a JSON Pointer. A place has one JSON Pointer, so this
/// reads back exactly what [`pointer_of`] writes.
fn places_of(pointer: &str) -> Option<Vec<String>> {
    if pointer.is_empty() {
        return Some(Vec::new());
    }
    pointer
        .strip_prefix('/')?
        .split('/')
        .map(|step| {
            // `~` only ever starts `~0` or `~1`.
            let escapes = step.split('~').skip(1);
            escapes
                .clone()
                .all(|escape| escape.starts_with(['0', '1']))
                .then(|| step.replace("~1", "/").replace("~0", "~"))
        })
        .collect()
}

/// Whether the JSON Pointer `pointer` names a place inside the one that
/// `around` names.
fn is_inside(pointer: &str, around: &str) -> bool {
    pointer
        .strip_prefix(around)
        .is_some_and(|rest| rest.starts_with('/'))
}

/// The position in a list that `step` names: digits, with no leading zero.
fn position(step: &str) -> Option<usize> {
    let written_once = step == "0" || !step.starts_with('0');
    (written_once && step.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| step.parse().ok())
        .flatten()
}

/// What `value` holds at `place`, or `None` when it has no such place.
fn find<'a, S: AsRef<str>>(value: &'a Value, place: &[S]) -> Option<&'a Value> {
    place.iter().try_fold(value, |value, step| match value {
        Value::Object(object) => object.get(step.as_ref()),
        Value::Array(items) => items.get(position(step.as_ref())?),
        _ => None,
    })
}

/// What `value` holds at `place`, to change it, or `None` when it has no
/// such place.
fn find_mut<'a>(value: &'a mut Value, place: &[&str]) -> Option<&'a mut Value> {
    place.iter().try_fold(value, |value, step| match value {
        Value::Object(object) => object.get_mut(*step),
        Value::Array(items) => items.get_mut(position(step)?),
        _ => None,
    })
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
    let mut schema = list_schema(
        &format!(
            "The masked markers of {what}: every place in it where a value was masked, once \
             each, in the order of their text. No marker is inside another."
        ),
        DEFINITION,
    );
    schema["uniqueItems"] = Value::Bool(true);
    schema
}
