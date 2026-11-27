// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Writing and reading stage records, through the public interface of
//! `brio_lens`.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use brio_lens::json::{Value, json};
use brio_lens::{
    Element, ElementTable, Envelope, InvalidPayloadKind, Payload, PayloadType, ReadError,
    RecordDefect, RecordInputs, RecordReader, RecordRefusal, RecordWriter, Stage, WriteError,
};

use support::{
    Forge, Gauge, Loose, Probe, assert_forgeries_are_corrupted, envelopes, files_under, inputs,
    nested, probe, refusal, resealed, stage, tamper, with_its_own_digest, workspace, write_record,
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

    write_record(&build_directory, &stage, &inputs, &written);
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

    write_record(&build_directory, &stage, &inputs, &written);
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
            write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
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
        let path = write_record(
            &build_directory,
            &stage::<Probe>("app::main"),
            &inputs("sensor a\n"),
            &probe(&[("a", 7)]),
        );
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

    let path = write_record(
        &build_directory,
        &stage::<Probe>("app::main"),
        &inputs("sensor a\n"),
        &probe(&[("a", 7)]),
    );

    assert!(path.starts_with(&build_directory));
    assert_eq!(files_under(&workspace), [path]);
}

#[test]
fn a_record_with_an_unknown_payload_type_reads_as_an_untyped_payload() {
    let build_directory = workspace("untyped").join(".brio");
    let stage = stage::<Gauge>("app::main");
    write_record(
        &build_directory,
        &stage,
        &inputs("sensor a\n"),
        &Gauge("steady".to_owned()),
    );
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
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
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
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));

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
    write_record(&build_directory, &stage, &original, &probe(&[("a", 7)]));
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
    let path = write_record(
        &build_directory,
        &stage::<Probe>("app::main"),
        &inputs("sensor a\n"),
        &probe(&[("a", 7)]),
    );

    let text = fs::read_to_string(path).expect("the record can be read");

    assert_eq!(with_its_own_digest(&text), text);
}

#[test]
fn a_record_the_writer_would_refuse_is_rejected_even_with_a_matching_digest() {
    let build_directory = workspace("forged").join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
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
    let stage = stage::<Probe>("app::main");
    let old_inputs = inputs("sensor a\n");
    let new_inputs = inputs("sensor a\nsensor b\n");
    let old_path = write_record(&build_directory, &stage, &old_inputs, &probe(&[("a", 7)]));
    let new_path = write_record(
        &build_directory,
        &stage,
        &new_inputs,
        &probe(&[("a", 7), ("b", 1)]),
    );

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
            || &inputs,
            &Gauge("steady".to_owned()),
        )
        .expect("the gauge record is written");
    let probe_path = write_record(
        &build_directory,
        &stage::<Probe>("app::main"),
        &inputs,
        &probe(&[("a", 7)]),
    );
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
    let count = |is_counted: fn(&ReadError) -> bool| {
        let errors = records.iter().filter_map(|record| record.as_ref().err());
        errors.filter(|error| is_counted(error)).count()
    };
    let corrupted = count(|error| {
        matches!(
            error,
            ReadError::Defective {
                defect: RecordDefect::Corrupted,
                ..
            }
        )
    });
    assert_eq!(corrupted, 1);
    assert_eq!(
        count(|error| matches!(error, ReadError::NotARecord { .. })),
        1
    );
    assert_eq!(records.len(), 3);
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
        vec![Element::new(stage.id(&["nest"]), nested(self.0))]
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

    let path = write_record(&build_directory, &stage, &inputs, &Nest(124));
    let record = RecordReader::in_build_directory(&build_directory)
        .read(&stage, &inputs)
        .expect("the deepest payload is read");
    assert_eq!(record.payload.0, 124);

    let error = writer
        .write(&stage, || &inputs, &Nest(125))
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
            matches!(&error, RecordRefusal::AbsoluteModulePath { module: refused } if refused == module),
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
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
    // A directory where the record belongs: nothing can be renamed onto it.
    fs::remove_file(&path).expect("the record can be removed");
    fs::create_dir(&path).expect("a directory can take its place");

    let error = writer
        .write(&stage, || &inputs, &probe(&[("a", 7)]))
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
    let build_directory = workspace("outline").join(".brio");
    let stage = stage::<Loose>("app::main");
    write_record(
        &build_directory,
        &stage,
        &inputs("sensor a\n"),
        &Loose(outline),
    );

    let envelopes = envelopes("outline");
    let elements = envelopes[0].elements();

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

    // Each parent before its children, in their order.
    let in_order: Vec<&Value> = elements.in_order().map(Element::payload).collect();
    assert_eq!(
        in_order,
        ["Document", "Install", "Usage", "Flags", "Appendix"]
    );
}

