// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Provenance and Reasons on the elements of stage records, through the
//! public interface of `brio_lens`.

mod support;

use brio_lens::json::{self, Value, json};
use brio_lens::{
    Element, Envelope, Payload, Reason, RecordRefusal, RecordWriter, SourceSpan, Stage, WriteError,
};

use support::{
    Forge, Loose, Probe, assert_forgeries_are_corrupted, build_directory, envelopes, files_under,
    inputs, nested, probe, resealed, stage, workspace,
};

/// The source of the module `app::main`.
const SOURCE: &str = "sensor a\nsensor b\n";

/// Writes the record of what the stage of payload `P` produced for
/// `app::main`, as `payload`, into the build directory of `test`.
fn write<P: Payload>(test: &str, payload: &P) -> Stage<P> {
    write_for(test, "app::main", payload)
}

/// Writes the record of what the stage of payload `P` produced for `module`,
/// as `payload`, into the build directory of `test`.
fn write_for<P: Payload>(test: &str, module: &str, payload: &P) -> Stage<P> {
    let stage = stage::<P>(module);
    RecordWriter::in_build_directory(build_directory(test))
        .write(&stage, &inputs(SOURCE), payload)
        .expect("the record is written");
    stage
}

#[test]
fn an_elements_provenance_resolves_back_to_its_source_span() {
    workspace("provenance_source");
    let stage = write(
        "provenance_source",
        &Loose(|stage| {
            vec![
                Element::new(stage.id(&["sensor", "b"]), json!("b")).from_source(SourceSpan {
                    file: "src/main.brio".to_owned(),
                    start: 9,
                    end: 17,
                }),
            ]
        }),
    );

    let envelopes = envelopes("provenance_source");
    let element = envelopes[0]
        .elements()
        .get(&stage.id(&["sensor", "b"]))
        .expect("the element is in the record");

    let [span] = element.provenance().source() else {
        panic!("the element comes from one span of source text");
    };
    assert_eq!(span.file, "src/main.brio");
    assert_eq!((span.start, span.end), (9, 17));
    // A span counts bytes of the file, from its start up to but not
    // including its end.
    let (start, end) = (span.start as usize, span.end as usize);
    assert_eq!(&SOURCE[start..end], "sensor b");
}

#[test]
fn a_source_file_named_by_an_absolute_path_is_not_written() {
    let workspace = workspace("provenance_absolute_path");
    let stage = stage::<Loose>("app::main");

    let error = RecordWriter::in_build_directory(workspace.join(".brio"))
        .write(
            &stage,
            &inputs(SOURCE),
            &Loose(|stage| {
                vec![
                    Element::new(stage.id(&["sensor", "b"]), json!("b")).from_source(SourceSpan {
                        file: "/home/ada/app/src/main.brio".to_owned(),
                        start: 9,
                        end: 17,
                    }),
                ]
            }),
        )
        .expect_err("an absolute path is refused");

    let sensor = stage.id(&["sensor", "b"]);
    assert!(matches!(
        &error,
        WriteError::Refused(RecordRefusal::AbsoluteSourcePath { element, file })
            if *element == sensor && file == "/home/ada/app/src/main.brio"
    ));
    assert_eq!(
        error.to_string(),
        format!(
            "the element `{sensor}` comes from the source file `/home/ada/app/src/main.brio`, \
             named by an absolute path, which a record never holds"
        )
    );
    assert_eq!(files_under(&workspace), Vec::<std::path::PathBuf>::new());
}

#[test]
fn an_elements_provenance_resolves_to_the_previous_stage_elements_it_derives_from() {
    workspace("provenance_previous_stage");
    // The previous stage reads every sensor; this one sums the readings.
    let probes = write("provenance_previous_stage", &probe(&[("a", 7), ("b", -2)]));
    let sums = write(
        "provenance_previous_stage",
        &Loose(|stage| {
            let probes = support::stage::<Probe>("app::main");
            vec![
                Element::new(stage.id(&["sum"]), json!(5))
                    .derived_from(probes.link(&["sensor", "a"]))
                    .derived_from(probes.link(&["sensor", "b"])),
            ]
        }),
    );

    // Read as a tool that knows neither payload type.
    let envelopes = envelopes("provenance_previous_stage");
    let sum = envelopes[0]
        .elements()
        .get(&sums.id(&["sum"]))
        .expect("the element is in the record");

    let [from_a, from_b] = sum.provenance().derived_from() else {
        panic!("the sum derives from two readings");
    };
    assert_eq!(from_a.module(), "app::main");
    assert_eq!(from_a.payload_kind(), "probe");
    assert_eq!(from_a.id(), &probes.id(&["sensor", "a"]));
    let a = from_a.resolve(&envelopes).expect("the reading is found");
    let b = from_b.resolve(&envelopes).expect("the reading is found");
    assert_eq!(a.payload(), &json!({ "sensor": "a", "value": 7 }));
    assert_eq!(b.payload(), &json!({ "sensor": "b", "value": -2 }));

    // A link finds nothing among records that do not hold its element.
    assert!(from_a.resolve(&envelopes[..1]).is_none());
    let elsewhere = support::stage::<Probe>("app::util").link(&["sensor", "a"]);
    assert!(elsewhere.resolve(&envelopes).is_none());
}

