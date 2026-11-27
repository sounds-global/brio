// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Writing records into the build directory and reading them back.

use std::borrow::Borrow;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::envelope::{RecordDefect, RecordRefusal};
use crate::payload::Payload;
use crate::{Envelope, RecordInputs, RecordKey, Stage};

/// A record: the envelope, and the payload it wraps.
///
/// The payload is a payload type, or [`Untyped`] when the reader does not
/// know the payload's type. Either way the envelope holds every element of
/// the record, in its element table.
#[derive(Debug, Clone, PartialEq)]
pub struct Record<P> {
    /// What every record shares, with the element table.
    pub envelope: Envelope,
    /// What the stage produced, in the types of its phase.
    pub payload: P,
}

/// The payload of a record read without its payload type.
///
/// It holds nothing itself: the payload of each element is in the element
/// table of the envelope, as JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Untyped;

impl Record<Untyped> {
    /// This record with its payload as the payload type `P`.
    ///
    /// # Errors
    ///
    /// Fails when the record holds another payload kind or payload version
    /// than `P` declares, or when its elements are not in the form `P` reads.
    pub fn typed<P: Payload>(self) -> Result<Record<P>, ReadError> {
        let envelope = self.envelope;
        // A record never holds an invalid payload kind, so a payload type
        // that declares one matches no record.
        if envelope.payload_kind() != P::KIND || envelope.payload_version() != P::VERSION {
            return Err(ReadError::WrongPayload {
                found_kind: envelope.payload_kind().to_owned(),
                found_version: envelope.payload_version(),
                expected_kind: P::KIND,
                expected_version: P::VERSION,
            });
        }
        match P::from_elements(envelope.elements()) {
            Some(payload) => Ok(Record { envelope, payload }),
            None => Err(ReadError::InvalidPayload {
                payload_kind: P::KIND,
                key: envelope.key().clone(),
            }),
        }
    }
}

/// Where the records of a build directory are kept.
fn records_directory(build_directory: PathBuf) -> PathBuf {
    build_directory.join("lens")
}

/// What ends the name of the file of a record, after its record key.
const RECORD_SUFFIX: &str = ".json";
/// What ends the name of the file of a record its writer has not finished.
const UNFINISHED_SUFFIX: &str = ".unfinished";

/// Where the record with `key` is kept.
fn record_path(records_directory: &Path, key: &RecordKey) -> PathBuf {
    records_directory.join(format!("{key}{RECORD_SUFFIX}"))
}

/// The record key that the file `file` is named for, or `None` when it is
/// not named as a record: a record is stored under its record key and under
/// no other name.
fn record_key_of(file: &Path) -> Option<RecordKey> {
    let key = file.file_name()?.to_str()?.strip_suffix(RECORD_SUFFIX)?;
    RecordKey::from_hex(key.to_owned())
}

/// The name of the file where the writer `writer` of the process `process`
/// keeps the record with `key` until it is complete. The name is that
/// writer's alone.
fn unfinished_name(key: &RecordKey, process: u32, writer: u64) -> String {
    format!("{key}.{process}-{writer}{UNFINISHED_SUFFIX}")
}

/// Whether `name` is a name that [`unfinished_name`] gives: the name of a
/// record that a writer, of any process, has not finished.
fn is_unfinished_name(name: &str) -> bool {
    let parts = || {
        let (key, numbers) = name.strip_suffix(UNFINISHED_SUFFIX)?.rsplit_once('.')?;
        let (process, writer) = numbers.split_once('-')?;
        let key = RecordKey::from_hex(key.to_owned())?;
        Some((key, process.parse().ok()?, writer.parse().ok()?))
    };
    // The name is recognised by making it again, so the writer and the
    // reader cannot come to name it differently.
    parts().is_some_and(|(key, process, writer)| unfinished_name(&key, process, writer) == name)
}

/// Writes records into a build directory, or, when Lens is off, drops them.
///
/// A phase hands every record to its writer, whether Lens is on or off: the
/// [disabled](RecordWriter::disabled) writer does no work for a record.
#[derive(Debug)]
pub struct RecordWriter {
    records: Records,
}

/// What becomes of the records a writer is handed.
#[derive(Debug)]
enum Records {
    /// Lens is on: records are kept in `directory`.
    Kept { directory: PathBuf },
    /// Lens is off: no record is made.
    Dropped,
}

impl RecordWriter {
    /// A writer of records into `build_directory`. It writes nowhere else.
    #[must_use]
    pub fn in_build_directory(build_directory: impl Into<PathBuf>) -> Self {
        Self {
            records: Records::Kept {
                directory: records_directory(build_directory.into()),
            },
        }
    }

