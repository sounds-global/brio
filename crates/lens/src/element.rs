// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The elements of a record, and the element table that holds them.

use std::collections::BTreeMap;

use crate::json::{Map, Value, json};
use crate::mask::{self, MaskedMarkers};
use crate::provenance::{self, OriginStep, Provenance};
use crate::schema::{list_schema, object_schema, reference};
use crate::{ElementLink, Reason, SourceSpan, StableId, fields, reason};

/// The JSON name of the parent of an element.
const PARENT: &str = "parent";
/// The JSON name of the children of an element.
const CHILDREN: &str = "children";
/// The JSON name of the payload of an element.
const PAYLOAD: &str = "payload";

/// The JSON name of the masked markers of an element.
const MASKED: &str = mask::MASKED;
/// The JSON name of the provenance of an element.
const PROVENANCE: &str = "provenance";
/// The JSON name of the reasons of an element.
const REASONS: &str = "reasons";

/// One element of a stage record: one thing the stage produced, such as a
/// token or a declaration.
///
/// A phase makes an element from its stable identifier and its payload, and
/// names its parent when it is inside another element. Lens works out the
/// children of every element from the parents the elements name.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub(crate) id: StableId,
    pub(crate) parent: Option<StableId>,
    pub(crate) children: Vec<StableId>,
    pub(crate) payload: Value,
    pub(crate) masked: MaskedMarkers,
    pub(crate) provenance: Provenance,
    pub(crate) reasons: Vec<Reason>,
}

impl Element {
    /// The element with the stable identifier `id`, which
    /// [`Stage::id`](crate::Stage::id) gives, and with `payload`: what the
    /// phase says about this element alone, in the form its
    /// [`PayloadType::schema_fragment`](crate::PayloadType::schema_fragment)
    /// describes.
    ///
    /// The payload says nothing of the elements inside this one: each of
    /// them is an element of its own that names this one as its parent.
    #[must_use]
    pub fn new(id: StableId, payload: Value) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            payload,
            masked: MaskedMarkers::default(),
            provenance: Provenance::default(),
            reasons: Vec::new(),
        }
    }

    /// This element, with the value at `place` in its payload masked.
    ///
    /// A phase calls this for every value of a `Secret` or `Sensitive` type
    /// in the payload. `place` is a JSON Pointer (RFC 6901) into the payload,
    /// such as `/value` or `/arguments/0`. The value is dropped here, so its
    /// content is in no record: the record holds null at `place`, and a
    /// masked marker that names it.
    ///
    /// The record writer refuses a marker at a place the payload does not
    /// have. Only the payload and the justification of a reason can be
    /// masked: a confidential value never goes in a stable identifier, in
    /// the provenance or in the decision of a reason.
    #[must_use]
    pub fn masked_at(mut self, place: &str) -> Self {
        self.masked.mask(&mut self.payload, place);
        self
    }

    /// This element, which comes from the source text at `span`.
    #[must_use]
    pub fn from_source(mut self, span: SourceSpan) -> Self {
        self.provenance.source.push(span);
        self
    }

    /// This element, which derives from the element of the previous stage
    /// that `link` leads to.
    #[must_use]
    pub fn derived_from(mut self, link: ElementLink) -> Self {
        self.provenance.derived_from.push(link);
        self
    }

    /// This element, which the compiler created: `by` made it from the
    /// element that `from` leads to.
    ///
    /// `by` names what the compiler did, such as `inlining`,
    /// `monomorphization` or `compile-time evaluation`. Call this once for
    /// every step of the chain of origin, starting with the step that made
    /// this element and going back to an element that comes from source
    /// text, so the element records its full chain.
    #[must_use]
    pub fn created_by(mut self, by: &str, from: ElementLink) -> Self {
        self.provenance.origin.push(OriginStep {
            by: by.to_owned(),
            from,
        });
        self
    }

    /// This element, with the reason for one decision the phase made about
    /// it.
    #[must_use]
    pub fn because(mut self, reason: Reason) -> Self {
        self.reasons.push(reason);
        self
    }

    /// This element, inside the element `parent` of the same record.
    #[must_use]
    pub fn child_of(mut self, parent: &StableId) -> Self {
        self.parent = Some(parent.clone());
        self
    }

    /// The stable identifier of this element.
    #[must_use]
    pub fn id(&self) -> &StableId {
        &self.id
    }

    /// The element this one is inside, or `None` for a root.
    #[must_use]
    pub fn parent(&self) -> Option<&StableId> {
        self.parent.as_ref()
    }

    /// The elements inside this one, in their order.
    #[must_use]
    pub fn children(&self) -> &[StableId] {
        &self.children
    }

    /// What the phase says about this element alone.
    #[must_use]
    pub fn payload(&self) -> &Value {
        &self.payload
    }

    /// Whether the value at `place` in the payload was masked: the payload
    /// holds null there in place of a confidential value.
    #[must_use]
    pub fn is_masked(&self, place: &str) -> bool {
        self.masked.has(place)
    }

    /// Every place in the payload where a value was masked, in order.
    pub fn masked(&self) -> impl Iterator<Item = &str> {
        self.masked.places()
    }

    /// Where this element comes from.
    #[must_use]
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// The reason for every decision the phase made about this element.
    #[must_use]
    pub fn reasons(&self) -> &[Reason] {
        &self.reasons
    }

    /// This element as it is written in the element table, under its stable
    /// identifier.
    fn to_json(&self) -> Value {
        fields::object([
            (
                PARENT,
                self.parent.as_ref().map_or(Value::Null, StableId::to_json),
            ),
            (CHILDREN, fields::list(&self.children, StableId::to_json)),
            (PAYLOAD, self.payload.clone()),
            (MASKED, self.masked.to_json()),
            (PROVENANCE, self.provenance.to_json()),
            (REASONS, fields::list(&self.reasons, Reason::to_json)),
        ])
    }

    /// The element written as `element` under the stable identifier `id`,
    /// or `None` when `element` is not an element.
    fn from_json(id: StableId, element: &Value) -> Option<Self> {
        let parent = match element.get(PARENT)? {
            Value::Null => None,
            parent => Some(StableId::from_json(parent)?),
        };
        Some(Self {
            id,
            parent,
            children: fields::list_from(element.get(CHILDREN)?, StableId::from_json)?,
            payload: element.get(PAYLOAD)?.clone(),
            masked: MaskedMarkers::from_json(element.get(MASKED)?)?,
            provenance: Provenance::from_json(element.get(PROVENANCE)?)?,
            reasons: fields::list_from(element.get(REASONS)?, Reason::from_json)?,
        })
    }
}

