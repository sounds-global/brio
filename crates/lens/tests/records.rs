// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Writing and reading stage records, through the public interface of
//! `brio_lens`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use brio_lens::json::{Value, json};
use brio_lens::{
    Envelope, InvalidPayloadKind, Payload, ReadError, RecordDefect, RecordInputs, RecordReader,
    RecordRefusal, RecordWriter, StableId, WriteError,
};

/// A payload no phase owns: the reading of every sensor a module declares.
#[derive(Debug, PartialEq)]
struct Probe {
    readings: Vec<Reading>,
}

/// One element of a [`Probe`].
#[derive(Debug, PartialEq)]
struct Reading {
    id: StableId,
    sensor: String,
    value: i64,
}

impl Payload for Probe {
    const KIND: &'static str = "probe";
    const VERSION: u32 = 3;

    fn schema_fragment() -> Value {
        json!({ "type": "object" })
    }

    fn to_json(&self) -> Value {
        let readings: Vec<Value> = self
            .readings
            .iter()
            .map(|reading| {
                json!({
                    "id": reading.id.to_json(),
                    "sensor": reading.sensor,
                    "value": reading.value
                })
            })
            .collect();
        json!({ "readings": readings })
    }

    fn from_json(payload: &Value) -> Option<Self> {
        let readings = payload["readings"]
            .as_array()?
            .iter()
            .map(|reading| {
                Some(Reading {
                    id: StableId::from_json(&reading["id"])?,
                    sensor: reading["sensor"].as_str()?.to_owned(),
                    value: reading["value"].as_i64()?,
                })
            })
            .collect::<Option<Vec<Reading>>>()?;
        Some(Self { readings })
    }
}

/// What the probe stage produces for `module` when it declares `sensors`.
///
/// A reading's place in the stage's structure is its sensor's name.
fn probe(module: &str, sensors: &[(&str, i64)]) -> Probe {
    let readings = sensors
        .iter()
        .map(|(sensor, value)| Reading {
            id: StableId::of::<Probe>(module, &["sensor", sensor]).expect("`probe` is a kind"),
            sensor: (*sensor).to_owned(),
            value: *value,
        })
        .collect();
    Probe { readings }
}

/// A second payload type, which a reader of [`Probe`] records does not expect.
#[derive(Debug, PartialEq)]
struct Gauge(String);

impl Payload for Gauge {
    const KIND: &'static str = "gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "string" })
    }

    fn to_json(&self) -> Value {
        Value::from(self.0.as_str())
    }

    fn from_json(payload: &Value) -> Option<Self> {
        Some(Self(payload.as_str()?.to_owned()))
    }
}

/// A payload of numbers that are not whole.
#[derive(Debug, PartialEq)]
struct Levels(Vec<f64>);

impl Payload for Levels {
    const KIND: &'static str = "levels";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "array", "items": { "type": "number" } })
    }

    fn to_json(&self) -> Value {
        Value::from(self.0.clone())
    }

    fn from_json(payload: &Value) -> Option<Self> {
        payload
            .as_array()?
            .iter()
            .map(Value::as_f64)
            .collect::<Option<Vec<f64>>>()
            .map(Self)
    }
}

/// The inputs of the module `app::main` when its source is `source`.
fn inputs(source: &str) -> RecordInputs {
    RecordInputs {
        module: "app::main".to_owned(),
        module_inputs: BTreeMap::from([("source".to_owned(), source.as_bytes().to_vec())]),
        toolchain_version: "0.1.0".to_owned(),
        target: "aarch64-unknown-linux-gnu".to_owned(),
        build_settings: BTreeMap::from([("profile".to_owned(), "dev".to_owned())]),
    }
}

/// An empty workspace for the test called `test`.
fn workspace(test: &str) -> PathBuf {
    let workspace = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("records")
        .join(test);
    if workspace.exists() {
        fs::remove_dir_all(&workspace).expect("the old workspace can be removed");
    }
    fs::create_dir_all(&workspace).expect("the workspace can be created");
    workspace
}