    /// The writer of a build with Lens off. It drops every record and does
    /// no work for it: it never asks for the inputs of a record or for the
    /// elements of a payload, and it serializes, hashes and writes nothing.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            records: Records::Dropped,
        }
    }

    /// Writes the record of what `stage` produced, as `payload`, from
    /// `inputs`, and gives the file it was written to, or `None` when this
    /// is the [disabled](RecordWriter::disabled) writer.
    ///
    /// `inputs` gives the inputs of the record, owned or borrowed, as in
    /// `|| &inputs`. It is called only when the record is made, so a phase
    /// gathers and copies nothing for a record that is not written.
    ///
    /// The record is canonical JSON: the same stage, inputs and payload give
    /// the same bytes in the same place under the build directory.
    ///
    /// # Errors
    ///
    /// Fails when Lens refuses to make the record, for a [`RecordRefusal`],
    /// or when the file cannot be written. The disabled writer never fails.
    pub fn write<P: Payload, I: Borrow<RecordInputs>>(
        &self,
        stage: &Stage<P>,
        inputs: impl FnOnce() -> I,
        payload: &P,
    ) -> Result<Option<PathBuf>, WriteError> {
        let Records::Kept { directory } = &self.records else {
            return Ok(None);
        };
        let (envelope, text) = Envelope::seal(stage, inputs().borrow(), payload)?;

        let path = record_path(directory, envelope.key());
        // A reader never sees half a record: the file appears under its
        // name only once it is complete. Its name until then is its
        // writer's alone, so two writers of one record never share it.
        static WRITES: AtomicU64 = AtomicU64::new(0);
        let unfinished = directory.join(unfinished_name(
            envelope.key(),
            std::process::id(),
            WRITES.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(directory)
            .and_then(|()| fs::write(&unfinished, text))
            .and_then(|()| fs::rename(&unfinished, &path))
            .map_err(|error| {
                // Nothing of a record that was not written stays behind.
                // The write already failed, and that is the error to report.
                let _ = fs::remove_file(&unfinished);
                WriteError::Io {
                    path: path.clone(),
                    error,
                }
            })?;
        Ok(Some(path))
    }
}

/// Why a record was not written.
#[derive(Debug)]
#[non_exhaustive]
pub enum WriteError {
    /// Lens refuses to make the record.
    Refused(RecordRefusal),
    /// The record's file could not be written.
    Io {
        /// The file of the record.
        path: PathBuf,
        /// What the file system reported.
        error: io::Error,
    },
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(refusal) => refusal.fmt(f),
            Self::Io { path, error } => {
                write!(
                    f,
                    "the record `{}` cannot be written: {error}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Refused(refusal) => refusal.source(),
            Self::Io { error, .. } => Some(error),
        }
    }
}

impl From<RecordRefusal> for WriteError {
    fn from(refusal: RecordRefusal) -> Self {
        Self::Refused(refusal)
    }
}

/// Reads the records of a build directory.
///
/// No record is given out with a [`RecordDefect`].
#[derive(Debug)]
pub struct RecordReader {
    directory: PathBuf,
}

impl RecordReader {
    /// A reader of the records in `build_directory`.
    #[must_use]
    pub fn in_build_directory(build_directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: records_directory(build_directory.into()),
        }
    }

    /// The record of what `stage` produced from `inputs`.
    ///
    /// # Errors
    ///
    /// Fails when there is no record for these inputs, when the record has
    /// a [`RecordDefect`], or when its payload is not a `P`.
    pub fn read<P: Payload>(
        &self,
        stage: &Stage<P>,
        inputs: &RecordInputs,
    ) -> Result<Record<P>, ReadError> {
        let key = stage.key(inputs);
        let path = record_path(&self.directory, &key);
        if !path.exists() {
            return Err(ReadError::Missing {
                module: stage.module().to_owned(),
                payload_kind: P::KIND,
            });
        }
        let record = read_record(&path, &key)?;
        // The record key says which stage the record is for, and so does
        // the record: one that names another has been changed since.
        if record.envelope.stage() != stage.identity() {
            return Err(ReadError::Defective {
                path,
                defect: RecordDefect::Corrupted,
            });
        }
        record.typed()
    }

    /// Every record in the build directory, each with an untyped payload, in
    /// the order of their file names. A record that cannot be read is given
    /// as its error, in its place, and hides no other record.
    ///
    /// So is anything else among the records of the build directory, as
    /// [`ReadError::NotARecord`]: a file with another name than a record
    /// has, a hidden file, a directory. A reader is never given the records
    /// of a build directory with one missing as if they were whole, so
    /// nothing is kept beside the records, not even a `.gitkeep`.
    ///
    /// Only a record its writer has not finished is passed over: it is no
    /// record yet. It is known by its name alone, so a record renamed to
    /// such a name is passed over too.
    ///
    /// This reads records of any payload kind, including those of payload
    /// types the caller does not know. It does not know the inputs of any
    /// record, so only [`RecordReader::read`] checks a record against the
    /// inputs it should come from.
    ///
    /// # Errors
    ///
    /// Fails when the build directory cannot be listed.
    pub fn read_all(&self) -> Result<Vec<Result<Record<Untyped>, ReadError>>, ReadError> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }
        let io_error = |error| ReadError::Io {
            path: self.directory.clone(),
            error,
        };
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.directory).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            // A writer keeps an unfinished record in a file: a directory or
            // a link with such a name is not one. A file that is gone
            // before its type is known was one, finished since.
            let unfinished = entry.file_name().to_str().is_some_and(is_unfinished_name)
                && entry.file_type().map_or(true, |kind| kind.is_file());
            if !unfinished {
                paths.push(entry.path());
            }
        }
        paths.sort();

        let read = |path: &PathBuf| {
            let key =
                record_key_of(path).ok_or_else(|| ReadError::NotARecord { path: path.clone() })?;
            read_record(path, &key)
        };
        Ok(paths.iter().map(read).collect())
    }
}

