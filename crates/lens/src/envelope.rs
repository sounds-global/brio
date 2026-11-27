// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The envelope: what every record shares, whatever its payload.
//!
//! This module owns the envelope's fields and its version, and decides what
//! a valid record is: every rule a record meets is applied here, the same
//! when a record is made as when it is read. The published schema describes
//! what is defined here, so records and schema cannot drift.

use std::collections::BTreeSet;
use std::fmt;

use crate::json::{Map, Value, json};

use crate::payload::{self, InvalidPayloadKind, Payload, PayloadType};
use crate::schema::{list_schema, object_schema, reference};
use crate::stage::{self, StageIdentity};
use crate::{
    Element, ElementTable, PayloadSchema, Reason, RecordInputs, RecordKey, SourceSpan, StableId,
    Stage, canonical, element, fields, hash, provenance, reason,
};

/// The JSON name of the envelope version.
const ENVELOPE_VERSION: &str = "envelope_version";
/// The JSON name of the record key.
const KEY: &str = "key";
/// The JSON name of the payload version.
const PAYLOAD_VERSION: &str = "payload_version";
/// The JSON name of the roots of the element table.
const ROOTS: &str = "roots";
/// The JSON name of the elements of the element table.
const ELEMENTS: &str = "elements";
/// The JSON name of the digest of all the other fields.
const DIGEST: &str = "digest";

/// How deep a record can nest arrays and objects, as JSON: a reader of JSON
/// stops at a fixed depth.
const MAX_RECORD_DEPTH: usize = 127;

/// How deep the justification of a reason can nest: it is inside its reason,
/// the reasons, their element, the elements and the record.
pub(crate) const MAX_REASON_DEPTH: usize = MAX_RECORD_DEPTH - 5;

/// What every record shares, whatever its payload type: what the record says
/// about itself, and the element table that holds what the stage produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    key: RecordKey,
    stage: StageIdentity,
    payload_version: u32,
    elements: ElementTable,
}

impl Envelope {
    /// The version of the envelope this toolchain writes, and the only one
    /// it reads.
    pub const VERSION: u32 = 1;

    /// How deep the payload of an element can nest arrays and objects, as
    /// JSON.
    ///
    /// A reader of JSON stops at a fixed depth, and the record writer refuses
    /// any payload a reader could not read back. The structure of a stage
    /// never counts: elements are written flat, each naming its parent by
    /// stable identifier.
    // The payload is inside its element, the elements and the record.
    pub const MAX_PAYLOAD_DEPTH: usize = MAX_RECORD_DEPTH - 3;

    /// The record key: the hash of every input of the record.
    #[must_use]
    pub fn key(&self) -> &RecordKey {
        &self.key
    }

    /// The module the record is for.
    #[must_use]
    pub fn module(&self) -> &str {
        self.stage.module()
    }

