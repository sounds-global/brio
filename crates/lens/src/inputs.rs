// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! The inputs of a record, and the record key that covers them.

use std::collections::BTreeMap;
use std::fmt;

#[cfg(doc)]
use crate::Stage;
use crate::hash::{self, FieldHasher};
use crate::stage::StageIdentity;

/// Everything a record is computed from, apart from the [`Stage`] that
/// computes it: the payload type and the module.
///
/// A record is stored under the hash of these inputs and of its stage, so a
/// change to any of them gives another record and never an out-of-date one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordInputs {
    /// The inputs of the module, by name: its source and the interfaces of
    /// the modules it depends on. Each is the content itself or a hash of it.
    pub module_inputs: BTreeMap<String, Vec<u8>>,
    /// The version of the toolchain.
    pub toolchain_version: String,
    /// The target the build is for.
    pub target: String,
    /// The build settings, by name.
    pub build_settings: BTreeMap<String, String>,
}

impl RecordInputs {
    /// The key of the record that `stage`, with this payload version,
    /// computes from these inputs.
    pub(crate) fn key(&self, stage: &StageIdentity, payload_version: u32) -> RecordKey {
        let mut hasher = FieldHasher::new("brio lens record key");
        hasher.text(stage.payload_kind());
        hasher.number(u64::from(payload_version));
        hasher.text(stage.module());
        hasher.count(self.module_inputs.len());
        for (name, content) in &self.module_inputs {
            hasher.text(name);
            hasher.bytes(content);
        }
        hasher.text(&self.toolchain_version);
        hasher.text(&self.target);
        hasher.count(self.build_settings.len());
        for (name, value) in &self.build_settings {
            hasher.text(name);
            hasher.text(value);
        }
        RecordKey(hash::hex(&hasher.finish()))
    }
}

/// The content-hash key of a record: a hash of the stage, the module and its
/// inputs, the toolchain version, the target and the build settings.
///
/// It is written as 64 lowercase hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordKey(String);

impl RecordKey {
    /// The key a record declares for itself, as read, or `None` when `hex`
    /// is not 64 lowercase hexadecimal digits.
    pub(crate) fn from_hex(hex: String) -> Option<Self> {
        hash::is_hex(&hex, 64).then_some(Self(hex))
    }
}

impl fmt::Display for RecordKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
