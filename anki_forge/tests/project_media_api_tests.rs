use std::path::PathBuf;

use anki_forge::prelude::*;

const MP3: &[u8] = b"fake-mp3-bytes-for-package-test";
const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[test]
fn product_media_helpers_render_anki_compatible_content() {
    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    let audio = project
        .media_mut()
        .add_bytes("raw-audio.bin", MP3.to_vec())
        .expect("bytes media")
        .export_as("hola.mp3")
        .expect("audio media");
    let image = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG.to_vec())
        .expect("bytes media")
        .export_as("chart.png")
        .expect("image media");

    let note = Note::new("basic")
        .stable_id("media:1")
        .text("Front", "hola")
        .sound("Back", audio.clone())
        .image("Picture", image.clone());

    assert_eq!(audio.sound().render(), "[sound:hola.mp3]");
    assert_eq!(image.image().render(), "<img src=\"chart.png\">");
    assert_eq!(
        note.rendered_fields().get("Back").map(String::as_str),
        Some("[sound:hola.mp3]")
    );
    assert_eq!(
        note.rendered_fields().get("Picture").map(String::as_str),
        Some("<img src=\"chart.png\">")
    );
}

#[test]
fn project_build_packages_product_media_and_reports_count() {
    let root = unique_artifacts_dir("project-media");
    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    let audio = project
        .media_mut()
        .add_bytes("hola-source.mp3", MP3.to_vec())
        .expect("bytes media")
        .export_as("hola.mp3")
        .expect("audio media");

    project
        .add_note(
            Note::basic("hola", "hello")
                .stable_id("media:hola")
                .sound("Back", audio),
        )
        .expect("add note");

    let report = project
        .write_apkg(root.join("media.apkg"))
        .expect("write apkg");

    report.ensure_success().expect("successful media build");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.media, 1);
}

#[test]
fn project_build_uses_export_name_for_declared_mime() {
    let root = unique_artifacts_dir("project-media-mime");
    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    let image = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG.to_vec())
        .expect("bytes media")
        .export_as("chart.png")
        .expect("image media");

    project
        .add_note(
            Note::basic("chart", "")
                .stable_id("media:chart")
                .image("Back", image),
        )
        .expect("add note");

    let report = project
        .write_apkg(root.join("media-mime.apkg"))
        .expect("write apkg");

    report.ensure_success().expect("successful media build");
    assert_eq!(report.counts.media, 1);
}

#[test]
fn project_build_keeps_file_media_path_backed_for_large_sources() {
    let root = unique_artifacts_dir("project-media-file");
    let source = root.join("large-source.bin");
    std::fs::write(&source, vec![b'a'; 70 * 1024]).expect("write large media");

    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    let media = project
        .media_mut()
        .add_file(&source)
        .expect("file media")
        .export_as("large.bin")
        .expect("export file media");

    project
        .add_note(
            Note::basic("large", "")
                .stable_id("media:large")
                .sound("Back", media),
        )
        .expect("add note");

    let report = project
        .write_apkg(root.join("media-file.apkg"))
        .expect("write apkg");

    report.ensure_success().expect("successful media build");
    assert_eq!(report.counts.media, 1);
}

#[test]
fn media_export_names_reject_helper_unsafe_characters() {
    let mut project = Project::new("Media");

    let image_error = project
        .media_mut()
        .add_bytes("raw-image.png", PNG.to_vec())
        .expect("bytes media")
        .export_as("bad\"name.png")
        .expect_err("quotes break img src helpers");
    assert!(image_error.to_string().contains("MEDIA.EXPORT_NAME"));

    let sound_error = project
        .media_mut()
        .add_bytes("raw-audio.mp3", MP3.to_vec())
        .expect("bytes media")
        .export_as("bad].mp3")
        .expect_err("closing bracket breaks sound helpers");
    assert!(sound_error.to_string().contains("MEDIA.EXPORT_NAME"));
}

#[test]
fn add_bytes_rejects_oversized_inline_payload_immediately() {
    let mut project = Project::new("Media");

    let error = project
        .media_mut()
        .add_bytes("too-big.bin", vec![b'x'; 64 * 1024 + 1])
        .expect_err("inline bytes above strict limit are rejected");

    assert!(error.to_string().contains("MEDIA.INLINE_TOO_LARGE"));
    assert_eq!(
        project
            .lower()
            .expect("lower product")
            .authoring_document
            .media
            .len(),
        0,
        "oversized add_bytes must not create a pending registry entry"
    );
}

#[test]
fn add_bytes_and_add_file_reject_zero_byte_sources() {
    let root = unique_artifacts_dir("project-media-empty");
    let empty_file = root.join("empty.bin");
    std::fs::write(&empty_file, []).expect("write empty media");

    let mut project = Project::new("Media");
    let bytes_error = project
        .media_mut()
        .add_bytes("empty-bytes.bin", Vec::new())
        .expect_err("empty bytes are rejected");
    assert!(bytes_error.to_string().contains("MEDIA.EMPTY_SOURCE"));

    let file_error = project
        .media_mut()
        .add_file(&empty_file)
        .expect_err("empty files are rejected");
    assert!(file_error.to_string().contains("MEDIA.EMPTY_SOURCE"));
}

