use anki_forge::build::{
    ProjectDeclaredMimeMismatchBehavior, ProjectMediaDiagnosticBehavior, ProjectMediaPolicy,
    ProjectNormalizeOptions,
};
use anki_forge::prelude::*;
use std::path::PathBuf;

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
        report
            .artifact
            .as_ref()
            .map(|artifact| artifact.path.as_path()),
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
fn project_validate_reports_duplicate_stable_ids() {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project
        .add_note(Note::basic("hola", "hello").stable_id("dup"))
        .expect("add first note");
    project
        .add_note(Note::basic("adios", "goodbye").stable_id("dup"))
        .expect("add second note");

    let report = project.validate();

    assert!(report.has_errors());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "AFID.STABLE_ID_DUPLICATE"));
}

#[test]
fn project_validate_reports_blank_stable_id() {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project
        .add_note(Note::basic("hola", "hello").stable_id("   "))
        .expect("add note");

    let report = project.validate();

    assert!(report.has_errors());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "AFID.STABLE_ID_BLANK"
            && diagnostic
                .source
                .as_ref()
                .is_some_and(|source| source.as_str() == "project.notes[0]")
    }));
}

#[test]
fn project_validate_reports_duplicate_notetype_ids_with_index_sources_and_names() {
    let mut project = Project::new("Duplicate Note Types")
        .stable_id("duplicate-notetypes")
        .default_deck("Duplicate Note Types");

    project
        .add_notetype(NoteType::custom("jp-vocab").name("Japanese Vocabulary"))
        .expect("add first note type");
    project
        .add_notetype(NoteType::custom("jp-vocab").name("Japanese Vocab Copy"))
        .expect("add second note type");

    let report = project.validate();
    let duplicate = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.ID_DUPLICATE")
        .expect("duplicate notetype diagnostic");

    assert_eq!(
        duplicate.source.as_ref().map(|source| source.as_str()),
        Some("project.note_types[1]")
    );
    assert!(duplicate.message.contains("Japanese Vocabulary"));
    assert!(duplicate.message.contains("Japanese Vocab Copy"));
}

#[test]
fn project_validate_reports_custom_notetype_id_collision_with_implicit_stock() {
    let mut project = Project::new("Implicit Duplicate")
        .stable_id("implicit-duplicate")
        .default_deck("Implicit Duplicate");
    project
        .add_note(Note::basic("front", "back").stable_id("basic:1"))
        .expect("add stock note");
    project
        .add_notetype(NoteType::custom("basic").name("Custom Basic"))
        .expect("add custom basic notetype");

    let report = project.validate();
    let duplicate = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.ID_DUPLICATE")
        .expect("duplicate notetype diagnostic");

    assert_eq!(
        duplicate.source.as_ref().map(|source| source.as_str()),
        Some("project.note_types[0]")
    );
    assert!(duplicate.message.contains("implicit stock"));
    assert!(duplicate.message.contains("Custom Basic"));
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

    assert_eq!(report.status, "success");
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
fn project_build_maps_missing_media_reference_to_index_source_for_blank_and_duplicate_stable_ids() {
    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    project
        .add_note(
            Note::new("basic")
                .stable_id("")
                .text("Front", "blank")
                .html("Back", "<img src=\"blank.png\">"),
        )
        .expect("add blank note");
    project
        .add_note(
            Note::new("basic")
                .stable_id("dup")
                .text("Front", "dup 1")
                .html("Back", "<img src=\"one.png\">"),
        )
        .expect("add first duplicate note");
    project
        .add_note(
            Note::new("basic")
                .stable_id("dup")
                .text("Front", "dup 2")
                .html("Back", "<img src=\"two.png\">"),
        )
        .expect("add second duplicate note");

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
fn project_lower_maps_duplicate_custom_notetype_sources_to_project_indices_when_stock_is_implicit()
{
    let mut project = Project::new("Shifted Note Types")
        .stable_id("shifted-notetypes")
        .default_deck("Shifted Note Types");
    project
        .add_note(Note::basic("front", "back").stable_id("basic:1"))
        .expect("add stock note");
    project
        .add_notetype(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recognition")
                        .key("recognition")
                        .front("{{Prompt}}")
                        .back("{{Prompt}}")
                        .browser_back("{{Prompt}}"),
                ),
        )
        .expect("add first custom notetype");
    project
        .add_notetype(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recall")
                        .key("recall")
                        .front("{{Prompt}}")
                        .back("{{Prompt}}")
                        .browser_front("{{Prompt}}"),
                ),
        )
        .expect("add second custom notetype");

    let plan = project.lower().expect("lower project");

    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[1].templates[\"Recognition\"].front"),
        Some("project.note_types[0].templates[\"Recognition\"].front")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[1].templates[\"Recognition\"].browser_back"
        ),
        Some("project.note_types[0].templates[\"Recognition\"].browser_back")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[1].css"),
        Some("project.note_types[0].css")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[2].templates[\"Recall\"].back"),
        Some("project.note_types[1].templates[\"Recall\"].back")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[2].templates[\"Recall\"].browser_front"
        ),
        Some("project.note_types[1].templates[\"Recall\"].browser_front")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[2].css"),
        Some("project.note_types[1].css")
    );
}