/// The elements of one record, each under its stable identifier.
///
/// The table is flat: an element names its parent and lists its children by
/// stable identifier, however deep the structure of the stage is. Any tool
/// finds an element by its stable identifier alone, without knowing the
/// payload type of the record.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElementTable {
    pub(crate) roots: Vec<StableId>,
    pub(crate) elements: BTreeMap<StableId, Element>,
}

impl ElementTable {
    /// The elements that are inside no other, in their order.
    #[must_use]
    pub fn roots(&self) -> &[StableId] {
        &self.roots
    }

    /// The element with the stable identifier `id`, or `None` when the
    /// record has no such element.
    #[must_use]
    pub fn get(&self, id: &StableId) -> Option<&Element> {
        self.elements.get(id)
    }

    /// Every element, in the order of their stable identifiers.
    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.elements.values()
    }

    /// How many elements the record has.
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Whether the record has no element.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Every element, each parent before its children, in their order: the
    /// order in which the stage produced them, when it produced each parent
    /// before its children.
    pub fn in_order(&self) -> impl Iterator<Item = &Element> {
        let mut ordered = Vec::with_capacity(self.elements.len());
        let mut next: Vec<&StableId> = self.roots.iter().rev().collect();
        while let Some(id) = next.pop() {
            // A table read from a record is walked before it is known to
            // meet the rules, so the walk never trusts its lists: it stops
            // once it has as many elements as the table holds, and passes
            // over a stable identifier the table does not have.
            if ordered.len() == self.elements.len() {
                break;
            }
            if let Some(element) = self.elements.get(id) {
                next.extend(element.children.iter().rev());
                ordered.push(element);
            }
        }
        ordered.into_iter()
    }

    /// The roots as they are written in a record.
    pub(crate) fn roots_to_json(&self) -> Value {
        fields::list(&self.roots, StableId::to_json)
    }

    /// The elements as they are written in a record.
    pub(crate) fn elements_to_json(&self) -> Value {
        let elements: Map<String, Value> = self
            .elements
            .iter()
            .map(|(id, element)| (id.to_string(), element.to_json()))
            .collect();
        Value::Object(elements)
    }

    /// The table written as `roots` and `elements`, or `None` when they are
    /// not the two parts of a table. Nothing here says the table meets the
    /// rules of a record: the envelope checks them.
    pub(crate) fn from_json(roots: &Value, elements: &Value) -> Option<Self> {
        let elements = elements
            .as_object()?
            .iter()
            .map(|(id, element)| {
                let id = StableId::from_json(&Value::from(id.as_str()))?;
                Some((id.clone(), Element::from_json(id, element)?))
            })
            .collect::<Option<BTreeMap<StableId, Element>>>()?;
        Some(Self {
            roots: fields::list_from(roots, StableId::from_json)?,
            elements,
        })
    }
}

/// The name of the definition of a stable identifier in the published
/// schema.
pub(crate) const ID_DEFINITION: &str = "stable_identifier";
/// The name of the definition of an element in the published schema.
pub(crate) const DEFINITION: &str = "element";

/// The JSON Schema of a stable identifier.
pub(crate) fn id_schema() -> Value {
    json!({
        "description": "A stable identifier: the name of one element, the same in every build.",
        "type": "string",
        "pattern": "^[0-9a-f]{32}$"
    })
}

/// The JSON Schema of one element of the element table.
pub(crate) fn schema() -> Value {
    let id = reference(ID_DEFINITION);
    object_schema(
        "One element of what the stage produced.",
        &[
            (
                PARENT,
                json!({
                    "description": "The element this one is inside, or null for a root.",
                    "oneOf": [id, { "type": "null" }]
                }),
            ),
            (
                CHILDREN,
                list_schema(
                    "The elements inside this one, in their order. Each names this element as \
                     its parent.",
                    ID_DEFINITION,
                ),
            ),
            (
                PAYLOAD,
                json!({
                    "description": "What the phase says about this element alone, in the form \
                                    its payload kind defines."
                }),
            ),
            (MASKED, mask::list_schema_for("the payload")),
            (PROVENANCE, reference(provenance::DEFINITION)),
            (
                REASONS,
                list_schema(
                    "The reason for every decision the phase made about this element.",
                    reason::DEFINITION,
                ),
            ),
        ],
    )
}

/// The part of the schema of an element that holds for one payload: the
/// schema `fragment` of its payload.
pub(crate) fn schema_case(fragment: &Value) -> Value {
    let mut properties = Map::new();
    properties.insert(PAYLOAD.to_owned(), fragment.clone());
    json!({ "properties": properties })
}
