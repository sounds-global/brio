// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Schema assembly, through the public interface of `brio_lens`.

use brio_lens::{Payload, PayloadSchema, Schema, SchemaError};
use serde_json::{Value, json};

/// A payload no phase owns: declared here, outside `brio_lens`.
struct Probe;

impl Payload for Probe {
    const STAGE_KIND: &'static str = "probe";
    const VERSION: u32 = 3;

    fn schema_fragment() -> Value {
        json!({
            "type": "object",
            "required": ["reading"],
            "properties": { "reading": { "type": "integer" } }
        })
    }
}

#[test]
fn assembled_schema_includes_the_payload_fragment_and_the_combined_version() {
    let schema = Schema::assemble([PayloadSchema::of::<Probe>()]).expect("one payload assembles");

    assert_eq!(schema.version(), "1+probe.3");

    let document: Value = serde_json::from_str(&schema.to_json()).expect("the schema is JSON");
    assert_eq!(document["version"], "1+probe.3");
    assert_eq!(
        document["oneOf"],
        json!([{
            "properties": {
                "payload_kind": { "const": "probe" },
                "payload_version": { "const": 3 },
                "payload": {
                    "type": "object",
                    "required": ["reading"],
                    "properties": { "reading": { "type": "integer" } }
                }
            }
        }])
    );
}

/// A second payload, so that the order of a payload set can vary.
struct Gauge;

impl Payload for Gauge {
    const STAGE_KIND: &'static str = "gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "string" })
    }
}

#[test]
fn the_same_payload_set_gives_byte_identical_output_in_any_order() {
    let one_way = Schema::assemble([PayloadSchema::of::<Probe>(), PayloadSchema::of::<Gauge>()])
        .expect("two payloads assemble");
    let other_way = Schema::assemble([PayloadSchema::of::<Gauge>(), PayloadSchema::of::<Probe>()])
        .expect("two payloads assemble");

    assert_eq!(one_way.version(), "1+gauge.1+probe.3");
    assert_eq!(other_way.version(), "1+gauge.1+probe.3");
    assert_eq!(one_way.to_json(), other_way.to_json());
}

/// A payload that claims the stage kind of [`Probe`].
struct Impostor;

impl Payload for Impostor {
    const STAGE_KIND: &'static str = "probe";
    const VERSION: u32 = 9;

    fn schema_fragment() -> Value {
        json!({ "type": "null" })
    }
}

#[test]
fn two_payloads_of_one_stage_kind_do_not_assemble() {
    let error = Schema::assemble([
        PayloadSchema::of::<Probe>(),
        PayloadSchema::of::<Impostor>(),
    ])
    .expect_err("one stage kind cannot have two payloads");

    assert_eq!(
        error,
        SchemaError::DuplicateStageKind {
            stage_kind: "probe"
        }
    );
    assert_eq!(
        error.to_string(),
        "two payloads declare the stage kind `probe`"
    );
}
