#![cfg(feature = "internal-tools")]

use anki_forge::build::{
    BuildStatus, ProjectDeclaredMimeMismatchBehavior, ProjectMediaDiagnosticBehavior,
    ProjectMediaPolicy, ProjectNormalizeOptions,
};
use anki_forge::diagnostics::{ErrorCode, Severity};
use anki_forge::prelude::*;
use anki_forge::product::ProductDocument;
use std::path::PathBuf;

const IO_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[test]
fn note_basic_constructor_uses_stock_basic_fields() {
    let note = Note::basic("AT&T", "<b>phone</b>").stable_id("basic:att");

    assert_eq!(note.stable_id_ref(), Some("basic:att"));
    assert_eq!(note.note_type_id(), "basic");
    assert_eq!(
        note.rendered_fields().get("Front").map(String::as_str),
        Some("AT&amp;T")
    );
    assert_eq!(
        note.rendered_fields().get("Back").map(String::as_str),
        Some("&lt;b&gt;phone&lt;/b&gt;")
    );
}

#[test]
fn note_html_constructor_preserves_raw_html() {
    let note = Note::new("custom")
        .stable_id("custom:1")
        .text("question", "AT&T")
        .html("answer", "<b>Bell</b>");

    assert_eq!(
        note.rendered_fields().get("question").map(String::as_str),
        Some("AT&amp;T")
    );
    assert_eq!(
        note.rendered_fields().get("answer").map(String::as_str),
        Some("<b>Bell</b>")
    );
}

#[test]
fn note_image_occlusion_builder_accumulates_rects_and_renders_fields() {
    let mut project = Project::new("IO");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");
    let note = Note::image_occlusion(image)
        .stable_id("  heart:io:1  ")
        .mode(IoMode::HideOneGuessOne)
        .rect(10, 20, 30, 40)
        .rect(100, 20, 30, 40)
        .header("Heart")
        .back_extra("Identify it")
        .comments("review carefully")
        .tag("anatomy")
        .build()
        .expect("build image occlusion note");

    assert_eq!(note.note_type_id(), "image_occlusion");
    assert_eq!(note.stable_id_ref(), Some("heart:io:1"));
    let fields = note.rendered_fields();
    assert_eq!(
        fields.get("Occlusion").map(String::as_str),
        Some("{{c1,2::image-occlusion:rect:left=10:top=20:width=30:height=40}}<br>{{c1,2::image-occlusion:rect:left=100:top=20:width=30:height=40}}<br>")
    );
    assert_eq!(
        fields.get("Image").map(String::as_str),
        Some("<img src=\"heart.png\">")
    );
    assert_eq!(fields.get("Header").map(String::as_str), Some("Heart"));
    assert_eq!(
        fields.get("Back Extra").map(String::as_str),
        Some("Identify it")
    );
    assert_eq!(
        fields.get("Comments").map(String::as_str),
        Some("review carefully")
    );
    assert_eq!(note.tags(), ["anatomy".to_string()].as_slice());
}

#[test]
fn note_image_occlusion_builder_requires_stable_id() {
    let mut project = Project::new("IO");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");
    let err = Note::image_occlusion(image)
        .rect(10, 20, 30, 40)
        .build()
        .expect_err("stable id required");

    assert_eq!(
        err.code(),
        anki_forge::diagnostics::ErrorCode::DeckMissingStableId
    );
}

#[test]
fn note_image_occlusion_builder_rejects_blank_stable_id() {
    let mut project = Project::new("IO");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");
    let err = Note::image_occlusion(image)
        .stable_id("   ")
        .rect(10, 20, 30, 40)
        .build()
        .expect_err("blank stable id rejected");

    assert_eq!(
        err.code(),
        anki_forge::diagnostics::ErrorCode::StableIdBlank
    );
}

#[test]
fn note_image_occlusion_builder_rejects_bad_rects() {
    let mut project = Project::new("IO");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");
    let err = Note::image_occlusion(image.clone())
        .stable_id("heart:empty")
        .build()
        .expect_err("empty masks rejected");
    assert_eq!(
        err.code(),
        anki_forge::diagnostics::ErrorCode::ImageOcclusionEmptyMasks
    );

    let err = Note::image_occlusion(image.clone())
        .stable_id("heart:zero")
        .rect(10, 20, 0, 40)
        .build()
        .expect_err("zero width rejected");
    assert_eq!(
        err.code(),
        anki_forge::diagnostics::ErrorCode::ImageOcclusionRectEmpty
    );

    let err = Note::image_occlusion(image)
        .stable_id("heart:dup")
        .rect(10, 20, 30, 40)
        .rect(10, 20, 30, 40)
        .build()
        .expect_err("duplicate rect rejected");
    assert_eq!(
        err.code(),
        anki_forge::diagnostics::ErrorCode::ImageOcclusionRectDuplicate
    );
}

#[test]
fn project_validate_accepts_stock_image_occlusion() {
    let mut project = Project::new("IO").stable_id("io").default_deck("IO");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");

    project
        .add_note(
            Note::new("image_occlusion")
                .stable_id("io:raw")
                .html(
                    "Occlusion",
                    "{{c1::image-occlusion:rect:left=0:top=0:width=1:height=1}}<br>",
                )
                .image("Image", image)
                .text("Header", "")
                .text("Back Extra", "")
                .text("Comments", ""),
        )
        .expect("add raw io note");

    let report = project.validate();
    assert!(!report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "PROJECT.UNSUPPORTED_NOTE_TYPE"));
}

