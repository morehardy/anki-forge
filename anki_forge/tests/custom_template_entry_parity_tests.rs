use std::path::Path;

use anki_forge::build::BuildOptions;
use anki_forge::prelude::*;
use anki_forge::product::ProductDocument;
use anki_forge::writer::inspect_apkg;

fn semantic_facts(apkg: &Path, notetype_id: &str) -> serde_json::Value {
    let inspected = inspect_apkg(apkg).expect("inspect apkg");
    let notetype = inspected
        .observations
        .notetypes
        .iter()
        .find(|value| value["id"] == notetype_id)
        .expect("note type");
    let fields = inspected
        .observations
        .fields
        .iter()
        .filter(|value| value["notetype_id"] == notetype_id)
        .map(|value| {
            serde_json::json!({
                "name": value["name"],
                "ord": value["ord"],
                "sort": value.get("sort").and_then(serde_json::Value::as_bool).unwrap_or(false),
            })
        })
        .collect::<Vec<_>>();
    let templates = inspected
        .observations
        .templates
        .iter()
        .filter(|value| value["notetype_id"] == notetype_id)
        .map(|value| {
            serde_json::json!({
                "name": value["name"],
                "ord": value["ord"],
                "front": value["question_format"],
                "back": value["answer_format"],
                "generation_requirement": value["generation_requirement"],
            })
        })
        .collect::<Vec<_>>();
    let browsers = inspected
        .observations
        .browser_templates
        .iter()
        .filter(|value| value["notetype_id"] == notetype_id)
        .map(|value| {
            serde_json::json!({
                "name": value["template_name"],
                "front": value["browser_question_format"],
                "back": value["browser_answer_format"],
            })
        })
        .collect::<Vec<_>>();
    let target_decks = inspected
        .observations
        .template_target_decks
        .iter()
        .filter(|value| value["notetype_id"] == notetype_id)
        .map(|value| value["target_deck_name"].clone())
        .collect::<Vec<_>>();
    let notes_and_cards = inspected
        .observations
        .references
        .iter()
        .filter(|value| value["notetype_id"] == notetype_id || value.get("note_id").is_some())
        .map(|value| {
            if let Some(note_id) = value.get("note_id") {
                serde_json::json!({"card_note_id": note_id, "ord": value["ord"]})
            } else {
                serde_json::json!({
                    "note_id": value["id"],
                    "deck": value["deck_name"],
                    "fields": value["fields"],
                })
            }
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "notetype": {
            "id": notetype["id"],
            "kind": notetype["kind"],
            "name": notetype["name"],
            "css": notetype["css"],
        },
        "fields": fields,
        "templates": templates,
        "browsers": browsers,
        "target_decks": target_decks,
        "references": notes_and_cards,
    })
}

fn build_project(project: &Project, output: &Path) {
    project
        .write_apkg(output)
        .expect("write project apkg")
        .ensure_success()
        .expect("successful project build");
}

#[test]
fn custom_normal_is_equivalent_across_project_bundle_and_product_v3() {
    let root = tempfile::tempdir().expect("tempdir");
    let bundle = root.path().join("normal-bundle");
    std::fs::create_dir(&bundle).expect("bundle dir");
    std::fs::write(
        bundle.join("anki-template.yaml"),
        r#"format_version: template-bundle-v1
note_type:
  id: parity-normal
  name: Parity Normal
  kind: normal
  fields:
    - {key: prompt, name: Prompt, identity: true, required: true}
    - {key: extra, name: Extra, sort: true, optional: true}
  templates:
    - key: card
      name: Card
      front_file: front.html
      back_file: back.html
      browser_front_file: browser-front.html
      browser_back_file: browser-back.html
      target_deck: Parity::Target
      generation_rule: {kind: all, fields: [prompt]}
css_file: style.css
"#,
    )
    .expect("manifest");
    for (name, value) in [
        ("front.html", "{{Prompt}}"),
        ("back.html", "{{Prompt}}<br>{{Extra}}"),
        ("browser-front.html", "{{Prompt}}"),
        ("browser-back.html", "{{Extra}}"),
        ("style.css", ".card { color: navy; }"),
    ] {
        std::fs::write(bundle.join(name), value).expect("bundle file");
    }

    let note_type = NoteType::custom("parity-normal")
        .name("Parity Normal")
        .field(Field::new("Prompt").key("prompt").identity().required())
        .field(Field::new("Extra").key("extra").sort().optional())
        .identity(IdentityRecipe::fields(["prompt"]))
        .template(
            Template::new("Card")
                .key("card")
                .front("{{Prompt}}")
                .back("{{Prompt}}<br>{{Extra}}")
                .browser_front("{{Prompt}}")
                .browser_back("{{Extra}}")
                .target_deck("Parity::Target")
                .generate_when(GenerationRule::all(["prompt"])),
        )
        .css(".card { color: navy; }");
    let mut direct = Project::new("Parity").stable_id("parity");
    direct.add_notetype(note_type).expect("direct note type");
    direct
        .add_note(
            Note::new("parity-normal")
                .stable_id("parity:normal:1")
                .deck("Parity::Target")
                .text("prompt", "hello"),
        )
        .expect("direct note");

    let mut bundled = Project::new("Parity").stable_id("parity");
    bundled
        .import_template_bundle(&bundle)
        .expect("bundle import");
    bundled
        .add_note(
            Note::new("parity-normal")
                .stable_id("parity:normal:1")
                .deck("Parity::Target")
                .text("prompt", "hello"),
        )
        .expect("bundle note");

    let product: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "parity",
        "note_types": [{
            "kind": "custom", "note_type_kind": "normal", "id": "parity-normal", "name": "Parity Normal",
            "fields": [
                {"key": "prompt", "name": "Prompt", "identity": true, "required": true},
                {"key": "extra", "name": "Extra", "sort": true}
            ],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "templates": [{
                "key": "card", "name": "Card", "front": "{{Prompt}}", "back": "{{Prompt}}<br>{{Extra}}",
                "browser_front": "{{Prompt}}", "browser_back": "{{Extra}}", "target_deck": "Parity::Target",
                "generation_rule": {"kind": "all", "fields": ["prompt"]}
            }],
            "css": ".card { color: navy; }"
        }],
        "notes": [{
            "kind": "custom", "note_type_id": "parity-normal", "stable_id": "parity:normal:1",
            "deck_name": "Parity::Target", "fields": {"prompt": {"kind": "html", "value": "hello"}}
        }]
    }))
    .expect("product document");

    let direct_apkg = root.path().join("direct.apkg");
    let bundle_apkg = root.path().join("bundle.apkg");
    let product_apkg = root.path().join("product.apkg");
    build_project(&direct, &direct_apkg);
    build_project(&bundled, &bundle_apkg);
    Project::from_product_document(product)
        .build(BuildOptions::new().output(&product_apkg))
        .expect("product build");

    let expected = semantic_facts(&direct_apkg, "parity-normal");
    assert_eq!(semantic_facts(&bundle_apkg, "parity-normal"), expected);
    assert_eq!(semantic_facts(&product_apkg, "parity-normal"), expected);
}

