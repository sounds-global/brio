// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The `brio` command line, run as a process.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use brio_lens::json::{Value, json};
use brio_lens::{Element, ElementTable, Payload, PayloadType, RecordInputs, RecordWriter, Stage};

/// The published schema while no phase is linked: the envelope alone.
const ENVELOPE_SCHEMA: &str = r##"{
  "$defs": {
    "element": {
      "additionalProperties": false,
      "description": "One element of what the stage produced.",
      "properties": {
        "children": {
          "description": "The elements inside this one, in their order. Each names this element as its parent.",
          "items": {
            "$ref": "#/$defs/stable_identifier"
          },
          "type": "array"
        },
        "masked": {
          "description": "The masked markers of the payload: every place in it where a value was masked, once each, in the order of their text. No marker is inside another.",
          "items": {
            "$ref": "#/$defs/masked_marker"
          },
          "type": "array",
          "uniqueItems": true
        },
        "parent": {
          "description": "The element this one is inside, or null for a root.",
          "oneOf": [
            {
              "$ref": "#/$defs/stable_identifier"
            },
            {
              "type": "null"
            }
          ]
        },
        "payload": {
          "description": "What the phase says about this element alone, in the form its payload kind defines."
        },
        "provenance": {
          "$ref": "#/$defs/provenance"
        },
        "reasons": {
          "description": "The reason for every decision the phase made about this element.",
          "items": {
            "$ref": "#/$defs/reason"
          },
          "type": "array"
        }
      },
      "required": [
        "parent",
        "children",
        "payload",
        "masked",
        "provenance",
        "reasons"
      ],
      "type": "object"
    },
    "element_link": {
      "additionalProperties": false,
      "description": "A link to one element of a record. The record is the one with this module and this payload kind, and the element is under this stable identifier in its element table.",
      "properties": {
        "id": {
          "$ref": "#/$defs/stable_identifier"
        },
        "module": {
          "$ref": "#/$defs/workspace_name"
        },
        "payload_kind": {
          "$ref": "#/$defs/payload_kind"
        }
      },
      "required": [
        "module",
        "payload_kind",
        "id"
      ],
      "type": "object"
    },
    "masked_marker": {
      "description": "A masked marker: the place, as a JSON Pointer (RFC 6901), where a confidential value was masked. The record holds null at that place and nothing of the value.",
      "format": "json-pointer",
      "type": "string"
    },
    "origin_step": {
      "additionalProperties": false,
      "description": "One step of a chain of origin: what made a compiler-created element, and the element it was made from.",
      "properties": {
        "by": {
          "description": "What made the element at this step, such as inlining, monomorphization or compile-time evaluation.",
          "type": "string"
        },
        "from": {
          "$ref": "#/$defs/element_link"
        }
      },
      "required": [
        "by",
        "from"
      ],
      "type": "object"
    },
    "payload_kind": {
      "description": "A payload kind: the name of one payload type, made of lowercase letters, digits and `_`.",
      "pattern": "^[a-z0-9_]+$",
      "type": "string"
    },
    "provenance": {
      "additionalProperties": false,
      "description": "The provenance of an element: where it comes from.",
      "properties": {
        "derived_from": {
          "description": "The elements of the previous stage that the element derives from.",
          "items": {
            "$ref": "#/$defs/element_link"
          },
          "type": "array"
        },
        "origin": {
          "description": "The chain of origin of an element the compiler created: every step that made it, starting with the last. Empty for any other element. The element the last step was made from comes from source text, or has a chain of its own.",
          "items": {
            "$ref": "#/$defs/origin_step"
          },
          "type": "array"
        },
        "source": {
          "description": "The spans of source text the element comes from.",
          "items": {
            "$ref": "#/$defs/source_span"
          },
          "type": "array"
        }
      },
      "required": [
        "source",
        "derived_from",
        "origin"
      ],
      "type": "object"
    },
    "reason": {
      "additionalProperties": false,
      "description": "A reason: the recorded justification for one decision a phase made about an element.",
      "properties": {
        "because": {
          "description": "What justifies the decision, in the form the phase defines."
        },
        "decision": {
          "description": "The phase's name for what it decided.",
          "type": "string"
        },
        "involves": {
          "description": "The elements the justification is about.",
          "items": {
            "$ref": "#/$defs/element_link"
          },
          "type": "array"
        },
        "masked": {
          "description": "The masked markers of the justification: every place in it where a value was masked, once each, in the order of their text. No marker is inside another.",
          "items": {
            "$ref": "#/$defs/masked_marker"
          },
          "type": "array",
          "uniqueItems": true
        }
      },
      "required": [
        "decision",
        "because",
        "masked",
        "involves"
      ],
      "type": "object"
    },
    "source_span": {
      "additionalProperties": false,
      "description": "A span of source text: some bytes of one source file.",
      "properties": {
        "end": {
          "description": "Where the span ends: how many bytes of the file are before its end. The byte at this place is not in the span.",
          "minimum": 0,
          "type": "integer"
        },
        "file": {
          "$ref": "#/$defs/workspace_name"
        },
        "start": {
          "description": "Where the span starts: how many bytes of the file are before it.",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "file",
        "start",
        "end"
      ],
      "type": "object"
    },
    "stable_identifier": {
      "description": "A stable identifier: the name of one element, the same in every build.",
      "pattern": "^[0-9a-f]{32}$",
      "type": "string"
    },
    "workspace_name": {
      "description": "A name in the workspace, such as a module or a source file. Never an absolute path.",
      "not": {
        "pattern": "^([/\\\\~]|[A-Za-z]:[/\\\\])"
      },
      "type": "string"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "description": "A record: what one stage produced for one module, or what one run did.",
  "properties": {
    "digest": {
      "description": "The BLAKE3 of this record without this field, as JSON text with sorted keys, an indent of two spaces and a final newline.",
      "pattern": "^[0-9a-f]{64}$",
      "type": "string"
    },
    "elements": {
      "additionalProperties": {
        "$ref": "#/$defs/element"
      },
      "description": "The element table: every element of what the stage produced, under its stable identifier. The table is flat, whatever the structure of the stage.",
      "propertyNames": {
        "$ref": "#/$defs/stable_identifier"
      },
      "type": "object"
    },
    "envelope_version": {
      "const": 1
    },
    "key": {
      "description": "The record key: the BLAKE3 of every input of the record. A record is stored under its key.",
      "pattern": "^[0-9a-f]{64}$",
      "type": "string"
    },
    "module": {
      "$ref": "#/$defs/workspace_name"
    },
    "payload_kind": {
      "$ref": "#/$defs/payload_kind"
    },
    "payload_version": {
      "minimum": 0,
      "type": "integer"
    },
    "roots": {
      "description": "The elements that are inside no other, in their order.",
      "items": {
        "$ref": "#/$defs/stable_identifier"
      },
      "type": "array"
    }
  },
  "required": [
    "envelope_version",
    "key",
    "module",
    "payload_kind",
    "payload_version",
    "roots",
    "elements",
    "digest"
  ],
  "title": "Brio Lens record",
  "type": "object",
  "version": "1"
}
"##;

fn brio(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_brio"))
        .args(arguments)
        .output()
        .expect("the brio binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn lens_schema_prints_the_envelope_schema_with_its_version() {
    let output = brio(&["lens", "schema"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), ENVELOPE_SCHEMA);
    assert_eq!(stderr(&output), "");
}

#[test]
fn lens_schema_prints_the_same_bytes_on_every_run() {
    let first = brio(&["lens", "schema"]);
    let second = brio(&["lens", "schema"]);

    assert!(first.status.success());
    assert!(!first.stdout.is_empty());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn an_unknown_command_is_a_usage_error_that_names_it() {
    let output = brio(&["lens", "polish"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("polish"));
}

#[test]
fn no_command_is_a_usage_error_that_prints_the_full_help() {
    let output = brio(&[]);
    let help = brio(&["--help"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert!(help.status.success());
    assert!(!help.stdout.is_empty());
    assert_eq!(stderr(&output), stdout(&help));
}

#[test]
fn version_prints_the_toolchain_version() {
    let output = brio(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        concat!("brio ", env!("CARGO_PKG_VERSION"), "\n")
    );
}

/// A payload no phase owns: the reading of every sensor a module declares.
struct Probe(Vec<(&'static str, i64)>);

impl PayloadType for Probe {
    const KIND: &'static str = "probe";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "integer" })
    }
}

impl Payload for Probe {
    fn to_elements(&self, stage: &Stage<Self>) -> Vec<Element> {
        let reading = |(sensor, value): &(&str, i64)| {
            Element::new(stage.id(&["sensor", sensor]), json!(value))
        };
        self.0.iter().map(reading).collect()
    }

    fn from_elements(_elements: &ElementTable) -> Option<Self> {
        None
    }
}

/// The probe stage of `module`.
fn probe_stage(module: &str) -> Stage<Probe> {
    Stage::of(module).expect("the payload kind and the module are valid")
}

/// An empty directory for the record sets of the test called `test`.
fn record_sets(test: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    if directory.exists() {
        fs::remove_dir_all(&directory).expect("the old record sets can be removed");
    }
    fs::create_dir_all(&directory).expect("the directory can be created");
    directory
}

/// Writes into the record set in `directory` the record of what the probe
/// stage produced for `module`, with the toolchain `toolchain_version`, and
/// gives the file of the record.
fn write_record(
    directory: &Path,
    module: &str,
    toolchain_version: &str,
    sensors: &[(&'static str, i64)],
) -> PathBuf {
    let inputs = RecordInputs {
        module_inputs: BTreeMap::new(),
        toolchain_version: toolchain_version.to_owned(),
        target: "aarch64-unknown-linux-gnu".to_owned(),
        build_settings: BTreeMap::new(),
    };
    RecordWriter::in_build_directory(directory)
        .write(&probe_stage(module), || &inputs, &Probe(sensors.to_vec()))
        .expect("the record is written")
        .expect("the writer is not the disabled writer")
}

fn lens_test(produced: &Path, reference: &Path) -> Output {
    let path = |directory: &Path| directory.to_str().expect("the path is text").to_owned();
    brio(&["lens", "test", &path(produced), &path(reference)])
}

#[test]
fn lens_test_succeeds_and_prints_nothing_when_the_record_sets_match() {
    let sets = record_sets("lens_test_match");
    let (produced, reference) = (sets.join("produced"), sets.join("reference"));
    write_record(&reference, "app::main", "0.1.0", &[("a", 1), ("b", 2)]);
    write_record(&produced, "app::main", "0.2.0", &[("a", 1), ("b", 2)]);

    let output = lens_test(&produced, &reference);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn lens_test_fails_and_prints_each_differing_stage_and_element() {
    let sets = record_sets("lens_test_differ");
    let (produced, reference) = (sets.join("produced"), sets.join("reference"));
    write_record(
        &reference,
        "app::main",
        "0.1.0",
        &[("a", 1), ("b", 2), ("c", 3)],
    );
    write_record(
        &produced,
        "app::main",
        "0.1.0",
        &[("a", 1), ("b", 5), ("d", 4)],
    );
    write_record(&reference, "app::motor", "0.1.0", &[("e", 1)]);
    write_record(&produced, "app::motor", "0.1.0", &[("e", 1)]);
    write_record(&produced, "app::valve", "0.1.0", &[("f", 1)]);

    let output = lens_test(&produced, &reference);

    let main = probe_stage("app::main");
    let valve = probe_stage("app::valve");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output),
        format!(
            "the `probe` record of module `app::main` differs from its reference record:\n\
             \x20 added: {}\n\
             \x20 removed: {}\n\
             \x20 changed: {}\n\
             the `probe` record of module `app::valve` has no reference record:\n\
             \x20 added: {}\n",
            main.id(&["sensor", "d"]),
            main.id(&["sensor", "c"]),
            main.id(&["sensor", "b"]),
            valve.id(&["sensor", "f"]),
        )
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn lens_test_is_an_error_when_a_record_set_is_not_a_directory() {
    let sets = record_sets("lens_test_no_directory");
    let (produced, reference) = (sets.join("produced"), sets.join("reference"));
    write_record(&produced, "app::main", "0.1.0", &[("a", 1)]);

    let output = lens_test(&produced, &reference);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!("brio: `{}` is not a directory\n", reference.display())
    );
}

#[test]
fn lens_test_is_an_error_when_a_record_cannot_be_read() {
    let sets = record_sets("lens_test_corrupted");
    let (produced, reference) = (sets.join("produced"), sets.join("reference"));
    write_record(&reference, "app::main", "0.1.0", &[("a", 1)]);
    let record = write_record(&produced, "app::main", "0.1.0", &[("a", 1)]);
    fs::write(&record, "{}\n").expect("the record can be changed");

    let output = lens_test(&produced, &reference);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!("brio: the record `{}` is corrupted\n", record.display())
    );
}

#[test]
fn lens_test_is_an_error_when_a_record_set_holds_no_record() {
    let sets = record_sets("lens_test_no_record");
    let (produced, reference) = (sets.join("produced"), sets.join("reference"));
    write_record(&produced, "app::main", "0.1.0", &[("a", 1)]);
    write_record(&reference, "app::main", "0.1.0", &[("a", 2)]);

    // The records directories inside the two build directories.
    let output = lens_test(&produced.join("lens"), &reference.join("lens"));

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "brio: `{}` holds no record\n",
            produced.join("lens").display()
        )
    );
}

#[test]
fn lens_test_is_an_error_when_a_record_set_holds_the_records_of_two_builds() {
    let sets = record_sets("lens_test_two_builds");
    let (produced, reference) = (sets.join("produced"), sets.join("reference"));
    write_record(&reference, "app::main", "0.1.0", &[("a", 1)]);
    write_record(&produced, "app::main", "0.1.0", &[("a", 1)]);
    write_record(&produced, "app::main", "0.2.0", &[("a", 1)]);

    let output = lens_test(&produced, &reference);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "brio: the produced records hold two `probe` records of module `app::main`: the set \
         holds the records of more than one build\n"
    );
}

#[test]
fn lens_test_is_an_error_when_a_record_is_misnamed_in_both_record_sets() {
    let sets = record_sets("lens_test_misnamed");
    let (produced, reference) = (sets.join("produced"), sets.join("reference"));
    // The record of one stage differs between the sets, and is misnamed in
    // both.
    let misnamed_record = |set: &Path, value: i64| {
        let record = write_record(set, "app::main", "0.1.0", &[("a", value)]);
        let misnamed = record.with_extension("JSON");
        fs::rename(&record, &misnamed).expect("the record can be renamed");
        misnamed
    };
    let misnamed = misnamed_record(&produced, 1);
    misnamed_record(&reference, 2);
    // Another stage, the same in both sets.
    write_record(&produced, "app::motor", "0.1.0", &[("e", 1)]);
    write_record(&reference, "app::motor", "0.1.0", &[("e", 1)]);

    let output = lens_test(&produced, &reference);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        format!(
            "brio: `{}` is among the records and is not named for a record key\n",
            misnamed.display()
        )
    );
}
