// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Stable identifiers of the elements of a record.

use std::fmt;

use crate::hash::{self, FieldHasher};
use crate::json::Value;
use crate::payload::{self, InvalidPayloadKind, Payload};

/// The stable identifier of one element of a stage record.
///
/// It comes from the element's identity alone: its module, the stage that
/// produced it and its place in that stage's structure. The same element has
/// the same identifier in every build, whatever else changes in the input.
///
/// It is written as 32 lowercase hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableId(String);

impl StableId {
    /// The identifier of the element at `place` in what the stage of payload
    /// `P` produced for `module`.
    ///
    /// `place` is the path to the element in the stage's own structure, from
    /// the outside in, such as `["main", "parameter", "count"]`. Build it
    /// from what identifies the element among its siblings, such as a name.
    /// Never build it from a text offset, a counter over the whole module or
    /// a memory address: these change when an unrelated element does.
    ///
    /// # Errors
    ///
    /// Fails when the payload kind of `P` breaks the rule on
    /// [`Payload::KIND`].
    pub fn of<P: Payload>(module: &str, place: &[&str]) -> Result<Self, InvalidPayloadKind> {
        let mut hasher = FieldHasher::new("brio lens stable identifier");
        hasher.text(module);
        hasher.text(payload::kind_of::<P>()?);
        hasher.count(place.len());
        for step in place {
            hasher.text(step);
        }
        Ok(Self(hash::hex(&hasher.finish()[..16])))
    }

    /// This identifier as JSON, for a payload to write.
    #[must_use]
    pub fn to_json(&self) -> Value {
        Value::from(self.0.as_str())
    }

    /// The identifier that [`StableId::to_json`] wrote as `value`, or `None`
    /// when `value` is not an identifier.
    #[must_use]
    pub fn from_json(value: &Value) -> Option<Self> {
        let text = value.as_str()?;
        hash::is_hex(text, 32).then(|| Self(text.to_owned()))
    }
}

impl fmt::Display for StableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
