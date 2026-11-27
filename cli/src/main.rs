// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The `brio` command.

use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

use brio_lens::{PayloadSchema, Schema};
use clap::{Parser, Subcommand};

/// The Brio language toolchain.
#[derive(Parser)]
#[command(name = "brio", version)]
struct Brio {
    #[command(subcommand)]
    command: Command,
}

/// A command of `brio`. Each one appears in the change that implements it.
#[derive(Subcommand)]
enum Command {
    /// Work with Lens records
    #[command(subcommand)]
    Lens(LensCommand),
}

/// A command of `brio lens`.
#[derive(Subcommand)]
enum LensCommand {
    /// Print the published record schema
    Schema,
}

fn main() -> ExitCode {
    let outcome = match Brio::parse().command {
        Command::Lens(LensCommand::Schema) => lens_schema(),
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("brio: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The payloads of every phase linked into this binary.
///
/// Each phase crate adds its payloads here in the change that introduces it.
fn linked_payloads() -> Vec<PayloadSchema> {
    Vec::new()
}

/// `brio lens schema`: prints the published record schema.
fn lens_schema() -> Result<(), Box<dyn Error>> {
    let schema = Schema::assemble(linked_payloads())?;
    io::stdout().write_all(schema.to_json().as_bytes())?;
    Ok(())
}
