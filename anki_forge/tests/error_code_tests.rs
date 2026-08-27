#![cfg(feature = "internal-tools")]

use std::path::PathBuf;

use anki_forge::deck::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, Deck, IoMode, MediaSource,
};
use anki_forge::diagnostics::{
    Diagnostic, DiagnosticCode, ErrorCode, ErrorCodeExt, Severity, SourcePath, ValidationReport,
};
use anki_forge::product::{Note, ProductDocument, Project};
use anyhow::Context;
use serde_json::json;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn io_fixture_image_path() -> PathBuf {
    repo_root().join(
        "contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png",
    )
}

#[test]
fn blank_stable_id_error_has_stable_code() {
    let mut deck = Deck::new("Spanish");

    let err = deck
        .basic()
        .note("hola", "hello")
        .stable_id("   ")
        .add()
        .expect_err("blank stable id must fail");

    assert_eq!(err.code(), ErrorCode::StableIdBlank);
    assert_eq!(err.code().as_str(), "DECK.BLANK_STABLE_ID");
}

#[test]
fn reserved_afid_namespace_error_has_stable_code() {
    let mut deck = Deck::new("Spanish");

    let err = deck
        .basic()
        .note("hola", "hello")
        .stable_id("afid:v1:deadbeef")
        .add()
        .expect_err("reserved namespace must fail");

    assert_eq!(err.code(), ErrorCode::ReservedAfidNamespace);
    assert_eq!(err.code().as_str(), "DECK.RESERVED_AFID_NAMESPACE");
}

#[test]
fn empty_identity_selection_error_has_stable_code() {
    let err =
        BasicIdentitySelection::new(std::iter::empty::<BasicIdentityField>()).expect_err("error");

    assert_eq!(err.code(), ErrorCode::IdentityFieldsEmpty);
    assert_eq!(err.code().as_str(), "AFID.IDENTITY_FIELDS_EMPTY");
}

#[test]
fn blank_identity_override_reason_error_has_stable_code() {
    let err = BasicIdentityOverride::new([BasicIdentityField::Front], "   ").expect_err("error");

    assert_eq!(
        err.code(),
        ErrorCode::NoteLevelIdentityOverrideReasonRequired
    );
    assert_eq!(
        err.code().as_str(),
        "AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"
    );
}

#[test]
fn deck_media_unsafe_filename_error_has_stable_code() {
    let mut deck = Deck::new("Anatomy");

    let err = deck
        .media()
        .add(MediaSource::from_bytes("../escape.png", vec![1, 2, 3]))
        .expect_err("path-like media names must fail");

    assert_eq!(err.code(), ErrorCode::MediaUnsafeFilename);
    assert_eq!(err.code().as_str(), "MEDIA.UNSAFE_FILENAME");
}

#[test]
fn image_occlusion_empty_masks_error_has_stable_code() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_file(io_fixture_image_path()))
        .expect("register image");

    let err = deck
        .image_occlusion()
        .note(image)
        .stable_id("io-empty")
        .mode(IoMode::HideOneGuessOne)
        .add()
        .expect_err("empty masks must fail");

    assert_eq!(err.code(), ErrorCode::ImageOcclusionEmptyMasks);
    assert_eq!(err.code().as_str(), "DECK.EMPTY_IO_MASKS");
}

#[test]
fn image_occlusion_rect_out_of_bounds_error_has_stable_code() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_file(io_fixture_image_path()))
        .expect("register image");

    let err = deck
        .image_occlusion()
        .note(image)
        .mode(IoMode::HideOneGuessOne)
        .rect(u32::MAX, 0, 1, 1)
        .add()
        .expect_err("out-of-bounds rect must fail");

    assert_eq!(err.code(), ErrorCode::ImageOcclusionRectOutOfBounds);
    assert_eq!(err.code().as_str(), "AFID.IO_RECT_OUT_OF_BOUNDS");
}

