// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The stage: where a payload type and a module enter Lens.

use std::fmt;
use std::marker::PhantomData;

use crate::envelope::{self, RecordRefusal};
use crate::json::Value;
use crate::payload::{self, PayloadType};
use crate::schema::reference;
use crate::{ElementLink, RecordInputs, RecordKey, StableId};

/// The JSON name of the module of a stage.
const MODULE: &str = "module";
/// The JSON name of the payload kind of a stage.
pub(crate) const PAYLOAD_KIND: &str = "payload_kind";

/// What identifies a stage, in a record and in every link to one of its
/// elements: the module it works on and the payload kind of what it
/// produces.
///
/// One is only ever made of a module and a payload kind that a stage can
/// have, so the rule on both is applied once, here.
///
/// Identities are in the order of their modules, then of their payload
/// kinds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StageIdentity {
    module: String,
    payload_kind: String,
}

impl StageIdentity {
    /// The identity of the stage with `payload_kind` at work on `module`, or
    /// `None` when no stage can have them: the module is named by an
    /// absolute path, or the payload kind breaks the rule on
    /// [`PayloadType::KIND`].
    pub(crate) fn admitted(module: &str, payload_kind: &str) -> Option<Self> {
        (!envelope::is_absolute_path(module) && payload::is_kind(payload_kind)).then(|| Self {
            module: module.to_owned(),
            payload_kind: payload_kind.to_owned(),
        })
    }

    /// The module the stage works on.
    pub(crate) fn module(&self) -> &str {
        &self.module
    }

    /// The payload kind of what the stage produces.
    pub(crate) fn payload_kind(&self) -> &str {
        &self.payload_kind
    }

    /// The stable identifier of the element at `place` in what the stage
    /// produces.
    pub(crate) fn id(&self, place: &[&str]) -> StableId {
        StableId::derive(&self.module, &self.payload_kind, place)
    }

    /// The fields that write this identity in a record.
    pub(crate) fn fields(&self) -> [(&'static str, Value); 2] {
        [
            (MODULE, Value::from(self.module.as_str())),
            (PAYLOAD_KIND, Value::from(self.payload_kind.as_str())),
        ]
    }

    /// The identity that the fields of `object` write, or `None` when they
    /// are not the identity of a stage.
    pub(crate) fn from_json(object: &Value) -> Option<Self> {
        Self::admitted(
            object.get(MODULE)?.as_str()?,
            object.get(PAYLOAD_KIND)?.as_str()?,
        )
    }

    /// The JSON Schema of the fields that write an identity.
    pub(crate) fn schema_fields() -> [(&'static str, Value); 2] {
        [
            (MODULE, reference(envelope::NAME_DEFINITION)),
            (PAYLOAD_KIND, reference(payload::KIND_DEFINITION)),
        ]
    }
}

/// One stage at work on one module: the stage of payload `P`, and the module
/// it produces a record for.
///
/// The payload kind of `P` and the name of the module are checked once, when
/// the stage is made. Everything that identifies a record or one of its
/// elements comes from this one checked identity: the stable identifiers,
/// the record key and the envelope.
pub struct Stage<P> {
    identity: StageIdentity,
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
    pub fn of(module: impl AsRef<str>) -> Result<Self, RecordRefusal> {
        Ok(Self {
            identity: envelope::admit::<P>(module.as_ref())?,
            payload_version: P::VERSION,
            payload_type: PhantomData,
        })
    }
}

impl<P> Stage<P> {
    /// The stable identifier of the element at `place` in what this stage
    /// produces for its module.
    ///
    /// `place` is the path to the element in the stage's own structure, from
    /// the outside in, such as `["main", "parameter", "count"]`. Build it
    /// from what identifies the element among its siblings, such as a name.
    /// Never build it from a text offset, a counter over the whole module or
    /// a memory address: these change when an unrelated element does. Never
    /// build it from a confidential value either: a stable identifier is
    /// written in the record, and nothing masks it.
    #[must_use]
    pub fn id(&self, place: &[&str]) -> StableId {
        self.identity.id(place)
    }

    /// A link to the element at `place` in what this stage produces for its
    /// module, for the provenance of an element of another record.
    ///
    /// `place` is the same path that [`Stage::id`] takes.
    #[must_use]
    pub fn link(&self, place: &[&str]) -> ElementLink {
        ElementLink::new(self.identity.clone(), self.id(place))
    }

    /// The module this stage produces a record for.
    #[must_use]
    pub fn module(&self) -> &str {
        self.identity.module()
    }

    /// The payload kind of this stage's payload.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        self.identity.payload_kind()
    }

    /// The version of the format of this stage's payload.
    #[must_use]
    pub fn payload_version(&self) -> u32 {
        self.payload_version
    }

    /// What identifies this stage in a record.
    pub(crate) fn identity(&self) -> &StageIdentity {
        &self.identity
    }

    /// The key of the record this stage computes from `inputs`.
    pub(crate) fn key(&self, inputs: &RecordInputs) -> RecordKey {
        inputs.key(&self.identity, self.payload_version)
    }
}

// Written by hand: a stage can be copied and shown whatever `P` can do.
impl<P> Clone for Stage<P> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            payload_version: self.payload_version,
            payload_type: PhantomData,
        }
    }
}

impl<P> fmt::Debug for Stage<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage")
            .field("module", &self.module())
            .field("payload_kind", &self.payload_kind())
            .field("payload_version", &self.payload_version)
            .finish()
    }
}
