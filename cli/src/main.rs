// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The `brio` command.

use std::error::Error;
use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use brio_lens::{Envelope, PayloadSchema, RecordReader, Schema, StageDifference, StageTest};
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
    /// Compare produced records against reference records
    Test {
        /// The directory of the produced records, laid out as a build directory
        produced: PathBuf,
        /// The directory of the reference records, laid out as a build directory
        reference: PathBuf,
    },
}

/// How a command that ran to its end ended.
///
/// A command that could not run to its end gives an error instead.
enum Outcome {
    /// The command did what it was asked.
    Succeeded,
    /// The command ran, and what it found is a failure: it carries its own
    /// report, which is its result and not an error.
    Failed {
        /// What the command found, as the lines it prints.
        report: String,
    },
}

fn main() -> ExitCode {
    let outcome = match Brio::parse().command {
        Command::Lens(LensCommand::Schema) => lens_schema(),
        Command::Lens(LensCommand::Test {
            produced,
            reference,
        }) => lens_test(&produced, &reference),
    };

    // The one place that says what exit status each end of a command has.
    match outcome {
        Ok(Outcome::Succeeded) => ExitCode::SUCCESS,
        Ok(Outcome::Failed { report }) => {
            print!("{report}");
            ExitCode::FAILURE
        }
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
fn lens_schema() -> Result<Outcome, Box<dyn Error>> {
    let schema = Schema::assemble(linked_payloads())?;
    io::stdout().write_all(schema.to_json().as_bytes())?;
    Ok(Outcome::Succeeded)
}

/// `brio lens test`: compares the records in the directory `produced`
/// against the reference records in the directory `reference`, and fails
/// with each differing stage and element when they differ.
fn lens_test(produced: &Path, reference: &Path) -> Result<Outcome, Box<dyn Error>> {
    let produced = record_set(produced)?;
    let reference = record_set(reference)?;
    let test = StageTest::compare(&produced, &reference)?;
    if test.passed() {
        return Ok(Outcome::Succeeded);
    }
    let report = test.differences().iter().map(difference_report).collect();
    Ok(Outcome::Failed { report })
}

/// The envelope of every record in `directory`, which is laid out as a
/// build directory.
fn record_set(directory: &Path) -> Result<Vec<Envelope>, Box<dyn Error>> {
    // A directory that is not there has no record, and so would compare as
    // an empty record set: a mistyped name must not pass a test.
    if !directory.is_dir() {
        return Err(format!("`{}` is not a directory", directory.display()).into());
    }
    let mut envelopes = Vec::new();
    for record in RecordReader::in_build_directory(directory).read_all()? {
        envelopes.push(record?.envelope);
    }
    Ok(envelopes)
}

/// The lines that `brio lens test` prints for one differing stage: the
/// stage, then each differing element under its stable identifier.
fn difference_report(difference: &StageDifference) -> String {
    let versions = (
        difference.produced_payload_version(),
        difference.reference_payload_version(),
    );
    let how = match versions {
        (Some(_), None) => "has no reference record",
        (None, Some(_)) => "was not produced",
        _ => "differs from its reference record",
    };
    let mut report = format!(
        "the `{}` record of module `{}` {how}:\n",
        difference.payload_kind(),
        difference.module()
    );
    // Writing into a `String` never fails.
    if let (Some(produced), Some(reference)) = versions
        && produced != reference
    {
        let _ = writeln!(
            report,
            "  payload version: {produced}, reference: {reference}"
        );
    }
    if difference.roots_reordered() {
        report.push_str("  roots: in another order\n");
    }
    let elements = [
        ("added", difference.added()),
        ("removed", difference.removed()),
        ("changed", difference.changed()),
    ];
    for (how, elements) in elements {
        for element in elements {
            let _ = writeln!(report, "  {how}: {element}");
        }
    }
    report
}