#[test]
fn a_record_of_another_module_under_the_right_record_key_is_rejected() {
    let build_directory = workspace("forged_module").join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
    let text = fs::read_to_string(&path).expect("the record can be read");

    // A whole record, with a digest to match, that says it is for another
    // module than the one its record key was computed for.
    let renamed = with_its_own_digest(&text.replacen(
        r#""module": "app::main""#,
        r#""module": "app::util""#,
        1,
    ));
    fs::write(&path, renamed).expect("the record can be changed");
    let read = RecordReader::in_build_directory(&build_directory).read(&stage, &inputs);

    assert!(matches!(
        read,
        Err(ReadError::Defective {
            defect: RecordDefect::Corrupted,
            ..
        })
    ));
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
    let path = write_record(&build_directory, &stage, &inputs, &Loose(outline));
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

    let missing = json!("0123456789abcdef0123456789abcdef");
    let forgeries: [(&str, Forge); 5] = [
        ("a parent that is not in the record", &|record| {
            record["elements"][&flags]["parent"] = missing.clone();
        }),
        ("a child its parent does not list", &|record| {
            record["elements"][&usage]["children"] = json!([]);
        }),
        ("a child listed under another parent", &|record| {
            record["elements"][&usage]["children"] = json!([]);
            record["elements"][&appendix]["children"] = json!([flags]);
        }),
        ("a child listed twice", &|record| {
            record["elements"][&usage]["children"] = json!([flags, flags]);
        }),
        ("a root that has a parent", &|record| {
            record["roots"] = json!([document, appendix, flags]);
        }),
    ];
    assert_forgeries_are_corrupted("forged_table", &path, &whole, &forgeries);
}

/// A payload that is the elements of another record, as they were read.
struct Copied(Vec<Element>);

impl PayloadType for Copied {
    const KIND: &'static str = "loose";
    const VERSION: u32 = 1;

    fn schema_fragment() -> Value {
        json!({})
    }
}

impl Payload for Copied {
    fn to_elements(&self, _stage: &Stage<Self>) -> Vec<Element> {
        self.0.clone()
    }

    fn from_elements(elements: &ElementTable) -> Option<Self> {
        Some(Self(elements.in_order().cloned().collect()))
    }
}

#[test]
fn elements_read_from_a_record_can_be_written_again_and_read_back() {
    let build_directory = workspace("rewritten").join(".brio");
    let writer = RecordWriter::in_build_directory(&build_directory);
    let reader = RecordReader::in_build_directory(&build_directory);
    let inputs = inputs("sensor a\n");
    writer
        .write(&stage::<Loose>("app::main"), || &inputs, &Loose(outline))
        .expect("the record is written");
    // The elements as read: each parent already lists its children.
    let copied = stage::<Copied>("app::main");
    let read = reader.read(&copied, &inputs).expect("the record is read");
    let document = copied.id(&["document"]);
    assert_eq!(
        read.envelope
            .elements()
            .get(&document)
            .map(|e| e.children().len()),
        Some(2)
    );

    writer
        .write(&copied, || &inputs, &read.payload)
        .expect("the record is written again");
    let again = reader
        .read(&copied, &inputs)
        .expect("the record is read again");

    let children = |record: &brio_lens::Record<Copied>| {
        record
            .envelope
            .elements()
            .get(&document)
            .map(|e| e.children().to_vec())
    };
    assert_eq!(children(&again), children(&read));
}

/// The inputs of a record that is never made.
fn inputs_never_asked_for() -> RecordInputs {
    panic!("the disabled writer asks for the inputs of a record")
}

#[test]
fn the_disabled_writer_writes_nothing() {
    let workspace = workspace("lens_off");

    let written = RecordWriter::disabled()
        .write(
            &stage::<Probe>("app::main"),
            || inputs("sensor a\n"),
            &probe(&[("a", 7)]),
        )
        .expect("nothing is refused");

    assert_eq!(written, None);
    assert_eq!(files_under(&workspace), [] as [PathBuf; 0]);
}