#[test]
fn project_lower_maps_custom_notetype_stock_collision_sources_to_project_index() {
    let mut project = Project::new("Implicit Duplicate")
        .stable_id("implicit-duplicate")
        .default_deck("Implicit Duplicate");
    project
        .add_note(Note::basic("front", "back").stable_id("basic:1"))
        .expect("add stock note");
    project
        .add_notetype(
            NoteType::custom("basic")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Custom Basic")
                        .front("{{Prompt}}")
                        .back("{{Prompt}}"),
                ),
        )
        .expect("add custom basic notetype");

    let plan = project.lower().expect("lower project");

    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[1].templates[\"Custom Basic\"].front"),
        Some("project.note_types[0].templates[\"Custom Basic\"].front")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[1].css"),
        Some("project.note_types[0].css")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[\"basic\"].templates[\"Custom Basic\"].front"
        ),
        None
    );
}

#[test]
fn project_build_reports_duplicate_notetype_template_and_css_media_sources_by_index() {
    let mut project = Project::new("Duplicate Note Type Media")
        .stable_id("duplicate-notetype-media")
        .default_deck("Duplicate Note Type Media");
    project
        .add_notetype(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recognition")
                        .front(r#"<img src="missing-first-template.png"> {{Prompt}}"#)
                        .back("{{Prompt}}"),
                )
                .css(r#".card { background: url("missing-first-css.png"); }"#),
        )
        .expect("add first custom notetype");
    project
        .add_notetype(
            NoteType::custom("dup")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Recall")
                        .front(r#"<img src="missing-second-template.png"> {{Prompt}}"#)
                        .back("{{Prompt}}"),
                )
                .css(r#".card { background: url("missing-second-css.png"); }"#),
        )
        .expect("add second custom notetype");

    let error = project
        .build(BuildOptions::new().inspect(false))
        .expect_err("duplicate notetype id and missing media references fail build");

    assert!(error
        .report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.ID_DUPLICATE"));

    let sources = error
        .report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .map(|diagnostic| {
            assert_eq!(diagnostic.severity, Severity::Error);
            diagnostic
                .source
                .as_ref()
                .map(|source| source.as_str())
                .expect("missing media diagnostic source")
        })
        .collect::<Vec<_>>();

    assert!(sources.contains(&"project.note_types[0].templates[\"Recognition\"].front"));
    assert!(sources.contains(&"project.note_types[0].css"));
    assert!(sources.contains(&"project.note_types[1].templates[\"Recall\"].front"));
    assert!(sources.contains(&"project.note_types[1].css"));
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

    assert_eq!(report.status, "success");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert!(!codes
        .iter()
        .any(|code| code == "PROJECT.UNSUPPORTED_CUSTOM_NOTETYPE"));
    assert!(!codes
        .iter()
        .any(|code| code == "PROJECT.UNSUPPORTED_NOTE_TYPE"));
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
