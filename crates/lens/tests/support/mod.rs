// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! What the tests of records share: payload types no phase owns, and the
//! workspaces records are written into.

#![allow(
    dead_code,
    reason = "each test binary uses its own part of this module"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use brio_lens::json::{self, Value, json};
use brio_lens::{
    Element, ElementTable, Envelope, Payload, PayloadType, ReadError, RecordDefect, RecordInputs,
    RecordReader, Stage,
};

/// A payload no phase owns: the reading of every sensor a module declares.
#[derive(Debug, PartialEq)]
pub struct Probe {
    pub readings: Vec<Reading>,
}

/// One element of a [`Probe`].
#[derive(Debug, PartialEq)]
pub struct Reading {
    pub sensor: String,
    pub value: i64,
}

impl PayloadType for Probe {
    const KIND: &'static str = "probe";
    const VERSION: u32 = 3;

    fn schema_fragment() -> Value {
        json!({
            "type": "object",
            "required": ["sensor", "value"],
            "properties": { "sensor": { "type": "string" }, "value": { "type": "integer" } }
        })
    }
}

impl Payload for Probe {
    /// A reading's place in the stage's structure is its sensor's name.
    fn to_elements(&self, stage: &Stage<Self>) -> Vec<Element> {
        self.readings
            .iter()
            .map(|reading| {
                Element::new(
                    stage.id(&["sensor", &reading.sensor]),
                    json!({ "sensor": reading.sensor, "value": reading.value }),
                )
            })
            .collect()
    }

    fn from_elements(elements: &ElementTable) -> Option<Self> {
        let readings = elements
            .roots()
            .iter()
            .map(|id| {
                let reading = elements.get(id)?.payload();
                Some(Reading {
                    sensor: reading["sensor"].as_str()?.to_owned(),
                    value: reading["value"].as_i64()?,
                })
            })
            .collect::<Option<Vec<Reading>>>()?;
        Some(Self { readings })
    }
}

/// What the probe stage produces for a module that declares `sensors`.
pub fn probe(sensors: &[(&str, i64)]) -> Probe {
    let readings = sensors
        .iter()
        .map(|(sensor, value)| Reading {
            sensor: (*sensor).to_owned(),
            value: *value,
        })
        .collect();
    Probe { readings }
}

/// A second payload type, which a reader of [`Probe`] records does not
/// expect: one element, the state of the module's gauge.
#[derive(Debug, PartialEq)]
pub struct Gauge(pub String);

impl PayloadType for Gauge {
    const KIND: &'static str = "gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "string" })
    }
}

impl Payload for Gauge {
    fn to_elements(&self, stage: &Stage<Self>) -> Vec<Element> {
        vec![Element::new(
            stage.id(&["gauge"]),
            Value::from(self.0.as_str()),
        )]
    }

    fn from_elements(elements: &ElementTable) -> Option<Self> {
        let [root] = elements.roots() else {
            return None;
        };
        Some(Self(elements.get(root)?.payload().as_str()?.to_owned()))
    }
}

/// A payload that is whatever elements a test describes. It is read back
/// through the element table alone, as a tool that knows no payload type
/// reads it.
pub struct Loose(pub fn(&Stage<Loose>) -> Vec<Element>);

impl PayloadType for Loose {
    const KIND: &'static str = "loose";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({})
    }
}

impl Payload for Loose {
    fn to_elements(&self, stage: &Stage<Self>) -> Vec<Element> {
        (self.0)(stage)
    }

    fn from_elements(_elements: &ElementTable) -> Option<Self> {
        Some(Self(|_| Vec::new()))
    }
}

/// The stage of payload `P` for `module`.
pub fn stage<P: PayloadType>(module: &str) -> Stage<P> {
    Stage::of(module).expect("the payload kind and the module are valid")
}

/// The inputs of a module whose source is `source`.
pub fn inputs(source: &str) -> RecordInputs {
    RecordInputs {
        module_inputs: BTreeMap::from([("source".to_owned(), source.as_bytes().to_vec())]),
        toolchain_version: "0.1.0".to_owned(),
        target: "aarch64-unknown-linux-gnu".to_owned(),
        build_settings: BTreeMap::from([("profile".to_owned(), "dev".to_owned())]),
    }
}

