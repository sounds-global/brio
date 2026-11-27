// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The Records lock, through the public interface of `brio_lens` and the
//! lock file of a build directory.
//!
//! A test holds the lock file itself, as another process would: its name
//! and the meaning of a shared and an exclusive lock are a promise of the
//! layout of a build directory.

mod support;

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use brio_lens::{RecordReader, RecordWriter, WriteError};

use support::{Probe, files_under, inputs, lock_file, probe, stage, workspace, write_record};

/// The lock file of `build_directory`, open as another process has it.
fn open_lock_file(build_directory: &Path) -> File {
    File::open(lock_file(build_directory)).expect("the lock file is there")
}

/// Long enough for anything a test starts to finish when nothing holds it
/// back: it takes far less than a millisecond.
const PAUSE: Duration = Duration::from_millis(300);

/// Far longer than anything a test starts takes once it is free to go on.
const LONG: Duration = Duration::from_secs(30);

/// Starts `work` beside the test, and gives where its result arrives.
fn start<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Receiver<T> {
    let (done, result) = mpsc::channel();
    std::thread::spawn(move || {
        // The test may be gone when the work is done: it has failed then.
        let _ = done.send(work());
    });
    result
}

/// How many records `build_directory` is read to hold, each of them whole.
fn count_records(build_directory: &Path) -> usize {
    let records = RecordReader::in_build_directory(build_directory)
        .read_all()
        .expect("the records are read");
    assert!(records.iter().all(Result::is_ok), "{records:?}");
    records.len()
}

/// Writes into `build_directory` the record of the probe stage of `module`.
fn write_probe(build_directory: &Path, module: &str) -> PathBuf {
    write_record(
        build_directory,
        &stage::<Probe>(module),
        &inputs("sensor a\n"),
        &probe(&[("a", 7)]),
    )
}

/// What a writer says when it is asked for the record of the probe stage
/// of `app::main` in `build_directory` and cannot write it.
fn failed_write(build_directory: &Path) -> WriteError {
    RecordWriter::in_build_directory(build_directory)
        .write(
            &stage::<Probe>("app::main"),
            || inputs("sensor a\n"),
            &probe(&[("a", 7)]),
        )
        .expect_err("the record cannot be written")
}

#[test]
fn a_writer_makes_the_lock_file_beside_the_records_and_a_reader_does_not_report_it() {
    let build_directory = workspace("lock_file").join(".brio");

    let record = write_probe(&build_directory, "app::main");

    let mut files = files_under(&build_directory);
    files.sort();
    let mut expected = [record, lock_file(&build_directory)];
    expected.sort();
    assert_eq!(files, expected);
    assert_eq!(count_records(&build_directory), 1);
}

#[test]
fn a_writer_makes_the_lock_file_before_it_changes_anything_among_the_records() {
    let build_directory = workspace("lock_file_first").join(".brio");
    fs::create_dir_all(&build_directory).expect("the build directory can be created");
    // A file where the records belong: no record can be written.
    fs::write(build_directory.join("lens"), "").expect("a file can take the place of the records");

    let error = failed_write(&build_directory);

    assert!(matches!(error, WriteError::Io { .. }), "{error}");
    assert!(lock_file(&build_directory).is_file());
}

#[test]
fn a_reader_makes_no_file() {
    let workspace = workspace("reader_makes_no_file");
    let (written, copied) = (workspace.join("written"), workspace.join("copied"));
    let record = write_probe(&written, "app::main");
    // A record set that no writer wrote into: the records alone, copied.
    fs::create_dir_all(copied.join("lens")).expect("the record set can be created");
    let copy = copied
        .join("lens")
        .join(record.file_name().expect("the record has a name"));
    fs::copy(&record, &copy).expect("the record can be copied");
    let before = files_under(&workspace);

    assert_eq!(count_records(&written), 1);
    assert_eq!(count_records(&copied), 1);

    assert_eq!(files_under(&workspace), before);
    assert_eq!(files_under(&copied), [copy]);
}

#[cfg(unix)]
#[test]
fn a_reader_reads_a_directory_it_cannot_write_to() {
    use std::os::unix::fs::PermissionsExt;

    let workspace = workspace("read_only");
    let (written, copied) = (workspace.join("written"), workspace.join("copied"));
    let record = write_probe(&written, "app::main");
    // A record set with no lock file as well.
    fs::create_dir_all(copied.join("lens")).expect("the record set can be created");
    let name = record.file_name().expect("the record has a name");
    fs::copy(&record, copied.join("lens").join(name)).expect("the record can be copied");
    let directories = [
        written.join("lens"),
        copied.join("lens"),
        written.clone(),
        copied.clone(),
    ];
    let set_mode = |mode| {
        for directory in &directories {
            fs::set_permissions(directory, fs::Permissions::from_mode(mode))
                .expect("the permissions can be changed");
        }
    };

    set_mode(0o555);
    let read = [&written, &copied].map(|set| RecordReader::in_build_directory(set).read_all());
    set_mode(0o755);

    for records in read {
        let records = records.expect("the records are read");
        assert_eq!(records.len(), 1);
        assert!(records[0].is_ok(), "{records:?}");
    }
}

#[test]
fn a_reader_that_lists_the_records_waits_while_the_exclusive_lock_is_held() {
    let build_directory = workspace("reader_waits").join(".brio");
    write_probe(&build_directory, "app::main");
    let lock = open_lock_file(&build_directory);
    lock.lock().expect("the lock can be taken");

    let directory = build_directory.clone();
    let listed = start(move || count_records(&directory));

    assert!(
        listed.recv_timeout(PAUSE).is_err(),
        "the reader did not wait"
    );
    lock.unlock().expect("the lock can be released");
    assert_eq!(listed.recv_timeout(LONG), Ok(1));
}

