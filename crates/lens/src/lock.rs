// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The Records lock: how the readers and writers of the records of one
//! build directory, in any number of processes, keep out of each other's
//! way.
//!
//! The lock is a file beside the records, which a reader locks to list
//! them and a writer locks to write one. Its name and what its two kinds of
//! lock mean are a promise of
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
    let lock_file = match open_lock_file(&lock_file, Holder::Writer)? {
        LockFile::NotThere => make_lock_file(&lock_file)?,
        there => there,
    };
    let _held = lock_file.locked_shared();
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
    // Something that is no file where the lock file belongs is no lock
    // file: a reader goes on without one.
    let lock_file = open_lock_file(&lock_file(records_directory), Holder::Reader);
    let _held = lock_file.unwrap_or(LockFile::NotThere).locked_shared();
    work()
}

/// The lock file of a build directory, as a holder finds it.
#[derive(Debug)]
enum LockFile {
    /// It is there, and open to be locked.
    Open(File),
    /// It is there and cannot be opened, so the lock cannot be taken.
    CannotBeOpened,
    /// Nothing has its name.
    NotThere,
}

impl LockFile {
    /// The lock file with the shared lock held on it, waiting for the lock,
    /// or `None` where it cannot be taken: the holder goes on without it.
    fn locked_shared(self) -> Option<File> {
        let Self::Open(lock_file) = self else {
            return None;
        };
        loop {
            match after_asking(&lock_file.lock_shared()) {
                Next::Hold => return Some(lock_file),
                Next::AskAgain => {}
                Next::GoOnWithout => return None,
            }
        }
    }
}

/// Who opens the lock file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Holder {
    /// A reader, which only ever takes the shared lock.
    Reader,
    /// A writer, which also takes the exclusive lock.
    Writer,
}

/// Opens the lock file `lock_file` for `holder` to lock.
///
/// Only a file is opened, and never through a link: opening what else can
/// have the name may never end, as with a pipe, or reach outside the build
/// directory.
///
/// # Errors
///
/// Fails when what has the name is not a file: no lock file can be there.
fn open_lock_file(lock_file: &Path, holder: Holder) -> io::Result<LockFile> {
    match fs::symlink_metadata(lock_file) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "what has its name is not a file",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(LockFile::NotThere),
        Err(error) => return Err(error),
    }
    // To be read is enough for a shared lock everywhere. Some file systems
    // on a network give the exclusive lock only on a file open to be
    // written to, so a writer opens it for both when it may, and a lock
    // file that can only be read still locks shared.
    let mut for_both = OpenOptions::new();
    let opened = match holder {
        Holder::Writer => for_both
            .read(true)
            .write(true)
            .open(lock_file)
            .or_else(|_| File::open(lock_file)),
        Holder::Reader => File::open(lock_file),
    };
    Ok(match opened {
        Ok(opened) => LockFile::Open(opened),
        // It was removed since it was looked at.
        Err(error) if error.kind() == io::ErrorKind::NotFound => LockFile::NotThere,
        Err(_) => LockFile::CannotBeOpened,
    })
}

/// Makes the lock file `lock_file`, which is not there, and opens it for a
/// writer to lock. Nothing is ever written into it.
fn make_lock_file(lock_file: &Path) -> io::Result<LockFile> {
    let mut new = OpenOptions::new();
    match new.read(true).write(true).create_new(true).open(lock_file) {
        Ok(made) => Ok(LockFile::Open(made)),
        // Another writer made it meanwhile.
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            open_lock_file(lock_file, Holder::Writer)
        }
        Err(error) => Err(error),
    }
}

/// What a holder does next, once it has asked for the lock.
#[derive(Debug, PartialEq, Eq)]
enum Next {
    /// It has the lock, and holds it.
    Hold,
    /// Waiting for the lock was cut short: it asks again.
    AskAgain,
    /// The lock cannot be taken here: it goes on without it.
    GoOnWithout,
}

/// What a holder does next, once it has asked for the lock and was
/// answered `answer`.
fn after_asking(answer: &io::Result<()>) -> Next {
    match answer {
        Ok(()) => Next::Hold,
        Err(error) if error.kind() == io::ErrorKind::Interrupted => Next::AskAgain,
        Err(_) => Next::GoOnWithout,
    }
}

/// What of the lock cannot be seen through the public interface: how long
/// it is held, and what a holder does when it cannot have it.
#[cfg(test)]
mod tests {
    use std::fs::TryLockError;

    use super::*;

    /// An empty build directory for the test called `test`.
    fn build_directory(test: &str) -> PathBuf {
        let build_directory =
            std::env::temp_dir().join(format!("brio_lens-lock-{}-{test}", std::process::id()));
        // The directory is there only when an earlier run was stopped.
        let _ = fs::remove_dir_all(&build_directory);
        fs::create_dir_all(&build_directory).expect("the build directory can be created");
        build_directory
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
        let build_directory = build_directory("writer_holds");
        let records_directory = build_directory.join("lens");

        let during = while_making_a_record(&records_directory, || {
            exclusive_lock_is_free(&records_directory)
        });

        assert_eq!(during.ok(), Some(false));
        assert!(exclusive_lock_is_free(&records_directory));
        fs::remove_dir_all(build_directory).expect("the build directory can be removed");
    }

    #[test]
    fn a_reader_holds_the_shared_lock_for_as_long_as_it_lists_the_records() {
        let build_directory = build_directory("reader_holds");
        let records_directory = build_directory.join("lens");
        while_making_a_record(&records_directory, || ()).expect("the lock file can be made");

        let during = while_listing_the_records(&records_directory, || {
            exclusive_lock_is_free(&records_directory)
        });

        assert!(!during);
        assert!(exclusive_lock_is_free(&records_directory));
        fs::remove_dir_all(build_directory).expect("the build directory can be removed");
    }

    #[test]
    fn a_writer_that_finds_the_lock_file_made_meanwhile_opens_that_one() {
        let build_directory = build_directory("made_meanwhile");
        let lock_file = build_directory.join(LOCK_FILE);
        // Another writer made the lock file since this one looked for it.
        fs::write(&lock_file, "").expect("the lock file can be made");

        let found = make_lock_file(&lock_file);

        assert!(matches!(found, Ok(LockFile::Open(_))), "{found:?}");
        fs::remove_dir_all(build_directory).expect("the build directory can be removed");
    }

    #[test]
    fn a_holder_asks_again_when_its_wait_is_cut_short_and_goes_on_without_a_lock_it_cannot_have() {
        let cut_short = Err(io::ErrorKind::Interrupted.into());
        let cannot_lock = Err(io::ErrorKind::Unsupported.into());

        assert_eq!(after_asking(&Ok(())), Next::Hold);
        assert_eq!(after_asking(&cut_short), Next::AskAgain);
        assert_eq!(after_asking(&cannot_lock), Next::GoOnWithout);
    }
}