/// Writes the records of a chain of origin into the build directory of
/// `test`, and gives the stages of `app::util` and `app::main`.
///
/// `app::util` declares a generic function. The compiler makes a copy of it
/// for `i64`, then puts the body of that copy where `main` calls it.
fn write_chain(test: &str) -> (Stage<Loose>, Stage<Loose>) {
    workspace(test);
    let util = write_for(
        test,
        "app::util",
        &Loose(|stage| {
            vec![
                Element::new(stage.id(&["scale"]), json!("fn scale<T>")).from_source(declaration()),
            ]
        }),
    );
    let main = write(
        test,
        &Loose(|stage| {
            let generic = support::stage::<Loose>("app::util").link(&["scale"]);
            vec![
                Element::new(stage.id(&["scale<i64>"]), json!("fn scale<i64>"))
                    .created_by("monomorphization", generic.clone()),
                Element::new(stage.id(&["main", "scale<i64>", "body"]), json!("x * 2"))
                    .created_by("inlining", stage.link(&["scale<i64>"]))
                    .created_by("monomorphization", generic),
            ]
        }),
    );
    (util, main)
}

/// Where `app::util` declares its generic function.
fn declaration() -> SourceSpan {
    SourceSpan {
        file: "src/util.brio".to_owned(),
        start: 12,
        end: 40,
    }
}

#[test]
fn a_compiler_created_element_records_its_full_chain_of_origin_back_to_source() {
    let (util, main) = write_chain("provenance_chain");

    let envelopes = envelopes("provenance_chain");
    let inlined = main
        .link(&["main", "scale<i64>", "body"])
        .resolve(&envelopes)
        .expect("the element is found");

    // The element has no source text of its own: the chain leads to it.
    assert_eq!(inlined.provenance().source(), []);
    let [inlining, monomorphization] = inlined.provenance().origin() else {
        panic!("the element was created in two steps");
    };
    assert_eq!(inlining.by, "inlining");
    assert_eq!(inlining.from, main.link(&["scale<i64>"]));
    assert_eq!(monomorphization.by, "monomorphization");
    assert_eq!(monomorphization.from, util.link(&["scale"]));

    let copy = inlining
        .from
        .resolve(&envelopes)
        .expect("the copy is found");
    assert_eq!(copy.payload(), &json!("fn scale<i64>"));
    let declared = monomorphization
        .from
        .resolve(&envelopes)
        .expect("the end of the chain is found");
    assert_eq!(declared.provenance().source(), [declaration()]);
    assert_eq!(declared.provenance().origin(), []);
}

#[test]
fn a_chain_of_origin_is_followed_to_source_from_any_element_on_it() {
    let (util, main) = write_chain("provenance_chain_followed");
    let envelopes = envelopes("provenance_chain_followed");
    let provenance = |link: brio_lens::ElementLink| {
        link.resolve(&envelopes)
            .expect("the element is found")
            .provenance()
    };
    let inlined = provenance(main.link(&["main", "scale<i64>", "body"]));
    let declaration = declaration();

    assert_eq!(inlined.origin_source(&envelopes), [&declaration]);
    let copy = provenance(main.link(&["scale<i64>"]));
    assert_eq!(copy.origin_source(&envelopes), [&declaration]);
    let declared = provenance(util.link(&["scale"]));
    assert_eq!(declared.origin_source(&envelopes), [&declaration]);

    // A chain that leaves the records at hand leads nowhere.
    let main_alone: Vec<&Envelope> = envelopes
        .iter()
        .filter(|envelope| envelope.module() == "app::main")
        .collect();
    assert!(inlined.origin_source(main_alone).is_empty());
}

