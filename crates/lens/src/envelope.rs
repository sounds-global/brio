// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The envelope: what every record shares, whatever its payload.
//!
//! This module owns the envelope's fields and its version, and decides what
//! a valid record is: every rule a record meets is applied here, the same
//! when a record is made as when it is read. The published schema describes
//! what is defined here, so records and schema cannot drift.

use std::fmt;

use crate::json::{Map, Value, json};

use crate::hash;
use crate::payload::{self, InvalidPayloadKind, Payload};
use crate::{PayloadSchema, RecordInputs, RecordKey, canonical};

/// The JSON name of the envelope version.
const ENVELOPE_VERSION: &str = "envelope_version";
/// The JSON name of the record key.
const KEY: &str = "key";
/// The JSON name of the module the record is for.
const MODULE: &str = "module";
/// The JSON name of the payload kind.
const PAYLOAD_KIND: &str = "payload_kind";
/// The JSON name of the payload version.
const PAYLOAD_VERSION: &str = "payload_version";
/// The JSON name of the payload.
const PAYLOAD: &str = "payload";
/// The JSON name of the digest of all the other fields.
const DIGEST: &str = "digest";

/// Every field of a record. All of them are required.
const FIELDS: [&str; 7] = [
    ENVELOPE_VERSION,
    KEY,
    MODULE,
    PAYLOAD_KIND,
    PAYLOAD_VERSION,
    PAYLOAD,
    DIGEST,
];

/// What a record says about itself, apart from its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub(crate) key: RecordKey,
    pub(crate) module: String,
    pub(crate) payload_kind: String,
    pub(crate) payload_version: u32,
}

impl Envelope {
    /// The version of the envelope this toolchain writes, and the only one
    /// it reads.
    pub const VERSION: u32 = 1;

    /// How deep a payload can nest arrays and objects, as JSON.
    ///
    /// A reader of JSON stops at a fixed depth, and the record writer refuses
    /// any payload a reader could not read back. A stage whose structure
    /// nests deeper writes it flat, with each element naming its parent by
    /// stable identifier.
    pub const MAX_PAYLOAD_DEPTH: usize = 126;

    /// The key of the record that the stage of payload `P` computes from
    /// `inputs`.
    pub(crate) fn key_of<P: Payload>(
        inputs: &RecordInputs,
    ) -> Result<RecordKey, InvalidPayloadKind> {
        Ok(inputs.key(payload::kind_of::<P>()?, P::VERSION))
    }

    /// The record key: the hash of every input of the record.
    #[must_use]
    pub fn key(&self) -> &RecordKey {
        &self.key
    }

    /// The module the record is for.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The payload kind of the record's payload.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        &self.payload_kind
    }

    /// The version of the format of the record's payload.
    #[must_use]
    pub fn payload_version(&self) -> u32 {
        self.payload_version
    }

    /// The record of what the stage of payload `P` produced, as `payload`,
    /// from `inputs`: its envelope, and its canonical JSON text with its
    /// digest.
    pub(crate) fn seal<P: Payload>(
        inputs: &RecordInputs,
        payload: &P,
    ) -> Result<(Self, String), RecordRefusal> {
        let envelope = Self {
            key: Self::key_of::<P>(inputs)?,
            module: inputs.module.clone(),
            payload_kind: P::KIND.to_owned(),
            payload_version: P::VERSION,
        };
        if is_absolute_path(&envelope.module) {
            return Err(RecordRefusal::AbsolutePath {
                module: envelope.module,
            });
        }
        let payload = payload.to_json();
        let depth = canonical::depth(&payload);
        if depth > Self::MAX_PAYLOAD_DEPTH {
            return Err(RecordRefusal::PayloadTooDeep {
                payload_kind: P::KIND,
                depth,
            });
        }

        let mut record = Map::new();
        record.insert(ENVELOPE_VERSION.to_owned(), Value::from(Self::VERSION));
        record.insert(KEY.to_owned(), Value::from(envelope.key.to_string()));
        record.insert(MODULE.to_owned(), Value::from(envelope.module.as_str()));
        record.insert(
            PAYLOAD_KIND.to_owned(),
            Value::from(envelope.payload_kind.as_str()),
        );
        record.insert(
            PAYLOAD_VERSION.to_owned(),
            Value::from(envelope.payload_version),
        );
        record.insert(PAYLOAD.to_owned(), payload);
        let digest = digest_of(record.clone());
        record.insert(DIGEST.to_owned(), Value::from(digest));
        Ok((envelope, canonical::to_text(Value::Object(record))))
    }

    /// The envelope and the payload of the record written as `text`, which
    /// is stored under the record key `stored_under`.
    ///
    /// The envelope version is checked first: no other field of a record
    /// with another envelope version is interpreted.
    pub(crate) fn open(
        text: &str,
        stored_under: &RecordKey,
    ) -> Result<(Self, Value), RecordDefect> {
        // The JSON reader stops at the depth the writer refuses beyond, so a
        // record with a deeper payload is not read as a record at all.
        let Ok(Value::Object(mut record)) = crate::json::from_str(text) else {
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

        // A record is only ever written as canonical JSON, so any other
        // text has been changed since, even when it means the same.
        if canonical::to_text(Value::Object(record.clone())) != text {
            return Err(RecordDefect::Corrupted);
        }
        let digest = record.remove(DIGEST).ok_or(RecordDefect::Corrupted)?;
        if digest.as_str() != Some(digest_of(record.clone()).as_str()) {
            return Err(RecordDefect::Corrupted);
        }

        let text_of = |record: &Map<String, Value>, field: &str| {
            record
                .get(field)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or(RecordDefect::Corrupted)
        };
        let envelope = Self {
            key: RecordKey::from_hex(text_of(&record, KEY)?).ok_or(RecordDefect::Corrupted)?,
            module: text_of(&record, MODULE)?,
            payload_kind: text_of(&record, PAYLOAD_KIND)?,
            payload_version: record
                .get(PAYLOAD_VERSION)
                .and_then(Value::as_u64)
                .and_then(|version| u32::try_from(version).ok())
                .ok_or(RecordDefect::Corrupted)?,
        };
        // What the writer refuses is not a record, whatever its digest says.
        if is_absolute_path(&envelope.module) {
            return Err(RecordDefect::Corrupted);
        }
        if envelope.key != *stored_under {
            return Err(RecordDefect::Stale);
        }
        let payload = record.remove(PAYLOAD).ok_or(RecordDefect::Corrupted)?;
        Ok((envelope, payload))
    }
}

