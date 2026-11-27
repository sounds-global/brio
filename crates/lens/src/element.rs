// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The elements of a record, and the element table that holds them.

use std::collections::BTreeMap;

use crate::json::{Map, Value, json};
use crate::provenance::{self, OriginStep, Provenance, list_from_json, list_schema, object_schema};
use crate::schema::reference;
use crate::{ElementLink, Reason, SourceSpan, StableId, reason};

/// The JSON name of the parent of an element.
const PARENT: &str = "parent";
/// The JSON name of the children of an element.
const CHILDREN: &str = "children";
/// The JSON name of the payload of an element.
const PAYLOAD: &str = "payload";

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
            provenance: Provenance::default(),
            reasons: Vec::new(),
        }
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
        let mut element = Map::new();
        element.insert(
            PARENT.to_owned(),
            self.parent.as_ref().map_or(Value::Null, StableId::to_json),
        );
        element.insert(CHILDREN.to_owned(), ids_to_json(&self.children));
        element.insert(PAYLOAD.to_owned(), self.payload.clone());
        element.insert(PROVENANCE.to_owned(), self.provenance.to_json());
        element.insert(
            REASONS.to_owned(),
            self.reasons.iter().map(Reason::to_json).collect(),
        );
        Value::Object(element)
    }

    /// The element written as `element` under the stable identifier `id`,
    /// or `None` when `element` is not an element.
    fn from_json(id: StableId, element: &Value) -> Option<Self> {
        let element = element.as_object()?;
        let parent = match element.get(PARENT)? {
            Value::Null => None,
            parent => Some(StableId::from_json(parent)?),
        };
        Some(Self {
            id,
            parent,
            children: ids_from_json(element.get(CHILDREN)?)?,
            payload: element.get(PAYLOAD)?.clone(),
            provenance: Provenance::from_json(element.get(PROVENANCE)?)?,
            reasons: list_from_json(element.get(REASONS)?, Reason::from_json)?,
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

    /// The roots as they are written in a record.
    pub(crate) fn roots_to_json(&self) -> Value {
        ids_to_json(&self.roots)
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
            roots: ids_from_json(roots)?,
            elements,
        })
    }
}

/// A list of stable identifiers as JSON.
fn ids_to_json(ids: &[StableId]) -> Value {
    ids.iter().map(StableId::to_json).collect()
}

/// The list of stable identifiers written as `ids`, or `None` when `ids` is
/// not one.
fn ids_from_json(ids: &Value) -> Option<Vec<StableId>> {
    ids.as_array()?.iter().map(StableId::from_json).collect()
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
