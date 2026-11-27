// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The envelope: what every record shares, whatever its payload.
//!
//! This module owns the envelope's fields and its version. The published
//! schema describes what is defined here, and records are written and read
//! here, so the two cannot drift.

use crate::json::{Map, Value, json};

use crate::hash;
use crate::{RecordKey, canonical};

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

    /// The canonical JSON text of the record made of this envelope and
    /// `payload`, with its digest.
    pub(crate) fn seal(&self, payload: Value) -> String {
        let mut record = Map::new();
        record.insert(ENVELOPE_VERSION.to_owned(), Value::from(Self::VERSION));
        record.insert(KEY.to_owned(), Value::from(self.key.to_string()));
        record.insert(MODULE.to_owned(), Value::from(self.module.as_str()));
        record.insert(
            PAYLOAD_KIND.to_owned(),
            Value::from(self.payload_kind.as_str()),
        );
        record.insert(
            PAYLOAD_VERSION.to_owned(),
            Value::from(self.payload_version),
        );
        record.insert(PAYLOAD.to_owned(), payload);
        let digest = digest_of(record.clone());
        record.insert(DIGEST.to_owned(), Value::from(digest));
        canonical::to_text(Value::Object(record))
    }

    /// The envelope and the payload of the record written as `text`.
    ///
    /// The envelope version is checked first: no other field of a record
    /// with another envelope version is interpreted.
    pub(crate) fn open(text: &str) -> Result<(Self, Value), Unreadable> {
        let Ok(Value::Object(mut record)) = crate::json::from_str(text) else {
            return Err(Unreadable::Corrupted);
        };

        let found = record
            .get(ENVELOPE_VERSION)
            .and_then(Value::as_u64)
            .ok_or(Unreadable::Corrupted)?;
        if found != u64::from(Self::VERSION) {
            return Err(Unreadable::UnsupportedEnvelopeVersion { found });
        }

        // A record is only ever written as canonical JSON, so any other
        // text has been changed since, even when it means the same.
        if canonical::to_text(Value::Object(record.clone())) != text {
            return Err(Unreadable::Corrupted);
        }
        let digest = record.remove(DIGEST).ok_or(Unreadable::Corrupted)?;
        if digest.as_str() != Some(digest_of(record.clone()).as_str()) {
            return Err(Unreadable::Corrupted);
        }

        let text_of = |record: &Map<String, Value>, field: &str| {
            record
                .get(field)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or(Unreadable::Corrupted)
        };
        let envelope = Self {
            key: RecordKey::from_hex(text_of(&record, KEY)?).ok_or(Unreadable::Corrupted)?,
            module: text_of(&record, MODULE)?,
            payload_kind: text_of(&record, PAYLOAD_KIND)?,
            payload_version: record
                .get(PAYLOAD_VERSION)
                .and_then(Value::as_u64)
                .and_then(|version| u32::try_from(version).ok())
                .ok_or(Unreadable::Corrupted)?,
        };
        let payload = record.remove(PAYLOAD).ok_or(Unreadable::Corrupted)?;
        Ok((envelope, payload))
    }
}

/// Why a text is not a record this toolchain can read.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Unreadable {
    /// The text is not a record, is not canonical JSON, or does not match
    /// its digest.
    Corrupted,
    /// The record has an envelope version other than [`Envelope::VERSION`].
    UnsupportedEnvelopeVersion {
        /// The envelope version of the record.
        found: u64,
    },
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
pub(crate) fn schema_case(payload_kind: &str, payload_version: u32, fragment: &Value) -> Value {
    let mut properties = Map::new();
    properties.insert(PAYLOAD_KIND.to_owned(), json!({ "const": payload_kind }));
    properties.insert(
        PAYLOAD_VERSION.to_owned(),
        json!({ "const": payload_version }),
    );
    properties.insert(PAYLOAD.to_owned(), fragment.clone());
    json!({ "properties": properties })
}
