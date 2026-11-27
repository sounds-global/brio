// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Masked values in stage records, through the public interface of
//! `brio_lens`.

mod support;

use std::fs;
use std::path::PathBuf;

use brio_lens::json::{self, Value, json};
use brio_lens::{
    Element, Reason, RecordRefusal, RecordWriter, Stage, WriteError, is_masked, masked,
};

use support::{
    Forge, Loose, assert_forgeries_are_corrupted, build_directory, envelopes, files_under, inputs,
    stage, workspace,
};

/// The source of the module `app::main`, which holds a confidential value.
const SOURCE: &str = "let token: Secret[Text] = \"hunter2\"\n";

/// What a stage produces for [`SOURCE`]: one binding, whose value is masked
/// in its payload and in the reason for a decision about it.
fn binding(stage: &Stage<Loose>) -> Vec<Element> {
    vec![
        Element::new(
            stage.id(&["binding", "token"]),
            json!({ "name": "token", "value": masked("hunter2") }),
        )
        .because(Reason::new(
            "folded constant",
            json!({ "folded_to": masked("hunter2") }),
        )),
    ]
}

/// Writes the record of [`binding`] into the build directory of `test`, and
/// gives its stage and its file.
fn write_binding(test: &str) -> (Stage<Loose>, PathBuf) {
    let stage = stage::<Loose>("app::main");
    let path = RecordWriter::in_build_directory(build_directory(test))
        .write(&stage, &inputs(SOURCE), &Loose(binding))
        .expect("the record is written")
        .expect("Lens is on");
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
        json!({ "name": "token", "value": { "$masked": true } })
    );
    assert_eq!(
        element["reasons"][0]["because"],
        json!({ "folded_to": { "$masked": true } })
    );

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

    assert!(is_masked(&element.payload()["value"]));
    assert!(!is_masked(&element.payload()["name"]));
    assert!(!is_masked(element.payload()));
    let [reason] = element.reasons() else {
        panic!("the element has one reason");
    };
    assert!(is_masked(&reason.because["folded_to"]));
}

/// What the writer says when asked to write `elements`.
fn refusal(test: &str, elements: fn(&Stage<Loose>) -> Vec<Element>) -> RecordRefusal {
    let workspace = workspace(test);
    let error = RecordWriter::in_build_directory(build_directory(test))
        .write(
            &stage::<Loose>("app::main"),
            &inputs(SOURCE),
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
fn a_masked_marker_that_holds_anything_is_not_written() {
    let token = stage::<Loose>("app::main").id(&["binding", "token"]);
    let malformed = RecordRefusal::MalformedMaskedMarker {
        element: token.clone(),
    };

    let in_a_payload = refusal("masked_with_content", |stage| {
        vec![Element::new(
            stage.id(&["binding", "token"]),
            json!({ "value": [{ "$masked": true, "hint": "hunter2" }] }),
        )]
    });
    assert_eq!(in_a_payload, malformed);
    assert_eq!(
        in_a_payload.to_string(),
        format!(
            "the element `{token}` holds an object with `$masked` that is not a masked marker: \
             a masked marker holds nothing else"
        )
    );

    let as_the_mark = refusal("masked_as_content", |stage| {
        vec![Element::new(
            stage.id(&["binding", "token"]),
            json!({ "$masked": "hunter2" }),
        )]
    });
    assert_eq!(as_the_mark, malformed);

    let in_a_reason = refusal("masked_reason_with_content", |stage| {
        vec![
            Element::new(stage.id(&["binding", "token"]), json!("token")).because(Reason::new(
                "folded constant",
                json!({ "folded_to": { "$masked": true, "hint": "hunter2" } }),
            )),
        ]
    });
    assert_eq!(in_a_reason, malformed);
}

#[test]
fn a_record_whose_masked_marker_holds_anything_is_rejected_even_with_a_matching_digest() {
    workspace("masked_forged");
    let (stage, path) = write_binding("masked_forged");
    let whole: Value = json::from_str(&fs::read_to_string(&path).expect("the record can be read"))
        .expect("the record is JSON");
    let id = stage.id(&["binding", "token"]).to_string();

    let in_the_payload: Forge = &|record| {
        record["elements"][&id]["payload"]["value"]["hint"] = json!("hunter2");
    };
    let in_the_reason: Forge = &|record| {
        record["elements"][&id]["reasons"][0]["because"]["folded_to"]["$masked"] = json!("hunter2");
    };
    assert_forgeries_are_corrupted(
        "masked_forged",
        &path,
        &whole,
        &[
            ("a masked marker that holds a hint", in_the_payload),
            ("a masked marker that holds a value", in_the_reason),
        ],
    );
}
