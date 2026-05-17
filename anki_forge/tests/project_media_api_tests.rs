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
        .export_as("hola.mp3")
        .expect("audio media");
    let image = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG.to_vec())
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
        .export_as("bad\"name.png")
        .expect_err("quotes break img src helpers");
    assert!(image_error.to_string().contains("MEDIA.EXPORT_NAME"));

    let sound_error = project
        .media_mut()
        .add_bytes("raw-audio.mp3", MP3.to_vec())
        .export_as("bad].mp3")
        .expect_err("closing bracket breaks sound helpers");
    assert!(sound_error.to_string().contains("MEDIA.EXPORT_NAME"));
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