/// Every file under `directory`, at any depth.
fn files_under(directory: &Path) -> Vec<PathBuf> {
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
fn tamper(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).expect("the record can be read");
    assert!(text.contains(from), "the record has `{from}`:\n{text}");
    fs::write(path, text.replacen(from, to, 1)).expect("the record can be changed");
}

#[test]
fn writing_then_reading_a_record_gives_back_the_same_envelope_and_payload() {
    let build_directory = workspace("round_trip").join(".brio");
    let inputs = inputs("sensor a\nsensor b\n");
    let written = probe("app::main", &[("a", 7), ("b", -2)]);

    RecordWriter::in_build_directory(&build_directory)
        .write(&inputs, &written)
        .expect("the record is written");
    let record = RecordReader::in_build_directory(&build_directory)
        .read::<Probe>(&inputs)
        .expect("the record is read");

    assert_eq!(record.payload, written);
    assert_eq!(record.envelope.module(), "app::main");
    assert_eq!(record.envelope.payload_kind(), "probe");
    assert_eq!(record.envelope.payload_version(), 3);
}

#[test]
fn numbers_that_are_not_whole_read_back_exactly() {
    let build_directory = workspace("numbers").join(".brio");
    let inputs = inputs("sensor a\n");
    // Numbers that a JSON reader without exact rounding gets wrong.
    let written = Levels(vec![
        0.1 + 0.2,
        1.0e-7,
        5.0e-324,
        f64::MAX,
        -0.0,
        2.0,
        0.063_914_958_278_444_53,
        7.038_531_e-26,
        1.797_693_134_862_315_7e308,
        8.988_465_674_311_58e307,
        2.225_073_858_507_201_4e-308,
        9_007_199_254_740_993.0,
        0.300_000_000_000_000_04,
        123_456_789.123_456_78,
        1.0 / 3.0,
        std::f64::consts::PI * 1.0e100,
        std::f64::consts::E * 1.0e-100,
    ]);

    RecordWriter::in_build_directory(&build_directory)
        .write(&inputs, &written)
        .expect("the record is written");
    let record = RecordReader::in_build_directory(&build_directory)
        .read::<Levels>(&inputs)
        .expect("the record is read");

    assert_eq!(record.payload, written);
}

#[test]
fn writers_of_the_same_record_at_the_same_time_all_succeed() {
    let build_directory = workspace("same_time").join(".brio");
    let inputs = inputs("sensor a\n");

    let write_often = || {
        for _ in 0..50 {
            RecordWriter::in_build_directory(&build_directory)
                .write(&inputs, &probe("app::main", &[("a", 7)]))
                .expect("the record is written");
        }
    };
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(write_often);
        }
    });

    assert_eq!(files_under(&build_directory).len(), 1);
    RecordReader::in_build_directory(&build_directory)
        .read::<Probe>(&inputs)
        .expect("the record is read");
}

#[test]
fn identical_input_gives_byte_identical_records() {
    let write = |test: &str| {
        let build_directory = workspace(test).join(".brio");
        let path = RecordWriter::in_build_directory(&build_directory)
            .write(&inputs("sensor a\n"), &probe("app::main", &[("a", 7)]))
            .expect("the record is written");
        let relative = path
            .strip_prefix(&build_directory)
            .expect("the record is in the build directory")
            .to_owned();
        (relative, fs::read(path).expect("the record can be read"))
    };

    let (first_path, first_bytes) = write("identical_first");
    let (second_path, second_bytes) = write("identical_second");

    assert_eq!(first_path, second_path);
    assert!(!first_bytes.is_empty());
    assert_eq!(first_bytes, second_bytes);
}