#[test]
fn project_add_notetype_rejects_custom_image_occlusion_reserved_stock_id() {
    let mut project = Project::new("IO").stable_id("io").default_deck("IO");

    let err = project
        .add_notetype(NoteType::custom("image_occlusion").name("Custom IO"))
        .expect_err("image occlusion is a reserved stock note type id");

    assert_eq!(err.diagnostic().code.as_str(), "NOTETYPE.ID_RESERVED");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.note_types[0]")
    );
}

#[test]
fn project_image_occlusion_build_writes_apkg() {
    let root = unique_artifacts_dir("project-image-occlusion-build");
    let mut project = Project::new("Anatomy")
        .stable_id("anatomy")
        .default_deck("Anatomy");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");

    project
        .add_note(
            Note::image_occlusion(image)
                .stable_id("heart:io:1")
                .mode(IoMode::HideAllGuessOne)
                .rect(0, 0, 1, 1)
                .header("Heart")
                .back_extra("Identify it")
                .build()
                .expect("build io note"),
        )
        .expect("add io note");

    let report = project
        .write_apkg(root.join("io.apkg"))
        .expect("write io apkg");

    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 1);
}

#[test]
fn project_image_occlusion_builder_missing_stable_id_fails_before_project_add_note() {
    let mut project = Project::new("IO");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");

    let err = Note::image_occlusion(image)
        .rect(0, 0, 1, 1)
        .build()
        .expect_err("builder rejects missing stable id");

    assert_eq!(
        err.code(),
        anki_forge::diagnostics::ErrorCode::DeckMissingStableId
    );
    assert_eq!(project.validate().diagnostics.len(), 0);
}

#[test]
fn project_raw_image_occlusion_without_stable_id_keeps_generated_fallback() {
    let mut project = Project::new("IO").stable_id("io").default_deck("IO");
    let image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");

    project
        .add_note(
            Note::new("image_occlusion")
                .html(
                    "Occlusion",
                    "{{c1::image-occlusion:rect:left=0:top=0:width=1:height=1}}<br>",
                )
                .image("Image", image)
                .text("Header", "")
                .text("Back Extra", "")
                .text("Comments", ""),
        )
        .expect("add raw io note");

    let plan = project.lower().expect("lower raw io note");
    let note = plan.authoring_document.notes.first().expect("note");
    assert_eq!(note.id, "generated:1");
    assert_eq!(note.notetype_id, "image_occlusion");
}

#[test]
fn project_image_occlusion_cross_project_media_reports_missing_reference() {
    let mut owner = Project::new("Owner");
    let image = owner
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");
    let mut project = Project::new("Target")
        .stable_id("target")
        .default_deck("Target");
    project
        .add_note(
            Note::image_occlusion(image)
                .stable_id("target:io")
                .rect(0, 0, 1, 1)
                .build()
                .expect("build io note"),
        )
        .expect("add io note");

    let err = project
        .normalize()
        .expect_err("unregistered media reference should fail");
    assert!(err.to_string().contains("MEDIA"));
}

#[test]
fn project_image_occlusion_lower_matches_deck_product_shape() {
    let mut deck = Deck::builder("Anatomy").stable_id("anatomy").build();
    let deck_image = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", IO_PNG.to_vec()))
        .expect("deck media");
    deck.image_occlusion()
        .note(deck_image)
        .mode(IoMode::HideAllGuessOne)
        .rect(0, 0, 1, 1)
        .stable_id("heart:io:1")
        .add()
        .expect("deck io");

    let mut project = Project::new("Anatomy")
        .stable_id("anatomy")
        .default_deck("Anatomy");
    let project_image = project
        .media_mut()
        .add_bytes("heart-source.png", IO_PNG.to_vec())
        .expect("media bytes")
        .export_as("heart.png")
        .expect("image export");
    project
        .add_note(
            Note::image_occlusion(project_image)
                .stable_id("heart:io:1")
                .mode(IoMode::HideAllGuessOne)
                .rect(0, 0, 1, 1)
                .build()
                .expect("project io"),
        )
        .expect("add project io");

    let deck_plan = deck
        .into_product_document()
        .expect("deck product")
        .lower()
        .expect("deck lower");
    let project_plan = project.lower().expect("project lower");
    let deck_io_notetype = deck_plan
        .authoring_document
        .notetypes
        .iter()
        .find(|notetype| notetype.original_stock_kind.as_deref() == Some("image_occlusion"))
        .expect("deck image occlusion notetype");
    let project_io_notetype = project_plan
        .authoring_document
        .notetypes
        .iter()
        .find(|notetype| notetype.original_stock_kind.as_deref() == Some("image_occlusion"))
        .expect("project image occlusion notetype");
    assert_eq!(
        deck_io_notetype.original_stock_kind,
        project_io_notetype.original_stock_kind
    );
    assert_eq!(
        deck_plan.authoring_document.notes[0].fields,
        project_plan.authoring_document.notes[0].fields
    );
}

#[test]
fn project_basic_note_writes_apkg_and_returns_report() {
    let root = unique_artifacts_dir("project-basic-build");
    let output = root.join("spanish-a1.apkg");

    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project.write_apkg(&output).expect("write apkg");

    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 0);
    assert_eq!(
        report.artifact.as_ref().map(|artifact| artifact.path()),
        Some(output.as_path())
    );
    assert!(output.exists());
}

