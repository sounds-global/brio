// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Stage tests: comparing produced records against reference records,
//! through the public interface of `brio_lens`.

mod support;

use brio_lens::json::{Value, json};
use brio_lens::{
    ComparedRecords, Element, ElementTable, Envelope, Payload, PayloadType, Reason, RecordSet,
    Stage, StageTest, StageTestError,
};

use support::{
    Loose, Probe, build_directory, envelopes, inputs, probe, stage, workspace, write_record,
};

/// The payload of a later version of the phase that produces [`Loose`]: it
/// has no element.
struct LaterLoose;

impl PayloadType for LaterLoose {
    const KIND: &'static str = Loose::KIND;
    const VERSION: u32 = Loose::VERSION + 1;

    fn schema_fragment() -> Value {
        json!({})
    }
}

impl Payload for LaterLoose {
    fn to_elements(&self, _stage: &Stage<Self>) -> Vec<Element> {
        Vec::new()
    }

    fn from_elements(_elements: &ElementTable) -> Option<Self> {
        Some(Self)
    }
}

/// Where the record set `set` of `test` is, as the name of the workspace of
/// a test.
fn set_of(test: &str, set: RecordSet) -> String {
    format!("{test}/{set}")
}

/// Writes the record of what the stage of payload `P` produced for `module`
/// from `source`, as `payload`, into the record set `set`.
fn write<P: Payload>(set: &str, module: &str, source: &str, payload: &P) {
    write_record(
        build_directory(set),
        &stage::<P>(module),
        &inputs(source),
        payload,
    );
}

/// The empty record sets of `test`: the produced one, then the reference.
fn record_sets(test: &str) -> (String, String) {
    workspace(test);
    (
        set_of(test, RecordSet::Produced),
        set_of(test, RecordSet::Reference),
    )
}

/// The Stage test of the record set `produced` against `reference`.
fn stage_test(produced: &str, reference: &str) -> StageTest {
    let produced: Vec<Envelope> = envelopes(produced);
    let reference: Vec<Envelope> = envelopes(reference);
    StageTest::compare(&produced, &reference).expect("each set has one record of each stage")
}

#[test]
fn identical_record_sets_report_no_differences() {
    let (produced, reference) = record_sets("stage_test_identical");
    for set in [&produced, &reference] {
        write(
            set,
            "app::main",
            "sensor a\n",
            &probe(&[("a", 7), ("b", -2)]),
        );
        write(set, "app::motor", "sensor c\n", &probe(&[("c", 1)]));
    }

    let test = stage_test(&produced, &reference);

    assert!(test.passed());
    assert!(test.differences().is_empty());
}

#[test]
fn added_removed_and_changed_elements_are_reported_separately_for_each_stage() {
    let (produced, reference) = record_sets("stage_test_differences");
    write(
        &reference,
        "app::main",
        "sensor a\n",
        &probe(&[("a", 1), ("b", 2), ("c", 3)]),
    );
    write(
        &produced,
        "app::main",
        "sensor d\n",
        &probe(&[("a", 1), ("b", 5), ("d", 4)]),
    );
    write(&reference, "app::motor", "sensor e\n", &probe(&[("e", 1)]));
    write(&produced, "app::motor", "sensor e\n", &probe(&[("e", 1)]));
    write(&reference, "app::valve", "sensor f\n", &probe(&[("f", 1)]));
    write(
        &produced,
        "app::valve",
        "sensor f\n",
        &probe(&[("f", 2), ("g", 3)]),
    );

    let test = stage_test(&produced, &reference);

    assert!(!test.passed());
    let [main, valve] = test.differences() else {
        panic!("two stages differ: {:?}", test.differences());
    };
    let main_stage = stage::<Probe>("app::main");
    assert_eq!((main.module(), main.payload_kind()), ("app::main", "probe"));
    assert_eq!(main.added(), [main_stage.id(&["sensor", "d"])]);
    assert_eq!(main.removed(), [main_stage.id(&["sensor", "c"])]);
    assert_eq!(main.changed(), [main_stage.id(&["sensor", "b"])]);
    let valve_stage = stage::<Probe>("app::valve");
    assert_eq!(
        (valve.module(), valve.payload_kind()),
        ("app::valve", "probe")
    );
    assert_eq!(valve.added(), [valve_stage.id(&["sensor", "g"])]);
    assert_eq!(valve.removed(), []);
    assert_eq!(valve.changed(), [valve_stage.id(&["sensor", "f"])]);
}

