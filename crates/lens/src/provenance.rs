// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Provenance: where an element of a record comes from.

use crate::json::{Map, Value, json};
use crate::schema::reference;
use crate::{Element, Envelope, StableId, element, envelope, payload};

/// The JSON name of the source spans of a provenance.
const SOURCE: &str = "source";
/// The JSON name of the previous-stage elements of a provenance.
const DERIVED_FROM: &str = "derived_from";
/// The JSON name of the chain of origin of a provenance.
const ORIGIN: &str = "origin";
/// The JSON name of what made the element, in a step of a chain of origin.
const BY: &str = "by";
/// The JSON name of what the element was made from, in a step of a chain of
/// origin.
const FROM: &str = "from";
/// The JSON name of the file of a source span.
const FILE: &str = "file";
/// The JSON name of the start of a source span.
const START: &str = "start";
/// The JSON name of the end of a source span.
const END: &str = "end";

/// The JSON name of the module of an element link.
const MODULE: &str = "module";
/// The JSON name of the payload kind of an element link.
const PAYLOAD_KIND: &str = "payload_kind";
/// The JSON name of the stable identifier of an element link.
const ID: &str = "id";

/// A span of source text: some bytes of one source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    /// The source file, by its name in the workspace, such as
    /// `src/main.brio`. The name is written in the record, so it is never
    /// an absolute path: the record writer refuses one.
    pub file: String,
    /// Where the span starts: how many bytes of the file are before it.
    pub start: u64,
    /// Where the span ends: how many bytes of the file are before its end.
    /// The byte at `end` is not in the span.
    pub end: u64,
}

impl SourceSpan {
    /// This span as it is written in a record.
    fn to_json(&self) -> Value {
        let mut span = Map::new();
        span.insert(FILE.to_owned(), Value::from(self.file.as_str()));
        span.insert(START.to_owned(), Value::from(self.start));
        span.insert(END.to_owned(), Value::from(self.end));
        Value::Object(span)
    }

    /// The span written as `span`, or `None` when `span` is not one.
    fn from_json(span: &Value) -> Option<Self> {
        Some(Self {
            file: span.get(FILE)?.as_str()?.to_owned(),
            start: span.get(START)?.as_u64()?,
            end: span.get(END)?.as_u64()?,
        })
    }
}

/// A link to one element of a record: the record, by its module and its
/// payload kind, and the element in it, by its stable identifier.
///
/// [`Stage::link`](crate::Stage::link) makes one, from the stage that
/// produced the element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementLink {
    module: String,
    payload_kind: String,
    id: StableId,
}

impl ElementLink {
    /// The link to the element `id` of the record that the stage with
    /// `payload_kind` produced for `module`.
    pub(crate) fn new(module: &str, payload_kind: &str, id: StableId) -> Self {
        Self {
            module: module.to_owned(),
            payload_kind: payload_kind.to_owned(),
            id,
        }
    }

    /// The module of the record the element is in.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The payload kind of the record the element is in.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        &self.payload_kind
    }

    /// The stable identifier of the element.
    #[must_use]
    pub fn id(&self) -> &StableId {
        &self.id
    }

    /// The element this link leads to, among the records with `envelopes`,
    /// or `None` when none of them holds it.
    ///
    /// The records can be of any payload type, known to the caller or not.
    pub fn resolve<'a>(
        &self,
        envelopes: impl IntoIterator<Item = &'a Envelope>,
    ) -> Option<&'a Element> {
        envelopes
            .into_iter()
            .filter(|envelope| {
                envelope.module() == self.module && envelope.payload_kind() == self.payload_kind
            })
            .find_map(|envelope| envelope.elements().get(&self.id))
    }

    /// This link as it is written in a record.
    pub(crate) fn to_json(&self) -> Value {
        let mut link = Map::new();
        link.insert(MODULE.to_owned(), Value::from(self.module.as_str()));
        link.insert(
            PAYLOAD_KIND.to_owned(),
            Value::from(self.payload_kind.as_str()),
        );
        link.insert(ID.to_owned(), self.id.to_json());
        Value::Object(link)
    }

    /// The link written as `link`, or `None` when `link` is not one: a link
    /// only ever names a module and a payload kind that a stage can have.
    pub(crate) fn from_json(link: &Value) -> Option<Self> {
        let module = link.get(MODULE)?.as_str()?;
        let payload_kind = link.get(PAYLOAD_KIND)?.as_str()?;
        if envelope::is_absolute_path(module) || !payload::is_kind(payload_kind) {
            return None;
        }
        Some(Self::new(
            module,
            payload_kind,
            StableId::from_json(link.get(ID)?)?,
        ))
    }
}

