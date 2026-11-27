// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Reasons: why a phase decided what it decided.

use crate::json::{Value, json};
use crate::mask::{self, MaskedMarkers};
use crate::schema::{list_schema, object_schema, text_schema};
use crate::{ElementLink, envelope, fields, provenance};

/// The JSON name of the decision of a reason.
const DECISION: &str = "decision";
/// The JSON name of the justification of a reason.
const BECAUSE: &str = "because";
/// The JSON name of the masked markers of a reason.
const MASKED: &str = mask::MASKED;
/// The JSON name of the elements a reason involves.
const INVOLVES: &str = "involves";

/// The recorded justification for one decision a phase made about an
/// element, such as the constraints behind an inferred type.
#[derive(Debug, Clone, PartialEq)]
pub struct Reason {
    /// The decision: the phase's name for what it decided, such as
    /// `inferred type`. Never a confidential value: nothing masks it.
    pub decision: String,
    /// What justifies the decision, in the phase's own form: the candidates
    /// considered, the constraints, the rule applied. It nests at most
    /// [`Reason::MAX_DEPTH`] levels.
    pub because: Value,
    /// The elements the justification is about, such as both accesses of a
    /// rejected borrow. Lens can find them, in any record.
    pub involves: Vec<ElementLink>,
    /// Where a value of the justification was masked.
    masked: MaskedMarkers,
}

impl Reason {
    /// How deep the justification of a reason can nest arrays and objects,
    /// as JSON. The record writer refuses any reason a reader could not read
    /// back.
    pub const MAX_DEPTH: usize = envelope::MAX_REASON_DEPTH;

    /// The reason for `decision`, justified by `because`.
    #[must_use]
    pub fn new(decision: &str, because: Value) -> Self {
        Self {
            decision: decision.to_owned(),
            because,
            involves: Vec::new(),
            masked: MaskedMarkers::default(),
        }
    }

    /// This reason, with the value at `place` in its justification masked.
    ///
    /// `place` is a JSON Pointer (RFC 6901) into the justification, as for
    /// [`Element::masked_at`](crate::Element::masked_at). The value is
    /// dropped here, so its content is in no record.
    #[must_use]
    pub fn masked_at(mut self, place: &str) -> Self {
        self.masked.mask(&mut self.because, place);
        self
    }

    /// Whether the value at `place` in the justification was masked: the
    /// justification holds null there in place of a confidential value.
    #[must_use]
    pub fn is_masked(&self, place: &str) -> bool {
        self.masked.has(place)
    }

    /// Every place in the justification where a value was masked, in order.
    pub fn masked(&self) -> impl Iterator<Item = &str> {
        self.masked.places()
    }

    /// A masked marker of this reason that names a place where its
    /// justification holds no masked value.
    pub(crate) fn misplaced_marker(&self) -> Option<&str> {
        self.masked.misplaced_in(&self.because)
    }

    /// This reason, with a justification that is also about the element
    /// that `link` leads to.
    #[must_use]
    pub fn involving(mut self, link: ElementLink) -> Self {
        self.involves.push(link);
        self
    }

    /// This reason as it is written in a record.
    pub(crate) fn to_json(&self) -> Value {
        fields::object([
            (DECISION, Value::from(self.decision.as_str())),
            (BECAUSE, self.because.clone()),
            (MASKED, self.masked.to_json()),
            (INVOLVES, fields::list(&self.involves, ElementLink::to_json)),
        ])
    }

    /// The reason written as `reason`, or `None` when `reason` is not one.
    pub(crate) fn from_json(reason: &Value) -> Option<Self> {
        Some(Self {
            decision: reason.get(DECISION)?.as_str()?.to_owned(),
            because: reason.get(BECAUSE)?.clone(),
            masked: MaskedMarkers::from_json(reason.get(MASKED)?)?,
            involves: fields::list_from(reason.get(INVOLVES)?, ElementLink::from_json)?,
        })
    }
}

/// The name of the definition of a reason in the published schema.
pub(crate) const DEFINITION: &str = "reason";

/// The JSON Schema of a reason.
pub(crate) fn schema() -> Value {
    object_schema(
        "A reason: the recorded justification for one decision a phase made about an element.",
        &[
            (
                DECISION,
                text_schema("The phase's name for what it decided."),
            ),
            (
                BECAUSE,
                json!({
                    "description": "What justifies the decision, in the form the phase defines."
                }),
            ),
            (MASKED, mask::list_schema_for("the justification")),
            (
                INVOLVES,
                list_schema(
                    "The elements the justification is about.",
                    provenance::LINK_DEFINITION,
                ),
            ),
        ],
    )
}
