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

/// A masked marker: the place in a payload, or in the justification of a
/// reason, where a confidential value was masked.
///
/// A place is a path of names and list positions from the outside in. A
/// marker is that path as a JSON Pointer (RFC 6901), which Lens writes
/// itself, so one place has one marker.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaskedMarker(String);

impl MaskedMarker {
    /// The marker of `place`, a path of names and list positions.
    fn of(place: &[&str]) -> Self {
        Self(
            place
                .iter()
                .map(|step| format!("/{}", step.replace('~', "~0").replace('/', "~1")))
                .collect(),
        )
    }

    /// The marker written as `pointer`, or `None` when `pointer` is not a
    /// JSON Pointer. A place has one JSON Pointer, so this reads back
    /// exactly what a marker is written as.
    fn from_pointer(pointer: &str) -> Option<Self> {
        // Every step starts with `/`, and `~` only ever starts `~0` or `~1`.
        let escapes = || pointer.split('~').skip(1);
        let is_a_pointer = (pointer.is_empty() || pointer.starts_with('/'))
            && escapes().all(|escape| escape.starts_with(['0', '1']));
        is_a_pointer.then(|| Self(pointer.to_owned()))
    }

    /// This marker as a JSON Pointer (RFC 6901), as it is written in a
    /// record.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this marker is at the place of `around`, or inside it.
    fn is_within(&self, around: &Self) -> bool {
        self.0
            .strip_prefix(around.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    }

    /// What `value` holds at the place of this marker, or `None` when it
    /// has no such place.
    fn find_in<'a>(&self, value: &'a Value) -> Option<&'a Value> {
        value.pointer(&self.0)
    }
}

impl fmt::Display for MaskedMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<str> for MaskedMarker {
    fn eq(&self, pointer: &str) -> bool {
        self.0 == pointer
    }
}

/// The masked markers of one JSON value, such as the payload of an element.
///
/// A marker is beside the value and never inside it, so a phase is free to
/// put anything in a payload: nothing there is taken for a marker. No
/// marker is inside another.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MaskedMarkers(BTreeSet<MaskedMarker>);

impl MaskedMarkers {
    /// Masks what `value` holds at `place`: drops it, and marks the place.
    /// What is inside a masked value is masked with it, under one marker.
    ///
    /// # Errors
    ///
    /// Gives the marker of `place` when `value` holds nothing there: no
    /// value was masked, and nothing is marked.
    pub(crate) fn mask(&mut self, value: &mut Value, place: &[&str]) -> Result<(), MaskedMarker> {
        let marker = MaskedMarker::of(place);
        if self.covers(&marker) {
            return Ok(());
        }
        match value.pointer_mut(marker.as_str()) {
            Some(masked) if !masked.is_null() => {
                *masked = Value::Null;
                self.mark(marker);
                Ok(())
            }
            _ => Err(marker),
        }
    }

    /// Marks `place`, where the value that `value` held was masked before:
    /// `value` holds nothing there.
    ///
    /// # Errors
    ///
    /// Gives the marker of `place` when `value` does not hold null there,
    /// and nothing is marked.
    pub(crate) fn keep(&mut self, value: &Value, place: &[&str]) -> Result<(), MaskedMarker> {
        let marker = MaskedMarker::of(place);
        if self.covers(&marker) {
            return Ok(());
        }
        if marker.find_in(value) == Some(&Value::Null) {
            self.mark(marker);
            Ok(())
        } else {
            Err(marker)
        }
    }

    /// Adds `marker`, which takes the place of every marker inside it.
    fn mark(&mut self, marker: MaskedMarker) {
        self.0.retain(|inside| !inside.is_within(&marker));
        self.0.insert(marker);
    }

    /// Whether the value at `place` was masked, alone or with a value it is
    /// inside.
    pub(crate) fn has(&self, place: &[&str]) -> bool {
        self.covers(&MaskedMarker::of(place))
    }

    /// Whether the place of `marker` is masked, alone or with a value it is
    /// inside.
    fn covers(&self, marker: &MaskedMarker) -> bool {
        self.0.iter().any(|around| marker.is_within(around))
    }

    /// Every masked marker, in the order of their text.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &MaskedMarker> {
        self.0.iter()
    }

    /// A marker that names a place where no value of `value` was masked.
    ///
    /// It is `mismarked`, a marker a phase asked for and was refused, unless
    /// a value around it was masked since. Otherwise it is one of these
    /// markers at a place that `value` does not have, or where it holds a
    /// value.
    pub(crate) fn misplaced_in<'a>(
        &'a self,
        value: &Value,
        mismarked: Option<&'a MaskedMarker>,
    ) -> Option<&'a MaskedMarker> {
        mismarked.filter(|marker| !self.covers(marker)).or_else(|| {
            self.iter()
                .find(|marker| marker.find_in(value) != Some(&Value::Null))
        })
    }

    /// These markers as they are written in a record.
    pub(crate) fn to_json(&self) -> Value {
        self.iter().map(MaskedMarker::as_str).collect()
    }

    /// The markers written as `markers`, or `None` when `markers` is not a
    /// list of JSON Pointers.
    pub(crate) fn from_json(markers: &Value) -> Option<Self> {
        markers
            .as_array()?
            .iter()
            .map(|marker| MaskedMarker::from_pointer(marker.as_str()?))
            .collect::<Option<BTreeSet<MaskedMarker>>>()
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