/// One step of a chain of origin: what made a compiler-created element, and
/// the element it was made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginStep {
    /// What made the element at this step, such as `inlining`,
    /// `monomorphization` or `compile-time evaluation`.
    pub by: String,
    /// The element it was made from.
    pub from: ElementLink,
}

impl OriginStep {
    /// This step as it is written in a record.
    fn to_json(&self) -> Value {
        let mut step = Map::new();
        step.insert(BY.to_owned(), Value::from(self.by.as_str()));
        step.insert(FROM.to_owned(), self.from.to_json());
        Value::Object(step)
    }

    /// The step written as `step`, or `None` when `step` is not one.
    fn from_json(step: &Value) -> Option<Self> {
        Some(Self {
            by: step.get(BY)?.as_str()?.to_owned(),
            from: ElementLink::from_json(step.get(FROM)?)?,
        })
    }
}

/// The provenance of one element: the links from it back to the source text
/// it comes from, and to the elements of the previous stage it derives from.
///
/// An element the compiler creates, such as the copy of a function body that
/// inlining makes, also has a chain of origin: every step that made it, from
/// the last one back to an element that comes from source text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Provenance {
    pub(crate) source: Vec<SourceSpan>,
    pub(crate) derived_from: Vec<ElementLink>,
    pub(crate) origin: Vec<OriginStep>,
}

impl Provenance {
    /// The spans of source text the element comes from.
    #[must_use]
    pub fn source(&self) -> &[SourceSpan] {
        &self.source
    }

    /// The elements of the previous stage that the element derives from.
    #[must_use]
    pub fn derived_from(&self) -> &[ElementLink] {
        &self.derived_from
    }

    /// The chain of origin of a compiler-created element: every step that
    /// made it, starting with the last. It is empty for an element the
    /// compiler did not create.
    ///
    /// Following the chain leads to source: the element the last step here
    /// was made from comes from source text, or has a chain of its own.
    #[must_use]
    pub fn origin(&self) -> &[OriginStep] {
        &self.origin
    }

    /// This provenance as it is written in a record.
    pub(crate) fn to_json(&self) -> Value {
        let mut provenance = Map::new();
        provenance.insert(
            SOURCE.to_owned(),
            self.source.iter().map(SourceSpan::to_json).collect(),
        );
        provenance.insert(
            DERIVED_FROM.to_owned(),
            self.derived_from.iter().map(ElementLink::to_json).collect(),
        );
        provenance.insert(
            ORIGIN.to_owned(),
            self.origin.iter().map(OriginStep::to_json).collect(),
        );
        Value::Object(provenance)
    }

    /// The provenance written as `provenance`, or `None` when `provenance`
    /// is not one.
    pub(crate) fn from_json(provenance: &Value) -> Option<Self> {
        Some(Self {
            source: list_from_json(provenance.get(SOURCE)?, SourceSpan::from_json)?,
            derived_from: list_from_json(provenance.get(DERIVED_FROM)?, ElementLink::from_json)?,
            origin: list_from_json(provenance.get(ORIGIN)?, OriginStep::from_json)?,
        })
    }
}

/// The list written as `list`, each item read by `item`, or `None` when
/// `list` is not a list of such items.
pub(crate) fn list_from_json<T>(list: &Value, item: fn(&Value) -> Option<T>) -> Option<Vec<T>> {
    list.as_array()?.iter().map(item).collect()
}