#[test]
fn a_reader_and_a_writer_of_a_new_record_do_not_wait_while_a_shared_lock_is_held() {
    let build_directory = workspace("shared_lock_held").join(".brio");
    write_probe(&build_directory, "app::main");
    let lock = open_lock_file(&build_directory);
    lock.lock_shared().expect("the lock can be taken");

    let directory = build_directory.clone();
    let listed = start(move || count_records(&directory));
    assert_eq!(listed.recv_timeout(LONG), Ok(1));

    let directory = build_directory.clone();
    let written = start(move || write_probe(&directory, "app::motor"));
    assert!(written.recv_timeout(LONG).is_ok(), "the writer waited");
    assert_eq!(count_records(&build_directory), 2);
}

#[test]
fn a_writer_of_a_new_record_waits_while_the_exclusive_lock_is_held() {
    let build_directory = workspace("writer_waits").join(".brio");
    write_probe(&build_directory, "app::main");
    let lock = open_lock_file(&build_directory);
    lock.lock().expect("the lock can be taken");

    let directory = build_directory.clone();
    let written = start(move || write_probe(&directory, "app::motor"));

    assert!(
        written.recv_timeout(PAUSE).is_err(),
        "the writer did not wait"
    );
    // Nothing of the record is there yet, not even unfinished.
    assert_eq!(files_under(&build_directory.join("lens")).len(), 1);
    lock.unlock().expect("the lock can be released");
    assert!(written.recv_timeout(LONG).is_ok());
}

#[test]
fn reading_one_record_and_writing_a_record_that_is_there_take_no_lock() {
    let build_directory = workspace("no_lock_taken").join(".brio");
    let record = write_probe(&build_directory, "app::main");
    let lock = open_lock_file(&build_directory);
    lock.lock().expect("the lock can be taken");

    let directory = build_directory.clone();
    let read = start(move || {
        RecordReader::in_build_directory(directory)
            .read(&stage::<Probe>("app::main"), &inputs("sensor a\n"))
            .is_ok()
    });
    assert_eq!(read.recv_timeout(LONG), Ok(true));

    let directory = build_directory.clone();
    let written = start(move || write_probe(&directory, "app::main"));
    assert_eq!(written.recv_timeout(LONG), Ok(record));
}

#[test]
fn a_writer_that_cannot_make_the_lock_file_writes_no_record() {
    let build_directory = workspace("no_lock_file_to_make").join(".brio");
    // A directory where the lock file belongs: no file can be made there.
    fs::create_dir_all(lock_file(&build_directory)).expect("a directory can be created");

    let error = failed_write(&build_directory);

    assert!(matches!(error, WriteError::LockFile { .. }), "{error}");
    assert_eq!(
        error.to_string().split(": ").next(),
        Some(
            format!(
                "the lock file `{}` of the records cannot be made",
                lock_file(&build_directory).display()
            )
            .as_str()
        )
    );
    assert_eq!(files_under(&build_directory), Vec::<PathBuf>::new());
}

#[test]
fn a_reader_reads_the_records_beside_a_lock_file_that_is_not_a_file() {
    let build_directory = workspace("lock_file_is_no_file").join(".brio");
    write_probe(&build_directory, "app::main");
    fs::remove_file(lock_file(&build_directory)).expect("the lock file can be removed");
    fs::create_dir(lock_file(&build_directory)).expect("a directory can take its place");

    let directory = build_directory.clone();
    let listed = start(move || count_records(&directory));

    assert_eq!(listed.recv_timeout(LONG), Ok(1));
}

#[cfg(unix)]
#[test]
fn a_writer_does_not_follow_a_link_where_the_lock_file_belongs() {
    let workspace = workspace("lock_file_is_a_link");
    let build_directory = workspace.join(".brio");
    fs::create_dir_all(&build_directory).expect("the build directory can be created");
    let outside = workspace.join("outside-the-build-directory");
    std::os::unix::fs::symlink(&outside, lock_file(&build_directory)).expect("a link can be made");

    let error = failed_write(&build_directory);

    assert!(matches!(error, WriteError::LockFile { .. }), "{error}");
    assert!(!outside.exists());
    assert!(!build_directory.join("lens").exists());
}

#[cfg(unix)]
#[test]
fn a_writer_takes_the_lock_on_a_lock_file_it_can_only_read() {
    use std::os::unix::fs::PermissionsExt;

    let build_directory = workspace("lock_file_read_only").join(".brio");
    write_probe(&build_directory, "app::main");
    fs::set_permissions(
        lock_file(&build_directory),
        fs::Permissions::from_mode(0o444),
    )
    .expect("the permissions can be changed");
    let lock = open_lock_file(&build_directory);
    lock.lock().expect("the lock can be taken");

    let directory = build_directory.clone();
    let written = start(move || write_probe(&directory, "app::motor"));

    assert!(
        written.recv_timeout(PAUSE).is_err(),
        "the writer did not wait"
    );
    lock.unlock().expect("the lock can be released");
    assert!(written.recv_timeout(LONG).is_ok());
}

#[cfg(unix)]
#[test]
fn a_writer_and_a_reader_go_on_without_a_lock_file_they_cannot_open() {
    use std::os::unix::fs::PermissionsExt;

    let build_directory = workspace("lock_file_closed").join(".brio");
    write_probe(&build_directory, "app::main");
    fs::set_permissions(
        lock_file(&build_directory),
        fs::Permissions::from_mode(0o000),
    )
    .expect("the permissions can be changed");

    write_probe(&build_directory, "app::motor");

    assert_eq!(count_records(&build_directory), 2);
}