#[test]
fn the_disabled_writer_does_no_work_for_a_record() {
    // A record is serialized from the elements of its payload, and hashed
    // from them and from its inputs. The disabled writer asks for neither,
    // so it accepts even what an enabled writer refuses.
    let written = RecordWriter::disabled()
        .write(
            &stage::<Loose>("app::main"),
            inputs_never_asked_for,
            &Loose(|_| panic!("the disabled writer asks a payload for its elements")),
        )
        .expect("nothing is refused");

    assert_eq!(written, None);
}

#[test]
fn a_file_that_is_not_named_as_a_record_is_given_as_an_error_and_hides_no_other_record() {
    let build_directory = workspace("misnamed").join(".brio");
    let inputs = inputs("sensor a\n");
    let probe_record = write_record(
        &build_directory,
        &stage::<Probe>("app::main"),
        &inputs,
        &probe(&[("a", 7)]),
    );
    write_record(
        &build_directory,
        &stage::<Gauge>("app::main"),
        &inputs,
        &Gauge("steady".to_owned()),
    );
    let misnamed = probe_record.with_extension("JSON");
    fs::rename(&probe_record, &misnamed).expect("the record can be renamed");
    let foreign = probe_record.with_file_name("notes.txt");
    fs::write(&foreign, "not a record\n").expect("a foreign file can be added");

    let records = RecordReader::in_build_directory(&build_directory)
        .read_all()
        .expect("the records are read");

    // Each file gives a record or an error, in the order of the file names.
    let mut files = files_under(&build_directory);
    files.sort();
    let expected: Vec<Option<&PathBuf>> = files
        .iter()
        .map(|file| (*file == misnamed || *file == foreign).then_some(file))
        .collect();
    let not_records: Vec<Option<&PathBuf>> = records
        .iter()
        .map(|record| match record {
            Err(ReadError::NotARecord { path }) => Some(path),
            Ok(_) => None,
            Err(other) => panic!("a file is read in an unexpected way: {other}"),
        })
        .collect();
    assert_eq!(not_records, expected);
    assert_eq!(not_records.iter().flatten().count(), 2);
}

/// The file where the writer `writer` of the process `process` keeps the
/// record of the file `record` until it is complete.
fn unfinished_file(record: &Path, process: &str, writer: &str) -> PathBuf {
    record.with_extension(format!("{process}-{writer}.unfinished"))
}

/// The file of every unfinished record that a writer of this process can
/// keep for the record of the file `record`: one for each writer the
/// process has, and it has fewer than these in the whole of these tests.
fn unfinished_files_of_this_process(record: &Path) -> impl Iterator<Item = PathBuf> {
    let process = std::process::id().to_string();
    (0..2000).map(move |writer| unfinished_file(record, &process, &writer.to_string()))
}

#[test]
fn a_record_that_a_writer_has_not_finished_is_no_record_yet() {
    let build_directory = workspace("unfinished").join(".brio");
    let path = write_record(
        &build_directory,
        &stage::<Probe>("app::main"),
        &inputs("sensor a\n"),
        &probe(&[("a", 7)]),
    );
    // What a writer of the same record leaves until its record is complete.
    fs::write(unfinished_file(&path, "4242", "0"), "{\n  \"dig")
        .expect("an unfinished file can be added");

    let records = RecordReader::in_build_directory(&build_directory)
        .read_all()
        .expect("the records are read");

    assert_eq!(records.len(), 1);
    assert!(records[0].is_ok());
}

#[test]
fn a_file_named_as_unfinished_that_no_writer_made_is_given_as_an_error() {
    let build_directory = workspace("unfinished_by_no_writer").join(".brio");
    let path = write_record(
        &build_directory,
        &stage::<Probe>("app::main"),
        &inputs("sensor a\n"),
        &probe(&[("a", 7)]),
    );
    fs::rename(&path, path.with_extension("unfinished")).expect("the record can be renamed");
    for (name, text) in [
        ("notes.4242-0.unfinished", "not a record\n"),
        ("4242-0.unfinished", ""),
    ] {
        fs::write(path.with_file_name(name), text).expect("a foreign file can be added");
    }
    // No process has so high a number, and no writer keeps a directory.
    fs::write(unfinished_file(&path, "99999999999", "0"), "").expect("a foreign file can be added");
    fs::create_dir(unfinished_file(&path, "4242", "1")).expect("a directory can be added");

    let records = RecordReader::in_build_directory(&build_directory)
        .read_all()
        .expect("the records are read");

    assert_eq!(records.len(), 5);
    for record in &records {
        assert!(matches!(record, Err(ReadError::NotARecord { .. })));
    }
}