/// Where the workspace of the test called `test` is.
fn workspace_path(test: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("records")
        .join(test)
}

/// The build directory in the workspace of the test called `test`.
pub fn build_directory(test: &str) -> PathBuf {
    workspace_path(test).join(".brio")
}

/// The envelopes of the records in the build directory of `test`, read as a
/// tool that knows no payload type reads them, in the order of their
/// payload kinds and then of their modules.
pub fn envelopes(test: &str) -> Vec<Envelope> {
    let mut envelopes: Vec<Envelope> = RecordReader::in_build_directory(build_directory(test))
        .read_all()
        .expect("the records are read")
        .into_iter()
        .map(|record| record.expect("the record is whole").envelope)
        .collect();
    envelopes.sort_by(|left, right| {
        (left.payload_kind(), left.module()).cmp(&(right.payload_kind(), right.module()))
    });
    envelopes
}

/// `depth` arrays nested inside each other, around nothing.
pub fn nested(depth: usize) -> Value {
    (0..depth).fold(Value::Null, |inner, _| json!([inner]))
}

/// A change that a tool outside the toolchain makes to a record.
pub type Forge<'a> = &'a dyn Fn(&mut Value);

/// Checks that each of `forgeries`, made to the one record `whole` of the
/// build directory of `test`, which is written at `path`, gives a record
/// that is rejected as corrupted although its digest matches.
pub fn assert_forgeries_are_corrupted(
    test: &str,
    path: &Path,
    whole: &Value,
    forgeries: &[(&str, Forge)],
) {
    let reader = RecordReader::in_build_directory(build_directory(test));
    for (what, forge) in forgeries {
        let mut forged = whole.clone();
        forge(&mut forged);
        fs::write(path, resealed(forged)).expect("the record can be changed");
        let read = reader.read_all().expect("the records are read");
        assert!(
            matches!(
                read[..],
                [Err(ReadError::Defective {
                    defect: RecordDefect::Corrupted,
                    ..
                })]
            ),
            "a record with {what} is read"
        );
    }
}

/// An empty workspace for the test called `test`.
pub fn workspace(test: &str) -> PathBuf {
    let workspace = workspace_path(test);
    if workspace.exists() {
        fs::remove_dir_all(&workspace).expect("the old workspace can be removed");
    }
    fs::create_dir_all(&workspace).expect("the workspace can be created");
    workspace
}

/// Every file under `directory`, at any depth.
pub fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).expect("the directory can be listed") {
        let path = entry.expect("the entry can be read").path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files
}

/// Changes the one written record at `path` by replacing `from` with `to`.
pub fn tamper(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).expect("the record can be read");
    assert!(text.contains(from), "the record has `{from}`:\n{text}");
    fs::write(path, text.replacen(from, to, 1)).expect("the record can be changed");
}

/// The record `text` with the digest the published schema describes: the
/// BLAKE3 of the record without its digest field. Worked out here, without
/// Lens, as any tool outside the toolchain would.
pub fn with_its_own_digest(text: &str) -> String {
    let (digest_line, rest) = text
        .strip_prefix("{\n")
        .and_then(|fields| fields.split_once('\n'))
        .expect("the digest is the first field of a record");
    assert!(digest_line.starts_with(r#"  "digest": ""#), "{digest_line}");
    let without_digest = format!("{{\n{rest}");
    let digest = blake3::hash(without_digest.as_bytes()).to_hex();
    format!("{{\n  \"digest\": \"{digest}\",\n{rest}")
}

/// The text of `record` as a record is written, with a digest to match:
/// what a tool outside the toolchain would write after changing a record.
pub fn resealed(record: Value) -> String {
    // The JSON model sorts the keys of every object it reads.
    let text = json::to_string_pretty(&record).expect("a record is JSON") + "\n";
    with_its_own_digest(&text)
}
