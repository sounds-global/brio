// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

//! Masked values in stage records, through the public interface of
//! `brio_lens`.

mod support;

use std::fs;
use std::path::PathBuf;

use brio_lens::json::{self, Value, json};
use brio_lens::{Element, MaskedPart, Reason, RecordRefusal, StableId, Stage};

use support::{
    Forge, Loose, assert_forgeries_are_corrupted, build_directory, envelopes, files_under, inputs,
    refusal, stage, workspace, write_record,
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
        .masked_at(&["value"])
        .because(
            Reason::new("folded constant", json!({ "folded_to": ["hunter2"] }))
                .masked_at(&["folded_to", "0"]),
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

    assert!(element.is_masked(&["value"]));
    assert_eq!(element.masked().collect::<Vec<_>>(), ["/value"]);
    // A value that is absent is not a masked one.
    assert!(!element.is_masked(&["unit"]));
    assert!(!element.is_masked(&["name"]));
    let [reason] = element.reasons() else {
        panic!("the element has one reason");
    };
    assert!(reason.is_masked(&["folded_to", "0"]));
    assert!(!reason.is_masked(&["folded_to"]));
    assert_eq!(reason.masked().collect::<Vec<_>>(), ["/folded_to/0"]);
}

#[test]
fn a_payload_is_free_to_hold_what_looks_like_a_masked_marker() {
    workspace("masked_lookalike");
    let stage = stage::<Loose>("app::main");
    write_record(
        build_directory("masked_lookalike"),
        &stage,
        &inputs(SOURCE),
        &Loose(|stage| {
            vec![Element::new(
                stage.id(&["literal"]),
                json!({ "masked": ["/value"], "$masked": true, "value": null }),
            )]
        }),
    );

    let envelopes = envelopes("masked_lookalike");
    let element = envelopes[0]
        .elements()
        .get(&stage.id(&["literal"]))
        .expect("the element is in the record");
    assert!(!element.is_masked(&["value"]));
    assert_eq!(element.masked().count(), 0);
}

/// The element, the part of it and the masked marker that `refusal` is
/// about.
fn misplaced_marker(refusal: &RecordRefusal) -> (StableId, MaskedPart, String) {
    match refusal {
        RecordRefusal::MisplacedMaskedMarker {
            element,
            part,
            marker,
        } => (element.clone(), *part, marker.as_str().to_owned()),
        other => panic!("the record is refused for another reason: {other}"),
    }
}

#[test]
fn a_masked_marker_where_no_value_was_masked_is_not_written() {
    let token = stage::<Loose>("app::main").id(&["binding", "token"]);
    let misplaced = |part: MaskedPart, marker: &str| (token.clone(), part, marker.to_owned());

    let nowhere = refusal("masked_nowhere", |stage| {
        vec![
            Element::new(
                stage.id(&["binding", "token"]),
                json!({ "value": "hunter2" }),
            )
            .masked_at(&["missing"]),
        ]
    });
    assert_eq!(
        misplaced_marker(&nowhere),
        misplaced(MaskedPart::Payload, "/missing")
    );
    assert_eq!(
        nowhere.to_string(),
        format!(
            "the element `{token}` has a masked marker at `/missing` in its payload, where no \
             value was masked"
        )
    );

    // A place that holds nothing is not one where a value was masked: the
    // value is somewhere else, and would be written.
    let on_nothing = refusal("masked_nothing", |stage| {
        vec![
            Element::new(
                stage.id(&["binding", "token"]),
                json!({ "value": "hunter2", "unit": null }),
            )
            .masked_at(&["unit"]),
        ]
    });
    assert_eq!(
        misplaced_marker(&on_nothing),
        misplaced(MaskedPart::Payload, "/unit")
    );

    let beyond_a_list = refusal("masked_beyond_a_list", |stage| {
        vec![Element::new(stage.id(&["binding", "token"]), json!(["hunter2"])).masked_at(&["01"])]
    });
    assert_eq!(
        misplaced_marker(&beyond_a_list),
        misplaced(MaskedPart::Payload, "/01")
    );

    let put_back = refusal("masked_in_a_reason", |stage| {
        let reason = Reason::new("folded constant", json!({ "folded_to": "hunter2" }))
            .masked_at(&["folded", "to"]);
        vec![Element::new(stage.id(&["binding", "token"]), json!("token")).because(reason)]
    });
    assert_eq!(
        misplaced_marker(&put_back),
        misplaced(MaskedPart::Reason, "/folded/to")
    );
    assert_eq!(
        put_back.to_string(),
        format!(
            "the element `{token}` has a masked marker at `/folded/to` in a reason, where no \
             value was masked"
        )
    );
}

/// The one element of the record that `elements` describes, written into the
/// build directory of `test` and read back.
fn read_back(test: &str, elements: fn(&Stage<Loose>) -> Vec<Element>) -> Element {
    workspace(test);
    let stage = stage::<Loose>("app::main");
    write_record(
        build_directory(test),
        &stage,
        &inputs(SOURCE),
        &Loose(elements),
    );
    let envelopes = envelopes(test);
    let mut read = envelopes[0].elements().iter().cloned();
    match (read.next(), read.next()) {
        (Some(element), None) => element,
        _ => panic!("the record has one element"),
    }
}

#[test]
fn a_value_masked_with_what_it_is_inside_leaves_one_masked_marker() {
    let inside_first = read_back("masked_inside_first", |stage| {
        vec![
            Element::new(
                stage.id(&["account"]),
                json!({ "owner": { "name": "ada", "token": "hunter2" } }),
            )
            .masked_at(&["owner", "token"])
            .masked_at(&["owner"]),
        ]
    });
    let outside_first = read_back("masked_outside_first", |stage| {
        vec![
            Element::new(
                stage.id(&["account"]),
                json!({ "owner": { "name": "ada", "token": "hunter2" } }),
            )
            .masked_at(&["owner"])
            .masked_at(&["owner", "token"])
            .masked_at(&["owner"]),
        ]
    });

    assert_eq!(inside_first, outside_first);
    assert_eq!(inside_first.payload(), &json!({ "owner": null }));
    assert_eq!(inside_first.masked().collect::<Vec<_>>(), ["/owner"]);
    // What is inside a masked value was masked with it.
    assert!(inside_first.is_masked(&["owner"]));
    assert!(inside_first.is_masked(&["owner", "token"]));
}

#[test]
fn a_value_is_masked_whatever_was_asked_of_what_is_inside_it() {
    // Inside the owner, one value is absent and another is not there at
    // all. Neither is refused once the owner itself is masked.
    let element = read_back("masked_around_nothing", |stage| {
        vec![
            Element::new(
                stage.id(&["account"]),
                json!({ "owner": { "name": "ada", "token": null } }),
            )
            .masked_at(&["owner", "token"])
            .masked_at(&["owner", "missing"])
            .masked_at(&["owner"]),
        ]
    });

    assert_eq!(element.payload(), &json!({ "owner": null }));
    assert_eq!(element.masked().collect::<Vec<_>>(), ["/owner"]);
}

#[test]
fn a_value_that_was_masked_before_stays_masked_in_a_record_made_again() {
    let first = read_back("masked_first", binding);
    // What a phase makes from the record it read: the value is gone, and
    // the place says so.
    let again = read_back("masked_again", |stage| {
        vec![
            Element::new(
                stage.id(&["binding", "token"]),
                json!({ "name": "token", "value": null, "unit": null }),
            )
            .still_masked_at(&["value"])
            .because(
                Reason::new("folded constant", json!({ "folded_to": [null] }))
                    .still_masked_at(&["folded_to", "0"]),
            ),
        ]
    });

    assert_eq!(again, first);
    assert!(again.is_masked(&["value"]));
    assert!(!again.is_masked(&["unit"]));
}

#[test]
fn a_value_is_not_written_where_one_is_said_to_be_masked_already() {
    let token = stage::<Loose>("app::main").id(&["binding", "token"]);

    let in_a_payload = refusal("masked_already_payload", |stage| {
        vec![
            Element::new(
                stage.id(&["binding", "token"]),
                json!({ "value": "hunter2" }),
            )
            .still_masked_at(&["value"]),
        ]
    });
    assert_eq!(
        misplaced_marker(&in_a_payload),
        (token.clone(), MaskedPart::Payload, "/value".to_owned())
    );

    let in_a_reason = refusal("masked_already_reason", |stage| {
        let reason = Reason::new("folded constant", json!({ "folded_to": "hunter2" }))
            .still_masked_at(&["folded_to"]);
        vec![Element::new(stage.id(&["binding", "token"]), json!("token")).because(reason)]
    });
    assert_eq!(
        misplaced_marker(&in_a_reason),
        (token, MaskedPart::Reason, "/folded_to".to_owned())
    );
}

#[test]
fn a_place_has_one_spelling_whatever_its_names_are_made_of() {
    let element = read_back("masked_spelling", |stage| {
        vec![
            Element::new(
                stage.id(&["literal"]),
                json!({ "~": "hunter2", "a/b": "hunter2", "": "hunter2", "~1": [0, "hunter2"] }),
            )
            .masked_at(&["~"])
            .masked_at(&["a/b"])
            .masked_at(&[""])
            .masked_at(&["~1", "1"]),
        ]
    });

    assert_eq!(
        element.payload(),
        &json!({ "~": null, "a/b": null, "": null, "~1": [0, null] })
    );
    // The places as JSON Pointers, in the order of their text.
    assert_eq!(
        element.masked().collect::<Vec<_>>(),
        ["/", "/a~1b", "/~0", "/~01/1"]
    );
    assert!(element.is_masked(&["~"]));
    assert!(element.is_masked(&["a/b"]));
    assert!(element.is_masked(&["~1", "1"]));
    assert!(!element.is_masked(&["~1", "0"]));
}

#[test]
fn the_whole_of_a_payload_can_be_masked() {
    let element = read_back("masked_whole", |stage| {
        vec![Element::new(stage.id(&["literal"]), json!("hunter2")).masked_at(&[])]
    });

    assert_eq!(element.payload(), &Value::Null);
    assert_eq!(element.masked().collect::<Vec<_>>(), [""]);
    assert!(element.is_masked(&[]));
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
    let a_marker_inside_another: Forge = &|record| {
        record["elements"][&id]["payload"] = json!(null);
        record["elements"][&id]["masked"] = json!(["", "/value"]);
    };
    let a_marker_that_is_no_pointer: Forge = &|record| {
        record["elements"][&id]["payload"] = json!({ "~": null });
        record["elements"][&id]["masked"] = json!(["/~"]);
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
            ("a masked marker inside another", a_marker_inside_another),
            (
                "a masked marker that is no JSON Pointer",
                a_marker_that_is_no_pointer,
            ),
        ],
    );
}
