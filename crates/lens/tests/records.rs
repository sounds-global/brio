// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Writing and reading stage records, through the public interface of
//! `brio_lens`.

mod support;

use std::fs;
use std::path::PathBuf;

use brio_lens::json::{Value, json};
use brio_lens::{
    Element, ElementTable, Envelope, InvalidPayloadKind, Payload, PayloadType, ReadError,
    RecordDefect, RecordReader, RecordRefusal, RecordWriter, Stage, WriteError,
};

use support::{
    Gauge, Loose, Probe, files_under, inputs, probe, resealed, stage, tamper, with_its_own_digest,
    workspace,
};

/// A payload of numbers that are not whole.
#[derive(Debug, PartialEq)]
struct Levels(Vec<f64>);

impl PayloadType for Levels {
    const KIND: &'static str = "levels";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "array", "items": { "type": "number" } })
    }
}

impl Payload for Levels {
    fn to_elements(&self, stage: &Stage<Self>) -> Vec<Element> {
        vec![Element::new(
            stage.id(&["levels"]),
            Value::from(self.0.clone()),
        )]
    }

    fn from_elements(elements: &ElementTable) -> Option<Self> {
        let [root] = elements.roots() else {
            return None;
        };
        elements
            .get(root)?
            .payload()
            .as_array()?
            .iter()
            .map(Value::as_f64)
            .collect::<Option<Vec<f64>>>()
            .map(Self)
    }
}

#[test]
fn writing_then_reading_a_record_gives_back_the_same_envelope_and_payload() {
    let build_directory = workspace("round_trip").join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\nsensor b\n");
    let written = probe(&[("a", 7), ("b", -2)]);

    RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &inputs, &written)
        .expect("the record is written");
    let record = RecordReader::in_build_directory(&build_directory)
        .read(&stage, &inputs)
        .expect("the record is read");

    assert_eq!(record.payload, written);
    assert_eq!(record.envelope.module(), "app::main");
    assert_eq!(record.envelope.payload_kind(), "probe");
    assert_eq!(record.envelope.payload_version(), 3);
}

#[test]
fn numbers_that_are_not_whole_read_back_exactly() {
    let build_directory = workspace("numbers").join(".brio");
    let stage = stage::<Levels>("app::main");
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
        .write(&stage, &inputs, &written)
        .expect("the record is written");
    let record = RecordReader::in_build_directory(&build_directory)
        .read(&stage, &inputs)
        .expect("the record is read");

    assert_eq!(record.payload, written);
}