#[test]
fn deck_validate_error_preserves_diagnostic_code() {
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

    let err = deck
        .validate()
        .expect_err("blank stable id must fail validation");

    assert_eq!(err.code(), ErrorCode::StableIdBlank);
    assert_eq!(err.code().as_str(), "DECK.BLANK_STABLE_ID");
}

#[test]
fn product_media_builder_errors_have_stable_codes() {
    let mut project = Project::new("Spanish");

    let err = project
        .media_mut()
        .add_bytes("empty.png", vec![])
        .expect_err("empty media bytes must fail");

    assert_eq!(err.code(), ErrorCode::MediaEmptySource);
    assert_eq!(err.code().as_str(), "MEDIA.EMPTY_SOURCE");
}

#[test]
fn product_lower_media_source_changed_errors_have_stable_codes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = temp.path().join("asset.bin");
    std::fs::write(&source, b"original bytes").expect("write source");

    let mut project = Project::new("Media");
    project
        .media_mut()
        .add_file(&source)
        .expect("register media")
        .export_as("asset.bin")
        .expect("export media");
    std::fs::write(&source, b"changed bytes").expect("change source");

    let err = project
        .lower()
        .expect_err("changed source should fail lower");

    assert_eq!(err.code(), ErrorCode::MediaSourceChanged);
    assert_eq!(err.code().as_str(), "MEDIA.SOURCE_CHANGED");
}

#[test]
fn product_lowering_errors_are_downcastable_from_anyhow() {
    let mut project = Project::from_product_document(ProductDocument::new("doc"));
    project
        .add_note(Note::basic("hola", "hello").stable_id("note-1"))
        .expect("add direct note");

    let err = project
        .lower()
        .expect_err("mixed ProductDocument and direct Project state must fail");

    assert_eq!(err.code(), ErrorCode::ProjectProductDocumentSourceMixed);
    assert_eq!(err.code().as_str(), "PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED");
}

#[test]
fn product_add_error_exposes_stable_code_and_downcasts_from_anyhow() {
    let mut project = Project::new("Spanish");
    let err = project
        .add_note(Note::basic("hola", "hello").stable_id("   "))
        .context("add note failed")
        .expect_err("blank stable id");

    assert_eq!(err.code(), ErrorCode::StableIdBlank);
    assert_eq!(err.code().as_str(), "DECK.BLANK_STABLE_ID");
}

#[test]
fn afid_blank_stable_id_maps_to_stable_id_blank_error_code() {
    let code = anki_forge::diagnostics::DiagnosticCode::new("AFID.STABLE_ID_BLANK");

    assert_eq!(code.error_code(), ErrorCode::StableIdBlank);
    assert_eq!(code.error_code().as_str(), "DECK.BLANK_STABLE_ID");
}

#[test]
fn project_add_error_is_send_sync_static_for_anyhow_downcast() {
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}

    assert_send_sync_static::<anki_forge::product::ProjectAddError>();
}

#[test]
fn product_validation_report_ensure_success_allows_warning_only_reports() {
    let report = ValidationReport {
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("NOTETYPE.IDENTITY_RECIPE_MISSING"),
            severity: Severity::Warning,
            domain: None,
            stage: None,
            message: "custom note type has no identity recipe".into(),
            source: Some(SourcePath::new("project.note_types[\"custom\"]")),
            help: Some("add IdentityRecipe::fields([...])".into()),
        }],
    };

    report
        .ensure_success()
        .expect("warning-only report is successful");
}

#[test]
fn product_validation_report_ensure_success_returns_typed_error() {
    let report = ValidationReport {
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED"),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message: "ProductDocument-backed projects cannot mix direct Project notes".into(),
            source: Some(SourcePath::new("project")),
            help: Some("build either from ProductDocument or direct Project state".into()),
        }],
    };

    let err = report
        .ensure_success()
        .expect_err("error diagnostic must fail validation report");

    assert_eq!(
        err.primary_code().as_str(),
        "PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED"
    );
    assert_eq!(
        err.report()
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count(),
        1
    );
    assert_eq!(err.code().as_str(), "PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED");
}
