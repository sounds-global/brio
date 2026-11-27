// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Masked values in stage records, through the public interface of
//! `brio_lens`.

mod support;

use std::fs;
use std::path::PathBuf;

use brio_lens::json::{self, Value, json};
use brio_lens::{Element, Reason, RecordRefusal, RecordWriter, Stage, WriteError};

use support::{
    Forge, Loose, assert_forgeries_are_corrupted, build_directory, envelopes, files_under, inputs,
    stage, workspace, write_record,
};

/// The source of the module `app::main`, which holds a confidential value.
const SOURCE: &str = "let token: Secret[Text] = \"hunter2\"\n";

/// What a stage produces for [`SOURCE`]: one binding, whose value is masked
/// in its payload and in the reason for a decision about it.
fn binding(stage: &Stage<Loose>) -> Vec<Element> {
    vec![
        Element::new(
            stage.id(&["binding", "token"]),
            json!({ "name": "token", "value": "hunter2", "unit": null }),
        )
        .masked_at("/value")
        .because(
            Reason::new("folded constant", json!({ "folded_to": ["hunter2"] }))
                .masked_at("/folded_to/0"),
        ),
    ]
}

/// Writes the record of [`binding`] into the build directory of `test`, and
/// gives its stage and its file.
fn write_binding(test: &str) -> (Stage<Loose>, PathBuf) {
    let stage = stage::<Loose>("app::main");
    let path = write_record(
        build_directory(test),
        &stage,
        &inputs(SOURCE),
        &Loose(binding),
    );
    (stage, path)
}

#[test]
fn a_masked_value_is_written_as_a_masked_marker_and_its_content_appears_nowhere() {
    let workspace = workspace("masked_written");
    let (stage, path) = write_binding("masked_written");

    let record: Value = json::from_str(&fs::read_to_string(&path).expect("the record can be read"))
        .expect("the record is JSON");
    let element = &record["elements"][stage.id(&["binding", "token"]).to_string()];
    assert_eq!(
        element["payload"],
        json!({ "name": "token", "value": null, "unit": null })
    );
    assert_eq!(element["masked"], json!(["/value"]));
    let reason = &element["reasons"][0];
    assert_eq!(reason["because"], json!({ "folded_to": [null] }));
    assert_eq!(reason["masked"], json!(["/folded_to/0"]));

    for file in files_under(&workspace) {
        let written = fs::read_to_string(&file).expect("the file can be read");
        assert!(
            !written.contains("hunter2"),
            "`{}` holds the masked value",
            file.display()
        );
    }
}

#[test]
fn a_masked_marker_reads_back_as_masked() {
    workspace("masked_read");
    let (stage, _) = write_binding("masked_read");

    let envelopes = envelopes("masked_read");
    let element = envelopes[0]
        .elements()
        .get(&stage.id(&["binding", "token"]))
        .expect("the element is in the record");

    assert!(element.is_masked("/value"));
    assert_eq!(element.masked().collect::<Vec<_>>(), ["/value"]);
    // A value that is absent is not a masked one.
    assert!(!element.is_masked("/unit"));
    assert!(!element.is_masked("/name"));
    let [reason] = element.reasons() else {
        panic!("the element has one reason");
    };
    assert!(reason.is_masked("/folded_to/0"));
    assert!(!reason.is_masked("/folded_to"));
}

#[test]
fn a_payload_is_free_to_hold_what_looks_like_a_masked_marker() {
    workspace("masked_lookalike");
    let stage = stage::<Loose>("app::main");
    RecordWriter::in_build_directory(build_directory("masked_lookalike"))
        .write(
            &stage,
            || inputs(SOURCE),
            &Loose(|stage| {
                vec![Element::new(
                    stage.id(&["literal"]),
                    json!({ "masked": ["/value"], "$masked": true, "value": null }),
                )]
            }),
        )
        .expect("the record is written");

    let envelopes = envelopes("masked_lookalike");
    let element = envelopes[0]
        .elements()
        .get(&stage.id(&["literal"]))
        .expect("the element is in the record");
    assert!(!element.is_masked("/value"));
    assert_eq!(element.masked().count(), 0);
}

/// What the writer says when asked to write `elements`.
fn refusal(test: &str, elements: fn(&Stage<Loose>) -> Vec<Element>) -> RecordRefusal {
    let workspace = workspace(test);
    let error = RecordWriter::in_build_directory(build_directory(test))
        .write(
            &stage::<Loose>("app::main"),
            || inputs(SOURCE),
            &Loose(elements),
        )
        .expect_err("the record is refused");
    assert_eq!(files_under(&workspace), Vec::<PathBuf>::new());
    match error {
        WriteError::Refused(refusal) => refusal,
        other => panic!("the write failed in an unexpected way: {other}"),
    }
}

#[test]
fn a_masked_marker_where_no_value_was_masked_is_not_written() {
    let token = stage::<Loose>("app::main").id(&["binding", "token"]);
    let misplaced = |place: &str| RecordRefusal::MisplacedMaskedMarker {
        element: token.clone(),
        place: place.to_owned(),
    };

    let nowhere = refusal("masked_nowhere", |stage| {
        vec![
            Element::new(
                stage.id(&["binding", "token"]),
                json!({ "value": "hunter2" }),
            )
            .masked_at("/missing"),
        ]
    });
    assert_eq!(nowhere, misplaced("/missing"));
    assert_eq!(
        nowhere.to_string(),
        format!(
            "the element `{token}` has a masked marker at `/missing`, where no value was masked"
        )
    );

    let not_a_place = refusal("masked_not_a_place", |stage| {
        vec![
            Element::new(
                stage.id(&["binding", "token"]),
                json!({ "value": "hunter2" }),
            )
            .masked_at("value"),
        ]
    });
    assert_eq!(not_a_place, misplaced("value"));

    let put_back = refusal("masked_put_back", |stage| {
        let mut reason = Reason::new("folded constant", json!({ "folded_to": "hunter2" }))
            .masked_at("/folded_to");
        reason.because = json!({ "folded_to": "hunter2" });
        vec![Element::new(stage.id(&["binding", "token"]), json!("token")).because(reason)]
    });
    assert_eq!(put_back, misplaced("/folded_to"));
}

#[test]
fn a_record_with_a_value_where_one_was_masked_is_rejected_even_with_a_matching_digest() {
    workspace("masked_forged");
    let (stage, path) = write_binding("masked_forged");
    let whole: Value = json::from_str(&fs::read_to_string(&path).expect("the record can be read"))
        .expect("the record is JSON");
    let id = stage.id(&["binding", "token"]).to_string();

    let a_value_put_back: Forge = &|record| {
        record["elements"][&id]["payload"]["value"] = json!("hunter2");
    };
    let a_reason_put_back: Forge = &|record| {
        record["elements"][&id]["reasons"][0]["because"]["folded_to"][0] = json!("hunter2");
    };
    let a_marker_elsewhere: Forge = &|record| {
        record["elements"][&id]["masked"] = json!(["/name"]);
    };
    let a_marker_twice: Forge = &|record| {
        record["elements"][&id]["masked"] = json!(["/value", "/value"]);
    };
    assert_forgeries_are_corrupted(
        "masked_forged",
        &path,
        &whole,
        &[
            ("a value where one was masked", a_value_put_back),
            ("a justification where one was masked", a_reason_put_back),
            ("a masked marker where a value is", a_marker_elsewhere),
            ("one masked marker twice", a_marker_twice),
        ],
    );
}
