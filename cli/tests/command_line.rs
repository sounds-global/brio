// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The `brio` command line, run as a process.

use std::process::{Command, Output};

/// The published schema while no phase is linked: the envelope alone.
const ENVELOPE_SCHEMA: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "properties": {
    "digest": {
      "description": "The SHA-256 of this record without this field, as JSON text with sorted keys, an indent of two spaces and a final newline.",
      "pattern": "^[0-9a-f]{64}$",
      "type": "string"
    },
    "envelope_version": {
      "const": 1
    },
    "key": {
      "description": "The record key: the SHA-256 of every input of the record. A record is stored under its key.",
      "pattern": "^[0-9a-f]{64}$",
      "type": "string"
    },
    "module": {
      "type": "string"
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
    "key",
    "module",
    "payload_kind",
    "payload_version",
    "payload",
    "digest"
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
