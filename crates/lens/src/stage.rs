// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The stage: where a payload type and a module enter Lens.

use std::fmt;
use std::marker::PhantomData;

use crate::envelope::{self, RecordRefusal};
use crate::payload::PayloadType;
use crate::{ElementLink, RecordInputs, RecordKey, StableId};

/// One stage at work on one module: the stage of payload `P`, and the module
/// it produces a record for.
///
/// The payload kind of `P` and the name of the module are checked once, when
/// the stage is made. Everything that identifies a record or one of its
/// elements comes from this one checked identity: the stable identifiers,
/// the record key and the envelope.
pub struct Stage<P> {
    module: String,
    payload_kind: &'static str,
    payload_version: u32,
    payload_type: PhantomData<fn() -> P>,
}

impl<P: PayloadType> Stage<P> {
    /// The stage of payload `P` for `module`, named as it is in the
    /// workspace.
    ///
    /// # Errors
    ///
    /// Fails when the payload kind of `P` breaks the rule on
    /// [`PayloadType::KIND`], or when `module` is named by an absolute path,
    /// which no record holds.
    pub fn of(module: impl Into<String>) -> Result<Self, RecordRefusal> {
        let module = module.into();
        let payload_kind = envelope::admit::<P>(&module)?;
        Ok(Self {
            module,
            payload_kind,
            payload_version: P::VERSION,
            payload_type: PhantomData,
        })
    }

    /// The stable identifier of the element at `place` in what this stage
    /// produces for its module.
    ///
    /// `place` is the path to the element in the stage's own structure, from
    /// the outside in, such as `["main", "parameter", "count"]`. Build it
    /// from what identifies the element among its siblings, such as a name.
    /// Never build it from a text offset, a counter over the whole module or
    /// a memory address: these change when an unrelated element does.
    #[must_use]
    pub fn id(&self, place: &[&str]) -> StableId {
        StableId::derive(&self.module, self.payload_kind, place)
    }

    /// A link to the element at `place` in what this stage produces for its
    /// module, for the provenance of an element of another record.
    ///
    /// `place` is the same path that [`Stage::id`] takes.
    #[must_use]
    pub fn link(&self, place: &[&str]) -> ElementLink {
        ElementLink::new(&self.module, self.payload_kind, self.id(place))
    }

    /// The key of the record this stage computes from `inputs`.
    pub(crate) fn key(&self, inputs: &RecordInputs) -> RecordKey {
        inputs.key(self.payload_kind, self.payload_version, &self.module)
    }
}

impl<P> Stage<P> {
    /// The module this stage produces a record for.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The payload kind of this stage's payload.
    #[must_use]
    pub fn payload_kind(&self) -> &'static str {
        self.payload_kind
    }

    /// The version of the format of this stage's payload.
    #[must_use]
    pub fn payload_version(&self) -> u32 {
        self.payload_version
    }
}

// Written by hand: a stage can be copied and shown whatever `P` can do.
impl<P> Clone for Stage<P> {
    fn clone(&self) -> Self {
        Self {
            module: self.module.clone(),
            payload_kind: self.payload_kind,
            payload_version: self.payload_version,
            payload_type: PhantomData,
        }
    }
}

impl<P> fmt::Debug for Stage<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("module", &self.module)
            .field("payload_kind", &self.payload_kind)
            .field("payload_version", &self.payload_version)
            .finish()
    }
}