#[test]
fn an_unrelated_change_elsewhere_does_not_change_an_elements_stable_identifier() {
    let before = probe("app::main", &[("b", 7)]);
    let after = probe("app::main", &[("a", 1), ("b", 7), ("c", 3)]);

    assert_eq!(before.readings[0].sensor, "b");
    assert_eq!(after.readings[1].sensor, "b");
    assert_eq!(before.readings[0].id, after.readings[1].id);
}

#[test]
fn a_stable_identifier_differs_by_module_by_stage_and_by_place() {
    let id = StableId::of::<Probe>("app::main", &["sensor", "a"]).expect("`probe` is a kind");

    let other_module = StableId::of::<Probe>("app::util", &["sensor", "a"]);
    let other_stage = StableId::of::<Gauge>("app::main", &["sensor", "a"]);
    let other_place = StableId::of::<Probe>("app::main", &["sensor", "b"]);
    let split_place = StableId::of::<Probe>("app::main", &["sensora"]);

    assert_ne!(Ok(&id), other_module.as_ref());
    assert_ne!(Ok(&id), other_stage.as_ref());
    assert_ne!(Ok(&id), other_place.as_ref());
    assert_ne!(Ok(&id), split_place.as_ref());
}

#[test]
fn records_are_written_under_the_build_directory_and_nowhere_else() {
    let workspace = workspace("build_directory");
    let build_directory = workspace.join(".brio");

    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&inputs("sensor a\n"), &probe("app::main", &[("a", 7)]))
        .expect("the record is written");

    assert!(path.starts_with(&build_directory));
    assert_eq!(files_under(&workspace), [path]);
}

#[test]
fn a_record_with_an_unknown_payload_type_reads_as_an_untyped_payload() {
    let build_directory = workspace("untyped").join(".brio");
    let inputs = inputs("sensor a\n");
    RecordWriter::in_build_directory(&build_directory)
        .write(&inputs, &Gauge("steady".to_owned()))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);

    let records = reader.read_all().expect("the records are read");

    assert_eq!(records.len(), 1);
    let record = records[0].as_ref().expect("the record is whole");
    assert_eq!(record.envelope.module(), "app::main");
    assert_eq!(record.envelope.payload_kind(), "gauge");
    assert_eq!(record.envelope.payload_version(), 1);
    assert_eq!(record.payload, json!("steady"));

    let as_probe = record.clone().typed::<Probe>();
    assert!(matches!(as_probe, Err(ReadError::WrongPayload { .. })));
    let as_gauge = record.clone().typed::<Gauge>();
    assert_eq!(
        as_gauge.expect("the record holds a gauge").payload,
        Gauge("steady".to_owned())
    );
}