    /// The payload kind of the record's payload.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        self.stage.payload_kind()
    }

    /// What identifies the stage that produced the record.
    pub(crate) fn stage(&self) -> &StageIdentity {
        &self.stage
    }

    /// The version of the format of the record's payload.
    #[must_use]
    pub fn payload_version(&self) -> u32 {
        self.payload_version
    }

    /// The elements of the record: what the stage produced.
    #[must_use]
    pub fn elements(&self) -> &ElementTable {
        &self.elements
    }

    /// The record of what `stage` produced, as `payload`, from `inputs`: its
    /// envelope, and its canonical JSON text with its digest.
    pub(crate) fn seal<P: Payload>(
        stage: &Stage<P>,
        inputs: &RecordInputs,
        payload: &P,
    ) -> Result<(Self, String), RecordRefusal> {
        let envelope = Self {
            key: stage.key(inputs),
            stage: stage.identity().clone(),
            payload_version: stage.payload_version(),
            elements: table_of(payload.to_elements(stage))?,
        };
        let text = envelope.to_text();
        Ok((envelope, text))
    }

    /// The envelope of the record written as `text`, which is stored under
    /// the record key `stored_under`.
    ///
    /// The envelope version is checked first: no other field of a record
    /// with another envelope version is interpreted.
    pub(crate) fn open(text: &str, stored_under: &RecordKey) -> Result<Self, RecordDefect> {
        // The JSON reader stops at the depth the writer refuses beyond, so a
        // deeper record is not read as a record at all.
        let Ok(record @ Value::Object(_)) = crate::json::from_str::<Value>(text) else {
            return Err(RecordDefect::Corrupted);
        };

        let found = record
            .get(ENVELOPE_VERSION)
            .and_then(Value::as_u64)
            .ok_or(RecordDefect::Corrupted)?;
        if found != u64::from(Self::VERSION) {
            return Err(RecordDefect::UnsupportedEnvelopeVersion {
                found,
                supported: Self::VERSION,
            });
        }

        let envelope = Self::admitted(&record).ok_or(RecordDefect::Corrupted)?;
        // A record is only ever the text the writer gives it. Any other text
        // has been changed since, even when it means the same: this is where
        // a digest that does not match is found, and a field no record has.
        if envelope.to_text() != text {
            return Err(RecordDefect::Corrupted);
        }
        if envelope.key != *stored_under {
            return Err(RecordDefect::Stale);
        }
        Ok(envelope)
    }

    /// The envelope that the fields of `record` describe, or `None` when
    /// they break a rule of a record. What the writer refuses is not a
    /// record, whatever its digest says.
    fn admitted(record: &Value) -> Option<Self> {
        let key = RecordKey::from_hex(record.get(KEY)?.as_str()?.to_owned())?;
        let stage = StageIdentity::from_json(record)?;
        let payload_version = record
            .get(PAYLOAD_VERSION)
            .and_then(Value::as_u64)
            .and_then(|version| u32::try_from(version).ok())?;
        // A table meets the rules when it is the table Lens makes from its
        // own elements, in their own order.
        let written = ElementTable::from_json(record.get(ROOTS)?, record.get(ELEMENTS)?)?;
        let elements = table_of(written.in_order().cloned().collect()).ok()?;
        (elements == written).then_some(Self {
            key,
            stage,
            payload_version,
            elements,
        })
    }

    /// This record as canonical JSON text, with its digest.
    fn to_text(&self) -> String {
        let mut record = fields::map(self.stage.fields().into_iter().chain([
            (ENVELOPE_VERSION, Value::from(Self::VERSION)),
            (KEY, Value::from(self.key.to_string())),
            (PAYLOAD_VERSION, Value::from(self.payload_version)),
            (ROOTS, self.elements.roots_to_json()),
            (ELEMENTS, self.elements.elements_to_json()),
        ]));
        let digest = digest_of(record.clone());
        record.insert(DIGEST.to_owned(), Value::from(digest));
        canonical::to_text(Value::Object(record))
    }
}

/// What identifies the stage of payload `P` at work on `module`, when they
/// can enter Lens.
pub(crate) fn admit<P: PayloadType>(module: &str) -> Result<StageIdentity, RecordRefusal> {
    let payload_kind = payload::kind_of::<P>()?;
    // The payload kind is valid, so only the module can be refused now.
    StageIdentity::admitted(module, payload_kind).ok_or_else(|| RecordRefusal::AbsoluteModulePath {
        module: module.to_owned(),
    })
}

/// The element table of `elements`, given in the order a stage produced
/// them, with the children of every element and the roots worked out from
/// the parents the elements name.
///
/// Every rule of the element table is applied here, and a table is made
/// nowhere else.
fn table_of(elements: Vec<Element>) -> Result<ElementTable, RecordRefusal> {
    let mut table = ElementTable::default();
    let mut parents = Vec::with_capacity(elements.len());
    for mut element in elements {
        admit_element(&element)?;
        // The children are worked out here, from the parents alone: an
        // element read from another record still lists the ones it had.
        element.children.clear();
        let id = element.id.clone();
        parents.push((id.clone(), element.parent.clone()));
        if table.elements.insert(id.clone(), element).is_some() {
            return Err(RecordRefusal::DuplicateElement { element: id });
        }
    }
    for (id, parent) in parents {
        match parent {
            None => table.roots.push(id),
            Some(parent) => match table.elements.get_mut(&parent) {
                Some(parent) => parent.children.push(id),
                None => {
                    return Err(RecordRefusal::MissingParent {
                        element: id,
                        parent,
                    });
                }
            },
        }
    }
    match outside_every_root(&table) {
        Some(element) => Err(RecordRefusal::ParentCycle { element }),
        None => Ok(table),
    }
}

/// Checks the rules that `element` meets on its own.
fn admit_element(element: &Element) -> Result<(), RecordRefusal> {
    let depth = canonical::depth(&element.payload);
    if depth > Envelope::MAX_PAYLOAD_DEPTH {
        return Err(RecordRefusal::PayloadTooDeep {
            element: element.id.clone(),
            depth,
        });
    }
    let from_an_absolute_path = |span: &&SourceSpan| is_absolute_path(&span.file);
    if let Some(span) = element.provenance.source.iter().find(from_an_absolute_path) {
        return Err(RecordRefusal::AbsoluteSourcePath {
            element: element.id.clone(),
            file: span.file.clone(),
        });
    }
    for reason in &element.reasons {
        let depth = canonical::depth(&reason.because);
        if depth > MAX_REASON_DEPTH {
            return Err(RecordRefusal::ReasonTooDeep {
                element: element.id.clone(),
                depth,
            });
        }
    }
    Ok(())
}

