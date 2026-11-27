// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Assembly of the published record schema.

use std::collections::BTreeMap;
use std::fmt;

use crate::json::{Map, Value, json};

use crate::PayloadSchema;

/// The version of the envelope every record is wrapped in.
const ENVELOPE_VERSION: u32 = 1;

/// The published schema of every record a toolchain can write.
///
/// It combines the envelope with the payloads of every linked phase.
#[derive(Debug)]
pub struct Schema {
    version: SchemaVersion,
    /// One payload for each payload kind, in the order of their kinds.
    payloads: Vec<PayloadSchema>,
}

/// The version of a [`Schema`]: the envelope version plus the version of
/// each payload.
///
/// It is written as the envelope version, then the kind and version of each
/// payload in the order of their kinds, as in `1+syntax.2+tokens.1`.
#[derive(Debug, PartialEq, Eq)]
pub struct SchemaVersion {
    envelope: u32,
    payloads: BTreeMap<&'static str, u32>,
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.envelope)?;
        for (payload_kind, version) in &self.payloads {
            write!(f, "+{payload_kind}.{version}")?;
        }
        Ok(())
    }
}

/// Why a set of payloads cannot be assembled into one schema.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaError {
    /// Two payloads declare the same payload kind, so records of that kind
    /// would have no single schema or version.
    DuplicatePayloadKind {
        /// The payload kind declared twice.
        payload_kind: &'static str,
    },
    /// A payload kind has characters that would make the schema version
    /// ambiguous.
    InvalidPayloadKind {
        /// The payload kind as declared.
        payload_kind: &'static str,
    },
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePayloadKind { payload_kind } => {
                write!(f, "two payloads declare the payload kind `{payload_kind}`")
            }
            Self::InvalidPayloadKind { payload_kind } => write!(
                f,
                "the payload kind `{payload_kind}` has characters other than lowercase letters, \
                 digits and `_`"
            ),
        }
    }
}

impl std::error::Error for SchemaError {}

impl Schema {
    /// Assembles the envelope and `payloads` into one schema.
    ///
    /// The result depends only on the set of payloads, not on their order.
    ///
    /// # Errors
    ///
    /// Fails when two payloads declare the same payload kind, or when a
    /// payload kind is empty or has characters other than lowercase ASCII
    /// letters, digits and `_`.
    pub fn assemble(
        payloads: impl IntoIterator<Item = PayloadSchema>,
    ) -> Result<Self, SchemaError> {
        let mut by_kind = BTreeMap::new();
        for payload in payloads {
            let payload_kind = payload.kind;
            if !is_valid_kind(payload_kind) {
                return Err(SchemaError::InvalidPayloadKind { payload_kind });
            }
            if by_kind.insert(payload_kind, payload).is_some() {
                return Err(SchemaError::DuplicatePayloadKind { payload_kind });
            }
        }

        let version = SchemaVersion {
            envelope: ENVELOPE_VERSION,
            payloads: by_kind
                .iter()
                .map(|(kind, payload)| (*kind, payload.version))
                .collect(),
        };
        Ok(Self {
            version,
            payloads: by_kind.into_values().collect(),
        })
    }

    /// The version of this schema.
    #[must_use]
    pub fn version(&self) -> &SchemaVersion {
        &self.version
    }

    /// This schema as a JSON Schema document.
    ///
    /// The text is canonical: the keys of every object are sorted, so the
    /// same schema always gives the same bytes.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut document = envelope_schema();
        document["version"] = Value::from(self.version.to_string());
        if !self.payloads.is_empty() {
            document["oneOf"] = self.payloads.iter().map(payload_case).collect();
        }
        format!("{:#}\n", with_sorted_keys(document))
    }
}

/// Whether `kind` can appear in a schema version without ambiguity.
fn is_valid_kind(kind: &str) -> bool {
    !kind.is_empty()
        && kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// What every record shares, whatever its payload.
fn envelope_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Brio Lens record",
        "type": "object",
        "required": ["envelope_version", "payload_kind", "payload_version", "payload"],
        "properties": {
            "envelope_version": { "const": ENVELOPE_VERSION },
            "payload_kind": { "type": "string" },
            "payload_version": { "type": "integer", "minimum": 0 },
            "payload": {}
        }
    })
}

/// The part of the schema that holds for records of one payload.
fn payload_case(payload: &PayloadSchema) -> Value {
    json!({
        "properties": {
            "payload_kind": { "const": payload.kind },
            "payload_version": { "const": payload.version },
            "payload": payload.fragment
        }
    })
}

/// `value` with the keys of every object in sorted order.
///
/// `serde_json` only sorts keys itself while its `preserve_order` feature is
/// off, and any crate in a build can turn that feature on.
fn with_sorted_keys(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<(String, Value)> = object.into_iter().collect();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            let sorted: Map<String, Value> = entries
                .into_iter()
                .map(|(key, value)| (key, with_sorted_keys(value)))
                .collect();
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(with_sorted_keys).collect()),
        scalar => scalar,
    }
}
