// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Assembly of the published record schema.

use std::collections::BTreeMap;
use std::fmt;

use crate::json::{Map, Value, json};

use crate::{Envelope, PayloadSchema, canonical, envelope};

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
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePayloadKind { payload_kind } => {
                write!(f, "two payloads declare the payload kind `{payload_kind}`")
            }
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
    /// Fails when two payloads declare the same payload kind.
    pub fn assemble(
        payloads: impl IntoIterator<Item = PayloadSchema>,
    ) -> Result<Self, SchemaError> {
        let mut by_kind = BTreeMap::new();
        for payload in payloads {
            let payload_kind = payload.kind;
            if by_kind.insert(payload_kind, payload).is_some() {
                return Err(SchemaError::DuplicatePayloadKind { payload_kind });
            }
        }

        let version = SchemaVersion {
            envelope: Envelope::VERSION,
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
    /// The text is canonical JSON, as records are: the keys of every object
    /// are sorted, so the same schema always gives the same bytes.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut document = envelope::schema();
        document["version"] = Value::from(self.version.to_string());
        if !self.payloads.is_empty() {
            document["oneOf"] = self.payloads.iter().map(envelope::schema_case).collect();
        }
        canonical::to_text(document)
    }
}

/// A reference, in the published schema, to the definition called `name`.
pub(crate) fn reference(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}

/// A JSON Schema of an object with `description`, in which every one of
/// `properties` is required and no other is admitted.
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
        "properties": properties,
        "additionalProperties": false
    })
}

/// A JSON Schema of a list with `description`, of values that each meet the
/// definition called `definition`.
pub(crate) fn list_schema(description: &str, definition: &str) -> Value {
    json!({ "description": description, "type": "array", "items": reference(definition) })
}

/// A JSON Schema of a text with `description`.
pub(crate) fn text_schema(description: &str) -> Value {
    json!({ "description": description, "type": "string" })
}
