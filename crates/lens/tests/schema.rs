// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Schema assembly, through the public interface of `brio_lens`.

mod support;

use std::collections::BTreeSet;
use std::fs;

use brio_lens::json::{self, Value, json};
use brio_lens::{
    Element, InvalidPayloadKind, PayloadSchema, PayloadType, Reason, RecordWriter, Schema,
    SchemaError, SourceSpan, masked,
};

use support::{Loose, inputs, stage, workspace};

/// A payload type no phase owns: declared here, outside `brio_lens`. Schema
/// assembly needs what it declares and nothing else.
struct Probe;

impl PayloadType for Probe {
    const KIND: &'static str = "probe";
    const VERSION: u32 = 3;

    fn schema_fragment() -> Value {
        json!({
            "type": "object",
            "required": ["reading"],
            "properties": { "reading": { "type": "integer" } }
        })
    }
}

/// What `P` declares, when its payload kind is valid.
fn declared<P: PayloadType>() -> PayloadSchema {
    PayloadSchema::of::<P>().expect("the payload kind is valid")
}

#[test]
fn assembled_schema_includes_the_payload_fragment_and_the_combined_version() {
    let schema = Schema::assemble([declared::<Probe>()]).expect("one payload assembles");

    assert_eq!(schema.version().to_string(), "1+probe.3");

    let document: Value = json::from_str(&schema.to_json()).expect("the schema is JSON");
    assert_eq!(document["version"], "1+probe.3");
    assert_eq!(
        document["oneOf"],
        json!([{
            "properties": {
                "payload_kind": { "const": "probe" },
                "payload_version": { "const": 3 },
                "elements": {
                    "additionalProperties": {
                        "properties": {
                            "payload": {
                                "type": "object",
                                "required": ["reading"],
                                "properties": { "reading": { "type": "integer" } }
                            }
                        }
                    }
                }
            }
        }])
    );
}

/// A second payload, so that the order of a payload set can vary.
struct Gauge;

impl PayloadType for Gauge {
    const KIND: &'static str = "gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "string" })
    }
}

#[test]
fn the_same_payload_set_gives_byte_identical_output_in_any_order() {
    let one_way = Schema::assemble([declared::<Probe>(), declared::<Gauge>()])
        .expect("two payloads assemble");
    let other_way = Schema::assemble([declared::<Gauge>(), declared::<Probe>()])
        .expect("two payloads assemble");

    assert_eq!(one_way.version().to_string(), "1+gauge.1+probe.3");
    assert_eq!(other_way.version().to_string(), "1+gauge.1+probe.3");
    assert_eq!(one_way.to_json(), other_way.to_json());
}

/// A payload that claims the payload kind of [`Probe`].
struct Impostor;

impl PayloadType for Impostor {
    const KIND: &'static str = "probe";
    const VERSION: u32 = 9;

    fn schema_fragment() -> Value {
        json!({ "type": "null" })
    }
}

#[test]
fn two_payloads_of_one_payload_kind_do_not_assemble() {
    let error = Schema::assemble([declared::<Probe>(), declared::<Impostor>()])
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

impl PayloadType for Label {
    const KIND: &'static str = "label";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "string", "minLength": 1, "description": "A label." })
    }
}

