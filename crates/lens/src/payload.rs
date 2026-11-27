// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The payload contract a phase implements.

use serde_json::Value;

/// What a phase declares for each payload type it records.
///
/// `brio_lens` knows no payload type: a phase crate implements this trait for
/// its own types.
pub trait Payload {
    /// The payload kind: the name of this payload type, such as `"tokens"`.
    ///
    /// It identifies the payload in every record and in the published schema,
    /// so it is unique among the payloads linked into one toolchain. It is
    /// made of lowercase ASCII letters, digits and `_`.
    const KIND: &'static str;

    /// The version of this payload's format, owned by the phase.
    const VERSION: u32;

    /// The JSON Schema of this payload, as it appears in the published schema.
    fn schema_fragment() -> Value;
}

/// The declaration of one payload type, with the type itself erased.
///
/// This is what schema assembly takes: a tool that links several phases
/// collects one of these for each payload type.
#[derive(Debug)]
pub struct PayloadSchema {
    pub(crate) kind: &'static str,
    pub(crate) version: u32,
    pub(crate) fragment: Value,
}

impl PayloadSchema {
    /// Captures what `P` declares against the payload contract.
    #[must_use]
    pub fn of<P: Payload>() -> Self {
        Self {
            kind: P::KIND,
            version: P::VERSION,
            fragment: P::schema_fragment(),
        }
    }
}