/// Whether `module` is named by an absolute path, on any platform: from a
/// root (`/…`, `\…`), a home directory (`~…`) or a drive (`C:\…`).
fn is_absolute_path(module: &str) -> bool {
    let module = module.as_bytes();
    let from_a_drive = module.len() >= 3
        && module[0].is_ascii_alphabetic()
        && module[1] == b':'
        && matches!(module[2], b'/' | b'\\');
    matches!(module.first(), Some(b'/' | b'\\' | b'~')) || from_a_drive
}

/// Why Lens refuses to make a record.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordRefusal {
    /// The payload declares an invalid payload kind.
    InvalidPayloadKind(InvalidPayloadKind),
    /// The module is named by an absolute path, which no record holds.
    AbsolutePath {
        /// The module as named in the inputs.
        module: String,
    },
    /// The payload nests deeper than [`Envelope::MAX_PAYLOAD_DEPTH`], so no
    /// reader could read the record back.
    PayloadTooDeep {
        /// The payload kind of the payload.
        payload_kind: &'static str,
        /// How deep the payload nests.
        depth: usize,
    },
}

impl fmt::Display for RecordRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayloadKind(error) => error.fmt(f),
            Self::AbsolutePath { module } => write!(
                f,
                "the module `{module}` is named by an absolute path, which a record never holds"
            ),
            Self::PayloadTooDeep {
                payload_kind,
                depth,
            } => write!(
                f,
                "the `{payload_kind}` payload nests {depth} deep, but a record holds at most {} \
                 levels",
                Envelope::MAX_PAYLOAD_DEPTH
            ),
        }
    }
}

impl std::error::Error for RecordRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPayloadKind(error) => Some(error),
            Self::AbsolutePath { .. } | Self::PayloadTooDeep { .. } => None,
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

/// The JSON Schema of what every record shares.
pub(crate) fn schema() -> Value {
    let hash = json!({ "type": "string", "pattern": "^[0-9a-f]{64}$" });
    let mut key = hash.clone();
    key["description"] = Value::from(
        "The record key: the BLAKE3 of every input of the record. A record is stored under \
         its key.",
    );
    let mut digest = hash;
    digest["description"] = Value::from(
        "The BLAKE3 of this record without this field, as JSON text with sorted keys, an \
         indent of two spaces and a final newline.",
    );

    let mut properties = Map::new();
    properties.insert(
        ENVELOPE_VERSION.to_owned(),
        json!({ "const": Envelope::VERSION }),
    );
    properties.insert(KEY.to_owned(), key);
    properties.insert(MODULE.to_owned(), json!({ "type": "string" }));
    properties.insert(PAYLOAD_KIND.to_owned(), json!({ "type": "string" }));
    properties.insert(
        PAYLOAD_VERSION.to_owned(),
        json!({ "type": "integer", "minimum": 0 }),
    );
    properties.insert(PAYLOAD.to_owned(), json!({}));
    properties.insert(DIGEST.to_owned(), digest);

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Brio Lens record",
        "type": "object",
        "required": FIELDS,
        "properties": properties
    })
}

/// The part of the schema that holds for records of one payload.
pub(crate) fn schema_case(payload: &PayloadSchema) -> Value {
    let mut properties = Map::new();
    properties.insert(PAYLOAD_KIND.to_owned(), json!({ "const": payload.kind }));
    properties.insert(
        PAYLOAD_VERSION.to_owned(),
        json!({ "const": payload.version }),
    );
    properties.insert(PAYLOAD.to_owned(), payload.fragment.clone());
    json!({ "properties": properties })
}
