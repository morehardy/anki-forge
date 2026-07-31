use anki_forge::prelude::*;
use anki_forge::writer::inspect_apkg;

#[test]
fn external_custom_cloze_bundle_builds_with_templates_css_and_card_ordinals() {
    let bundle = tempfile::tempdir().expect("bundle");
    std::fs::write(
        bundle.path().join("anki-template.yaml"),
        r#"
format_version: template-bundle-v1
note_type:
  id: language-cloze
  name: Language Cloze
  kind: cloze
  cloze_field: text
  fields:
    - key: text
      name: Sentence
      identity: true
      sort: true
    - key: extra
      name: Extra
      optional: true
  templates:
    - key: cloze
      name: Cloze
      front_file: front.html
      back_file: back.html
      browser_front_file: browser-front.html
css_file: style.css
"#,
    )
    .expect("manifest");
    std::fs::write(bundle.path().join("front.html"), "{{cloze:Sentence}}").expect("front");
    std::fs::write(
        bundle.path().join("back.html"),
        "{{cloze:Sentence}}<br>{{Extra}}",
    )
    .expect("back");
    std::fs::write(
        bundle.path().join("browser-front.html"),
        "{{text:Sentence}}",
    )
    .expect("browser front");
    std::fs::write(bundle.path().join("style.css"), ".cloze { color: #c00; }").expect("css");

    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("bundle.apkg");
    let mut project = Project::new("Bundle")
        .stable_id("bundle")
        .default_deck("Bundle");
    project
        .import_template_bundle(bundle.path())
        .expect("import bundle");
    project
        .add_note(
            Note::new("language-cloze")
                .stable_id("bundle:1")
                .text("text", "{{c1::Madrid}} is in {{c2::Spain}}")
                .text("extra", "geography"),
        )
        .expect("add note");

    let report = project.write_apkg(&apkg).expect("write apkg");
    assert_eq!(report.counts.cards, 2);

    let inspected = inspect_apkg(&apkg).expect("inspect");
    assert!(inspected.observations.notetypes.iter().any(|value| {
        value["id"] == "language-cloze"
            && value["kind"] == "cloze"
            && value["css"] == ".cloze { color: #c00; }"
    }));
    assert!(inspected.observations.templates.iter().any(|value| {
        value["notetype_id"] == "language-cloze"
            && value["question_format"] == "{{cloze:Sentence}}"
            && value["answer_format"] == "{{cloze:Sentence}}<br>{{Extra}}"
    }));
    assert!(inspected
        .observations
        .browser_templates
        .iter()
        .any(|value| {
            value["notetype_id"] == "language-cloze"
                && value["browser_question_format"] == "{{text:Sentence}}"
        }));
}

#[test]
fn template_bundle_rejects_parent_directory_paths() {
    let root = tempfile::tempdir().expect("root");
    let bundle_path = root.path().join("bundle");
    std::fs::create_dir(&bundle_path).expect("bundle dir");
    std::fs::write(root.path().join("outside.html"), "{{Front}}").expect("outside");
    std::fs::write(bundle_path.join("back.html"), "{{Front}}").expect("back");
    std::fs::write(
        bundle_path.join("anki-template.yaml"),
        r#"
format_version: template-bundle-v1
note_type:
  id: unsafe
  kind: normal
  fields:
    - key: front
      name: Front
      identity: true
  templates:
    - key: card
      name: Card
      front_file: ../outside.html
      back_file: back.html
"#,
    )
    .expect("manifest");
    let mut project = Project::new("Unsafe");

    let error = project
        .import_template_bundle(&bundle_path)
        .expect_err("unsafe path should fail");

    assert_eq!(error.code(), "TEMPLATE.BUNDLE_PATH_UNSAFE");
}

#[cfg(unix)]
#[test]
fn template_bundle_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let bundle_path = root.path().join("bundle");
    std::fs::create_dir(&bundle_path).expect("bundle dir");
    std::fs::write(root.path().join("outside.html"), "{{Front}}").expect("outside");
    symlink(
        root.path().join("outside.html"),
        bundle_path.join("front.html"),
    )
    .expect("symlink");
    std::fs::write(bundle_path.join("back.html"), "{{Front}}").expect("back");
    std::fs::write(
        bundle_path.join("anki-template.yaml"),
        r#"
format_version: template-bundle-v1
note_type:
  id: unsafe-symlink
  kind: normal
  fields:
    - key: front
      name: Front
      identity: true
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
"#,
    )
    .expect("manifest");
    let mut project = Project::new("Unsafe Symlink");

    let error = project
        .import_template_bundle(&bundle_path)
        .expect_err("symlink escape should fail");

    assert_eq!(error.code(), "TEMPLATE.BUNDLE_PATH_UNSAFE");
}