/// The name of the definition of a provenance in the published schema.
pub(crate) const DEFINITION: &str = "provenance";
/// The name of the definition of an element link in the published schema.
pub(crate) const LINK_DEFINITION: &str = "element_link";
/// The name of the definition of a source span in the published schema.
const SPAN_DEFINITION: &str = "source_span";
/// The name of the definition of a step of a chain of origin in the
/// published schema.
const STEP_DEFINITION: &str = "origin_step";

/// A JSON Schema of an object with `description`, in which every one of
/// `properties` is required.
pub(crate) fn object_schema(description: &str, properties: &[(&str, Value)]) -> Value {
    let required: Vec<&str> = properties.iter().map(|(name, _)| *name).collect();
    let properties: Map<String, Value> = properties
        .iter()
        .map(|(name, schema)| ((*name).to_owned(), schema.clone()))
        .collect();
    json!({
        "description": description,
        "type": "object",
        "required": required,
        "properties": properties
    })
}

/// A JSON Schema of a list with `description`, of items that each meet the
/// definition called `item`.
pub(crate) fn list_schema(description: &str, item: &str) -> Value {
    json!({ "description": description, "type": "array", "items": reference(item) })
}

/// A JSON Schema of a text with `description`.
fn text_schema(description: &str) -> Value {
    json!({ "description": description, "type": "string" })
}

/// A JSON Schema of a count of bytes with `description`.
fn offset_schema(description: &str) -> Value {
    json!({ "description": description, "type": "integer", "minimum": 0 })
}

/// The JSON Schema of a source span.
fn span_schema() -> Value {
    object_schema(
        "A span of source text: some bytes of one source file.",
        &[
            (
                FILE,
                text_schema(
                    "The source file, by its name in the workspace. Never an absolute path.",
                ),
            ),
            (
                START,
                offset_schema("Where the span starts: how many bytes of the file are before it."),
            ),
            (
                END,
                offset_schema(
                    "Where the span ends: how many bytes of the file are before its end. The \
                     byte at this place is not in the span.",
                ),
            ),
        ],
    )
}

/// The JSON Schema of an element link.
fn link_schema() -> Value {
    object_schema(
        "A link to one element of a record. The record is the one with this module and this \
         payload kind, and the element is under this stable identifier in its element table.",
        &[
            (
                MODULE,
                text_schema("The module of the record the element is in."),
            ),
            (
                PAYLOAD_KIND,
                text_schema("The payload kind of the record the element is in."),
            ),
            (ID, reference(element::ID_DEFINITION)),
        ],
    )
}

/// The JSON Schema of a step of a chain of origin.
fn step_schema() -> Value {
    object_schema(
        "One step of a chain of origin: what made a compiler-created element, and the element \
         it was made from.",
        &[
            (
                BY,
                text_schema(
                    "What made the element at this step, such as inlining, monomorphization or \
                     compile-time evaluation.",
                ),
            ),
            (FROM, reference(LINK_DEFINITION)),
        ],
    )
}

/// The JSON Schema of a provenance.
fn schema() -> Value {
    object_schema(
        "The provenance of an element: where it comes from.",
        &[
            (
                SOURCE,
                list_schema(
                    "The spans of source text the element comes from.",
                    SPAN_DEFINITION,
                ),
            ),
            (
                DERIVED_FROM,
                list_schema(
                    "The elements of the previous stage that the element derives from.",
                    LINK_DEFINITION,
                ),
            ),
            (
                ORIGIN,
                list_schema(
                    "The chain of origin of an element the compiler created: every step that \
                     made it, starting with the last. Empty for any other element. The element \
                     the last step was made from comes from source text, or has a chain of \
                     its own.",
                    STEP_DEFINITION,
                ),
            ),
        ],
    )
}

/// The JSON Schema of everything a provenance is made of, each under the
/// name of its definition.
pub(crate) fn definitions() -> [(&'static str, Value); 4] {
    [
        (DEFINITION, schema()),
        (SPAN_DEFINITION, span_schema()),
        (LINK_DEFINITION, link_schema()),
        (STEP_DEFINITION, step_schema()),
    ]
}