#[test]
fn a_corrupted_record_is_rejected_not_trusted() {
    let build_directory = workspace("corrupted").join(".brio");
    let inputs = inputs("sensor a\n");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&inputs, &probe("app::main", &[("a", 7)]))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);

    tamper(&path, r#""value": 7"#, r#""value": 8"#);
    let changed = reader.read::<Probe>(&inputs);
    assert!(matches!(
        changed,
        Err(ReadError::Defective {
            defect: RecordDefect::Corrupted,
            ..
        })
    ));
    let all = reader.read_all().expect("the records are read");
    assert!(matches!(
        all[..],
        [Err(ReadError::Defective {
            defect: RecordDefect::Corrupted,
            ..
        })]
    ));

    tamper(&path, r#""value": 8"#, r#""value": 7"#);
    reader
        .read::<Probe>(&inputs)
        .expect("the record is whole again");
    tamper(&path, r#""value": 7"#, r#""value":  7"#);
    let respaced = reader.read::<Probe>(&inputs);
    assert!(matches!(
        respaced,
        Err(ReadError::Defective {
            defect: RecordDefect::Corrupted,
            ..
        })
    ));

    fs::write(&path, "{ \"envelope_version\": 1,").expect("the record can be cut short");
    let cut_short = reader.read::<Probe>(&inputs);
    assert!(matches!(
        cut_short,
        Err(ReadError::Defective {
            defect: RecordDefect::Corrupted,
            ..
        })
    ));
}

#[test]
fn an_unsupported_envelope_version_is_rejected_naming_the_found_and_supported_versions() {
    let build_directory = workspace("envelope_version").join(".brio");
    let inputs = inputs("sensor a\n");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&inputs, &probe("app::main", &[("a", 7)]))
        .expect("the record is written");

    assert_eq!(Envelope::VERSION, 1);
    tamper(
        &path,
        r#""envelope_version": 1"#,
        r#""envelope_version": 2"#,
    );
    let error = RecordReader::in_build_directory(&build_directory)
        .read::<Probe>(&inputs)
        .expect_err("envelope version 2 is not supported");

    assert!(matches!(
        error,
        ReadError::Defective {
            defect: RecordDefect::UnsupportedEnvelopeVersion {
                found: 2,
                supported: 1,
            },
            ..
        }
    ));
    assert_eq!(
        error.to_string(),
        format!(
            "the record `{}` has envelope version 2, but this toolchain reads envelope version 1",
            path.display()
        )
    );
}

#[test]
fn a_change_to_any_input_gives_another_record_key() {
    let build_directory = workspace("key").join(".brio");
    let original = inputs("sensor a\n");
    RecordWriter::in_build_directory(&build_directory)
        .write(&original, &probe("app::main", &[("a", 7)]))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);
    let key = reader
        .read::<Probe>(&original)
        .expect("the record is read")
        .envelope
        .key()
        .clone();
    assert_eq!(key.to_string().len(), 64);

    let mut other_module = original.clone();
    other_module.module = "app::util".to_owned();
    let mut other_toolchain = original.clone();
    other_toolchain.toolchain_version = "0.2.0".to_owned();
    let mut other_target = original.clone();
    other_target.target = "x86_64-unknown-linux-gnu".to_owned();
    let mut other_settings = original.clone();
    other_settings
        .build_settings
        .insert("profile".to_owned(), "release".to_owned());
    let mut other_interface = original.clone();
    other_interface
        .module_inputs
        .insert("interface app::util".to_owned(), b"fn help()".to_vec());

    for changed in [
        inputs("sensor a\nsensor b\n"),
        other_module,
        other_toolchain,
        other_target,
        other_settings,
        other_interface,
    ] {
        let read = reader.read::<Probe>(&changed);
        assert!(
            matches!(read, Err(ReadError::Missing { .. })),
            "{changed:?} still finds the record"
        );
    }
    // The stage is an input too: the same inputs give another stage another record.
    assert!(matches!(
        reader.read::<Gauge>(&original),
        Err(ReadError::Missing { .. })
    ));
}