/// The record in `file`, which is stored under the key `stored_under`.
fn read_record(file: &Path, stored_under: &RecordKey) -> Result<Record<Untyped>, ReadError> {
    let defective = |defect| ReadError::Defective {
        path: file.to_owned(),
        defect,
    };
    let bytes = fs::read(file).map_err(|error| ReadError::Io {
        path: file.to_owned(),
        error,
    })?;
    let text = String::from_utf8(bytes).map_err(|_| defective(RecordDefect::Corrupted))?;
    let envelope = Envelope::open(&text, stored_under).map_err(defective)?;
    Ok(Record {
        envelope,
        payload: Untyped,
    })
}

/// Why a record was not read.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReadError {
    /// The build directory has no record for these inputs.
    Missing {
        /// The module the record was asked for.
        module: String,
        /// The payload kind the record was asked for.
        payload_kind: &'static str,
    },
    /// A file or directory could not be read.
    Io {
        /// The file or directory.
        path: PathBuf,
        /// What the file system reported.
        error: io::Error,
    },
    /// Something among the records of a build directory is not named for a
    /// record key, as every record is, so it is not read as a record.
    NotARecord {
        /// The file or directory.
        path: PathBuf,
    },
    /// The file does not hold a record this toolchain trusts.
    Defective {
        /// The file of the record.
        path: PathBuf,
        /// What is wrong with it.
        defect: RecordDefect,
    },
    /// The record holds another payload than the payload type asked for.
    WrongPayload {
        /// The payload kind of the record.
        found_kind: String,
        /// The payload version of the record.
        found_version: u32,
        /// The payload kind of the payload type asked for.
        expected_kind: &'static str,
        /// The payload version of the payload type asked for.
        expected_version: u32,
    },
    /// The record's elements are not in the form its payload type reads.
    InvalidPayload {
        /// The payload kind of the record.
        payload_kind: &'static str,
        /// The record key of the record.
        key: RecordKey,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing {
                module,
                payload_kind,
            } => write!(
                f,
                "there is no `{payload_kind}` record of module `{module}` for these inputs"
            ),
            Self::Io { path, error } => write!(f, "`{}` cannot be read: {error}", path.display()),
            Self::NotARecord { path } => write!(
                f,
                "`{}` is among the records and is not named for a record key",
                path.display()
            ),
            Self::Defective { path, defect } => {
                write!(f, "the record `{}` {defect}", path.display())
            }
            Self::WrongPayload {
                found_kind,
                found_version,
                expected_kind,
                expected_version,
            } => write!(
                f,
                "the record holds the payload `{found_kind}` version {found_version}, not \
                 `{expected_kind}` version {expected_version}"
            ),
            Self::InvalidPayload { payload_kind, key } => write!(
                f,
                "the elements of the `{payload_kind}` record `{key}` are not a `{payload_kind}` \
                 payload"
            ),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { error, .. } => Some(error),
            _ => None,
        }
    }
}