#[test]
fn project_normalize_basic_note_returns_normalized_ir() {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let normalized = project.normalize().expect("normalize");

    assert_eq!(normalized.document_id, "spanish-a1");
    assert_eq!(normalized.notes.len(), 1);
    assert_eq!(normalized.notetypes.len(), 1);
    assert_eq!(
        normalized.notes[0].fields.get("Front").map(String::as_str),
        Some("hola")
    );
}

#[test]
fn product_document_builds_directly_without_project_authoring_state() {
    let document = ProductDocument::new("direct-doc")
        .with_basic("basic-main")
        .add_basic_note("basic-main", "note-1", "Default", "front", "back");
    let report = document.build(BuildOptions::new()).unwrap();
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
}

#[test]
fn project_validate_warns_for_auto_derived_custom_field_key() {
    let note_type = NoteType::custom("auto-key")
        .field(Field::new("Expression"))
        .template(
            Template::new("Card 1")
                .front("{{Expression}}")
                .back("{{Expression}}"),
        );
    let mut project = Project::new("Auto Key")
        .stable_id("auto-key")
        .default_deck("Auto Key");
    project.add_notetype(note_type).expect("add note type");

    let report = project.validate();

    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.FIELD_KEY_AUTO_DERIVED"));
}

#[test]
fn project_cloze_card_count_fallback_counts_distinct_ords_when_inspect_disabled() {
    let root = unique_artifacts_dir("project-cloze-no-inspect");
    let mut project = Project::new("Cloze")
        .stable_id("cloze")
        .default_deck("Cloze");
    project
        .add_note(
            Note::cloze("{{c1::Madrid}} is in {{c2::Spain}} and {{c1::Europe}}")
                .stable_id("cloze:1"),
        )
        .expect("add cloze");

    let report = project
        .build(
            BuildOptions::new()
                .output(root.join("cloze.apkg"))
                .inspect(false),
        )
        .expect("build cloze");

    assert_eq!(report.counts.cards, 2);
}

#[test]
fn project_build_preserves_normalization_diagnostics_on_invalid_output() {
    let mut project = Project::new("   ").stable_id("   ").default_deck("Broken");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("blank document id should fail normalization");

    assert_eq!(err.cause, anki_forge::build::BuildFailureCause::Diagnostics);
    assert!(
        err.report
            .diagnostic_codes()
            .iter()
            .any(|code| code == "PHASE2.MISSING_DOCUMENT_ID"),
        "diagnostics: {:?}",
        err.report.diagnostic_codes()
    );
}

#[test]
fn project_build_maps_missing_media_reference_to_stable_note_field_source() {
    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    project
        .add_note(
            Note::new("basic")
                .stable_id("media:missing")
                .text("Front", "front")
                .html("Back", "<img src=\"missing.png\">"),
        )
        .expect("add note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing media reference fails build");
    assert_eq!(error.report.media.references, 1);
    assert_eq!(error.report.media.missing_references, 1);
    assert_eq!(error.report.media.unsafe_references, 0);
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(diagnostic.code.as_str(), "MEDIA.MISSING_REFERENCE");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.notes[\"media:missing\"].fields[\"Back\"]")
    );
    assert!(diagnostic.message.contains("missing.png"));
    assert!(diagnostic.help.as_deref().is_some_and(|help| help
        .contains("project.media_mut().add_file")
        && help.contains("local filename")));

    let media_index = error
        .report
        .diagnostics
        .iter()
        .position(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("media diagnostic index");
    let normalize_index = error
        .report
        .diagnostics
        .iter()
        .position(|diagnostic| diagnostic.code.as_str() == "PROJECT.NORMALIZE_FAILED")
        .expect("normalize failed diagnostic index");
    assert!(
        media_index < normalize_index,
        "specific media diagnostics should precede generic normalization failure"
    );
}

#[test]
fn project_build_maps_missing_inline_style_media_reference_to_note_field_source() {
    let mut project = Project::new("Inline Style Media")
        .stable_id("inline-style-media")
        .default_deck("Inline Style Media");
    project
        .add_note(
            Note::new("basic")
                .stable_id("media:inline-style")
                .text("Front", "front")
                .html(
                    "Back",
                    r#"<div style="background:url(missing-style.png)"></div>"#,
                ),
        )
        .expect("add note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing inline style media reference fails build");
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.notes[\"media:inline-style\"].fields[\"Back\"]")
    );
}

#[test]
fn project_build_uses_normalization_skips_for_non_packaged_media_refs() {
    let mut project = Project::new("Skipped Media")
        .stable_id("skipped-media")
        .default_deck("Skipped Media");
    project
        .add_note(
            Note::new("basic")
                .stable_id("media:skipped")
                .html(
                    "Front",
                    r#"<img src="https://example.test/remote.png"><img src="//cdn.example.test/asset.png">"#,
                )
                .html(
                    "Back",
                    r##"<img src="{{DynamicImage}}"><img src="?v=1"><img src="#fragment">"##,
                ),
        )
        .expect("add note");

    let report = project
        .build(BuildOptions::new().inspect(false))
        .expect("skipped references should not fail writer build");

    assert_eq!(report.status, BuildStatus::Success);
    assert!(!report
        .diagnostic_codes()
        .iter()
        .any(|code| code == "PHASE3.UNRESOLVED_MEDIA_REFERENCE"));
    assert!(!report
        .diagnostic_codes()
        .iter()
        .any(|code| code.starts_with("MEDIA.")));
}

