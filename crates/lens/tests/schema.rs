// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Schema assembly, through the public interface of `brio_lens`.

use brio_lens::json::{self, Value, json};
use brio_lens::{InvalidPayloadKind, Payload, PayloadSchema, Schema, SchemaError};

/// A payload no phase owns: declared here, outside `brio_lens`.
struct Probe;

impl Payload for Probe {
    const KIND: &'static str = "probe";
    const VERSION: u32 = 3;

    fn schema_fragment() -> Value {
        json!({
            "type": "object",
            "required": ["reading"],
            "properties": { "reading": { "type": "integer" } }
        })
    }

    fn to_json(&self) -> Value {
        Value::Null
    }

    fn from_json(_payload: &Value) -> Option<Self> {
        Some(Self)
    }
}

/// What `P` declares, when its payload kind is valid.
fn of<P: Payload>() -> PayloadSchema {
    PayloadSchema::of::<P>().expect("the payload kind is valid")
}

#[test]
fn assembled_schema_includes_the_payload_fragment_and_the_combined_version() {
    let schema = Schema::assemble([of::<Probe>()]).expect("one payload assembles");

    assert_eq!(schema.version().to_string(), "1+probe.3");

    let document: Value = json::from_str(&schema.to_json()).expect("the schema is JSON");
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
    const KIND: &'static str = "gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "string" })
    }

    fn to_json(&self) -> Value {
        Value::Null
    }

    fn from_json(_payload: &Value) -> Option<Self> {
        Some(Self)
    }
}

#[test]
fn the_same_payload_set_gives_byte_identical_output_in_any_order() {
    let one_way = Schema::assemble([of::<Probe>(), of::<Gauge>()]).expect("two payloads assemble");
    let other_way =
        Schema::assemble([of::<Gauge>(), of::<Probe>()]).expect("two payloads assemble");

    assert_eq!(one_way.version().to_string(), "1+gauge.1+probe.3");
    assert_eq!(other_way.version().to_string(), "1+gauge.1+probe.3");
    assert_eq!(one_way.to_json(), other_way.to_json());
}

/// A payload that claims the payload kind of [`Probe`].
struct Impostor;

impl Payload for Impostor {
    const KIND: &'static str = "probe";
    const VERSION: u32 = 9;

    fn schema_fragment() -> Value {
        json!({ "type": "null" })
    }

    fn to_json(&self) -> Value {
        Value::Null
    }

    fn from_json(_payload: &Value) -> Option<Self> {
        Some(Self)
    }
}

#[test]
fn two_payloads_of_one_payload_kind_do_not_assemble() {
    let error = Schema::assemble([of::<Probe>(), of::<Impostor>()])
        .expect_err("one payload kind cannot have two payloads");

    assert_eq!(
        error,
        SchemaError::DuplicatePayloadKind {
            payload_kind: "probe"
        }
    );
    assert_eq!(
        error.to_string(),
        "two payloads declare the payload kind `probe`"
    );
}

/// A payload whose fragment declares its keys out of order.
struct Label;

impl Payload for Label {
    const KIND: &'static str = "label";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "string", "minLength": 1, "description": "A label." })
    }

    fn to_json(&self) -> Value {
        Value::Null
    }

    fn from_json(_payload: &Value) -> Option<Self> {
        Some(Self)
    }
}

#[test]
fn object_keys_are_written_in_sorted_order_whatever_order_a_fragment_declares() {
    let schema = Schema::assemble([of::<Label>()]).expect("one payload assembles");

    let json = schema.to_json();
    let fragment = &json[json
        .find(r#""payload": {"#)
        .expect("the fragment is published")..];
    let position = |key: &str| fragment.find(key).expect("the fragment has this key");
    assert!(position(r#""description""#) < position(r#""minLength""#));
    assert!(position(r#""minLength""#) < position(r#""type""#));
}

/// A payload whose kind would make a schema version ambiguous.
struct Dotted;

impl Payload for Dotted {
    const KIND: &'static str = "probe.3+gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "null" })
    }

    fn to_json(&self) -> Value {
        Value::Null
    }

    fn from_json(_payload: &Value) -> Option<Self> {
        Some(Self)
    }
}

#[test]
fn a_payload_kind_with_other_than_lowercase_letters_digits_and_underscores_does_not_enter_lens() {
    let error = PayloadSchema::of::<Dotted>()
        .expect_err("the kind would read as two payloads in the version");

    assert_eq!(
        error,
        InvalidPayloadKind {
            payload_kind: "probe.3+gauge"
        }
    );
    assert_eq!(
        error.to_string(),
        "the payload kind `probe.3+gauge` is not made of lowercase letters, digits and `_`"
    );
}