/// An element of `table` that no root leads to, if there is one: it is its
/// own ancestor. Every element of `table` is listed once, under its parent
/// or as a root.
fn outside_every_root(table: &ElementTable) -> Option<StableId> {
    let mut reached = BTreeSet::new();
    let mut next: Vec<&StableId> = table.roots.iter().collect();
    while let Some(id) = next.pop() {
        reached.insert(id);
        next.extend(&table.elements[id].children);
    }
    table
        .elements
        .keys()
        .find(|id| !reached.contains(id))
        .cloned()
}

/// Whether `name` is an absolute path, on any platform: from a root (`/…`,
/// `\…`), a home directory (`~…`) or a drive (`C:\…`).
pub(crate) fn is_absolute_path(name: &str) -> bool {
    let name = name.as_bytes();
    let from_a_drive = name.len() >= 3
        && name[0].is_ascii_alphabetic()
        && name[1] == b':'
        && matches!(name[2], b'/' | b'\\');
    matches!(name.first(), Some(b'/' | b'\\' | b'~')) || from_a_drive
}

/// Why Lens refuses a stage, or the record of what it produced.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordRefusal {
    /// The payload type declares an invalid payload kind.
    InvalidPayloadKind(InvalidPayloadKind),
    /// The module is named by an absolute path, which no record holds.
    AbsoluteModulePath {
        /// The module as named.
        module: String,
    },
    /// The payload of an element nests deeper than
    /// [`Envelope::MAX_PAYLOAD_DEPTH`], so no reader could read the record
    /// back.
    PayloadTooDeep {
        /// The element.
        element: StableId,
        /// How deep its payload nests.
        depth: usize,
    },
    /// A reason of an element nests deeper than [`Reason::MAX_DEPTH`], so no
    /// reader could read the record back.
    ReasonTooDeep {
        /// The element.
        element: StableId,
        /// How deep the justification of the reason nests.
        depth: usize,
    },
    /// An element comes from a source file named by an absolute path, which
    /// no record holds.
    AbsoluteSourcePath {
        /// The element.
        element: StableId,
        /// The source file as named.
        file: String,
    },
    /// Two elements of the record have one stable identifier.
    DuplicateElement {
        /// The stable identifier of both.
        element: StableId,
    },
    /// An element names a parent that is not in the record.
    MissingParent {
        /// The element.
        element: StableId,
        /// The parent it names.
        parent: StableId,
    },
    /// An element is its own ancestor, so no root leads to it.
    ParentCycle {
        /// The element.
        element: StableId,
    },
}

impl fmt::Display for RecordRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayloadKind(error) => error.fmt(f),
            Self::AbsoluteModulePath { module } => write!(
                f,
                "the module `{module}` is named by an absolute path, which a record never holds"
            ),
            Self::PayloadTooDeep { element, depth } => write!(
                f,
                "the payload of the element `{element}` nests {depth} deep, but an element's \
                 payload nests at most {} levels",
                Envelope::MAX_PAYLOAD_DEPTH
            ),
            Self::ReasonTooDeep { element, depth } => write!(
                f,
                "a reason of the element `{element}` nests {depth} deep, but a reason nests at \
                 most {} levels",
                Reason::MAX_DEPTH
            ),
            Self::AbsoluteSourcePath { element, file } => write!(
                f,
                "the element `{element}` comes from the source file `{file}`, named by an \
                 absolute path, which a record never holds"
            ),
            Self::DuplicateElement { element } => {
                write!(f, "two elements have the stable identifier `{element}`")
            }
            Self::MissingParent { element, parent } => write!(
                f,
                "the element `{element}` names the parent `{parent}`, which is not in the record"
            ),
            Self::ParentCycle { element } => {
                write!(f, "the element `{element}` is its own ancestor")
            }
        }
    }
}

impl std::error::Error for RecordRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPayloadKind(error) => Some(error),
            _ => None,
        }
    }
}

impl From<InvalidPayloadKind> for RecordRefusal {
    fn from(error: InvalidPayloadKind) -> Self {
        Self::InvalidPayloadKind(error)
    }
}

