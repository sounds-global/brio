// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The payload contract a phase implements, and the check every payload
//! passes when it enters Lens.

use std::fmt;

use crate::json::{Value, json};
use crate::{Element, ElementTable, Stage};

/// What a phase declares for each payload type it records.
///
/// `brio_lens` knows no payload type: a phase crate implements this trait for
/// its own types. It is all that schema assembly needs. A payload type that
/// is also written and read implements [`Payload`].
pub trait PayloadType {
    /// The payload kind: the name of this payload type, such as `"tokens"`.
    ///
    /// It identifies the payload in every record and in the published schema,
    /// so it is unique among the payloads linked into one toolchain. It is
    /// not empty and is made of lowercase ASCII letters, digits and `_`.
    /// Lens refuses any other kind with [`InvalidPayloadKind`], where the
    /// payload type enters it: [`Stage::of`] and [`PayloadSchema::of`].
    const KIND: &'static str;

    /// The version of this payload's format, owned by the phase.
    const VERSION: u32;

    /// The JSON Schema of the payload of one element, as it appears in the
    /// published schema.
    ///
    /// Build it with [`json!`](crate::json::json) from [`crate::json`]. Where
    /// a value can be masked with
    /// [`Element::masked_at`](crate::Element::masked_at), the fragment admits
    /// null in its place.
    fn schema_fragment() -> Value;
}

/// A payload that is written and read: what one stage produced for one
/// module.
///
/// A phase gives Lens the elements of what it produced, flat, and Lens owns
/// the structure around them: the stable identifier of each element, its
/// parent and the order of its children.
pub trait Payload: PayloadType + Sized {
    /// The elements of this payload, as `stage` produced them.
    ///
    /// The elements are flat, whatever the structure of the payload: an
    /// element that is inside another names it with [`Element::child_of`],
    /// and the children of one parent keep the order they have here. The
    /// stable identifier of each element comes from [`Stage::id`].
    ///
    /// The result depends only on the payload: no timestamps, no absolute
    /// paths, and nothing that follows the iteration order of a hash table.
    /// Lens sorts the keys of every object when it writes the record.
    ///
    /// Lens calls this only when it writes the record, so a phase does no
    /// work for a record that is not written.
    fn to_elements(&self, stage: &Stage<Self>) -> Vec<Element>;

    /// The payload that [`Payload::to_elements`] gave as `elements`, or
    /// `None` when `elements` is not in that form.
    fn from_elements(elements: &ElementTable) -> Option<Self>;
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

/// The name of the definition of a payload kind in the published schema.
pub(crate) const KIND_DEFINITION: &str = "payload_kind";

/// The JSON Schema of a payload kind: the rule on [`PayloadType::KIND`], as
/// [`is_kind`] applies it.
pub(crate) fn kind_schema() -> Value {
    json!({
        "description": "A payload kind: the name of one payload type, made of lowercase \
                        letters, digits and `_`.",
        "type": "string",
        "pattern": "^[a-z0-9_]+$"
    })
}

/// Whether `payload_kind` meets the rule on [`PayloadType::KIND`].
pub(crate) fn is_kind(payload_kind: &str) -> bool {
    !payload_kind.is_empty()
        && payload_kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The payload kind of `P`, checked against the rule on
/// [`PayloadType::KIND`].
///
/// A payload type enters Lens through here, and nowhere else.
pub(crate) fn kind_of<P: PayloadType>() -> Result<&'static str, InvalidPayloadKind> {
    let payload_kind = P::KIND;
    if is_kind(payload_kind) {
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
    /// [`PayloadType::KIND`].
    pub fn of<P: PayloadType>() -> Result<Self, InvalidPayloadKind> {
        Ok(Self {
            kind: kind_of::<P>()?,
            version: P::VERSION,
            fragment: P::schema_fragment(),
        })
    }
}
