// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Brio Lens: the envelope shared by every stage record and execution record.
//!
//! This crate owns stable identifiers, provenance, reasons, content-hash keys
//! and versioning. Each phase crate defines its own payload types against it.

mod payload;
mod schema;

pub use payload::{Payload, PayloadSchema};
pub use schema::{ENVELOPE_VERSION, Schema, SchemaError};