#[test]
fn writers_of_the_same_record_at_the_same_time_all_succeed() {
    let build_directory = workspace("same_time").join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");

    let write_often = || {
        for _ in 0..50 {
            RecordWriter::in_build_directory(&build_directory)
                .write(&stage, &inputs, &probe(&[("a", 7)]))
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
        .read(&stage, &inputs)
        .expect("the record is read");
}

#[test]
fn identical_input_gives_byte_identical_records() {
    let write = |test: &str| {
        let build_directory = workspace(test).join(".brio");
        let path = RecordWriter::in_build_directory(&build_directory)
            .write(
                &stage::<Probe>("app::main"),
                &inputs("sensor a\n"),
                &probe(&[("a", 7)]),
            )
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
    let stage = stage::<Probe>("app::main");
    let before = probe(&[("b", 7)]).to_elements(&stage);
    let after = probe(&[("a", 1), ("b", 7), ("c", 3)]).to_elements(&stage);

    assert_eq!(before[0].payload()["sensor"], "b");
    assert_eq!(after[1].payload()["sensor"], "b");
    assert_eq!(before[0].id(), after[1].id());
}

#[test]
fn a_stable_identifier_differs_by_module_by_stage_and_by_place() {
    let id = stage::<Probe>("app::main").id(&["sensor", "a"]);

    assert_eq!(id.to_string().len(), 32);
    assert_eq!(id, stage::<Probe>("app::main").id(&["sensor", "a"]));
    assert_ne!(id, stage::<Probe>("app::util").id(&["sensor", "a"]));
    assert_ne!(id, stage::<Gauge>("app::main").id(&["sensor", "a"]));
    assert_ne!(id, stage::<Probe>("app::main").id(&["sensor", "b"]));
    assert_ne!(id, stage::<Probe>("app::main").id(&["sensora"]));
}

#[test]
fn records_are_written_under_the_build_directory_and_nowhere_else() {
    let workspace = workspace("build_directory");
    let build_directory = workspace.join(".brio");

    let path = RecordWriter::in_build_directory(&build_directory)
        .write(
            &stage::<Probe>("app::main"),
            &inputs("sensor a\n"),
            &probe(&[("a", 7)]),
        )
        .expect("the record is written");

    assert!(path.starts_with(&build_directory));
    assert_eq!(files_under(&workspace), [path]);
}

#[test]
fn a_record_with_an_unknown_payload_type_reads_as_an_untyped_payload() {
    let build_directory = workspace("untyped").join(".brio");
    let stage = stage::<Gauge>("app::main");
    RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &inputs("sensor a\n"), &Gauge("steady".to_owned()))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);

    let records = reader.read_all().expect("the records are read");

    assert_eq!(records.len(), 1);
    let record = records[0].as_ref().expect("the record is whole");
    assert_eq!(record.envelope.module(), "app::main");
    assert_eq!(record.envelope.payload_kind(), "gauge");
    assert_eq!(record.envelope.payload_version(), 1);
    // The reader knows no payload type, and still finds the element by its
    // stable identifier.
    let element = record
        .envelope
        .elements()
        .get(&stage.id(&["gauge"]))
        .expect("the element is in the record");
    assert_eq!(element.payload(), &json!("steady"));

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
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &inputs, &probe(&[("a", 7)]))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);

    tamper(&path, r#""value": 7"#, r#""value": 8"#);
    let changed = reader.read(&stage, &inputs);
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
        .read(&stage, &inputs)
        .expect("the record is whole again");
    tamper(&path, r#""value": 7"#, r#""value":  7"#);
    let respaced = reader.read(&stage, &inputs);
    assert!(matches!(
        respaced,
        Err(ReadError::Defective {
            defect: RecordDefect::Corrupted,
            ..
        })
    ));

    fs::write(&path, "{ \"envelope_version\": 1,").expect("the record can be cut short");
    let cut_short = reader.read(&stage, &inputs);
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
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &inputs, &probe(&[("a", 7)]))
        .expect("the record is written");

    assert_eq!(Envelope::VERSION, 1);
    tamper(
        &path,
        r#""envelope_version": 1"#,
        r#""envelope_version": 2"#,
    );
    let error = RecordReader::in_build_directory(&build_directory)
        .read(&stage, &inputs)
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
    let stage = stage::<Probe>("app::main");
    let original = inputs("sensor a\n");
    RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &original, &probe(&[("a", 7)]))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);
    let key = reader
        .read(&stage, &original)
        .expect("the record is read")
        .envelope
        .key()
        .clone();
    assert_eq!(key.to_string().len(), 64);

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
        other_toolchain,
        other_target,
        other_settings,
        other_interface,
    ] {
        let read = reader.read(&stage, &changed);
        assert!(
            matches!(read, Err(ReadError::Missing { .. })),
            "{changed:?} still finds the record"
        );
    }
    // The module and the stage are inputs too: the same inputs give another
    // module, or another stage, another record.
    assert!(matches!(
        reader.read(&support::stage::<Probe>("app::util"), &original),
        Err(ReadError::Missing { .. })
    ));
    assert!(matches!(
        reader.read(&support::stage::<Gauge>("app::main"), &original),
        Err(ReadError::Missing { .. })
    ));
}

#[test]
fn the_digest_of_a_record_is_the_one_the_published_schema_describes() {
    let build_directory = workspace("digest").join(".brio");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(
            &stage::<Probe>("app::main"),
            &inputs("sensor a\n"),
            &probe(&[("a", 7)]),
        )
        .expect("the record is written");

    let text = fs::read_to_string(path).expect("the record can be read");

    assert_eq!(with_its_own_digest(&text), text);
}

