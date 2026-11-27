// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The unfinished records of a writer, through the public interface of
//! `brio_lens` and the files of a build directory.
//!
//! A writer numbers its unfinished records in the whole of its process,
//! from 0. This file has one test, so that nothing else in its process
//! writes a record and the test knows the number of each one: with a second
//! test beside it, it would no longer know.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use brio_lens::{RecordReader, RecordWriter, WriteError};

use support::{Probe, files_under_but_the_lock, inputs, probe, stage, workspace, write_record};

/// What the test leaves in the file of an unfinished record of another
/// writer.
const ANOTHER_WRITERS: &str = "the unfinished record of another writer\n";

/// Leaves, for the record of the file `record`, the unfinished record of
/// another writer under the names with `numbers` that a writer of this
/// process gives. That other writer is of another process with the same
/// process identifier, or was stopped. Gives the files it left.
fn take_unfinished_names(record: &Path, numbers: std::ops::Range<u64>) -> Vec<PathBuf> {
    let process = std::process::id();
    let left: Vec<PathBuf> = numbers
        .map(|number| record.with_extension(format!("{process}-{number}.unfinished")))
        .collect();
    for file in &left {
        fs::write(file, ANOTHER_WRITERS).expect("an unfinished record can be added");
    }
    left
}

/// Checks that each of the files `left` still holds the unfinished record
/// the test left there.
fn assert_untouched(left: &[PathBuf]) {
    for file in left {
        let held = fs::read_to_string(file).ok();
        assert_eq!(held.as_deref(), Some(ANOTHER_WRITERS), "{}", file.display());
    }
}

#[test]
fn a_writer_takes_the_next_free_name_for_its_unfinished_record_and_touches_no_other() {
    let workspace = workspace("unfinished_records");
    let build_directory = workspace.join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    // The first unfinished record of the process, with the number 0.
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
    assert_eq!(
        files_under_but_the_lock(&workspace),
        std::slice::from_ref(&path)
    );

    // The next three names are taken. The same record key with another
    // payload is to replace the record, so the writer needs an unfinished
    // record: it passes the three names and takes the fourth.
    let mut left = take_unfinished_names(&path, 1..4);
    write_record(&build_directory, &stage, &inputs, &probe(&[("a", 8)]));

    assert_untouched(&left);
    let replaced = fs::read(&path).expect("the record can be read");
    assert!(String::from_utf8_lossy(&replaced).contains(r#""value": 8"#));
    // Nothing of the writer's own unfinished record stays behind.
    assert_eq!(files_under_but_the_lock(&workspace).len(), left.len() + 1);

    // Every name the writer tries next is taken: it gives up, and gives
    // itself no other name.
    left.extend(take_unfinished_names(&path, 5..500));
    let error = RecordWriter::in_build_directory(&build_directory)
        .write(&stage, || &inputs, &probe(&[("a", 9)]))
        .expect_err("the writer has no name left for its unfinished record");

    assert!(matches!(error, WriteError::Io { .. }));
    assert_eq!(
        error.to_string(),
        format!(
            "the record `{}` cannot be written: every name for its unfinished record is taken",
            path.display()
        )
    );
    assert_untouched(&left);
    assert_eq!(fs::read(&path).expect("the record can be read"), replaced);
    assert_eq!(files_under_but_the_lock(&workspace).len(), left.len() + 1);

    // A reader passes over every one of these names, and is given the
    // record.
    let records = RecordReader::in_build_directory(&build_directory)
        .read_all()
        .expect("the records are read");
    assert_eq!(records.len(), 1, "{records:?}");
    assert!(records[0].is_ok(), "{records:?}");
}