#[test]
fn project_build_missing_and_unsafe_refs_fail_in_normalization_not_writer() {
    let mut missing_project = Project::new("Missing Media")
        .stable_id("missing-media")
        .default_deck("Missing Media");
    missing_project
        .add_note(
            Note::new("basic")
                .stable_id("media:missing")
                .text("Front", "front")
                .html("Back", r#"<img src="missing.png">"#),
        )
        .expect("add missing note");

    let missing_error = missing_project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing reference fails normalization");

    assert!(missing_error
        .report
        .diagnostic_codes()
        .iter()
        .any(|code| code == "MEDIA.MISSING_REFERENCE"));
    assert!(!missing_error
        .report
        .diagnostic_codes()
        .iter()
        .any(|code| code == "PHASE3.UNRESOLVED_MEDIA_REFERENCE"));

    let mut unsafe_project = Project::new("Unsafe Media")
        .stable_id("unsafe-media")
        .default_deck("Unsafe Media");
    unsafe_project
        .add_note(
            Note::new("basic")
                .stable_id("media:unsafe")
                .text("Front", "front")
                .html("Back", r#"<img src="bad%2Fname.png">"#),
        )
        .expect("add unsafe note");

    let unsafe_error = unsafe_project
        .build(BuildOptions::new().inspect(false))
        .expect_err("unsafe reference fails normalization");

    assert!(unsafe_error
        .report
        .diagnostic_codes()
        .iter()
        .any(|code| code == "MEDIA.UNSAFE_REFERENCE"));
    assert!(!unsafe_error
        .report
        .diagnostic_codes()
        .iter()
        .any(|code| code == "PHASE3.UNRESOLVED_MEDIA_REFERENCE"));
}

#[test]
fn media_policy_does_not_demote_missing_or_unsafe_references() {
    let permissive_policy = ProjectMediaPolicy::strict()
        .unused_binding_behavior(ProjectMediaDiagnosticBehavior::Ignore)
        .unknown_mime_behavior(ProjectMediaDiagnosticBehavior::Ignore)
        .declared_mime_mismatch_behavior(ProjectDeclaredMimeMismatchBehavior::Warning);

    let mut missing_project = Project::new("Missing Media")
        .stable_id("policy-missing-media")
        .default_deck("Missing Media");
    missing_project
        .add_note(
            Note::new("basic")
                .stable_id("policy:missing")
                .text("Front", "front")
                .html("Back", r#"<img src="missing.png">"#),
        )
        .expect("add missing note");

    let missing_error =
        missing_project
            .build(BuildOptions::new().inspect(false).normalize_options(
                ProjectNormalizeOptions::strict().media_policy(permissive_policy),
            ))
            .expect_err("missing reference remains an error");
    let missing = missing_error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");
    assert_eq!(missing.severity, Severity::Error);

    let mut unsafe_project = Project::new("Unsafe Media")
        .stable_id("policy-unsafe-media")
        .default_deck("Unsafe Media");
    unsafe_project
        .add_note(
            Note::new("basic")
                .stable_id("policy:unsafe")
                .text("Front", "front")
                .html("Back", r#"<img src="bad%2Fname.png">"#),
        )
        .expect("add unsafe note");

    let unsafe_error =
        unsafe_project
            .build(BuildOptions::new().inspect(false).normalize_options(
                ProjectNormalizeOptions::strict().media_policy(permissive_policy),
            ))
            .expect_err("unsafe reference remains an error");
    let unsafe_reference = unsafe_error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.UNSAFE_REFERENCE")
        .expect("unsafe reference diagnostic");
    assert_eq!(unsafe_reference.severity, Severity::Error);
}

#[test]
fn project_build_maps_unsafe_media_reference_to_product_note_field_source_and_help() {
    let mut project = Project::new("Unsafe Media")
        .stable_id("unsafe-media-source")
        .default_deck("Unsafe Media");
    project
        .add_note(
            Note::new("basic")
                .stable_id("media:unsafe-source")
                .text("Front", "front")
                .html("Back", r#"<img src="bad%2Fname.png">"#),
        )
        .expect("add unsafe note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("unsafe reference fails normalization");
    assert_eq!(error.report.media.references, 1);
    assert_eq!(error.report.media.missing_references, 0);
    assert_eq!(error.report.media.unsafe_references, 1);
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.UNSAFE_REFERENCE")
        .expect("unsafe reference diagnostic");

    assert_eq!(diagnostic.code.as_str(), "MEDIA.UNSAFE_REFERENCE");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.notes[\"media:unsafe-source\"].fields[\"Back\"]")
    );
    assert!(diagnostic.message.contains("bad%2Fname.png"));
    assert!(diagnostic.help.as_deref().is_some_and(|help| {
        help.contains("bare local filename") && help.contains("packaged media")
    }));
}

#[test]
fn project_build_maps_custom_note_field_diagnostic_to_product_field_key() {
    let mut project = Project::new("Custom Media")
        .stable_id("custom-media")
        .default_deck("Custom Media");
    project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expression_key"))
                .template(
                    Template::new("Recognition")
                        .front("{{Expression}}")
                        .back("{{Expression}}"),
                ),
        )
        .expect("add custom notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .html("expression_key", "<img src=\"missing.png\">"),
        )
        .expect("add custom note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing media reference fails build");
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.notes[\"jp:taberu\"].fields[\"expression_key\"]")
    );
}

#[test]
fn project_build_maps_missing_template_media_reference_to_product_template_source() {
    let mut project = Project::new("Template Media")
        .stable_id("template-media")
        .default_deck("Template Media");
    project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expression"))
                .template(
                    Template::new("Recognition")
                        .front(r#"<img src="missing-template.png"> {{Expression}}"#)
                        .back("{{Expression}}"),
                ),
        )
        .expect("add custom notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:template")
                .text("expression", "taberu"),
        )
        .expect("add custom note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing media reference fails build");
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(diagnostic.code.as_str(), "MEDIA.MISSING_REFERENCE");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.note_types[\"jp-vocab\"].templates[\"Recognition\"].front")
    );
    assert!(diagnostic.message.contains("missing-template.png"));
}

