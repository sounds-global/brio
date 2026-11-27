// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The payload contract a phase implements, and the check every payload
//! passes when it enters Lens.

use std::fmt;

use crate::json::Value;

/// What a phase declares for each payload type it records.
///
/// `brio_lens` knows no payload type: a phase crate implements this trait for
/// its own types.
pub trait Payload: Sized {
    /// The payload kind: the name of this payload type, such as `"tokens"`.
    ///
    /// It identifies the payload in every record and in the published schema,
    /// so it is unique among the payloads linked into one toolchain. It is
    /// not empty and is made of lowercase ASCII letters, digits and `_`.
    /// Lens refuses any other kind with [`InvalidPayloadKind`], wherever the
    /// payload enters it.
    const KIND: &'static str;

    /// The version of this payload's format, owned by the phase.
    const VERSION: u32;

    /// The JSON Schema of this payload, as it appears in the published schema.
    ///
    /// Build it with [`json!`](crate::json::json) from [`crate::json`].
    fn schema_fragment() -> Value;

    /// This payload as JSON, in the form [`Payload::schema_fragment`]
    /// describes.
    ///
    /// The result depends only on the payload: no timestamps, no absolute
    /// paths, and nothing that follows the iteration order of a hash table.
    /// Lens sorts the keys of every object when it writes the record.
    fn to_json(&self) -> Value;

    /// The payload that [`Payload::to_json`] wrote as `payload`, or `None`
    /// when `payload` is not in that form.
    fn from_json(payload: &Value) -> Option<Self>;
}

/// A payload kind that is empty or has characters other than lowercase ASCII
/// letters, digits and `_`.
///
/// Such a kind would make a schema version ambiguous, so no payload that
/// declares one enters Lens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidPayloadKind {
    /// The payload kind as declared.
    pub payload_kind: &'static str,
}

impl fmt::Display for InvalidPayloadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the payload kind `{}` is not made of lowercase letters, digits and `_`",
            self.payload_kind
        )
    }
}

impl std::error::Error for InvalidPayloadKind {}

/// The payload kind of `P`, checked against the rule on [`Payload::KIND`].
///
/// Everything in Lens that takes a payload type gets its kind from here.
pub(crate) fn kind_of<P: Payload>() -> Result<&'static str, InvalidPayloadKind> {
    let payload_kind = P::KIND;
    let valid = !payload_kind.is_empty()
        && payload_kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if valid {
        Ok(payload_kind)
    } else {
        Err(InvalidPayloadKind { payload_kind })
    }
}

/// The declaration of one payload type, with the type itself erased.
///
/// This is what schema assembly takes: a tool that links several phases
/// collects one of these for each payload type.
#[derive(Debug)]
pub struct PayloadSchema {
    pub(crate) kind: &'static str,
    pub(crate) version: u32,
    pub(crate) fragment: Value,
}

impl PayloadSchema {
    /// Captures what `P` declares against the payload contract.
    ///
    /// # Errors
    ///
    /// Fails when the payload kind of `P` breaks the rule on
    /// [`Payload::KIND`].
    pub fn of<P: Payload>() -> Result<Self, InvalidPayloadKind> {
        Ok(Self {
            kind: kind_of::<P>()?,
            version: P::VERSION,
            fragment: P::schema_fragment(),
        })
    }
}
