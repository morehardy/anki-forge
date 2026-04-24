use anki_forge::{
    BasicIdentityField, BasicIdentityOverride, Deck, IoMode, MediaSource, ValidationCode,
};
use serde_json::json;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn io_fixture_image_path() -> PathBuf {
    repo_root().join(
        "contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png",
    )
}

#[test]
fn add_basic_infers_afid_and_validate_report_does_not_warn() {
    let mut deck = Deck::new("Spanish");
    deck.add_basic("hola", "hello").expect("add basic note");

    assert!(deck.notes()[0].id().starts_with("afid:v1:"));

    let report = deck.validate_report().expect("validation report");
    assert!(!report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::MissingStableId));
}

#[test]
fn validation_code_catalog_includes_note_identity_codes() {
    let codes = [
        ValidationCode::MissingStableId,
        ValidationCode::DuplicateStableId,
        ValidationCode::BlankStableId,
        ValidationCode::EmptyIoMasks,
        ValidationCode::UnknownMediaRef,
        ValidationCode::NoteLevelIdentityOverrideUsed,
        ValidationCode::IdentityDuplicatePayload,
        ValidationCode::IdentityCollision,
        ValidationCode::StableIdDuplicate,
    ];

    assert_eq!(codes.len(), 9);
}

#[test]
fn note_level_override_emits_warning_diagnostic() {
    let override_cfg = BasicIdentityOverride::new(
        [BasicIdentityField::Front, BasicIdentityField::Back],
        "sense",
    )
    .expect("override");
    let mut deck = Deck::new("Spanish");
    deck.basic()
        .note("hola", "hello")
        .identity_override(override_cfg)
        .add()
        .expect("add note with identity override");

    let report = deck.validate_report().expect("validation report");
    assert!(!report.has_errors());
    assert!(report.diagnostics().iter().any(|item| {
        item.code == ValidationCode::NoteLevelIdentityOverrideUsed && item.severity == "warning"
    }));
}

#[test]
fn legacy_generated_note_still_deserializes_and_reports_warning() {
    let deck: Deck = serde_json::from_value(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "generated:Spanish:1",
                    "stable_id": null,
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": true
                }
            }
        ],
        "next_generated_note_id": 2,
        "media": {}
    }))
    .expect("legacy generated deck should deserialize");

    let report = deck.validate_report().expect("validation report");
    assert!(!report.has_errors());
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::MissingStableId && item.severity == "warning"));
}

#[test]
fn blank_explicit_stable_id_fails_at_add_time() {
    let mut deck = Deck::new("Spanish");

    let err = deck
        .basic()
        .note("hola", "hello")
        .stable_id("   ")
        .add()
        .expect_err("blank stable id must fail");

    assert!(err.to_string().contains("stable_id"));
}

#[test]
fn cloze_lane_sugar_adds_note_with_metadata() {
    let mut deck = Deck::new("Spanish");

    deck.cloze()
        .note("A {{c1::cloze}} card")
        .stable_id("cloze-1")
        .extra("extra context")
        .tags(["study", "spanish"])
        .add()
        .expect("add cloze note");

    let note = &deck.notes()[0];
    assert_eq!(note.id(), "cloze-1");
    assert!(matches!(note, anki_forge::DeckNote::Cloze(_)));
}

#[test]
fn image_occlusion_lane_requires_rect_at_add_time_and_accepts_rects() {
    let mut deck = Deck::new("Spanish");
    let image = deck
        .media()
        .add(MediaSource::from_bytes("image.png", vec![1, 2, 3]))
        .expect("register image");

    let err = deck
        .image_occlusion()
        .note(image.clone())
        .mode(IoMode::HideOneGuessOne)
        .add()
        .expect_err("image occlusion without rect must fail");
    assert!(err.to_string().contains("rect"));

    let image = deck
        .media()
        .add(MediaSource::from_file(io_fixture_image_path()))
        .expect("register valid image");
    deck.image_occlusion()
        .note(image)
        .mode(IoMode::HideOneGuessOne)
        .rect(10, 20, 30, 40)
        .header("header")
        .back_extra("back extra")
        .comments("comments")
        .add()
        .expect("image occlusion with rect");

    assert_eq!(deck.notes().len(), 1);
    assert!(deck.notes()[0].id().starts_with("afid:v1:"));
    assert!(matches!(
        deck.notes()[0],
        anki_forge::DeckNote::ImageOcclusion(_)
    ));
}