#[test]
fn project_build_maps_missing_css_media_reference_to_product_css_source() {
    let mut project = Project::new("CSS Media")
        .stable_id("css-media")
        .default_deck("CSS Media");
    project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expression"))
                .template(
                    Template::new("Recognition")
                        .front("{{Expression}}")
                        .back("{{Expression}}"),
                )
                .css(r#".card { background: url("missing-css.png"); }"#),
        )
        .expect("add custom notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:css")
                .text("expression", "taberu"),
        )
        .expect("add custom note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing media reference fails build");
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(diagnostic.code.as_str(), "MEDIA.MISSING_REFERENCE");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.note_types[\"jp-vocab\"].css")
    );
    assert!(diagnostic.message.contains(r#"url("missing-css.png")"#));
    assert!(diagnostic.message.contains("line 1"));
    assert!(diagnostic.help.as_deref().is_some_and(|help| {
        help.contains("project.media_mut().add_file")
            && help.contains("CSS")
            && help.contains("local filename")
            && help.contains("conservative")
            && help.contains("rule/import")
    }));
}

#[test]
fn project_build_explains_missing_css_import_media_reference() {
    let mut project = Project::new("CSS Import Media")
        .stable_id("css-import-media")
        .default_deck("CSS Import Media");
    project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expression"))
                .template(
                    Template::new("Recognition")
                        .front("{{Expression}}")
                        .back("{{Expression}}"),
                )
                .css(r#"@import url("theme.css");"#),
        )
        .expect("add custom notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:css-import")
                .text("expression", "taberu"),
        )
        .expect("add custom note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing CSS import media reference fails build");
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.note_types[\"jp-vocab\"].css")
    );
    assert!(diagnostic.message.contains(r#"url("theme.css")"#));
    assert!(diagnostic.help.as_deref().is_some_and(|help| {
        help.contains("Register") && help.contains("external") && help.contains("rule/import")
    }));
}

#[test]
fn project_build_maps_missing_media_reference_to_index_source_for_generated_note_ids() {
    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    project
        .add_note(
            Note::new("basic")
                .text("Front", "generated 1")
                .html("Back", "<img src=\"one.png\">"),
        )
        .expect("add first generated note");
    project
        .add_note(
            Note::new("basic")
                .text("Front", "generated 2")
                .html("Back", "<img src=\"two.png\">"),
        )
        .expect("add second generated note");
    project
        .add_note(
            Note::new("basic")
                .text("Front", "generated 3")
                .html("Back", "<img src=\"three.png\">"),
        )
        .expect("add third generated note");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing media references fail build");
    let sources = error
        .report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .filter_map(|diagnostic| diagnostic.source.as_ref().map(|source| source.as_str()))
        .collect::<Vec<_>>();

    assert!(sources.contains(&"project.notes[0].fields[\"Back\"]"));
    assert!(sources.contains(&"project.notes[1].fields[\"Back\"]"));
    assert!(sources.contains(&"project.notes[2].fields[\"Back\"]"));
}

#[test]
fn deck_backed_project_maps_missing_media_reference_to_deck_note_index_source() {
    let mut deck = Deck::builder("Deck Media").stable_id("deck-media").build();
    deck.basic()
        .note("front", "<img src=\"missing.png\">")
        .stable_id("deck:stable")
        .add()
        .expect("add deck note");

    let error = Project::from(deck)
        .build(BuildOptions::new().inspect(false))
        .expect_err("missing media reference fails build");
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.notes[0].fields[\"Back\"]")
    );
}

#[test]
fn deck_backed_project_lower_maps_note_fields_to_deck_note_index_source() {
    let mut deck = Deck::builder("Deck Lower").stable_id("deck-lower").build();
    deck.basic()
        .note("front", "back")
        .stable_id("deck:stable")
        .add()
        .expect("add deck note");

    let plan = Project::from(deck)
        .lower()
        .expect("lower deck-backed project");

    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.notes[\"deck:stable\"].fields[\"Back\"]"),
        Some("project.notes[0].fields[\"Back\"]")
    );
    assert_ne!(
        plan.source_map
            .source_for_authoring_path("authoring.notes[\"deck:stable\"].fields[\"Back\"]"),
        Some("project.notes[\"deck:stable\"].fields[\"Back\"]")
    );
}

