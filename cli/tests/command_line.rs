// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The `brio` command line, run as a process.

use std::process::{Command, Output};

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
