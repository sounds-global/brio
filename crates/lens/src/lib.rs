// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Brio Lens: the envelope shared by every stage record and execution record.
//!
//! This crate owns stable identifiers, the element table, provenance,
//! reasons, content-hash keys, masked markers and versioning. Each phase
//! crate defines its own payload types against it.
//!
//! A phase declares a payload type with [`PayloadType`], and says how its
//! payload is made of elements with [`Payload`]. It enters Lens once for
//! each module, with [`Stage::of`], and hands the stage and its payload to a
//! [`RecordWriter`]. Any tool reads the records back with a [`RecordReader`],
//! with or without knowing their payload types.
//!
//! A phase keeps a confidential value out of its records with
//! [`Element::masked_at`] and [`Reason::masked_at`]. When Lens is off, it
//! hands its records to [`RecordWriter::disabled`], which drops them and
//! does no work.
//!
//! A [`StageTest`] compares the records a build produced against reviewed
//! reference records, and says what was added, removed and changed in each
//! stage.

mod canonical;
mod element;
mod envelope;
mod fields;
mod hash;
mod identity;
mod inputs;
mod mask;
mod payload;
mod provenance;
mod reason;
mod record;
mod schema;
mod stage;
mod stage_test;

/// The JSON model of Lens: the types a schema fragment and a payload are
/// written in.
///
/// A phase crate uses this module and never depends on a JSON library of its
/// own, so every phase shares one version of the model.
pub use serde_json as json;

pub use element::{Element, ElementTable};
pub use envelope::{Envelope, RecordDefect, RecordRefusal};
pub use identity::StableId;
pub use inputs::{RecordInputs, RecordKey};
pub use mask::{MaskedMarker, MaskedPart};
pub use payload::{InvalidPayloadKind, Payload, PayloadSchema, PayloadType};
pub use provenance::{ElementLink, OriginStep, Provenance, SourceSpan};
pub use reason::Reason;
pub use record::{ReadError, Record, RecordReader, RecordWriter, Untyped, WriteError};
pub use schema::{Schema, SchemaError, SchemaVersion};
pub use stage::Stage;
pub use stage_test::{AmbiguousStage, RecordSet, StageDifference, StageTest};