/// Why a stored text is not a record this toolchain trusts.
///
/// It reads as the end of a sentence about the record, as in "the record
/// `…` is corrupted".
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordDefect {
    /// The text is not a record as Lens writes it, or does not match its
    /// digest.
    Corrupted,
    /// The record was computed from other inputs than those of the record
    /// key it is stored under.
    Stale,
    /// The record has an envelope version this toolchain does not read.
    UnsupportedEnvelopeVersion {
        /// The envelope version of the record.
        found: u64,
        /// The envelope version this toolchain reads: [`Envelope::VERSION`].
        supported: u32,
    },
}

impl fmt::Display for RecordDefect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corrupted => f.write_str("is corrupted"),
            Self::Stale => f.write_str("is stale: it was computed from other inputs"),
            Self::UnsupportedEnvelopeVersion { found, supported } => write!(
                f,
                "has envelope version {found}, but this toolchain reads envelope version \
                 {supported}"
            ),
        }
    }
}

/// The digest of a record: the BLAKE3, in lowercase hexadecimal, of the
/// canonical JSON text of all its fields except the digest.
fn digest_of(record_without_digest: Map<String, Value>) -> String {
    let text = canonical::to_text(Value::Object(record_without_digest));
    hash::hex(&hash::of(text.as_bytes()))
}

/// The name of the definition, in the published schema, of a name in the
/// workspace.
pub(crate) const NAME_DEFINITION: &str = "workspace_name";

/// The JSON Schema of a name in the workspace, such as a module or a source
/// file: any text that [`is_absolute_path`] does not refuse.
fn name_schema() -> Value {
    json!({
        "description": "A name in the workspace, such as a module or a source file. Never an \
                        absolute path.",
        "type": "string",
        "not": { "pattern": r"^([/\\~]|[A-Za-z]:[/\\])" }
    })
}

/// The JSON Schema of a hash with `description`, as 64 hexadecimal digits.
fn hash_schema(description: &str) -> Value {
    json!({ "description": description, "type": "string", "pattern": "^[0-9a-f]{64}$" })
}

/// The JSON Schema of everything a record is made of, each under the name of
/// its definition.
fn definitions() -> Map<String, Value> {
    let mut definitions: Map<String, Value> = provenance::definitions()
        .into_iter()
        .map(|(name, schema)| (name.to_owned(), schema))
        .collect();
    definitions.insert(NAME_DEFINITION.to_owned(), name_schema());
    definitions.insert(payload::KIND_DEFINITION.to_owned(), payload::kind_schema());
    definitions.insert(element::ID_DEFINITION.to_owned(), element::id_schema());
    definitions.insert(element::DEFINITION.to_owned(), element::schema());
    definitions.insert(reason::DEFINITION.to_owned(), reason::schema());
    definitions
}

/// The JSON Schema of what every record shares.
pub(crate) fn schema() -> Value {
    let [module, payload_kind] = StageIdentity::schema_fields();
    let elements = json!({
        "description": "The element table: every element of what the stage produced, under its \
                        stable identifier. The table is flat, whatever the structure of the \
                        stage.",
        "type": "object",
        "propertyNames": reference(element::ID_DEFINITION),
        "additionalProperties": reference(element::DEFINITION)
    });
    let mut schema = object_schema(
        "A record: what one stage produced for one module, or what one run did.",
        &[
            (ENVELOPE_VERSION, json!({ "const": Envelope::VERSION })),
            (
                KEY,
                hash_schema(
                    "The record key: the BLAKE3 of every input of the record. A record is \
                     stored under its key.",
                ),
            ),
            module,
            payload_kind,
            (PAYLOAD_VERSION, json!({ "type": "integer", "minimum": 0 })),
            (
                ROOTS,
                list_schema(
                    "The elements that are inside no other, in their order.",
                    element::ID_DEFINITION,
                ),
            ),
            (ELEMENTS, elements),
            (
                DIGEST,
                hash_schema(
                    "The BLAKE3 of this record without this field, as JSON text with sorted \
                     keys, an indent of two spaces and a final newline.",
                ),
            ),
        ],
    );
    schema["$schema"] = Value::from("https://json-schema.org/draft/2020-12/schema");
    schema["title"] = Value::from("Brio Lens record");
    schema["$defs"] = Value::Object(definitions());
    schema
}

/// The part of the schema that holds for records of one payload.
pub(crate) fn schema_case(payload: &PayloadSchema) -> Value {
    let properties = fields::object([
        (stage::PAYLOAD_KIND, json!({ "const": payload.kind })),
        (PAYLOAD_VERSION, json!({ "const": payload.version })),
        (
            ELEMENTS,
            json!({ "additionalProperties": element::schema_case(&payload.fragment) }),
        ),
    ]);
    json!({ "properties": properties })
}
