// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Stable identifiers of the elements of a record.

use std::fmt;

#[cfg(doc)]
use crate::Stage;
use crate::hash::{self, FieldHasher};
use crate::json::Value;

/// The stable identifier of one element of a stage record.
///
/// It comes from the element's identity alone: its module, the stage that
/// produced it and its place in that stage's structure. The same element has
/// the same identifier in every build, whatever else changes in the input.
///
/// It is written as 32 lowercase hexadecimal digits. [`Stage::id`] makes one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableId(String);

impl StableId {
    /// The identifier of the element at `place` in what the stage with
    /// `payload_kind` produced for `module`. [`Stage::id`] is the way in.
    pub(crate) fn derive(module: &str, payload_kind: &str, place: &[&str]) -> Self {
        let mut hasher = FieldHasher::new("brio lens stable identifier");
        hasher.text(module);
        hasher.text(payload_kind);
        hasher.count(place.len());
        for step in place {
            hasher.text(step);
        }
        Self(hash::hex(&hasher.finish()[..16]))
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
