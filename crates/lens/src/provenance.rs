// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Provenance: where an element of a record comes from.

use crate::json::{Value, json};
use crate::schema::{list_schema, object_schema, reference, text_schema};
use crate::stage::StageIdentity;
use crate::{Element, Envelope, StableId, element, envelope, fields};

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
        fields::object([
            (FILE, Value::from(self.file.as_str())),
            (START, Value::from(self.start)),
            (END, Value::from(self.end)),
        ])
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
    stage: StageIdentity,
    id: StableId,
}

impl ElementLink {
    /// The link to the element `id` of the record that `stage` produced.
    pub(crate) fn new(stage: StageIdentity, id: StableId) -> Self {
        Self { stage, id }
    }

    /// The module of the record the element is in.
    #[must_use]
    pub fn module(&self) -> &str {
        self.stage.module()
    }

    /// The payload kind of the record the element is in.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        self.stage.payload_kind()
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
    /// They are the records of one build: a link does not say which inputs
    /// its record was computed from, so among records of the same stage and
    /// module from several builds it leads to the first that holds the
    /// element.
    pub fn resolve<'a>(
        &self,
        envelopes: impl IntoIterator<Item = &'a Envelope>,
    ) -> Option<&'a Element> {
        envelopes
            .into_iter()
            .filter(|envelope| *envelope.stage() == self.stage)
            .find_map(|envelope| envelope.elements().get(&self.id))
    }

    /// This link as it is written in a record.
    pub(crate) fn to_json(&self) -> Value {
        fields::object(
            self.stage
                .fields()
                .into_iter()
                .chain([(ID, self.id.to_json())]),
        )
    }

    /// The link written as `link`, or `None` when `link` is not one.
    pub(crate) fn from_json(link: &Value) -> Option<Self> {
        Some(Self {
            stage: StageIdentity::from_json(link)?,
            id: StableId::from_json(link.get(ID)?)?,
        })
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
        fields::object([
            (BY, Value::from(self.by.as_str())),
            (FROM, self.from.to_json()),
        ])
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

    /// The source text that the chain of origin leads to, among the records
    /// with `envelopes`: the spans this element comes from, or else the
    /// spans of the first element back along the chain that comes from
    /// source text.
    ///
    /// It is empty when the chain leaves `envelopes` or never reaches source
    /// text. The records are those of one build, as for
    /// [`ElementLink::resolve`].
    pub fn origin_source<'a>(
        &'a self,
        envelopes: impl IntoIterator<Item = &'a Envelope>,
    ) -> Vec<&'a SourceSpan> {
        let envelopes: Vec<&Envelope> = envelopes.into_iter().collect();
        let mut provenance = self;
        let mut followed: Vec<&ElementLink> = Vec::new();
        while provenance.source.is_empty() {
            // The last step of a chain is the furthest back.
            let Some(step) = provenance.origin.last() else {
                break;
            };
            // A chain that comes back to an element it left leads nowhere.
            if followed.contains(&&step.from) {
                break;
            }
            followed.push(&step.from);
            match step.from.resolve(envelopes.iter().copied()) {
                Some(element) => provenance = element.provenance(),
                None => break,
            }
        }
        provenance.source.iter().collect()
    }

    /// This provenance as it is written in a record.
    pub(crate) fn to_json(&self) -> Value {
        fields::object([
            (SOURCE, fields::list(&self.source, SourceSpan::to_json)),
            (
                DERIVED_FROM,
                fields::list(&self.derived_from, ElementLink::to_json),
            ),
            (ORIGIN, fields::list(&self.origin, OriginStep::to_json)),
        ])
    }

    /// The provenance written as `provenance`, or `None` when `provenance`
    /// is not one.
    pub(crate) fn from_json(provenance: &Value) -> Option<Self> {
        Some(Self {
            source: fields::list_from(provenance.get(SOURCE)?, SourceSpan::from_json)?,
            derived_from: fields::list_from(provenance.get(DERIVED_FROM)?, ElementLink::from_json)?,
            origin: fields::list_from(provenance.get(ORIGIN)?, OriginStep::from_json)?,
        })
    }
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

/// A JSON Schema of a count of bytes with `description`.
fn offset_schema(description: &str) -> Value {
    json!({ "description": description, "type": "integer", "minimum": 0 })
}

/// The JSON Schema of a source span.
fn span_schema() -> Value {
    object_schema(
        "A span of source text: some bytes of one source file.",
        &[
            (FILE, reference(envelope::NAME_DEFINITION)),
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
    let [module, payload_kind] = StageIdentity::schema_fields();
    object_schema(
        "A link to one element of a record. The record is the one with this module and this \
         payload kind, and the element is under this stable identifier in its element table.",
        &[
            module,
            payload_kind,
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
