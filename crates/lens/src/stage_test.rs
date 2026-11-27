// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Stage tests: comparing produced records against reference records.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::stage::StageIdentity;
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
    ) -> Result<Self, StageTestError> {
        let produced = by_stage(RecordSet::Produced, produced)?;
        let reference = by_stage(RecordSet::Reference, reference)?;
        let stages: BTreeSet<&StageIdentity> =
            produced.keys().chain(reference.keys()).copied().collect();
        let differences = stages
            .into_iter()
            .filter_map(|stage| {
                let record = |set: &BTreeMap<_, &'a Envelope>| set.get(stage).copied();
                StageDifference::between(stage, record(&produced), record(&reference))
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

/// The records of `set` with `envelopes`, each under its stage.
fn by_stage<'a>(
    set: RecordSet,
    envelopes: impl IntoIterator<Item = &'a Envelope>,
) -> Result<BTreeMap<&'a StageIdentity, &'a Envelope>, StageTestError> {
    let mut records = BTreeMap::new();
    for envelope in envelopes {
        let stage = envelope.stage();
        if records.insert(stage, envelope).is_some() {
            return Err(StageTestError::TwoRecordsOfOneStage {
                set,
                module: stage.module().to_owned(),
                payload_kind: stage.payload_kind().to_owned(),
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
    stage: StageIdentity,
    records: ComparedRecords,
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
        stage: &StageIdentity,
        produced: Option<&Envelope>,
        reference: Option<&Envelope>,
    ) -> Option<Self> {
        let records = match (produced, reference) {
            (Some(produced), Some(reference)) => ComparedRecords::Both {
                produced_payload_version: produced.payload_version(),
                reference_payload_version: reference.payload_version(),
            },
            (Some(_), None) => ComparedRecords::ProducedOnly,
            (None, Some(_)) => ComparedRecords::ReferenceOnly,
            // A stage with no record is in neither set.
            (None, None) => return None,
        };
        let none = ElementTable::default();
        let produced_elements = produced.map_or(&none, Envelope::elements);
        let reference_elements = reference.map_or(&none, Envelope::elements);
        let only_in = |these: &ElementTable, those: &ElementTable| -> Vec<StableId> {
            let missing = these
                .iter()
                .filter(|element| those.get(element.id()).is_none());
            missing.map(|element| element.id().clone()).collect()
        };
        let changed = produced_elements.iter().filter(|element| {
            reference_elements
                .get(element.id())
                .is_some_and(|counterpart| counterpart != *element)
        });
        // The roots are in no element, so their order is compared apart:
        // the roots that both records have, in the order of each.
        let shared_roots = |these: &ElementTable, those: &ElementTable| -> Vec<StableId> {
            let shared = these.roots().iter().filter(|id| those.roots().contains(id));
            shared.cloned().collect()
        };

        let difference = Self {
            stage: stage.clone(),
            records,
            added: only_in(produced_elements, reference_elements),
            removed: only_in(reference_elements, produced_elements),
            changed: changed.map(|element| element.id().clone()).collect(),
            roots_reordered: shared_roots(produced_elements, reference_elements)
                != shared_roots(reference_elements, produced_elements),
        };
        (!difference.is_none()).then_some(difference)
    }

    /// Whether this says that the two records are the same.
    fn is_none(&self) -> bool {
        let same_version = match self.records {
            ComparedRecords::Both {
                produced_payload_version,
                reference_payload_version,
            } => produced_payload_version == reference_payload_version,
            ComparedRecords::ProducedOnly | ComparedRecords::ReferenceOnly => false,
        };
        same_version
            && self.added.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && !self.roots_reordered
    }

    /// The module the stage works on.
    #[must_use]
    pub fn module(&self) -> &str {
        self.stage.module()
    }

    /// The payload kind of what the stage produces.
    #[must_use]
    pub fn payload_kind(&self) -> &str {
        self.stage.payload_kind()
    }

    /// Which of the two record sets have a record of the stage.
    #[must_use]
    pub fn records(&self) -> ComparedRecords {
        self.records
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

/// The records of one stage that a Stage test found in its two record sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparedRecords {
    /// Both sets have a record of the stage.
    Both {
        /// The payload version of the produced record.
        produced_payload_version: u32,
        /// The payload version of the reference record.
        reference_payload_version: u32,
    },
    /// The stage has no reference record: every element is added.
    ProducedOnly,
    /// No record of the stage was produced: every element is removed.
    ReferenceOnly,
}

/// Why two record sets cannot be compared.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StageTestError {
    /// One set holds two records of one stage, so nothing says which of the
    /// two to compare.
    TwoRecordsOfOneStage {
        /// The record set that holds the two records.
        set: RecordSet,
        /// The module of the stage.
        module: String,
        /// The payload kind of the stage.
        payload_kind: String,
    },
}

impl fmt::Display for StageTestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TwoRecordsOfOneStage {
                set,
                module,
                payload_kind,
            } => write!(
                f,
                "the {set} records hold two `{payload_kind}` records of module `{module}`: \
                 the set holds the records of more than one build"
            ),
        }
    }
}

impl std::error::Error for StageTestError {}
