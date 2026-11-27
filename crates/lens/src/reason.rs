// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Reasons: why a phase decided what it decided.

use crate::json::{Map, Value, json};
use crate::provenance::{self, list_from_json, list_schema, object_schema};
use crate::{ElementLink, envelope};

/// The JSON name of the decision of a reason.
const DECISION: &str = "decision";
/// The JSON name of the justification of a reason.
const BECAUSE: &str = "because";
/// The JSON name of the elements a reason involves.
const ELEMENTS: &str = "elements";

/// The recorded justification for one decision a phase made about an
/// element, such as the constraints behind an inferred type.
#[derive(Debug, Clone, PartialEq)]
pub struct Reason {
    /// The decision: the phase's name for what it decided, such as
    /// `inferred type`.
    pub decision: String,
    /// What justifies the decision, in the phase's own form: the candidates
    /// considered, the constraints, the rule applied. It nests at most
    /// [`Reason::MAX_DEPTH`] levels.
    pub because: Value,
    /// The elements the justification is about, such as both accesses of a
    /// rejected borrow. Lens can find them, in any record.
    pub elements: Vec<ElementLink>,
}

impl Reason {
    /// How deep the justification of a reason can nest arrays and objects,
    /// as JSON. The record writer refuses any reason a reader could not read
    /// back.
    // The justification is inside its reason, the reasons, their element,
    // the elements and the record.
    pub const MAX_DEPTH: usize = envelope::MAX_RECORD_DEPTH - 5;

    /// The reason for `decision`, justified by `because`.
    #[must_use]
    pub fn new(decision: &str, because: Value) -> Self {
        Self {
            decision: decision.to_owned(),
            because,
            elements: Vec::new(),
        }
    }

    /// This reason, with a justification that is also about the element
    /// that `link` leads to.
    #[must_use]
    pub fn involving(mut self, link: ElementLink) -> Self {
        self.elements.push(link);
        self
    }

    /// This reason as it is written in a record.
    pub(crate) fn to_json(&self) -> Value {
        let mut reason = Map::new();
        reason.insert(DECISION.to_owned(), Value::from(self.decision.as_str()));
        reason.insert(BECAUSE.to_owned(), self.because.clone());
        reason.insert(
            ELEMENTS.to_owned(),
            self.elements.iter().map(ElementLink::to_json).collect(),
        );
        Value::Object(reason)
    }

    /// The reason written as `reason`, or `None` when `reason` is not one.
    pub(crate) fn from_json(reason: &Value) -> Option<Self> {
        Some(Self {
            decision: reason.get(DECISION)?.as_str()?.to_owned(),
            because: reason.get(BECAUSE)?.clone(),
            elements: list_from_json(reason.get(ELEMENTS)?, ElementLink::from_json)?,
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
                json!({
                    "description": "The phase's name for what it decided.",
                    "type": "string"
                }),
            ),
            (
                BECAUSE,
                json!({
                    "description": "What justifies the decision, in the form the phase defines."
                }),
            ),
            (
                ELEMENTS,
                list_schema(
                    "The elements the justification is about.",
                    provenance::LINK_DEFINITION,
                ),
            ),
        ],
    )
}