#[test]
fn a_stage_with_a_record_in_one_set_alone_differs_even_with_no_element() {
    let (produced, reference) = record_sets("stage_test_one_set_alone");
    write(&produced, "app::main", "sensor a\n", &probe(&[("a", 1)]));
    write(&produced, "app::motor", "", &probe(&[]));
    write(&reference, "app::valve", "sensor f\n", &probe(&[("f", 1)]));

    let test = stage_test(&produced, &reference);

    let [main, motor, valve] = test.differences() else {
        panic!("three stages differ: {:?}", test.differences());
    };
    assert_eq!(
        main.added(),
        [stage::<Probe>("app::main").id(&["sensor", "a"])]
    );
    assert_eq!(main.removed(), []);
    assert_eq!(main.records(), ComparedRecords::ProducedOnly);
    assert_eq!(motor.module(), "app::motor");
    assert_eq!(motor.added(), []);
    assert_eq!(motor.records(), ComparedRecords::ProducedOnly);
    assert_eq!(valve.added(), []);
    assert_eq!(
        valve.removed(),
        [stage::<Probe>("app::valve").id(&["sensor", "f"])]
    );
    assert_eq!(valve.records(), ComparedRecords::ReferenceOnly);
}

#[test]
fn a_record_with_another_payload_version_than_its_reference_differs() {
    let (produced, reference) = record_sets("stage_test_payload_version");
    write(&produced, "app::main", "", &LaterLoose);
    write(&reference, "app::main", "", &Loose(|_| Vec::new()));

    let test = stage_test(&produced, &reference);

    let [main] = test.differences() else {
        panic!("one stage differs: {:?}", test.differences());
    };
    assert_eq!(
        main.records(),
        ComparedRecords::Both {
            produced_payload_version: 2,
            reference_payload_version: 1
        }
    );
    assert_eq!(
        (main.added(), main.removed(), main.changed()),
        (&[][..], &[][..], &[][..])
    );
}

#[test]
fn an_element_with_another_reason_or_other_children_is_changed() {
    let (produced, reference) = record_sets("stage_test_reason_and_children");
    write(
        &reference,
        "app::main",
        "",
        &Loose(|stage| {
            vec![
                Element::new(stage.id(&["a"]), json!("a")),
                Element::new(stage.id(&["b"]), json!("b"))
                    .because(Reason::new("kept", json!("it is read"))),
            ]
        }),
    );
    write(
        &produced,
        "app::main",
        "",
        &Loose(|stage| {
            vec![
                Element::new(stage.id(&["a"]), json!("a")),
                Element::new(stage.id(&["a", "inner"]), json!("inner")).child_of(&stage.id(&["a"])),
                Element::new(stage.id(&["b"]), json!("b"))
                    .because(Reason::new("kept", json!("it is written"))),
            ]
        }),
    );

    let test = stage_test(&produced, &reference);

    let [main] = test.differences() else {
        panic!("one stage differs: {:?}", test.differences());
    };
    let stage = stage::<Loose>("app::main");
    let mut changed = vec![stage.id(&["a"]), stage.id(&["b"])];
    changed.sort();
    assert_eq!(main.added(), [stage.id(&["a", "inner"])]);
    assert_eq!(main.changed(), changed);
    assert!(!main.roots_reordered());
}

#[test]
fn roots_in_another_order_differ_although_no_element_changed() {
    let (produced, reference) = record_sets("stage_test_roots_order");
    write(&reference, "app::main", "", &probe(&[("a", 1), ("b", 2)]));
    write(
        &produced,
        "app::main",
        "",
        &probe(&[("b", 2), ("a", 1), ("c", 3)]),
    );
    write(&reference, "app::motor", "", &probe(&[("a", 1), ("b", 2)]));
    write(
        &produced,
        "app::motor",
        "",
        &probe(&[("a", 1), ("c", 3), ("b", 2)]),
    );

    let test = stage_test(&produced, &reference);

    let [main, motor] = test.differences() else {
        panic!("two stages differ: {:?}", test.differences());
    };
    assert!(main.roots_reordered());
    assert_eq!(main.changed(), []);
    assert!(!motor.roots_reordered());
}

#[test]
fn a_set_with_two_records_of_one_stage_is_not_compared() {
    let (produced, reference) = record_sets("stage_test_two_records");
    write(&produced, "app::main", "sensor a\n", &probe(&[("a", 1)]));
    write(&reference, "app::main", "sensor a\n", &probe(&[("a", 1)]));
    write(&reference, "app::main", "sensor b\n", &probe(&[("b", 1)]));

    let error = StageTest::compare(&envelopes(&produced), &envelopes(&reference))
        .expect_err("the reference records are ambiguous");

    assert_eq!(
        error,
        StageTestError::TwoRecordsOfOneStage {
            set: RecordSet::Reference,
            module: "app::main".to_owned(),
            payload_kind: "probe".to_owned()
        }
    );
    assert_eq!(
        error.to_string(),
        "the reference records hold two `probe` records of module `app::main`: the set holds \
         the records of more than one build"
    );
}

#[test]
fn the_comparison_depends_on_no_other_workspace_crate() {
    // The Lens viewer is a workspace crate, as every phase is: a Stage test
    // that needed one of them would name it in the manifest of `brio_lens`.
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("the manifest can be read");

    let workspace_crates: Vec<&str> = manifest
        .lines()
        .filter(|line| line.starts_with("brio_"))
        .collect();
    assert_eq!(workspace_crates, Vec::<&str>::new());
}
