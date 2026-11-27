// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The Records lock: how the readers and writers of the records of one
//! build directory, in any number of processes, keep out of each other's
//! way.
//!
//! The lock is a file beside the records, which every reader and writer
//! locks. Its name and what its two kinds of lock mean are a promise of
//! the layout of a build directory, so that another tool can take part:
//!
//! - the holder of a *shared* lock lists the records, or makes a new record
//!   without replacing anything;
//! - the holder of the *exclusive* lock is alone at work among the records.
//!
//! Lens itself does not keep the whole of that promise yet: a writer still
//! puts a record in the place of a file with other bytes under the shared
//! lock, and nothing takes the exclusive one.
//!
//! The lock is advisory: it binds the programs that take it. A holder that
//! ends, however it ends, holds it no more.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

/// The name of the lock file, in a build directory, beside its records.
const LOCK_FILE: &str = "lens.lock";

/// Where the lock file of the records in `records_directory` is: beside
/// that directory, never in it, so that the records directory holds
/// records and unfinished records alone.
pub(crate) fn lock_file(records_directory: &Path) -> PathBuf {
    records_directory.with_file_name(LOCK_FILE)
}

/// Does `work`, which makes a new record among those in `records_directory`,
/// under the shared lock, and gives what it gives.
///
/// The lock file is made first when it is not there, and so before a writer
/// changes anything among the records.
///
/// Where the lock file is there and the lock cannot be taken, `work` is
/// done without it, with the care that writers take of each other without
/// a lock.
///
/// # Errors
///
/// Fails, without doing `work`, when the lock file is not there and cannot
/// be made: a writer never works among records that have no lock file.
pub(crate) fn while_making_a_record<T>(
    records_directory: &Path,
    work: impl FnOnce() -> T,
) -> io::Result<T> {
    let lock_file = lock_file(records_directory);
    if let Some(build_directory) = lock_file.parent() {
        fs::create_dir_all(build_directory)?;
    }
    let held = match opened(&lock_file)? {
        Some(opened) => opened,
        None => made(&lock_file)?,
    };
    let _held = held.filter(lock_shared);
    Ok(work())
}

/// Does `work`, which lists the records in `records_directory`, under the
/// shared lock, and gives what it gives.
///
/// A reader makes nothing: it takes the lock only when the lock file is
/// there. Where there is none, or the lock cannot be taken, `work` is done
/// without it.
pub(crate) fn while_listing_the_records<T>(
    records_directory: &Path,
    work: impl FnOnce() -> T,
) -> T {
    let opened = opened(&lock_file(records_directory));
    let _held = opened.ok().flatten().flatten().filter(lock_shared);
    work()
}

/// The lock file `lock_file`, open to be locked.
///
/// It is `None` when nothing has that name, and `Some(None)` when the file
/// is there and cannot be opened: the lock cannot be taken then.
///
/// Only a file is opened, and never through a link: opening what else can
/// have the name may never end, as with a pipe, or reach outside the build
/// directory.
///
/// # Errors
///
/// Fails when what has the name is not a file: no lock file can be there.
fn opened(lock_file: &Path) -> io::Result<Option<Option<File>>> {
    match fs::symlink_metadata(lock_file) {
        // To be read is enough to be locked, shared or exclusive, so a lock
        // file that cannot be written to locks as any other.
        Ok(metadata) if metadata.is_file() => Ok(Some(File::open(lock_file).ok())),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "what has its name is not a file",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// The lock file `lock_file`, which was not there, made and open to be
/// locked. Nothing is ever written into it.
fn made(lock_file: &Path) -> io::Result<Option<File>> {
    let mut new = OpenOptions::new();
    match new.read(true).write(true).create_new(true).open(lock_file) {
        Ok(made) => Ok(Some(made)),
        // Another writer made it meanwhile.
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            opened(lock_file).map(Option::flatten)
        }
        Err(error) => Err(error),
    }
}

/// What a holder does after it asked for the lock.
#[derive(Debug, PartialEq, Eq)]
enum Asked {
    /// The lock is held.
    Held,
    /// Waiting for the lock was cut short: the holder asks again.
    AskAgain,
    /// The lock cannot be taken here: the holder goes on without it.
    GoOnWithout,
}

/// What a holder does after it asked for the lock and was answered `answer`.
fn asked(answer: &io::Result<()>) -> Asked {
    match answer {
        Ok(()) => Asked::Held,
        Err(error) if error.kind() == io::ErrorKind::Interrupted => Asked::AskAgain,
        Err(_) => Asked::GoOnWithout,
    }
}

/// Takes the shared lock on `lock_file`, waiting for it, and says whether it
/// is held.
fn lock_shared(lock_file: &File) -> bool {
    loop {
        match asked(&lock_file.lock_shared()) {
            Asked::Held => return true,
            Asked::AskAgain => {}
            Asked::GoOnWithout => return false,
        }
    }
}

/// What of the lock cannot be seen through the public interface: how long
/// it is held, and what a holder does when it cannot have it.
#[cfg(test)]
mod tests {
    use std::fs::TryLockError;

    use super::*;

    /// The records directory of an empty build directory for the test
    /// called `test`.
    fn records_directory(test: &str) -> PathBuf {
        let build_directory =
            std::env::temp_dir().join(format!("brio_lens-lock-{}-{test}", std::process::id()));
        // The directory is there only when an earlier run was stopped.
        let _ = fs::remove_dir_all(&build_directory);
        fs::create_dir_all(&build_directory).expect("the build directory can be created");
        build_directory.join("lens")
    }

    /// Whether another holder can have the exclusive lock of the records in
    /// `records_directory` now, without waiting. It lets go of it at once.
    fn exclusive_lock_is_free(records_directory: &Path) -> bool {
        let lock_file = File::open(lock_file(records_directory)).expect("the lock file is there");
        match lock_file.try_lock() {
            Ok(()) => true,
            Err(TryLockError::WouldBlock) => false,
            Err(TryLockError::Error(error)) => panic!("the lock cannot be asked for: {error}"),
        }
    }

    #[test]
    fn a_writer_holds_the_shared_lock_for_as_long_as_it_makes_its_record() {
        let records_directory = records_directory("writer_holds");

        let during = while_making_a_record(&records_directory, || {
            exclusive_lock_is_free(&records_directory)
        });

        assert_eq!(during.ok(), Some(false));
        assert!(exclusive_lock_is_free(&records_directory));
        let _ = fs::remove_dir_all(lock_file(&records_directory).with_file_name(""));
    }

    #[test]
    fn a_reader_holds_the_shared_lock_for_as_long_as_it_lists_the_records() {
        let records_directory = records_directory("reader_holds");
        while_making_a_record(&records_directory, || ()).expect("the lock file can be made");

        let during = while_listing_the_records(&records_directory, || {
            exclusive_lock_is_free(&records_directory)
        });

        assert!(!during);
        assert!(exclusive_lock_is_free(&records_directory));
        let _ = fs::remove_dir_all(lock_file(&records_directory).with_file_name(""));
    }

    #[test]
    fn a_holder_asks_again_when_its_wait_is_cut_short_and_goes_on_without_a_lock_it_cannot_have() {
        let cut_short = Err(io::ErrorKind::Interrupted.into());
        let cannot_lock = Err(io::ErrorKind::Unsupported.into());

        assert_eq!(asked(&Ok(())), Asked::Held);
        assert_eq!(asked(&cut_short), Asked::AskAgain);
        assert_eq!(asked(&cannot_lock), Asked::GoOnWithout);
    }
}
