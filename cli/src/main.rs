// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The `brio` command.

use std::io::{self, Write};
use std::process::ExitCode;

use brio_lens::{PayloadSchema, Schema};

/// The exit code for a command line `brio` does not understand.
const USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();

    match words.as_slice() {
        ["lens", "schema"] => lens_schema(),
        [] => usage_error(),
        _ => {
            eprintln!("brio: unknown command `{}`", words.join(" "));
            usage_error()
        }
    }
}

/// Says how to call `brio`, and fails.
fn usage_error() -> ExitCode {
    eprintln!("usage: brio lens schema");
    ExitCode::from(USAGE_ERROR)
}

/// The payloads of every phase linked into this binary.
///
/// Each phase crate adds its payloads here in the change that introduces it.
fn linked_payloads() -> Vec<PayloadSchema> {
    Vec::new()
}

/// `brio lens schema`: prints the published record schema.
fn lens_schema() -> ExitCode {
    let schema = match Schema::assemble(linked_payloads()) {
        Ok(schema) => schema,
        Err(error) => {
            eprintln!("brio: {error}");
            return ExitCode::FAILURE;
        }
    };

    match io::stdout().write_all(schema.to_json().as_bytes()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("brio: cannot write the schema: {error}");
            ExitCode::FAILURE
        }
    }
}
