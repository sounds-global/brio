// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Brio Lens: the envelope shared by every stage record and execution record.
//!
//! This crate owns stable identifiers, provenance, reasons, content-hash keys
//! and versioning. Each phase crate defines its own payload types against it.

mod payload;
mod schema;

/// The JSON model of Lens: the types a schema fragment is written in.
///
/// A phase crate uses this module and never depends on a JSON library of its
/// own, so every phase shares one version of the model.
pub use serde_json as json;

pub use payload::{Payload, PayloadSchema};
pub use schema::{Schema, SchemaError, SchemaVersion};