#[test]
fn a_record_the_writer_would_refuse_is_rejected_even_with_a_matching_digest() {
    let build_directory = workspace("forged").join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &inputs, &probe(&[("a", 7)]))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);
    let text = fs::read_to_string(&path).expect("the record can be read");

    // A changed value with a digest to match reads as a whole record: the
    // digest guards against accidents, and only the record key ties a
    // record to its inputs.
    let revalued = with_its_own_digest(&text.replacen(r#""value": 7"#, r#""value": 8"#, 1));
    fs::write(&path, revalued).expect("the record can be changed");
    let record = reader.read(&stage, &inputs).expect("the digest matches");
    assert_eq!(record.payload, probe(&[("a", 8)]));

    for (from, to) in [
        (
            r#""module": "app::main""#,
            r#""module": "/home/ada/app/main.brio""#,
        ),
        (
            r#""payload_kind": "probe""#,
            r#""payload_kind": "probe.3+gauge""#,
        ),
    ] {
        let forged = with_its_own_digest(&text.replacen(from, to, 1));
        assert_ne!(forged, text);
        fs::write(&path, forged).expect("the record can be changed");
        let read = reader.read(&stage, &inputs);
        assert!(
            matches!(
                read,
                Err(ReadError::Defective {
                    defect: RecordDefect::Corrupted,
                    ..
                })
            ),
            "{to} is read"
        );
    }
}

#[test]
fn a_stale_record_is_rejected_not_trusted() {
    let build_directory = workspace("stale").join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let stage = stage::<Probe>("app::main");
    let old_inputs = inputs("sensor a\n");
    let new_inputs = inputs("sensor a\nsensor b\n");
    let old_path = writer
        .write(&stage, &old_inputs, &probe(&[("a", 7)]))
        .expect("the old record is written");
    let new_path = writer
        .write(&stage, &new_inputs, &probe(&[("a", 7), ("b", 1)]))
        .expect("the new record is written");

    // The record of the old inputs now sits where the new inputs' record belongs.
    fs::copy(&old_path, &new_path).expect("the old record can be copied");
    let read = RecordReader::in_build_directory(&build_directory).read(&stage, &new_inputs);

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
        .write(
            &stage::<Gauge>("app::main"),
            &inputs,
            &Gauge("steady".to_owned()),
        )
        .expect("the gauge record is written");
    let probe_path = writer
        .write(&stage::<Probe>("app::main"), &inputs, &probe(&[("a", 7)]))
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

/// A payload of one element, whose payload is arrays nested inside each
/// other and nothing else.
struct Nest(usize);

impl PayloadType for Nest {
    const KIND: &'static str = "nest";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "array" })
    }
}

impl Payload for Nest {
    fn to_elements(&self, stage: &Stage<Self>) -> Vec<Element> {
        let nested = (0..self.0).fold(Value::Null, |inner, _| json!([inner]));
        vec![Element::new(stage.id(&["nest"]), nested)]
    }

    fn from_elements(elements: &ElementTable) -> Option<Self> {
        let [root] = elements.roots() else {
            return None;
        };
        let mut depth = 0;
        let mut inner = elements.get(root)?.payload();
        while let Some(items) = inner.as_array() {
            depth += 1;
            inner = items.first()?;
        }
        Some(Self(depth))
    }
}

#[test]
fn an_elements_payload_is_read_back_at_the_deepest_nesting_and_refused_beyond_it() {
    let workspace = workspace("nesting");
    let build_directory = workspace.join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let stage = stage::<Nest>("app::main");
    let inputs = inputs("sensor a\n");
    assert_eq!(Envelope::MAX_PAYLOAD_DEPTH, 124);

    let path = writer
        .write(&stage, &inputs, &Nest(124))
        .expect("the deepest payload is written");
    let record = RecordReader::in_build_directory(&build_directory)
        .read(&stage, &inputs)
        .expect("the deepest payload is read");
    assert_eq!(record.payload.0, 124);

    let error = writer
        .write(&stage, &inputs, &Nest(125))
        .expect_err("no reader could read this payload back");
    let nest = stage.id(&["nest"]);
    assert!(matches!(
        &error,
        WriteError::Refused(RecordRefusal::PayloadTooDeep { element, depth: 125 }) if *element == nest
    ));
    assert_eq!(
        error.to_string(),
        format!(
            "the payload of the element `{nest}` nests 125 deep, but an element's payload nests \
             at most 124 levels"
        )
    );
    assert_eq!(files_under(&workspace), [path]);
}

#[test]
fn a_module_named_by_an_absolute_path_does_not_enter_lens() {
    for module in [
        "/home/ada/app/main.brio",
        "~/app/main.brio",
        r"C:\Users\ada\app\main.brio",
        "C:/Users/ada/app/main.brio",
        r"\\server\share\main.brio",
    ] {
        let error = Stage::<Probe>::of(module).expect_err("an absolute path is refused");
        assert!(
            matches!(&error, RecordRefusal::AbsolutePath { module: refused } if refused == module),
            "{module} gives {error}"
        );
    }

    let relative = Stage::<Probe>::of("app/main.brio").expect("a path in the workspace is a name");
    assert_eq!(relative.module(), "app/main.brio");
}

#[cfg(unix)]
#[test]
fn a_write_that_fails_leaves_nothing_behind() {
    let workspace = workspace("failed_write");
    let build_directory = workspace.join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = writer
        .write(&stage, &inputs, &probe(&[("a", 7)]))
        .expect("the record is written");
    // A directory where the record belongs: nothing can be renamed onto it.
    fs::remove_file(&path).expect("the record can be removed");
    fs::create_dir(&path).expect("a directory can take its place");

    let error = writer
        .write(&stage, &inputs, &probe(&[("a", 7)]))
        .expect_err("the record cannot be written");

    assert!(matches!(error, WriteError::Io { .. }));
    assert_eq!(files_under(&workspace), Vec::<PathBuf>::new());
}

/// A payload type whose kind breaks the rule on payload kinds.
struct Dotted;

impl PayloadType for Dotted {
    const KIND: &'static str = "probe.3+gauge";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({ "type": "null" })
    }
}

