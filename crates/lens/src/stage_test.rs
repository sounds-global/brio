// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Stage tests: comparing produced records against reference records.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{ElementTable, Envelope, StableId};

/// A Stage test: the comparison of a set of produced records against a set
/// of reviewed reference records.
///
/// The record of a stage is matched with the reference record of the same
/// module and payload kind, and their elements are matched by stable
/// identifier. The record keys are not compared: a reference record is
/// computed from other inputs, such as another toolchain version.
///
/// It only reads and compares records: it renders nothing, and needs
/// nothing of the Lens viewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageTest {
    differences: Vec<StageDifference>,
}

impl StageTest {
    /// Compares the records with the envelopes `produced` against those
    /// with the envelopes `reference`.
    ///
    /// # Errors
    ///
    /// Fails when one set holds two records of one stage, such as a build
    /// directory that still holds the record of an earlier build: nothing
    /// says which of the two to compare.
    pub fn compare<'a>(
        produced: impl IntoIterator<Item = &'a Envelope>,
        reference: impl IntoIterator<Item = &'a Envelope>,
    ) -> Result<Self, AmbiguousStage> {
        let produced = by_stage(RecordSet::Produced, produced)?;
        let reference = by_stage(RecordSet::Reference, reference)?;
        let stages: BTreeSet<&StageName> = produced.keys().chain(reference.keys()).collect();
        let differences = stages
            .into_iter()
            .filter_map(|stage| {
                let record = |set: &BTreeMap<_, &'a Envelope>| set.get(stage).copied();
                StageDifference::between(*stage, record(&produced), record(&reference))
            })
            .collect();
        Ok(Self { differences })
    }

    /// Whether the produced records are the same as the reference records.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.differences.is_empty()
    }

    /// Every stage whose produced record differs from its reference record,
    /// in the order of their modules and then of their payload kinds.
    #[must_use]
    pub fn differences(&self) -> &[StageDifference] {
        &self.differences
    }
}

/// What names a stage in a record set: its module, then its payload kind.
type StageName<'a> = (&'a str, &'a str);

/// The records of `set` with `envelopes`, each under the name of its stage.
fn by_stage<'a>(
    set: RecordSet,
    envelopes: impl IntoIterator<Item = &'a Envelope>,
) -> Result<BTreeMap<StageName<'a>, &'a Envelope>, AmbiguousStage> {
    let mut records = BTreeMap::new();
    for envelope in envelopes {
        let stage = (envelope.module(), envelope.payload_kind());
        if records.insert(stage, envelope).is_some() {
            return Err(AmbiguousStage {
                set,
                module: stage.0.to_owned(),
                payload_kind: stage.1.to_owned(),
            });
        }
    }
    Ok(records)
}

/// How the produced record of one stage differs from its reference record.
///
/// Every list of elements is in the order of their stable identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageDifference {
    module: String,
    payload_kind: String,
    produced_payload_version: Option<u32>,
    reference_payload_version: Option<u32>,
    added: Vec<StableId>,
    removed: Vec<StableId>,
    changed: Vec<StableId>,
    roots_reordered: bool,
}

impl StageDifference {
    /// How the record `produced` of `stage` differs from the record
    /// `reference`, or `None` when they are the same. A set that has no
    /// record of the stage gives `None` in place of the record.
    fn between(
        stage: StageName,
        produced: Option<&Envelope>,
        reference: Option<&Envelope>,
    ) -> Option<Self> {
        let none = ElementTable::default();
        let new = produced.map_or(&none, Envelope::elements);
        let old = reference.map_or(&none, Envelope::elements);
        let only_in = |these: &ElementTable, those: &ElementTable| -> Vec<StableId> {
            let missing = these
                .iter()
                .filter(|element| those.get(element.id()).is_none());
            missing.map(|element| element.id().clone()).collect()
        };
        let changed = new.iter().filter(|element| {
            old.get(element.id())
                .is_some_and(|reference| reference != *element)
        });
        // The roots are in no element, so their order is compared apart:
        // the roots that both records have, in the order of each.
        let shared_roots = |these: &ElementTable, those: &ElementTable| -> Vec<StableId> {
            let shared = these.roots().iter().filter(|id| those.roots().contains(id));
            shared.cloned().collect()
        };

        let difference = Self {
            module: stage.0.to_owned(),
            payload_kind: stage.1.to_owned(),
            produced_payload_version: produced.map(Envelope::payload_version),
            reference_payload_version: reference.map(Envelope::payload_version),
            added: only_in(new, old),
            removed: only_in(old, new),
            changed: changed.map(|element| element.id().clone()).collect(),
            roots_reordered: shared_roots(new, old) != shared_roots(old, new),
        };
        let same = difference.produced_payload_version == difference.reference_payload_version
            && difference.added.is_empty()
            && difference.removed.is_empty()
            && difference.changed.is_empty()
            && !difference.roots_reordered;
        (!same).then_some(difference)
    }

    /// The module the stage works on.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The payload kind of what the stage produces.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        &self.payload_kind
    }

    /// The payload version of the produced record, or `None` when no record
    /// of the stage was produced: every element is then removed.
    #[must_use]
    pub fn produced_payload_version(&self) -> Option<u32> {
        self.produced_payload_version
    }

    /// The payload version of the reference record, or `None` when the
    /// stage has no reference record: every element is then added.
    #[must_use]
    pub fn reference_payload_version(&self) -> Option<u32> {
        self.reference_payload_version
    }

    /// The elements that only the produced record has.
    #[must_use]
    pub fn added(&self) -> &[StableId] {
        &self.added
    }

    /// The elements that only the reference record has.
    #[must_use]
    pub fn removed(&self) -> &[StableId] {
        &self.removed
    }

    /// The elements that both records have, and that are not the same in
    /// both: in their payload, their masked markers, their provenance,
    /// their reasons, their parent or their children.
    #[must_use]
    pub fn changed(&self) -> &[StableId] {
        &self.changed
    }

    /// Whether the roots that both records have are in another order in the
    /// produced record. The order of the other elements is in their
    /// parents, so a parent whose children are in another order is changed.
    #[must_use]
    pub fn roots_reordered(&self) -> bool {
        self.roots_reordered
    }
}

/// One of the two record sets of a Stage test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordSet {
    /// The records a build produced.
    Produced,
    /// The reviewed records the produced ones are compared against.
    Reference,
}

impl fmt::Display for RecordSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Produced => "produced",
            Self::Reference => "reference",
        })
    }
}

/// Why two record sets cannot be compared: one of them holds two records of
/// one stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousStage {
    set: RecordSet,
    module: String,
    payload_kind: String,
}

impl AmbiguousStage {
    /// The record set that holds the two records.
    #[must_use]
    pub fn set(&self) -> RecordSet {
        self.set
    }

    /// The module of the stage.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The payload kind of the stage.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        &self.payload_kind
    }
}

impl fmt::Display for AmbiguousStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {} records hold two `{}` records of module `{}`",
            self.set, self.payload_kind, self.module
        )
    }
}

impl std::error::Error for AmbiguousStage {}