#[test]
fn export_as_reuses_same_filename_for_same_content_and_conflicts_for_different_content() {
    let mut project = Project::new("Media");

    let first = project
        .media_mut()
        .add_bytes("first-audio", MP3.to_vec())
        .expect("bytes media")
        .export_as("sound.mp3")
        .expect("first export");
    let second = project
        .media_mut()
        .add_bytes("same-audio", MP3.to_vec())
        .expect("bytes media")
        .export_as("sound.mp3")
        .expect("same content same filename is reused");

    assert_eq!(first.filename(), "sound.mp3");
    assert_eq!(second.filename(), "sound.mp3");
    assert_eq!(
        project
            .lower()
            .expect("lower product")
            .authoring_document
            .media
            .len(),
        1
    );

    let conflict = project
        .media_mut()
        .add_bytes("different-audio", b"different bytes".to_vec())
        .expect("bytes media")
        .export_as("sound.mp3")
        .expect_err("same filename with different content conflicts");
    assert!(conflict
        .to_string()
        .contains("MEDIA.DUPLICATE_FILENAME_CONFLICT"));
}

#[test]
fn same_content_can_export_under_different_filenames() {
    let mut project = Project::new("Media");

    project
        .media_mut()
        .add_bytes("first", MP3.to_vec())
        .expect("bytes media")
        .export_as("one.mp3")
        .expect("first export");
    project
        .media_mut()
        .add_bytes("second", MP3.to_vec())
        .expect("bytes media")
        .export_as("two.mp3")
        .expect("second export");

    let lowered = project.lower().expect("lower product");
    assert_eq!(lowered.authoring_document.media.len(), 2);
}

#[test]
fn add_bytes_validates_source_label_without_filename_rules() {
    let mut project = Project::new("Media");

    let empty = project
        .media_mut()
        .add_bytes("   ", MP3.to_vec())
        .expect_err("blank source label rejected");
    assert!(empty.to_string().contains("MEDIA.INVALID_SOURCE_LABEL"));

    let control = project
        .media_mut()
        .add_bytes("bad\nlabel", MP3.to_vec())
        .expect_err("control characters rejected");
    assert!(control.to_string().contains("MEDIA.INVALID_SOURCE_LABEL"));

    project
        .media_mut()
        .add_bytes("logical source label with spaces", MP3.to_vec())
        .expect("source label is not a helper-safe filename")
        .export_as("safe.mp3")
        .expect("safe export filename");
}

#[test]
fn failed_export_does_not_mutate_registry() {
    let mut project = Project::new("Media");

    project
        .media_mut()
        .add_bytes("raw-image", PNG.to_vec())
        .expect("bytes media")
        .export_as("../chart.png")
        .expect_err("unsafe export name fails");

    let lowered = project.lower().expect("lower product");
    assert_eq!(lowered.authoring_document.media.len(), 0);
}

#[test]
fn product_build_reports_file_source_changed_after_registration() {
    let root = unique_artifacts_dir("project-media-source-changed");
    let source = root.join("source.bin");
    std::fs::write(&source, b"original bytes").expect("write source");

    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    let media = project
        .media_mut()
        .add_file(&source)
        .expect("file media")
        .export_as("source.bin")
        .expect("export file media");
    std::fs::write(&source, b"changed bytes").expect("change source");

    project
        .add_note(
            Note::basic("source", "")
                .stable_id("media:source")
                .sound("Back", media),
        )
        .expect("add note");

    let error = project
        .write_apkg(root.join("source-changed.apkg"))
        .expect_err("changed source fails build");
    assert!(error
        .report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "MEDIA.SOURCE_CHANGED"));
}

#[test]
fn product_build_reports_file_source_changed_when_source_becomes_empty() {
    let root = unique_artifacts_dir("project-media-source-empty-change");
    let source = root.join("source.bin");
    std::fs::write(&source, b"original bytes").expect("write source");

    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    let media = project
        .media_mut()
        .add_file(&source)
        .expect("file media")
        .export_as("source.bin")
        .expect("export file media");
    std::fs::write(&source, []).expect("empty source");

    project
        .add_note(
            Note::basic("source", "")
                .stable_id("media:source")
                .sound("Back", media),
        )
        .expect("add note");

    let error = project
        .write_apkg(root.join("source-empty-change.apkg"))
        .expect_err("emptied source fails build");
    assert!(error
        .report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "MEDIA.SOURCE_CHANGED"));
}

#[test]
fn product_build_reports_file_source_missing_after_registration() {
    let root = unique_artifacts_dir("project-media-source-missing");
    let source = root.join("source.bin");
    std::fs::write(&source, b"original bytes").expect("write source");

    let mut project = Project::new("Media")
        .stable_id("media")
        .default_deck("Media");
    let media = project
        .media_mut()
        .add_file(&source)
        .expect("file media")
        .export_as("source.bin")
        .expect("export file media");
    std::fs::remove_file(&source).expect("delete source");

    project
        .add_note(
            Note::basic("source", "")
                .stable_id("media:source")
                .sound("Back", media),
        )
        .expect("add note");

    let error = project
        .write_apkg(root.join("source-missing.apkg"))
        .expect_err("missing source fails build");
    assert!(error
        .report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "MEDIA.SOURCE_MISSING"));
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