#[test]
fn a_payload_type_with_an_invalid_payload_kind_does_not_enter_lens() {
    let error = Stage::<Dotted>::of("app::main").expect_err("the kind is refused");

    assert!(matches!(
        error,
        RecordRefusal::InvalidPayloadKind(InvalidPayloadKind {
            payload_kind: "probe.3+gauge"
        })
    ));
}

/// The elements of a small outline: a document with two sections, the
/// second of which has one of its own.
fn outline(stage: &Stage<Loose>) -> Vec<Element> {
    let document = stage.id(&["document"]);
    let usage = stage.id(&["document", "usage"]);
    vec![
        Element::new(document.clone(), json!("Document")),
        Element::new(stage.id(&["document", "install"]), json!("Install")).child_of(&document),
        Element::new(usage.clone(), json!("Usage")).child_of(&document),
        Element::new(stage.id(&["document", "usage", "flags"]), json!("Flags")).child_of(&usage),
        Element::new(stage.id(&["appendix"]), json!("Appendix")),
    ]
}

#[test]
fn an_element_names_its_parent_and_lists_its_children_in_order() {
    let build_directory = workspace("tree").join(".brio");
    let stage = stage::<Loose>("app::main");
    RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &inputs("sensor a\n"), &Loose(outline))
        .expect("the record is written");

    let records = RecordReader::in_build_directory(&build_directory)
        .read_all()
        .expect("the records are read");
    let elements = records[0]
        .as_ref()
        .expect("the record is whole")
        .envelope
        .elements();

    let document = stage.id(&["document"]);
    let install = stage.id(&["document", "install"]);
    let usage = stage.id(&["document", "usage"]);
    let flags = stage.id(&["document", "usage", "flags"]);
    let appendix = stage.id(&["appendix"]);
    assert_eq!(elements.len(), 5);
    assert_eq!(elements.roots(), [document.clone(), appendix.clone()]);
    let element = |id| elements.get(id).expect("the element is in the record");
    assert_eq!(element(&document).parent(), None);
    assert_eq!(
        element(&document).children(),
        [install.clone(), usage.clone()]
    );
    assert_eq!(element(&install).parent(), Some(&document));
    assert_eq!(element(&install).children(), []);
    assert_eq!(element(&usage).children(), std::slice::from_ref(&flags));
    assert_eq!(element(&flags).parent(), Some(&usage));
    assert_eq!(element(&flags).payload(), &json!("Flags"));
    assert_eq!(element(&appendix).children(), []);
}

/// What the writer says when asked to write `elements`.
fn refusal(test: &str, elements: fn(&Stage<Loose>) -> Vec<Element>) -> RecordRefusal {
    let workspace = workspace(test);
    let error = RecordWriter::in_build_directory(workspace.join(".brio"))
        .write(
            &stage::<Loose>("app::main"),
            &inputs("sensor a\n"),
            &Loose(elements),
        )
        .expect_err("the record is refused");
    assert_eq!(files_under(&workspace), Vec::<PathBuf>::new());
    match error {
        WriteError::Refused(refusal) => refusal,
        other @ WriteError::Io { .. } => panic!("the write failed: {other}"),
        other => panic!("the write failed in an unexpected way: {other}"),
    }
}

#[test]
fn two_elements_with_one_stable_identifier_are_not_written() {
    let stage = stage::<Loose>("app::main");
    let twice = stage.id(&["a"]);

    let refused = refusal("duplicate_element", |stage| {
        vec![
            Element::new(stage.id(&["a"]), json!(1)),
            Element::new(stage.id(&["a"]), json!(2)),
        ]
    });

    assert_eq!(
        refused,
        RecordRefusal::DuplicateElement {
            element: twice.clone()
        }
    );
    assert_eq!(
        refused.to_string(),
        format!("two elements have the stable identifier `{twice}`")
    );
}