#[test]
fn a_reader_beside_writers_of_a_record_is_always_given_that_record() {
    let build_directory = workspace("reader_beside_rewriters").join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));

    let write_often = || {
        for _ in 0..200 {
            write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
        }
    };
    let read_often = || {
        let reader = RecordReader::in_build_directory(&build_directory);
        for _ in 0..200 {
            let records = reader.read_all().expect("the records are read");
            // A record that is written again is never gone for a moment.
            assert_eq!(records.len(), 1, "{records:?}");
            assert!(records[0].is_ok(), "{records:?}");
        }
    };
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(write_often);
        }
        scope.spawn(read_often);
    });
}

#[test]
fn writing_a_record_again_leaves_the_file_that_is_there() {
    let workspace = workspace("written_again");
    let build_directory = workspace.join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
    // Another name of the file the record is in.
    let same_file = workspace.join("same-file");
    fs::hard_link(&path, &same_file).expect("the record can be given a second name");

    write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));

    // The record is still in that file, and no other took its place: what
    // is written under the other name is read under the name of the record.
    fs::write(&same_file, "the same file\n").expect("the file can be written");
    assert_eq!(
        fs::read_to_string(&path).expect("the record can be read"),
        "the same file\n"
    );
    assert_eq!(files_under(&build_directory), [path]);
}

#[test]
fn a_writer_keeps_its_unfinished_record_under_a_name_that_readers_pass_over() {
    let build_directory = workspace("unfinished_name").join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
    let whole = fs::read(&path).expect("the record can be read");
    // Every name a writer of this process can keep its unfinished record
    // under is taken, by what no writer can make its file of.
    for taken in unfinished_files_of_this_process(&path) {
        fs::create_dir(taken).expect("a directory can be added");
    }

    // The same record key, and another payload: the record is to be
    // replaced, so the writer needs its unfinished record.
    let error = RecordWriter::in_build_directory(&build_directory)
        .write(&stage, || &inputs, &probe(&[("a", 8)]))
        .expect_err("the writer has nowhere to keep its unfinished record");

    assert!(matches!(error, WriteError::Io { .. }));
    assert_eq!(fs::read(&path).expect("the record can be read"), whole);
}

#[test]
fn writing_a_record_again_puts_it_in_the_place_of_a_corrupted_one() {
    let workspace = workspace("written_over_corrupted");
    let build_directory = workspace.join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
    let whole = fs::read(&path).expect("the record can be read");
    tamper(&path, r#""value": 7"#, r#""value": 8"#);

    write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));

    assert_eq!(fs::read(&path).expect("the record can be read"), whole);
    assert_eq!(files_under(&workspace), [path]);
}

#[test]
fn a_name_a_stopped_writer_left_is_never_written_through_and_is_passed_over() {
    let workspace = workspace("stopped_writer");
    let build_directory = workspace.join(".brio");
    let stage = stage::<Probe>("app::main");
    let inputs = inputs("sensor a\n");
    let path = write_record(&build_directory, &stage, &inputs, &probe(&[("a", 7)]));
    let whole = fs::read(&path).expect("the record can be read");
    // A reader that has the record open: another name of the same file.
    let open_record = workspace.join("open-record");
    fs::hard_link(&path, &open_record).expect("the record can be given a second name");
    // What a writer leaves when it is stopped once its record is in place:
    // its unfinished name, still a name of the record. The next writers of
    // this process make the same names.
    for left in unfinished_files_of_this_process(&path) {
        fs::hard_link(&path, left).expect("the record can be given a second name");
    }

    // The same record key, and another payload.
    write_record(&build_directory, &stage, &inputs, &probe(&[("a", 8)]));

    assert_eq!(
        fs::read(&open_record).expect("the open record can be read"),
        whole
    );
    assert_ne!(fs::read(&path).expect("the record can be read"), whole);
    // A reader passes over every name that is left, and is given the record.
    let records = RecordReader::in_build_directory(&build_directory)
        .read_all()
        .expect("the records are read");
    assert_eq!(records.len(), 1, "{records:?}");
    assert!(records[0].is_ok(), "{records:?}");
}
