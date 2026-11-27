// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Writing records into the build directory and reading them back.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::envelope::Unreadable;
use crate::json::Value;
use crate::payload::{self, InvalidPayloadKind, Payload};
use crate::{Envelope, RecordInputs, RecordKey, canonical};

/// A record: the envelope and the payload it wraps.
///
/// The payload is a payload type, or [`Value`] when the reader does not know
/// the payload's type.
#[derive(Debug, Clone, PartialEq)]
pub struct Record<P> {
    /// What the record says about itself.
    pub envelope: Envelope,
    /// What the stage produced.
    pub payload: P,
}

impl Record<Value> {
    /// This record with its payload as the payload type `P`.
    ///
    /// # Errors
    ///
    /// Fails when the record holds another payload kind or payload version
    /// than `P` declares, or when its payload is not in the form `P` reads.
    pub fn typed<P: Payload>(self) -> Result<Record<P>, ReadError> {
        let payload_kind = payload::kind_of::<P>()?;
        let envelope = self.envelope;
        if envelope.payload_kind != payload_kind || envelope.payload_version != P::VERSION {
            return Err(ReadError::WrongPayload {
                found_kind: envelope.payload_kind,
                found_version: envelope.payload_version,
                expected_kind: payload_kind,
                expected_version: P::VERSION,
            });
        }
        match P::from_json(&self.payload) {
            Some(payload) => Ok(Record { envelope, payload }),
            None => Err(ReadError::InvalidPayload {
                payload_kind,
                key: envelope.key,
            }),
        }
    }
}

/// Where the records of a build directory are kept.
fn records_directory(build_directory: PathBuf) -> PathBuf {
    build_directory.join("lens")
}

/// Where the record with `key` is kept.
fn record_path(records_directory: &Path, key: &RecordKey) -> PathBuf {
    records_directory.join(format!("{key}.json"))
}

/// Writes records into a build directory.
#[derive(Debug)]
pub struct RecordWriter {
    directory: PathBuf,
}

impl RecordWriter {
    /// A writer of records into `build_directory`. It writes nowhere else.
    #[must_use]
    pub fn in_build_directory(build_directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: records_directory(build_directory.into()),
        }
    }

    /// Writes the record of what a stage produced, as `payload`, from
    /// `inputs`, and gives the file it was written to.
    ///
    /// The record is canonical JSON: the same inputs and payload give the
    /// same bytes in the same place under the build directory.
    ///
    /// # Errors
    ///
    /// Fails when the payload kind of `P` breaks the rule on
    /// [`Payload::KIND`], when the module of `inputs` is named by an
    /// absolute path, when the payload nests deeper than
    /// [`Envelope::MAX_PAYLOAD_DEPTH`], or when the file cannot be written.
    pub fn write<P: Payload>(
        &self,
        inputs: &RecordInputs,
        payload: &P,
    ) -> Result<PathBuf, WriteError> {
        let envelope = Envelope::of::<P>(inputs)?;
        if inputs.module_is_an_absolute_path() {
            return Err(WriteError::AbsolutePath {
                module: inputs.module.clone(),
            });
        }
        let payload = payload.to_json();
        let depth = canonical::depth(&payload);
        if depth > Envelope::MAX_PAYLOAD_DEPTH {
            return Err(WriteError::PayloadTooDeep {
                payload_kind: P::KIND,
                depth,
            });
        }
        let text = envelope.seal(payload);

        let path = record_path(&self.directory, &envelope.key);
        // A reader never sees half a record: the file appears under its
        // name only once it is complete. Its name until then is its
        // writer's alone, so two writers of one record never share it.
        static WRITES: AtomicU64 = AtomicU64::new(0);
        let unfinished = path.with_extension(format!(
            "{}-{}.unfinished",
            std::process::id(),
            WRITES.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&self.directory)
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
        Ok(path)
    }
}

/// Why a record was not written.
#[derive(Debug)]
#[non_exhaustive]
pub enum WriteError {
    /// The payload declares an invalid payload kind.
    InvalidPayloadKind(InvalidPayloadKind),
    /// The module is named by an absolute path, which no record holds.
    AbsolutePath {
        /// The module as named in the inputs.
        module: String,
    },
    /// The payload nests deeper than [`Envelope::MAX_PAYLOAD_DEPTH`], so no
    /// reader could read the record back.
    PayloadTooDeep {
        /// The payload kind of the payload.
        payload_kind: &'static str,
        /// How deep the payload nests.
        depth: usize,
    },
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
            Self::InvalidPayloadKind(error) => error.fmt(f),
            Self::AbsolutePath { module } => write!(
                f,
                "the module `{module}` is named by an absolute path, which a record never holds"
            ),
            Self::PayloadTooDeep {
                payload_kind,
                depth,
            } => write!(
                f,
                "the `{payload_kind}` payload nests {depth} deep, but a record holds at most {} \
                 levels",
                Envelope::MAX_PAYLOAD_DEPTH
            ),
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
            Self::InvalidPayloadKind(error) => Some(error),
            Self::Io { error, .. } => Some(error),
            Self::AbsolutePath { .. } | Self::PayloadTooDeep { .. } => None,
        }
    }
}

