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
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "MEDIA.MISSING_REFERENCE")
        .expect("missing reference diagnostic");

    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.notes[\"media:missing\"].fields[\"Back\"]")
    );
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