/// The record `text` with the digest the published schema describes: the
/// BLAKE3 of the record without its digest field. Worked out here, without
/// Lens, as any tool outside the toolchain would.
fn with_its_own_digest(text: &str) -> String {
    let (digest_line, rest) = text
        .strip_prefix("{\n")
        .and_then(|fields| fields.split_once('\n'))
        .expect("the digest is the first field of a record");
    assert!(digest_line.starts_with(r#"  "digest": ""#), "{digest_line}");
    let without_digest = format!("{{\n{rest}");
    let digest = blake3::hash(without_digest.as_bytes()).to_hex();
    format!("{{\n  \"digest\": \"{digest}\",\n{rest}")
}

#[test]
fn the_digest_of_a_record_is_the_one_the_published_schema_describes() {
    let build_directory = workspace("digest").join(".brio");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&inputs("sensor a\n"), &probe("app::main", &[("a", 7)]))
        .expect("the record is written");

    let text = fs::read_to_string(path).expect("the record can be read");

    assert_eq!(with_its_own_digest(&text), text);
}

#[test]
fn a_record_the_writer_would_refuse_is_rejected_even_with_a_matching_digest() {
    let build_directory = workspace("forged").join(".brio");
    let inputs = inputs("sensor a\n");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&inputs, &probe("app::main", &[("a", 7)]))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);
    let text = fs::read_to_string(&path).expect("the record can be read");

    // A changed value with a digest to match reads as a whole record: the
    // digest guards against accidents, and only the record key ties a
    // record to its inputs.
    let revalued = with_its_own_digest(&text.replacen(r#""value": 7"#, r#""value": 8"#, 1));
    fs::write(&path, revalued).expect("the record can be changed");
    let record = reader.read::<Probe>(&inputs).expect("the digest matches");
    assert_eq!(record.payload.readings[0].value, 8);

    let renamed = with_its_own_digest(&text.replacen(
        r#""module": "app::main""#,
        r#""module": "/home/ada/app/main.brio""#,
        1,
    ));
    assert_ne!(renamed, text);
    fs::write(&path, renamed).expect("the record can be changed");
    let read = reader.read::<Probe>(&inputs);
    assert!(matches!(
        read,
        Err(ReadError::Defective {
            defect: RecordDefect::Corrupted,
            ..
        })
    ));
}

#[test]
fn a_stale_record_is_rejected_not_trusted() {
    let build_directory = workspace("stale").join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let old_inputs = inputs("sensor a\n");
    let new_inputs = inputs("sensor a\nsensor b\n");
    let old_path = writer
        .write(&old_inputs, &probe("app::main", &[("a", 7)]))
        .expect("the old record is written");
    let new_path = writer
        .write(&new_inputs, &probe("app::main", &[("a", 7), ("b", 1)]))
        .expect("the new record is written");

    // The record of the old inputs now sits where the new inputs' record belongs.
    fs::copy(&old_path, &new_path).expect("the old record can be copied");
    let read = RecordReader::in_build_directory(&build_directory).read::<Probe>(&new_inputs);

    assert!(matches!(
        read,
        Err(ReadError::Defective {
            defect: RecordDefect::Stale,
            ..
        })
    ));
}

#[test]
fn a_record_that_cannot_be_read_hides_no_other_record() {
    let build_directory = workspace("one_bad_record").join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let inputs = inputs("sensor a\n");
    writer
        .write(&inputs, &Gauge("steady".to_owned()))
        .expect("the gauge record is written");
    let probe_path = writer
        .write(&inputs, &probe("app::main", &[("a", 7)]))
        .expect("the probe record is written");
    tamper(&probe_path, r#""value": 7"#, r#""value": 8"#);
    fs::write(build_directory.join("lens").join("notes.json"), "{}\n")
        .expect("a foreign file can be added");

    let records = RecordReader::in_build_directory(&build_directory)
        .read_all()
        .expect("the records are read");

    let whole: Vec<&str> = records
        .iter()
        .flatten()
        .map(|record| record.envelope.payload_kind())
        .collect();
    assert_eq!(whole, ["gauge"]);
    let corrupted = records
        .iter()
        .filter(|record| {
            matches!(
                record,
                Err(ReadError::Defective {
                    defect: RecordDefect::Corrupted,
                    ..
                })
            )
        })
        .count();
    assert_eq!(corrupted, 2);
}

/// A payload of arrays nested inside each other, and nothing else.
struct Nest(usize);

impl Payload for Nest {
    const KIND: &'static str = "nest";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "array" })
    }

    fn to_json(&self) -> Value {
        (0..self.0).fold(Value::Null, |inner, _| json!([inner]))
    }

    fn from_json(payload: &Value) -> Option<Self> {
        let mut depth = 0;
        let mut inner = payload;
        while let Some(items) = inner.as_array() {
            depth += 1;
            inner = items.first()?;
        }
        Some(Self(depth))
    }
}

#[test]
fn a_payload_is_read_back_at_the_deepest_nesting_and_refused_beyond_it() {
    let workspace = workspace("nesting");
    let build_directory = workspace.join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let inputs = inputs("sensor a\n");
    assert_eq!(Envelope::MAX_PAYLOAD_DEPTH, 126);

    let path = writer
        .write(&inputs, &Nest(126))
        .expect("the deepest payload is written");
    let record = RecordReader::in_build_directory(&build_directory)
        .read::<Nest>(&inputs)
        .expect("the deepest payload is read");
    assert_eq!(record.payload.0, 126);

    let error = writer
        .write(&inputs, &Nest(127))
        .expect_err("no reader could read this payload back");
    assert!(matches!(
        error,
        WriteError::Refused(RecordRefusal::PayloadTooDeep {
            payload_kind: "nest",
            depth: 127
        })
    ));
    assert_eq!(
        error.to_string(),
        "the `nest` payload nests 127 deep, but a record holds at most 126 levels"
    );
    assert_eq!(files_under(&workspace), [path]);
}

#[test]
fn a_module_named_by_an_absolute_path_is_not_written() {
    let workspace = workspace("absolute_path");
    let build_directory = workspace.join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);

    for module in [
        "/home/ada/app/main.brio",
        "~/app/main.brio",
        r"C:\Users\ada\app\main.brio",
        "C:/Users/ada/app/main.brio",
        r"\\server\share\main.brio",
    ] {
        let mut inputs = inputs("sensor a\n");
        inputs.module = module.to_owned();
        let error = writer
            .write(&inputs, &probe(module, &[("a", 7)]))
            .expect_err("an absolute path is refused");
        assert!(
            matches!(&error, WriteError::Refused(RecordRefusal::AbsolutePath { module: refused }) if refused == module),
            "{module} gives {error}"
        );
    }
    assert_eq!(files_under(&workspace), Vec::<PathBuf>::new());

    let mut relative = inputs("sensor a\n");
    relative.module = "app/main.brio".to_owned();
    writer
        .write(&relative, &probe("app/main.brio", &[("a", 7)]))
        .expect("a path inside the workspace is a name");
}

