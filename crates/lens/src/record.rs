// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Writing records into the build directory and reading them back.

use std::borrow::Borrow;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write as _};
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

/// The name of the file of an unfinished record: where a writer keeps the
/// record with one record key until it is complete. It has the identifier
/// of the writer's process and a number.
///
/// Two processes can have one identifier, so the name alone is not that
/// writer's own: a writer takes a name only by making a new file with it,
/// in [`new_unfinished_file`].
///
/// The writer makes the name and the reader knows it again, both here, so
/// they cannot come to name it differently.
struct UnfinishedName {
    key: RecordKey,
    process: u32,
    /// What tells this unfinished record from every other of its process.
    number: u64,
}

impl UnfinishedName {
    /// The unfinished name that `name` is, or `None` when it is the name of
    /// something else: no writer, of any process, gives it.
    fn parse(name: &str) -> Option<Self> {
        let (key, numbers) = name.strip_suffix(UNFINISHED_SUFFIX)?.rsplit_once('.')?;
        let (process, number) = numbers.split_once('-')?;
        let parsed = Self {
            key: RecordKey::from_hex(key.to_owned())?,
            process: process.parse().ok()?,
            number: number.parse().ok()?,
        };
        // A number can be written in more than one way, and a writer writes
        // it in one.
        (parsed.to_string() == name).then_some(parsed)
    }
}

impl fmt::Display for UnfinishedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            key,
            process,
            number,
        } = self;
        write!(f, "{key}.{process}-{number}{UNFINISHED_SUFFIX}")
    }
}

/// How many names a writer tries for one unfinished record before it gives
/// up. A name is taken only when another process has the same identifier
/// and has given the same number, or when a writer with that name was
/// stopped.
const UNFINISHED_NAMES_TRIED: usize = 64;

/// Whether `file` is a file and holds the record with the text `text`,
/// whole.
fn holds_record(file: &Path, text: &str) -> bool {
    // Only a file is read: reading what else can have the name, such as a
    // pipe, may never end.
    fs::metadata(file).is_ok_and(|metadata| metadata.is_file())
        && fs::read(file).is_ok_and(|held| held == text.as_bytes())
}

/// Makes, in `records_directory`, the file of an unfinished record with
/// `key`, and gives it with its place.
///
/// The file is new, and this writer's alone. A name that is taken is left
/// to the writer that has it: writing into it, or removing it, would
/// reach the unfinished record of another process with the same identifier
/// and number, or a record itself, which a writer that was stopped left
/// under a second name.
fn new_unfinished_file(records_directory: &Path, key: &RecordKey) -> io::Result<(PathBuf, File)> {
    static UNFINISHED_RECORDS: AtomicU64 = AtomicU64::new(0);
    for _ in 0..UNFINISHED_NAMES_TRIED {
        let name = UnfinishedName {
            key: key.clone(),
            process: std::process::id(),
            number: UNFINISHED_RECORDS.fetch_add(1, Ordering::Relaxed),
        };
        let place = records_directory.join(name.to_string());
        match File::create_new(&place) {
            Ok(file) => return Ok((place, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "every name for its unfinished record is taken",
    ))
}

/// Writes the record with `key` and the text `text` into `records_directory`,
/// under the name of that record.
fn write_record(records_directory: &Path, key: &RecordKey, text: &str) -> io::Result<()> {
    fs::create_dir_all(records_directory)?;
    // A reader never sees half a record: the record is written under an
    // unfinished name, and gets its own name only once it is complete.
    let (unfinished, mut file) = new_unfinished_file(records_directory, key)?;
    let placed = file
        .write_all(text.as_bytes())
        .and_then(|()| put_in_place(&unfinished, &record_path(records_directory, key), text));
    // Nothing of this writer's unfinished record stays behind, written or
    // not. Once the record was renamed, the name is this writer's no more:
    // what has it since is the unfinished record of another, and is left.
    // When the write failed, that is the error to report.
    if !matches!(placed, Ok(Placed::Renamed)) {
        let _ = fs::remove_file(&unfinished);
    }
    placed.map(|_| ())
}

/// How a complete record came to be in its place.
enum Placed {
    /// The record is there, and the unfinished name is still a name of this
    /// writer's file.
    BesideUnfinished,
    /// The unfinished file was renamed: its unfinished name is free.
    Renamed,
}

/// Puts the complete record in `unfinished`, with the text `text`, in its
/// place, `file`.
fn put_in_place(unfinished: &Path, file: &Path, text: &str) -> io::Result<Placed> {
    // A second name for the same file takes the place of nothing, so no
    // record is gone for a moment. It fails when `file` is there, and where
    // the file system gives no second name to a file. When it fails because
    // another writer put the same record there meanwhile, there is nothing
    // left to do.
    if fs::hard_link(unfinished, file).is_ok() || holds_record(file, text) {
        return Ok(Placed::BesideUnfinished);
    }
    // The record is renamed into its place, and onto what is there: a
    // corrupted record, a record of the same record key that a stage made
    // otherwise, or something that is no record. Nothing is there when the
    // file system only gave the record no second name.
    fs::rename(unfinished, file).map(|()| Placed::Renamed)
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
    /// A record that is there already is left as it is, so a reader beside
    /// a writer of the same record is given that record at every moment.
    /// Only a file with other bytes in the place of the record is replaced.
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
        // A record that is there already stays as it is. Nothing is put in
        // its place, not even the same record: to a reader that lists the
        // records meanwhile, a file that is replaced can be gone.
        if holds_record(&path, &text) {
            return Ok(Some(path));
        }
        write_record(directory, envelope.key(), &text).map_err(|error| WriteError::Io {
            path: path.clone(),
            error,
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
    /// has, a hidden file, a directory under any name. A record with the
    /// wrong name would
    /// otherwise be missing from the records as if they were whole, so
    /// nothing is kept beside the records, not even a `.gitkeep`.
    ///
    /// Only a record its writer has not finished is passed over: it is no
    /// record yet. It is a file with the name a writer gives it, and
    /// nothing else tells it apart, so a record renamed to such a name is
    /// passed over too.
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
            // a link with such a name is not one. What has such a name and
            // no type that can be told is passed over as well: it is gone
            // since it was listed, as an unfinished record is once it is
            // finished.
            let named_unfinished = |name: &str| UnfinishedName::parse(name).is_some();
            let unfinished = entry.file_name().to_str().is_some_and(named_unfinished)
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
    let io_error = |error| ReadError::Io {
        path: file.to_owned(),
        error,
    };
    // Only a file is read: reading what else can have the name of a record,
    // such as a pipe, may never end.
    if !fs::metadata(file).map_err(io_error)?.is_file() {
        return Err(ReadError::NotARecord {
            path: file.to_owned(),
        });
    }
    let bytes = fs::read(file).map_err(io_error)?;
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
    /// Something among the records of a build directory is not the file of
    /// a record, so it is not read as one: it is not named for a record
    /// key, as every record is, or it is not a file.
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
                "`{}` is among the records and is not the file of a record",
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
