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
fn external_normal_bundle_preserves_field_template_and_asset_semantics() {
    let bundle = tempfile::tempdir().expect("bundle");
    std::fs::create_dir(bundle.path().join("assets")).expect("assets dir");
    std::fs::write(
        bundle.path().join("anki-template.yaml"),
        r#"
format_version: template-bundle-v1
note_type:
  id: language-card
  name: Language Card
  kind: normal
  fields:
    - key: prompt
      name: Prompt
      identity: true
      required: true
    - key: extra
      name: Extra
      sort: true
      optional: true
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
      browser_front_file: browser-front.html
      browser_back_file: browser-back.html
      target_deck: Languages::Custom
      generation_rule:
        kind: all
        fields: [prompt]
css_file: style.css
assets:
  - path: assets/icon.svg
    export_as: icon.svg
"#,
    )
    .expect("manifest");
    std::fs::write(bundle.path().join("front.html"), "{{Prompt}}").expect("front");
    std::fs::write(
        bundle.path().join("back.html"),
        "{{Prompt}}<br>{{Extra}}<img src=\"icon.svg\">",
    )
    .expect("back");
    std::fs::write(bundle.path().join("browser-front.html"), "{{Prompt}}").expect("browser front");
    std::fs::write(bundle.path().join("browser-back.html"), "{{Extra}}").expect("browser back");
    std::fs::write(
        bundle.path().join("style.css"),
        ".card { background-image: url(icon.svg); }",
    )
    .expect("css");
    std::fs::write(
        bundle.path().join("assets/icon.svg"),
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"#,
    )
    .expect("asset");

    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("normal-bundle.apkg");
    let mut project = Project::new("Bundle")
        .stable_id("normal-bundle")
        .default_deck("Bundle");
    project
        .import_template_bundle(bundle.path())
        .expect("import bundle");
    project
        .add_note(
            Note::new("language-card")
                .stable_id("bundle:normal:1")
                .text("prompt", "hello"),
        )
        .expect("add note");

    let report = project.write_apkg(&apkg).expect("write apkg");
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 1);

    let inspected = inspect_apkg(&apkg).expect("inspect");
    let extra = inspected
        .observations
        .fields
        .iter()
        .find(|value| value["selector"] == "notetype[id='language-card']::field[Extra]")
        .expect("extra field");
    assert_eq!(extra["sort"], true);
    let template = inspected
        .observations
        .templates
        .iter()
        .find(|value| value["selector"] == "notetype[id='language-card']::template[Card]")
        .expect("template");
    assert_eq!(
        template["generation_requirement"],
        serde_json::json!({"kind": "all", "field_names": ["Prompt"]})
    );
    assert!(inspected
        .observations
        .browser_templates
        .iter()
        .any(|value| {
            value["notetype_id"] == "language-card" && value["browser_answer_format"] == "{{Extra}}"
        }));
    assert!(inspected
        .observations
        .template_target_decks
        .iter()
        .any(|value| {
            value["notetype_id"] == "language-card"
                && value["target_deck_name"] == "Languages::Custom"
        }));
    assert!(inspected
        .observations
        .media
        .iter()
        .any(|value| value["filename"] == "icon.svg"));
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

#[test]
fn template_bundle_rejects_empty_fields_and_normal_cloze_field() {
    for (name, note_type) in [
        (
            "empty-fields",
            r#"
  id: invalid
  kind: normal
  fields: []
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
"#,
        ),
        (
            "normal-cloze-field",
            r#"
  id: invalid
  kind: normal
  cloze_field: text
  fields:
    - key: text
      name: Text
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
"#,
        ),
        (
            "blank-field-key",
            r#"
  id: invalid
  kind: normal
  fields:
    - key: ""
      name: Text
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
"#,
        ),
    ] {
        let bundle = tempfile::tempdir().expect("bundle");
        std::fs::write(bundle.path().join("front.html"), "{{Text}}").expect("front");
        std::fs::write(bundle.path().join("back.html"), "{{Text}}").expect("back");
        std::fs::write(
            bundle.path().join("anki-template.yaml"),
            format!("format_version: template-bundle-v1\nnote_type:\n{note_type}"),
        )
        .expect("manifest");
        let mut project = Project::new(name);

        let error = project
            .import_template_bundle(bundle.path())
            .expect_err("invalid manifest should fail");

        assert_eq!(error.code(), "TEMPLATE.BUNDLE_MANIFEST_INVALID");
    }
}

#[test]
fn template_bundle_rejects_conflicting_field_modes_without_mutating_project() {
    let bundle = tempfile::tempdir().expect("bundle");
    std::fs::write(bundle.path().join("front.html"), "{{Prompt}}").expect("front");
    std::fs::write(bundle.path().join("back.html"), "{{Prompt}}").expect("back");
    std::fs::write(
        bundle.path().join("anki-template.yaml"),
        r#"
format_version: template-bundle-v1
note_type:
  id: conflicting-card
  kind: normal
  fields:
    - key: prompt
      name: Prompt
      required: true
      optional: true
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
"#,
    )
    .expect("manifest");
    let mut project = Project::new("Conflicting field");

    let error = project
        .import_template_bundle(bundle.path())
        .expect_err("conflicting field modes must fail");

    assert_eq!(error.code(), "TEMPLATE.BUNDLE_FIELD_MODE_CONFLICT");
    project
        .add_notetype(
            NoteType::custom("conflicting-card")
                .field(Field::new("Prompt").key("prompt"))
                .template(
                    Template::new("Card")
                        .key("card")
                        .front("{{Prompt}}")
                        .back("{{Prompt}}"),
                ),
        )
        .expect("failed bundle import must not register its note type");
}

#[test]
fn template_bundle_template_errors_report_the_source_file() {
    let bundle = tempfile::tempdir().expect("bundle");
    std::fs::write(bundle.path().join("front.html"), "{{Typo}}").expect("front");
    std::fs::write(bundle.path().join("back.html"), "{{Front}}").expect("back");
    std::fs::write(
        bundle.path().join("anki-template.yaml"),
        r#"
format_version: template-bundle-v1
note_type:
  id: invalid-source
  kind: normal
  fields:
    - key: front
      name: Front
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
"#,
    )
    .expect("manifest");
    let mut project = Project::new("Invalid source");

    let error = project
        .import_template_bundle(bundle.path())
        .expect_err("unknown field should fail");

    assert_eq!(error.code(), "TEMPLATE.RENDER_FIELD_UNKNOWN");
    let expected_path = bundle.path().join("front.html").canonicalize().unwrap();
    assert_eq!(error.path(), Some(expected_path.as_path()));
    assert_eq!(error.byte_offset(), Some(0));
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