#[test]
fn object_keys_are_written_in_sorted_order_whatever_order_a_fragment_declares() {
    let schema = Schema::assemble([declared::<Label>()]).expect("one payload assembles");

    let json = schema.to_json();
    // The cases of the payloads come last, after everything the envelope defines.
    let cases = &json[json.rfind(r#""oneOf""#).expect("the payload is published")..];
    let fragment = &cases[cases
        .find(r#""payload": {"#)
        .expect("the fragment is published")..];
    let position = |key: &str| fragment.find(key).expect("the fragment has this key");
    assert!(position(r#""description""#) < position(r#""minLength""#));
    assert!(position(r#""minLength""#) < position(r#""type""#));
}

/// A payload whose kind would make a schema version ambiguous.
struct Dotted;

impl PayloadType for Dotted {
    const KIND: &'static str = "probe.3+gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "null" })
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

/// A reference, in the published schema, to the definition called `name`.
fn to(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}

/// The published schema of a toolchain with no payload, as a document.
fn envelope_schema() -> Value {
    let schema = Schema::assemble([]).expect("no payload assembles");
    json::from_str(&schema.to_json()).expect("the schema is JSON")
}

#[test]
fn the_schema_describes_the_provenance_of_every_element() {
    let schema = envelope_schema();
    let defined = |name: &str| &schema["$defs"][name];

    let element = &defined("element")["properties"];
    assert_eq!(element["provenance"], to("provenance"));

    let provenance = &defined("provenance")["properties"];
    assert_eq!(provenance["source"]["items"], to("source_span"));
    assert_eq!(provenance["derived_from"]["items"], to("element_link"));
    assert_eq!(provenance["origin"]["items"], to("origin_step"));

    let span = &defined("source_span")["properties"];
    assert_eq!(span["file"], to("workspace_name"));
    assert_eq!(span["start"]["type"], "integer");
    assert_eq!(span["end"]["type"], "integer");

    let link = &defined("element_link")["properties"];
    assert_eq!(link["module"], to("workspace_name"));
    assert_eq!(link["payload_kind"], to("payload_kind"));
    assert_eq!(link["id"], to("stable_identifier"));

    let step = &defined("origin_step")["properties"];
    assert_eq!(step["by"]["type"], "string");
    assert_eq!(step["from"], to("element_link"));
}

#[test]
fn the_schema_describes_everything_it_defines_in_words() {
    let schema = envelope_schema();

    for name in [
        "element",
        "provenance",
        "source_span",
        "element_link",
        "origin_step",
        "reason",
    ] {
        let described = schema["$defs"][name]["description"]
            .as_str()
            .unwrap_or_default();
        assert!(!described.is_empty(), "`{name}` is not described");
    }
}

#[test]
fn the_schema_describes_the_reasons_of_every_element() {
    let schema = envelope_schema();

    let reasons = &schema["$defs"]["element"]["properties"]["reasons"];
    assert_eq!(reasons["type"], "array");
    assert_eq!(reasons["items"], to("reason"));

    let reason = &schema["$defs"]["reason"]["properties"];
    assert_eq!(reason["decision"]["type"], "string");
    assert!(reason["because"].is_object());
    assert_eq!(reason["involves"]["items"], to("element_link"));
}

/// The names of the fields of the object `value`.
fn fields(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect()
}

/// The names of the fields the schema `definition` requires.
fn required(definition: &Value) -> BTreeSet<&str> {
    let required: BTreeSet<&str> = definition["required"]
        .as_array()
        .expect("the definition requires fields")
        .iter()
        .map(|name| name.as_str().expect("a field has a name"))
        .collect();
    assert_eq!(
        required,
        fields(&definition["properties"]),
        "every field is described and required"
    );
    required
}

#[test]
fn a_written_record_has_exactly_the_fields_the_schema_describes() {
    let build_directory = workspace("schema_fields").join(".brio");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(
            &stage::<Loose>("app::main"),
            &inputs("sensor a\n"),
            &Loose(|stage| {
                let other = stage.link(&["other"]);
                vec![
                    Element::new(stage.id(&["a"]), json!(1))
                        .from_source(SourceSpan {
                            file: "src/main.brio".to_owned(),
                            start: 0,
                            end: 8,
                        })
                        .derived_from(other.clone())
                        .created_by("inlining", other.clone())
                        .because(Reason::new("a", json!("why")).involving(other)),
                ]
            }),
        )
        .expect("the record is written")
        .expect("Lens is on");
    let text = fs::read_to_string(path).expect("the record can be read");
    let record: Value = json::from_str(&text).expect("the record is JSON");
    let schema = envelope_schema();
    let defined = |name: &str| required(&schema["$defs"][name]);

    assert_eq!(fields(&record), required(&schema));
    let (_, element) = record["elements"]
        .as_object()
        .and_then(|elements| elements.iter().next())
        .expect("the record has an element");
    assert_eq!(fields(element), defined("element"));
    let provenance = &element["provenance"];
    assert_eq!(fields(provenance), defined("provenance"));
    assert_eq!(fields(&provenance["source"][0]), defined("source_span"));
    assert_eq!(
        fields(&provenance["derived_from"][0]),
        defined("element_link")
    );
    assert_eq!(fields(&provenance["origin"][0]), defined("origin_step"));
    assert_eq!(fields(&element["reasons"][0]), defined("reason"));
}

#[test]
fn the_schema_admits_no_field_and_no_name_that_a_reader_rejects() {
    let schema = envelope_schema();

    // A record, and everything in it that Lens defines, has the fields the
    // schema describes and no other.
    assert_eq!(schema["additionalProperties"], false);
    for name in [
        "element",
        "provenance",
        "source_span",
        "element_link",
        "origin_step",
        "reason",
    ] {
        assert_eq!(
            schema["$defs"][name]["additionalProperties"], false,
            "`{name}` admits other fields"
        );
    }

    // A module, like a source file, is never named by an absolute path, and
    // a payload kind is made of lowercase letters, digits and `_`.
    assert_eq!(schema["properties"]["module"], to("workspace_name"));
    assert_eq!(schema["properties"]["payload_kind"], to("payload_kind"));
    assert_eq!(
        schema["$defs"]["workspace_name"]["not"],
        json!({ "pattern": r"^([/\\~]|[A-Za-z]:[/\\])" })
    );
    assert_eq!(schema["$defs"]["payload_kind"]["pattern"], "^[a-z0-9_]+$");
}

#[test]
fn the_schema_describes_the_masked_marker_as_it_is_written() {
    let schema = envelope_schema();
    let marker = &schema["$defs"]["masked"];

    assert!(
        !marker["description"]
            .as_str()
            .unwrap_or_default()
            .is_empty()
    );
    assert_eq!(marker["additionalProperties"], false);
    assert_eq!(fields(&masked("hunter2")), required(marker));
    assert_eq!(marker["properties"]["$masked"], json!({ "const": true }));
    assert_eq!(masked("hunter2"), json!({ "$masked": true }));
}