#[test]
fn an_element_whose_parent_is_not_in_the_record_is_not_written() {
    let stage = stage::<Loose>("app::main");
    let (child, parent) = (stage.id(&["a", "b"]), stage.id(&["a"]));

    let refused = refusal("missing_parent", |stage| {
        vec![Element::new(stage.id(&["a", "b"]), json!(1)).child_of(&stage.id(&["a"]))]
    });

    assert_eq!(
        refused,
        RecordRefusal::MissingParent {
            element: child.clone(),
            parent: parent.clone()
        }
    );
    assert_eq!(
        refused.to_string(),
        format!("the element `{child}` names the parent `{parent}`, which is not in the record")
    );
}

#[test]
fn elements_that_are_their_own_ancestors_are_not_written() {
    let refused = refusal("parent_cycle", |stage| {
        let (a, b) = (stage.id(&["a"]), stage.id(&["b"]));
        vec![
            Element::new(stage.id(&["root"]), json!(0)),
            Element::new(a.clone(), json!(1)).child_of(&b),
            Element::new(b, json!(2)).child_of(&a),
        ]
    });

    let stage = stage::<Loose>("app::main");
    let (a, b) = (stage.id(&["a"]), stage.id(&["b"]));
    let RecordRefusal::ParentCycle { element } = &refused else {
        panic!("the refusal is {refused}");
    };
    assert!(*element == a || *element == b);
    assert_eq!(
        refused.to_string(),
        format!("the element `{element}` is its own ancestor")
    );
}

#[test]
fn a_record_whose_element_table_breaks_its_rules_is_rejected_even_with_a_matching_digest() {
    let build_directory = workspace("forged_table").join(".brio");
    let stage = stage::<Loose>("app::main");
    let inputs = inputs("sensor a\n");
    let path = RecordWriter::in_build_directory(&build_directory)
        .write(&stage, &inputs, &Loose(outline))
        .expect("the record is written");
    let reader = RecordReader::in_build_directory(&build_directory);
    let text = fs::read_to_string(&path).expect("the record can be read");
    let whole: Value = brio_lens::json::from_str(&text).expect("the record is JSON");
    assert_eq!(resealed(whole.clone()), text);

    let document = stage.id(&["document"]).to_string();
    let usage = stage.id(&["document", "usage"]).to_string();
    let flags = stage.id(&["document", "usage", "flags"]).to_string();
    let appendix = stage.id(&["appendix"]).to_string();

    // Another order of the same children is another record, and a whole one.
    let mut reordered = whole.clone();
    reordered["roots"] = json!([appendix, document]);
    fs::write(&path, resealed(reordered)).expect("the record can be changed");
    let record = reader.read(&stage, &inputs).expect("the table holds");
    assert_eq!(record.envelope.elements().roots()[0].to_string(), appendix);

    type Forge = fn(&mut Value, &[String]);
    let forgeries: [(&str, Forge); 5] = [
        ("a parent that is not in the record", |record, ids| {
            record["elements"][&ids[2]]["parent"] = json!("0123456789abcdef0123456789abcdef");
        }),
        ("a child its parent does not list", |record, ids| {
            record["elements"][&ids[1]]["children"] = json!([]);
        }),
        ("a child listed under another parent", |record, ids| {
            record["elements"][&ids[1]]["children"] = json!([]);
            record["elements"][&ids[3]]["children"] = json!([ids[2]]);
        }),
        ("a child listed twice", |record, ids| {
            record["elements"][&ids[1]]["children"] = json!([ids[2], ids[2]]);
        }),
        ("a root that has a parent", |record, ids| {
            record["roots"] = json!([ids[0], ids[3], ids[2]]);
        }),
    ];
    let ids = [document, usage, flags, appendix];
    for (what, forge) in forgeries {
        let mut forged = whole.clone();
        forge(&mut forged, &ids);
        fs::write(&path, resealed(forged)).expect("the record can be changed");
        let read = reader.read(&stage, &inputs);
        assert!(
            matches!(
                read,
                Err(ReadError::Defective {
                    defect: RecordDefect::Corrupted,
                    ..
                })
            ),
            "a record with {what} is read"
        );
    }
}
