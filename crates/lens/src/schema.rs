// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Assembly of the published record schema.

use std::fmt;

use serde_json::{Value, json};

use crate::PayloadSchema;

/// The version of the envelope every record is wrapped in.
pub const ENVELOPE_VERSION: u32 = 1;

/// The published schema of every record a toolchain can write.
///
/// It combines the envelope with the payloads of every linked phase.
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    version: String,
    document: Value,
}

/// Why a set of payloads cannot be assembled into one schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaError {
    /// Two payloads declare the same stage kind, so records of that kind
    /// would have no single schema or version.
    DuplicateStageKind {
        /// The stage kind declared twice.
        stage_kind: &'static str,
    },
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateStageKind { stage_kind } => {
                write!(f, "two payloads declare the stage kind `{stage_kind}`")
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
    /// Fails when two payloads declare the same stage kind.
    pub fn assemble(
        payloads: impl IntoIterator<Item = PayloadSchema>,
    ) -> Result<Self, SchemaError> {
        let mut payloads: Vec<PayloadSchema> = payloads.into_iter().collect();
        payloads.sort_by_key(|payload| payload.stage_kind);
        if let Some(pair) = payloads
            .windows(2)
            .find(|pair| pair[0].stage_kind == pair[1].stage_kind)
        {
            return Err(SchemaError::DuplicateStageKind {
                stage_kind: pair[0].stage_kind,
            });
        }

        let mut version = ENVELOPE_VERSION.to_string();
        for payload in &payloads {
            version.push_str(&format!("+{}.{}", payload.stage_kind, payload.version));
        }

        let mut document = envelope_schema();
        document["version"] = Value::from(version.clone());
        if !payloads.is_empty() {
            document["oneOf"] = payloads.into_iter().map(payload_case).collect();
        }

        Ok(Self { version, document })
    }

    /// The version of this schema: the envelope version, then the stage kind
    /// and version of each payload, as in `1+syntax.2+tokens.1`.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// This schema as a JSON Schema document.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!("{:#}\n", self.document)
    }
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
fn payload_case(payload: PayloadSchema) -> Value {
    json!({
        "properties": {
            "payload_kind": { "const": payload.stage_kind },
            "payload_version": { "const": payload.version },
            "payload": payload.fragment
        }
    })
}