#[cfg(unix)]
#[test]
fn a_write_that_fails_leaves_nothing_behind() {
    let workspace = workspace("failed_write");
    let build_directory = workspace.join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let inputs = inputs("sensor a\n");
    let path = writer
        .write(&inputs, &probe("app::main", &[("a", 7)]))
        .expect("the record is written");
    // A directory where the record belongs: nothing can be renamed onto it.
    fs::remove_file(&path).expect("the record can be removed");
    fs::create_dir(&path).expect("a directory can take its place");

    let error = writer
        .write(&inputs, &probe("app::main", &[("a", 7)]))
        .expect_err("the record cannot be written");

    assert!(matches!(error, WriteError::Io { .. }));
    assert_eq!(files_under(&workspace), Vec::<PathBuf>::new());
}

/// A payload whose kind breaks the rule on payload kinds.
struct Dotted;

impl Payload for Dotted {
    const KIND: &'static str = "probe.3+gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "null" })
    }

    fn to_json(&self) -> Value {
        Value::Null
    }

    fn from_json(_payload: &Value) -> Option<Self> {
        Some(Self)
    }
}

#[test]
fn a_payload_with_an_invalid_payload_kind_is_not_written() {
    let workspace = workspace("invalid_kind");
    let build_directory = workspace.join(".brio");

    let error = RecordWriter::in_build_directory(&build_directory)
        .write(&inputs("sensor a\n"), &Dotted)
        .expect_err("the kind is refused");

    assert!(matches!(
        error,
        WriteError::Refused(RecordRefusal::InvalidPayloadKind(InvalidPayloadKind {
            payload_kind: "probe.3+gauge"
        }))
    ));
    assert_eq!(files_under(&workspace), Vec::<PathBuf>::new());
    assert_eq!(
        StableId::of::<Dotted>("app::main", &["a"]),
        Err(InvalidPayloadKind {
            payload_kind: "probe.3+gauge"
        })
    );
}