#[test]
fn project_build_accepts_custom_inputs_after_lowering_lands() {
    let custom_notetype = NoteType::custom("custom")
        .field(Field::new("Prompt").key("prompt"))
        .template(
            Template::new("Card 1")
                .front("{{Prompt}}")
                .back("{{Prompt}}"),
        );
    let mut project = Project::new("Custom")
        .stable_id("custom")
        .default_deck("Custom");
    project
        .add_notetype(custom_notetype)
        .expect("add custom notetype");
    project
        .add_note(
            Note::new("custom")
                .stable_id("custom:1")
                .text("Prompt", "hola"),
        )
        .expect("add custom note");

    let report = project
        .build(BuildOptions::new().inspect(false))
        .expect("custom inputs build");
    let codes = report.diagnostic_codes();

    assert_eq!(report.status, BuildStatus::Success);
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert!(!codes
        .iter()
        .any(|code| code == "PROJECT.UNSUPPORTED_CUSTOM_NOTETYPE"));
    assert!(!codes
        .iter()
        .any(|code| code == "PROJECT.UNSUPPORTED_NOTE_TYPE"));
}

#[test]
fn project_add_note_rejects_blank_note_type_id() {
    let mut project = Project::new("Blank Note Type")
        .stable_id("blank-note-type")
        .default_deck("Blank Note Type");

    let err = project
        .add_note(Note::new(" \n\t").stable_id("blank:type"))
        .expect_err("blank note type id must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "PROJECT.UNSUPPORTED_NOTE_TYPE"
    );
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[0]")
    );
}

#[test]
fn project_add_note_prioritizes_blank_note_type_id_before_blank_stable_id() {
    let mut project = Project::new("Priority")
        .stable_id("priority")
        .default_deck("Priority");

    let err = project
        .add_note(Note::new(" \n\t").stable_id(" \n\t"))
        .expect_err("blank note type id has higher priority");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "PROJECT.UNSUPPORTED_NOTE_TYPE"
    );
}

#[test]
fn project_add_note_rejects_blank_stable_id_without_mutating_project() {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    let err = project
        .add_note(Note::basic("hola", "hello").stable_id(" \t\n"))
        .expect_err("blank stable id must fail at add-time");

    assert_eq!(err.code(), ErrorCode::StableIdBlank);
    assert_eq!(err.diagnostic().code.as_str(), "AFID.STABLE_ID_BLANK");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[0]")
    );

    let normalized = project
        .normalize()
        .expect("failed add must not mutate project");
    assert_eq!(normalized.notes.len(), 0);
}

#[test]
fn project_add_note_rejects_duplicate_stable_id_without_mutating_project() {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");
    project
        .add_note(Note::basic("hola", "hello").stable_id("dup"))
        .expect("first note");

    let err = project
        .add_note(Note::basic("adios", "goodbye").stable_id("dup"))
        .expect_err("duplicate stable id must fail at add-time");

    assert_eq!(err.code(), ErrorCode::StableIdDuplicate);
    assert_eq!(err.diagnostic().code.as_str(), "AFID.STABLE_ID_DUPLICATE");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[1]")
    );

    let normalized = project
        .normalize()
        .expect("failed add must not mutate project");
    assert_eq!(normalized.notes.len(), 1);
    assert_eq!(normalized.notes[0].id, "dup");
}

#[test]
fn project_add_note_keeps_duplicate_location_after_clone_and_failed_adds() {
    let mut project = Project::new("Indexed IDs").default_deck("Indexed IDs");
    project.add_note(Note::basic("implicit", "first")).unwrap();
    project
        .add_note(Note::basic("explicit", "second").stable_id("existing"))
        .unwrap();
    let mut cloned = project.clone();

    cloned
        .add_note(Note::new("missing").stable_id("available"))
        .expect_err("a rejected note must not reserve its stable ID");
    cloned
        .add_note(Note::basic("valid", "third").stable_id("available"))
        .unwrap();
    let err = cloned
        .add_note(Note::basic("duplicate", "fourth").stable_id("existing"))
        .expect_err("cloning must preserve duplicate detection");
    assert_eq!(err.code(), ErrorCode::StableIdDuplicate);
    assert_eq!(
        err.diagnostic().message,
        "duplicate stable_id 'existing' at project.notes[3]; first definition is project.notes[1]"
    );
    assert_eq!(cloned.normalize().unwrap().notes.len(), 3);

    project
        .add_note(Note::basic("independent", "third").stable_id("available"))
        .expect("a clone must not reserve IDs on the original project");
    assert_eq!(project.normalize().unwrap().notes.len(), 3);
}

#[test]
fn project_add_note_rejects_unsupported_note_type() {
    let mut project = Project::new("Unknown Type")
        .stable_id("unknown-type")
        .default_deck("Unknown Type");

    let err = project
        .add_note(Note::new("missing").stable_id("missing:1"))
        .expect_err("unsupported note type must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "PROJECT.UNSUPPORTED_NOTE_TYPE"
    );
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[0]")
    );
}