impl From<InvalidPayloadKind> for WriteError {
    fn from(error: InvalidPayloadKind) -> Self {
        Self::InvalidPayloadKind(error)
    }
}

/// Reads the records of a build directory.
///
/// Every record is checked before it is given out: its text is canonical
/// JSON and matches its digest, and its record key is the one it is stored
/// under.
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

    /// The record of what the stage of payload `P` produced from `inputs`.
    ///
    /// # Errors
    ///
    /// Fails when there is no record for these inputs, when the record is
    /// corrupted or stale, when its envelope version is not
    /// [`Envelope::VERSION`], or when its payload is not a `P`.
    pub fn read<P: Payload>(&self, inputs: &RecordInputs) -> Result<Record<P>, ReadError> {
        let payload_kind = payload::kind_of::<P>()?;
        let key = inputs.key(payload_kind, P::VERSION);
        let path = record_path(&self.directory, &key);
        if !path.exists() {
            return Err(ReadError::Missing {
                module: inputs.module.clone(),
                payload_kind,
            });
        }
        read_record(&path, &key)?.typed()
    }

    /// Every record in the build directory, each with an untyped payload, in
    /// the order of their file names. A record that cannot be read is given
    /// as its error, in its place, and hides no other record.
    ///
    /// This reads records of any payload kind, including those of payload
    /// types the caller does not know. It does not know the inputs of any
    /// record, so only [`RecordReader::read`] checks a record against the
    /// inputs it should come from.
    ///
    /// # Errors
    ///
    /// Fails when the build directory cannot be listed.
    pub fn read_all(&self) -> Result<Vec<Result<Record<Value>, ReadError>>, ReadError> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }
        let io_error = |error| ReadError::Io {
            path: self.directory.clone(),
            error,
        };
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.directory).map_err(io_error)? {
            let path = entry.map_err(io_error)?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                paths.push(path);
            }
        }
        paths.sort();

        let read = |path: &PathBuf| {
            // A record is stored under its record key and under no other name.
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            let stored_under = RecordKey::from_hex(name.into_owned())
                .ok_or_else(|| ReadError::Corrupted { path: path.clone() })?;
            read_record(path, &stored_under)
        };
        Ok(paths.iter().map(read).collect())
    }
}

/// The record in `file`, which is stored under the key `stored_under`.
fn read_record(file: &Path, stored_under: &RecordKey) -> Result<Record<Value>, ReadError> {
    let path = || file.to_owned();
    let text = match fs::read(file) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|_| ReadError::Corrupted { path: path() })?,
        Err(error) => {
            return Err(ReadError::Io {
                path: path(),
                error,
            });
        }
    };
    let (envelope, payload) = Envelope::open(&text).map_err(|unreadable| match unreadable {
        Unreadable::Corrupted => ReadError::Corrupted { path: path() },
        Unreadable::UnsupportedEnvelopeVersion { found } => ReadError::UnsupportedEnvelopeVersion {
            path: path(),
            found,
            supported: Envelope::VERSION,
        },
    })?;
    if envelope.key != *stored_under {
        return Err(ReadError::Stale { path: path() });
    }
    Ok(Record { envelope, payload })
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
    /// The file is not a record as Lens writes it, or its fields do not
    /// match its digest.
    Corrupted {
        /// The file of the record.
        path: PathBuf,
    },
    /// The record was computed from other inputs than those of the record
    /// key it is stored under.
    Stale {
        /// The file of the record.
        path: PathBuf,
    },
    /// The record has an envelope version this toolchain does not read.
    UnsupportedEnvelopeVersion {
        /// The file of the record.
        path: PathBuf,
        /// The envelope version of the record.
        found: u64,
        /// The envelope version this toolchain reads: [`Envelope::VERSION`].
        supported: u32,
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
    /// The record's payload is not in the form its payload type reads.
    InvalidPayload {
        /// The payload kind of the record.
        payload_kind: &'static str,
        /// The record key of the record.
        key: RecordKey,
    },
    /// The payload type asked for declares an invalid payload kind.
    InvalidPayloadKind(InvalidPayloadKind),
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
            Self::Corrupted { path } => {
                write!(f, "the record `{}` is corrupted", path.display())
            }
            Self::Stale { path } => write!(
                f,
                "the record `{}` is stale: it was computed from other inputs",
                path.display()
            ),
            Self::UnsupportedEnvelopeVersion {
                path,
                found,
                supported,
            } => write!(
                f,
                "the record `{}` has envelope version {found}, but this toolchain reads \
                 envelope version {supported}",
                path.display()
            ),
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
                "the payload of the `{payload_kind}` record `{key}` is not a `{payload_kind}` \
                 payload"
            ),
            Self::InvalidPayloadKind(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { error, .. } => Some(error),
            Self::InvalidPayloadKind(error) => Some(error),
            _ => None,
        }
    }
}

impl From<InvalidPayloadKind> for ReadError {
    fn from(error: InvalidPayloadKind) -> Self {
        Self::InvalidPayloadKind(error)
    }
}