#[test]
fn custom_cloze_is_equivalent_across_project_bundle_and_product_v3() {
    let root = tempfile::tempdir().expect("tempdir");
    let bundle = root.path().join("cloze-bundle");
    std::fs::create_dir(&bundle).expect("bundle dir");
    std::fs::write(
        bundle.join("anki-template.yaml"),
        r#"format_version: template-bundle-v1
note_type:
  id: parity-cloze
  name: Parity Cloze
  kind: cloze
  cloze_field: text
  fields:
    - {key: text, name: Text, identity: true, sort: true, required: true}
    - {key: extra, name: Extra, optional: true}
  templates:
    - {key: cloze, name: Cloze, front_file: front.html, back_file: back.html, target_deck: "Parity::Cloze"}
css_file: style.css
"#,
    )
    .expect("manifest");
    std::fs::write(bundle.join("front.html"), "{{cloze:Text}}").expect("front");
    std::fs::write(bundle.join("back.html"), "{{cloze:Text}}<br>{{Extra}}").expect("back");
    std::fs::write(bundle.join("style.css"), ".cloze { color: maroon; }").expect("css");

    let note_type = NoteType::custom_cloze("parity-cloze", "text")
        .name("Parity Cloze")
        .field(Field::new("Text").key("text").identity().sort().required())
        .field(Field::new("Extra").key("extra").optional())
        .identity(IdentityRecipe::fields(["text"]))
        .template(
            Template::new("Cloze")
                .key("cloze")
                .front("{{cloze:Text}}")
                .back("{{cloze:Text}}<br>{{Extra}}")
                .target_deck("Parity::Cloze"),
        )
        .css(".cloze { color: maroon; }");
    let add_note = || {
        Note::new("parity-cloze")
            .stable_id("parity:cloze:1")
            .deck("Parity::Cloze")
            .text("text", "{{c1::Madrid}} is in {{c2::Spain}}")
    };
    let mut direct = Project::new("Parity").stable_id("parity");
    direct.add_notetype(note_type).expect("direct note type");
    direct.add_note(add_note()).expect("direct note");
    let mut bundled = Project::new("Parity").stable_id("parity");
    bundled
        .import_template_bundle(&bundle)
        .expect("bundle import");
    bundled.add_note(add_note()).expect("bundle note");

    let product: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3", "document_id": "parity",
        "note_types": [{
            "kind": "custom", "note_type_kind": "cloze", "cloze_field": "text",
            "id": "parity-cloze", "name": "Parity Cloze",
            "fields": [
                {"key": "text", "name": "Text", "identity": true, "sort": true, "required": true},
                {"key": "extra", "name": "Extra"}
            ],
            "identity": {"kind": "fields", "fields": ["text"]},
            "templates": [{
                "key": "cloze", "name": "Cloze", "front": "{{cloze:Text}}",
                "back": "{{cloze:Text}}<br>{{Extra}}", "target_deck": "Parity::Cloze"
            }],
            "css": ".cloze { color: maroon; }"
        }],
        "notes": [{
            "kind": "custom", "note_type_id": "parity-cloze", "stable_id": "parity:cloze:1",
            "deck_name": "Parity::Cloze",
            "fields": {"text": {"kind": "html", "value": "{{c1::Madrid}} is in {{c2::Spain}}"}}
        }]
    }))
    .expect("product document");

    let direct_apkg = root.path().join("direct-cloze.apkg");
    let bundle_apkg = root.path().join("bundle-cloze.apkg");
    let product_apkg = root.path().join("product-cloze.apkg");
    build_project(&direct, &direct_apkg);
    build_project(&bundled, &bundle_apkg);
    Project::from_product_document(product)
        .build(BuildOptions::new().output(&product_apkg))
        .expect("product build");

    let expected = semantic_facts(&direct_apkg, "parity-cloze");
    assert_eq!(semantic_facts(&bundle_apkg, "parity-cloze"), expected);
    assert_eq!(semantic_facts(&product_apkg, "parity-cloze"), expected);
}