#[test]
fn deserializing_duplicate_stable_id_even_when_one_note_is_generated_fails() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "generated:Spanish:1",
                    "stable_id": null,
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": true
                }
            },
            {
                "Basic": {
                    "id": "generated:Spanish:1",
                    "stable_id": "generated:Spanish:1",
                    "front": "adios",
                    "back": "bye",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 3,
        "media": {}
    }))
    .expect_err("duplicate stable ids should fail at load time");

    assert!(err
        .to_string()
        .contains("AFID.STABLE_ID_DUPLICATE: generated:Spanish:1"));
}

#[test]
fn validate_report_detects_blank_stable_id_from_deserialized_note() {
    let deck: Deck = serde_json::from_value(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "",
                    "stable_id": "   ",
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect("deserialize deck");

    let report = deck.validate_report().expect("validation report");
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::BlankStableId));
}

#[test]
fn validate_report_detects_empty_image_occlusion_geometry_and_validate_errors() {
    let deck: Deck = serde_json::from_value(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "ImageOcclusion": {
                    "id": "io-1",
                    "stable_id": "io-1",
                    "image": "image.png",
                    "mode": "HideAllGuessOne",
                    "rects": [],
                    "header": "",
                    "back_extra": "",
                    "comments": "",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect("deserialize deck");

    let report = deck.validate_report().expect("validation report");
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::EmptyIoMasks));
    assert!(deck.validate().is_err());
}

#[test]
fn validate_report_detects_unknown_media_ref_from_deserialized_note() {
    let deck: Deck = serde_json::from_value(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "ImageOcclusion": {
                    "id": "io-1",
                    "stable_id": "io-1",
                    "image": "missing.png",
                    "mode": "HideAllGuessOne",
                    "rects": [{"x": 1, "y": 2, "width": 3, "height": 4}],
                    "header": "",
                    "back_extra": "",
                    "comments": "",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect("deserialize deck");

    let report = deck.validate_report().expect("validation report");
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::UnknownMediaRef));
    assert!(deck.validate().is_err());
}

#[test]
fn add_time_rejects_forged_unknown_media_ref() {
    let mut deck = Deck::new("Anatomy");
    let image: anki_forge::MediaRef =
        serde_json::from_value(json!("missing.png")).expect("forge media ref");

    let err = deck
        .image_occlusion()
        .note(image)
        .stable_id("io-unknown")
        .rect(1, 2, 3, 4)
        .add()
        .expect_err("unknown media ref must fail at add time");

    assert!(err.to_string().contains("unknown media"));
}

#[test]
fn media_registry_reuses_same_name_same_content_and_rejects_conflicts() {
    let mut deck = Deck::new("Anatomy");

    let first = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("first registration");
    let second = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("same-bytes registration");

    assert_eq!(first, second);

    let err = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![9, 9, 9]))
        .expect_err("different bytes must fail");

    assert!(err.to_string().contains("heart.png"));
}

#[test]
fn media_registry_rejects_pathlike_names_at_add_time() {
    let mut deck = Deck::new("Anatomy");

    for invalid_name in ["../escape.png", "nested/escape.png", "/tmp/escape.png"] {
        let err = deck
            .media()
            .add(MediaSource::from_bytes(invalid_name, vec![1, 2, 3]))
            .expect_err("path-like media names must fail");

        assert!(
            err.to_string().contains("media filename"),
            "unexpected error for {invalid_name}: {err}"
        );
    }
}

#[test]
fn image_occlusion_lane_accepts_tags() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("register media");

    deck.image_occlusion()
        .note(image)
        .stable_id("io-tags")
        .rect(1, 2, 3, 4)
        .tags(["anatomy", "io"])
        .add()
        .expect("io note with tags");

    let note_json = serde_json::to_value(&deck.notes()[0]).expect("serialize note");
    assert_eq!(
        note_json["ImageOcclusion"]["tags"],
        json!(["anatomy", "io"])
    );
}

#[test]
fn image_occlusion_without_rects_fails_at_add_time() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("register media");

    let err = deck
        .image_occlusion()
        .note(image)
        .stable_id("io-1")
        .add()
        .expect_err("io note without rects must fail");

    assert!(err.to_string().contains("rect"));
}