#[test]
fn project_add_note_rejects_unknown_stock_field_key_case_sensitively() {
    let mut project = Project::new("Stock Fields")
        .stable_id("stock-fields")
        .default_deck("Stock Fields");

    let err = project
        .add_note(
            Note::new("basic")
                .stable_id("basic:case")
                .text("front", "lowercase is not a Rust Product stock field"),
        )
        .expect_err("unknown stock field key must fail at add-time");

    assert_eq!(err.diagnostic().code.as_str(), "PRODUCT.FIELD_UNKNOWN");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[0].fields[\"front\"]")
    );
}

#[test]
fn project_add_note_rejects_unknown_custom_field_key() {
    let note_type = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .identity(IdentityRecipe::fields(["expr"]));
    let mut project = Project::new("Custom Fields")
        .stable_id("custom-fields")
        .default_deck("Custom Fields");
    project.add_notetype(note_type).expect("add note type");

    let err = project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("missing", "食べる"),
        )
        .expect_err("unknown custom field key must fail at add-time");

    assert_eq!(err.diagnostic().code.as_str(), "PRODUCT.FIELD_UNKNOWN");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[0].fields[\"missing\"]")
    );
}

#[test]
fn project_add_note_rejects_note_identity_override_unknown_field_key() {
    let note_type = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .identity(IdentityRecipe::fields(["expr"]));
    let mut project = Project::new("Identity Override")
        .stable_id("identity-override")
        .default_deck("Identity Override");
    project.add_notetype(note_type).expect("add note type");

    let err = project
        .add_note(
            Note::new("jp-vocab")
                .identity(["missing"])
                .text("expr", "食べる"),
        )
        .expect_err("unknown identity field key must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "PRODUCT.IDENTITY_FIELD_UNKNOWN"
    );
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[0]")
    );
}

#[test]
fn project_add_note_rejects_custom_note_without_stable_id_or_identity_recipe() {
    let note_type = NoteType::custom("jp-vocab").field(Field::new("Expression").key("expr"));
    let mut project = Project::new("Missing Identity")
        .stable_id("missing-identity")
        .default_deck("Missing Identity");
    project
        .add_notetype(note_type)
        .expect("add note type without identity");

    let err = project
        .add_note(Note::new("jp-vocab").text("expr", "食べる"))
        .expect_err("missing note identity must fail at add-time");

    assert_eq!(err.diagnostic().code.as_str(), "PRODUCT.IDENTITY_MISSING");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.notes[0]")
    );
}

#[test]
fn project_add_note_accepts_custom_note_with_explicit_stable_id_without_notetype_identity() {
    let note_type = NoteType::custom("jp-vocab").field(Field::new("Expression").key("expr"));
    let mut project = Project::new("Explicit Identity")
        .stable_id("explicit-identity")
        .default_deck("Explicit Identity");
    project
        .add_notetype(note_type)
        .expect("add note type without identity");

    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "食べる"),
        )
        .expect("explicit stable id should satisfy add-time identity requirement");

    let normalized = project.normalize().expect("normalize");
    assert_eq!(normalized.notes.len(), 1);
    assert_eq!(normalized.notes[0].id, "jp:taberu");
}

fn unique_artifacts_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "anki-forge-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp artifacts dir");
    dir
}

#[test]
fn project_add_notetype_rejects_blank_id_without_mutating_project() {
    let mut project = Project::new("Blank Type")
        .stable_id("blank-type")
        .default_deck("Blank Type");

    let err = project
        .add_notetype(NoteType::custom(" \n\t"))
        .expect_err("blank note type id must fail at add-time");

    assert_eq!(err.diagnostic().code.as_str(), "NOTETYPE.ID_BLANK");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.note_types[0]")
    );

    let normalized = project
        .normalize()
        .expect("failed add must not mutate project");
    assert_eq!(normalized.notetypes.len(), 0);
}

#[test]
fn project_add_notetype_rejects_reserved_stock_id() {
    let mut project = Project::new("Reserved Type")
        .stable_id("reserved-type")
        .default_deck("Reserved Type");

    let err = project
        .add_notetype(NoteType::custom("basic"))
        .expect_err("stock note type ids are reserved");

    assert_eq!(err.diagnostic().code.as_str(), "NOTETYPE.ID_RESERVED");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.note_types[0]")
    );
}

#[test]
fn project_add_notetype_prioritizes_reserved_stock_id_before_field_errors() {
    let mut project = Project::new("Reserved Priority")
        .stable_id("reserved-priority")
        .default_deck("Reserved Priority");

    let err = project
        .add_notetype(
            NoteType::custom("basic")
                .field(Field::new("Front").key("front"))
                .field(Field::new("Duplicate Front").key("front")),
        )
        .expect_err("reserved stock id has higher priority than field errors");

    assert_eq!(err.diagnostic().code.as_str(), "NOTETYPE.ID_RESERVED");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.note_types[0]")
    );
}

#[test]
fn project_add_notetype_rejects_duplicate_custom_id_without_mutating_project() {
    let mut project = Project::new("Duplicate Type")
        .stable_id("duplicate-type")
        .default_deck("Duplicate Type");
    project
        .add_notetype(NoteType::custom("jp-vocab").field(Field::new("Expression").key("expr")))
        .expect("first note type");

    let err = project
        .add_notetype(NoteType::custom("jp-vocab").field(Field::new("Meaning").key("meaning")))
        .expect_err("duplicate note type id must fail at add-time");

    assert_eq!(err.diagnostic().code.as_str(), "NOTETYPE.ID_DUPLICATE");
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.note_types[1]")
    );

    let normalized = project
        .normalize()
        .expect("failed add must not mutate project");
    assert_eq!(normalized.notetypes.len(), 1);
    assert_eq!(normalized.notetypes[0].id, "jp-vocab");
    assert_eq!(normalized.notetypes[0].fields.len(), 1);
    assert_eq!(normalized.notetypes[0].fields[0].name, "Expression");

    let report = project.validate();
    assert!(!report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.ID_DUPLICATE"));
}

