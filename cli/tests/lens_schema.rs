// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! `brio lens schema`, run as a process.

use std::process::{Command, Output};

/// The published schema while no phase is linked: the envelope alone.
const ENVELOPE_SCHEMA: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "properties": {
    "envelope_version": {
      "const": 1
    },
    "payload": {},
    "payload_kind": {
      "type": "string"
    },
    "payload_version": {
      "minimum": 0,
      "type": "integer"
    }
  },
  "required": [
    "envelope_version",
    "payload_kind",
    "payload_version",
    "payload"
  ],
  "title": "Brio Lens record",
  "type": "object",
  "version": "1"
}
"#;

fn brio(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_brio"))
        .args(arguments)
        .output()
        .expect("the brio binary runs")
}

#[test]
fn lens_schema_prints_the_envelope_schema_with_its_version() {
    let output = brio(&["lens", "schema"]);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), ENVELOPE_SCHEMA);
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
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
fn an_unknown_command_fails_and_says_how_to_call_brio() {
    let output = brio(&["lens", "polish"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "brio: unknown command `lens polish`\nusage: brio lens schema\n"
    );
}

#[test]
fn no_command_fails_and_says_how_to_call_brio() {
    let output = brio(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "usage: brio lens schema\n"
    );
}