#[test]
fn a_reason_attached_to_a_decision_round_trips_unchanged() {
    workspace("reasons");
    let probes = write("reasons", &probe(&[("a", 7), ("b", -2)]));
    // This stage decides which reading is the alarm, and says why.
    let alarms = write(
        "reasons",
        &Loose(|stage| {
            let probes = support::stage::<Probe>("app::main");
            vec![
                Element::new(stage.id(&["alarm"]), json!("b"))
                    .because(
                        Reason::new(
                            "alarm",
                            json!({ "rule": "lowest reading", "candidates": { "a": 7, "b": -2 } }),
                        )
                        .involving(probes.link(&["sensor", "a"]))
                        .involving(probes.link(&["sensor", "b"])),
                    )
                    .because(Reason::new("severity", json!("below zero"))),
            ]
        }),
    );

    let envelopes = envelopes("reasons");
    let alarm = alarms
        .link(&["alarm"])
        .resolve(&envelopes)
        .expect("the element is found");

    let [chosen, severity] = alarm.reasons() else {
        panic!("the element records two decisions");
    };
    assert_eq!(chosen.decision, "alarm");
    assert_eq!(
        chosen.because,
        json!({ "rule": "lowest reading", "candidates": { "a": 7, "b": -2 } })
    );
    assert_eq!(
        chosen.involves,
        [probes.link(&["sensor", "a"]), probes.link(&["sensor", "b"])]
    );
    let lowest = chosen.involves[1]
        .resolve(&envelopes)
        .expect("the reading is found");
    assert_eq!(lowest.payload()["value"], -2);
    assert_eq!(severity, &Reason::new("severity", json!("below zero")));
}

#[test]
fn a_reason_is_read_back_at_the_deepest_nesting_and_refused_beyond_it() {
    let workspace = workspace("reason_nesting");
    let stage = stage::<Loose>("app::main");
    let writer = RecordWriter::in_build_directory(workspace.join(".brio"));
    assert_eq!(Reason::MAX_DEPTH, 122);

    let path = writer
        .write(
            &stage,
            &inputs(SOURCE),
            &Loose(|stage| {
                vec![
                    Element::new(stage.id(&["a"]), json!(1)).because(Reason::new("a", nested(122))),
                ]
            }),
        )
        .expect("the deepest reason is written");
    let envelopes = envelopes("reason_nesting");
    let read = stage.link(&["a"]).resolve(&envelopes).expect("it is found");
    assert_eq!(read.reasons()[0].because, nested(122));

    let error = writer
        .write(
            &stage,
            &inputs(SOURCE),
            &Loose(|stage| {
                vec![
                    Element::new(stage.id(&["a"]), json!(1)).because(Reason::new("a", nested(123))),
                ]
            }),
        )
        .expect_err("no reader could read this reason back");
    let a = stage.id(&["a"]);
    assert!(matches!(
        &error,
        WriteError::Refused(RecordRefusal::ReasonTooDeep { element, depth: 123 }) if *element == a
    ));
    assert_eq!(
        error.to_string(),
        format!(
            "a reason of the element `{a}` nests 123 deep, but a reason nests at most 122 levels"
        )
    );
    assert_eq!(files_under(&workspace), [path]);
}

/// One element with a source span, a previous-stage element and a reason.
fn with_provenance_and_a_reason(stage: &Stage<Loose>) -> Vec<Element> {
    let other = support::stage::<Probe>("app::main").link(&["sensor", "a"]);
    vec![
        Element::new(stage.id(&["a"]), json!(1))
            .from_source(SourceSpan {
                file: "src/main.brio".to_owned(),
                start: 0,
                end: 8,
            })
            .derived_from(other.clone())
            .because(Reason::new("a", json!("why")).involving(other)),
    ]
}

#[test]
fn a_record_whose_provenance_the_writer_would_refuse_is_rejected_even_with_a_matching_digest() {
    let workspace = workspace("provenance_forged");
    let stage = stage::<Loose>("app::main");
    let path = RecordWriter::in_build_directory(workspace.join(".brio"))
        .write(
            &stage,
            &inputs(SOURCE),
            &Loose(with_provenance_and_a_reason),
        )
        .expect("the record is written");
    let text = std::fs::read_to_string(&path).expect("the record can be read");
    let whole: Value = json::from_str(&text).expect("the record is JSON");
    assert_eq!(resealed(whole.clone()), text);
    let a = stage.id(&["a"]).to_string();

    let forgeries: [(&str, Forge); 5] = [
        ("a source file named by an absolute path", &|record| {
            record["elements"][&a]["provenance"]["source"][0]["file"] =
                json!("/home/ada/app/src/main.brio");
        }),
        ("a link to a module named by an absolute path", &|record| {
            record["elements"][&a]["provenance"]["derived_from"][0]["module"] =
                json!("~/app/main.brio");
        }),
        ("a link with an invalid payload kind", &|record| {
            record["elements"][&a]["reasons"][0]["involves"][0]["payload_kind"] =
                json!("probe.3+gauge");
        }),
        ("a field no element has", &|record| {
            record["elements"][&a]["provenance"]["note"] = json!("extra");
        }),
        ("no reasons", &|record| {
            record["elements"][&a]
                .as_object_mut()
                .expect("an object")
                .remove("reasons");
        }),
    ];
    assert_forgeries_are_corrupted("provenance_forged", &path, &whole, &forgeries);
}