#[test]
fn project_add_notetype_rejects_duplicate_field_key() {
    let mut project = Project::new("Duplicate Field Key")
        .stable_id("duplicate-field-key")
        .default_deck("Duplicate Field Key");

    let err = project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expr"))
                .field(Field::new("Prompt").key("expr")),
        )
        .expect_err("duplicate field key must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "NOTETYPE.FIELD_KEY_DUPLICATE"
    );
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.note_types[0].fields[\"Prompt\"]")
    );
}

#[test]
fn project_add_notetype_rejects_duplicate_field_name() {
    let mut project = Project::new("Duplicate Field Name")
        .stable_id("duplicate-field-name")
        .default_deck("Duplicate Field Name");

    let err = project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expr"))
                .field(Field::new("Expression").key("expr2")),
        )
        .expect_err("duplicate field name must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "NOTETYPE.FIELD_NAME_DUPLICATE"
    );
}

#[test]
fn project_add_notetype_rejects_duplicate_sort_field() {
    let mut project = Project::new("Duplicate Sort")
        .stable_id("duplicate-sort")
        .default_deck("Duplicate Sort");

    let err = project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expr").sort())
                .field(Field::new("Reading").key("reading").sort()),
        )
        .expect_err("duplicate sort field must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "NOTETYPE.SORT_FIELD_DUPLICATE"
    );
}

#[test]
fn project_add_notetype_rejects_duplicate_template_key() {
    let mut project = Project::new("Duplicate Template")
        .stable_id("duplicate-template")
        .default_deck("Duplicate Template");

    let err = project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expr"))
                .template(
                    Template::new("Recognition")
                        .key("card")
                        .front("{{Expression}}")
                        .back("{{Expression}}"),
                )
                .template(
                    Template::new("Production")
                        .key("card")
                        .front("{{Expression}}")
                        .back("{{Expression}}"),
                ),
        )
        .expect_err("duplicate template key must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "NOTETYPE.TEMPLATE_KEY_DUPLICATE"
    );
}

#[test]
fn project_add_notetype_rejects_duplicate_template_name_without_mutating_project() {
    let mut project = Project::new("Duplicate Template Name")
        .stable_id("duplicate-template-name")
        .default_deck("Duplicate Template Name");

    let err = project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expr"))
                .template(
                    Template::new("Recognition")
                        .key("recognition")
                        .front("{{Expression}}")
                        .back("{{Expression}}"),
                )
                .template(
                    Template::new("Recognition")
                        .key("production")
                        .front("{{Expression}}")
                        .back("{{Expression}}"),
                ),
        )
        .expect_err("duplicate template name must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "NOTETYPE.TEMPLATE_NAME_DUPLICATE"
    );
    assert_eq!(
        err.diagnostic()
            .source
            .as_ref()
            .map(|source| source.as_str()),
        Some("project.note_types[0].templates[\"Recognition\"]")
    );

    let normalized = project
        .normalize()
        .expect("failed add must not mutate project");
    assert_eq!(normalized.notetypes.len(), 0);
}

#[test]
fn project_add_notetype_rejects_template_rule_unknown_field_key() {
    let mut project = Project::new("Template Unknown Field")
        .stable_id("template-unknown-field")
        .default_deck("Template Unknown Field");

    let err = project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expr"))
                .template(
                    Template::new("Recognition")
                        .key("recognition")
                        .front("{{Expression}}")
                        .back("{{Expression}}")
                        .generate_when(GenerationRule::all(["missing"])),
                ),
        )
        .expect_err("unknown template rule field must fail at add-time");

    assert_eq!(err.diagnostic().code.as_str(), "TEMPLATE.FIELD_UNKNOWN");
}

#[test]
fn project_add_notetype_rejects_identity_recipe_unknown_field_key() {
    let mut project = Project::new("Identity Unknown Field")
        .stable_id("identity-unknown-field")
        .default_deck("Identity Unknown Field");

    let err = project
        .add_notetype(
            NoteType::custom("jp-vocab")
                .field(Field::new("Expression").key("expr"))
                .identity(IdentityRecipe::fields(["missing"])),
        )
        .expect_err("unknown identity recipe field must fail at add-time");

    assert_eq!(
        err.diagnostic().code.as_str(),
        "PRODUCT.IDENTITY_FIELD_UNKNOWN"
    );
}

#[test]
fn project_add_notetype_allows_missing_identity_recipe_and_validate_warns() {
    let mut project = Project::new("Identity Warning")
        .stable_id("identity-warning")
        .default_deck("Identity Warning");

    project
        .add_notetype(NoteType::custom("jp-vocab").field(Field::new("Expression").key("expr")))
        .expect("missing identity recipe is an add-time warning candidate, not an add-time error");

    let report = project.validate();
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.IDENTITY_RECIPE_MISSING")
        .expect("missing identity recipe warning");
    assert_eq!(diagnostic.severity, Severity::Warning);
}
