// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The `brio` command.

use std::error::Error;
use std::fmt::{self, Write as _};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use brio_lens::{
    ComparedRecords, Envelope, PayloadSchema, RecordReader, Schema, StageDifference, StageTest,
};
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

    // The one place that says what exit status each end of a command has:
    // 0 when it succeeded, 1 when it failed with its report, and 2 when an
    // error kept it from running to its end, as for a usage error.
    match outcome {
        Ok(Outcome::Succeeded) => ExitCode::SUCCESS,
        Ok(Outcome::Failed { report }) => match io::stdout().write_all(report.as_bytes()) {
            Ok(()) => ExitCode::from(1),
            Err(error) => stopped_by(&error),
        },
        Err(error) => stopped_by(&*error),
    }
}

/// Prints `error`, which kept a command from running to its end, and gives
/// the exit status of such a command.
fn stopped_by(error: &dyn Error) -> ExitCode {
    eprintln!("brio: {error}");
    ExitCode::from(2)
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
    if !directory.is_dir() {
        return Err(NotARecordSet::NotADirectory(directory.to_owned()).into());
    }
    let mut envelopes = Vec::new();
    for record in RecordReader::in_build_directory(directory).read_all()? {
        envelopes.push(record?.envelope);
    }
    // No test compares nothing: a set with no record is the wrong
    // directory, such as the records directory inside a build directory.
    if envelopes.is_empty() {
        return Err(NotARecordSet::NoRecord(directory.to_owned()).into());
    }
    Ok(envelopes)
}

/// Why `brio lens test` takes no record set from a directory.
#[derive(Debug)]
enum NotARecordSet {
    /// There is no such directory.
    NotADirectory(PathBuf),
    /// The directory holds no record.
    NoRecord(PathBuf),
}

impl fmt::Display for NotARecordSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADirectory(directory) => {
                write!(f, "`{}` is not a directory", directory.display())
            }
            Self::NoRecord(directory) => write!(f, "`{}` holds no record", directory.display()),
        }
    }
}

impl Error for NotARecordSet {}

/// The lines that `brio lens test` prints for one differing stage: the
/// stage, then each differing element under its stable identifier.
fn difference_report(difference: &StageDifference) -> String {
    let verdict = match difference.records() {
        ComparedRecords::Both { .. } => "differs from its reference record",
        ComparedRecords::ProducedOnly => "has no reference record",
        ComparedRecords::ReferenceOnly => "was not produced",
    };
    let mut report = format!(
        "the `{}` record of module `{}` {verdict}:\n",
        difference.payload_kind(),
        difference.module()
    );
    // Writing into a `String` never fails.
    if let ComparedRecords::Both {
        produced_payload_version: produced,
        reference_payload_version: reference,
    } = difference.records()
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
    let differing = [
        ("added", difference.added()),
        ("removed", difference.removed()),
        ("changed", difference.changed()),
    ];
    for (how, elements) in differing {
        for element in elements {
            let _ = writeln!(report, "  {how}: {element}");
        }
    }
    report
}
