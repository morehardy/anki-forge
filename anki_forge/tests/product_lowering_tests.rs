use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::prelude::{Field, IdentityRecipe, Note, NoteType, Project, Template};
use anki_forge::product::model::{CustomField, CustomNote, CustomNoteType, CustomTemplate};
use anki_forge::product::ProductDocument;
use anki_forge::update_safety::model::NoteIdentityEntry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[test]
fn basic_product_document_lowers_to_authoring_ir_with_mapping_evidence() {
    let plan = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .add_basic_note_with_tags(
            "basic-main",
            "note-1",
            "Default",
            "front".to_string(),
            "back".to_string(),
            ["demo"],
        )
        .lower()
        .expect("lower should succeed");

    assert_eq!(plan.authoring_document.kind, "authoring-ir");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.original_stock_kind.as_deref(), Some("basic"));

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(note.fields.get("Front").map(String::as_str), Some("front"));
    assert_eq!(note.tags, vec!["demo"]);

    assert_eq!(plan.mappings.len(), 2);
    assert!(plan.product_diagnostics.is_empty());
    assert!(plan.lowering_diagnostics.is_empty());
}

#[test]
fn cloze_and_image_occlusion_lanes_lower_to_stock_compatible_authoring_shapes() {
    let cloze_text = "A {{c1::cloze}} card";
    let plan = ProductDocument::new("cloze-doc")
        .with_cloze("cloze-main")
        .add_cloze_note_with_tags(
            "cloze-main",
            "note-1",
            "Default",
            cloze_text,
            "extra",
            ["tagged"],
        )
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "cloze");
    assert_eq!(notetype.original_stock_kind.as_deref(), Some("cloze"));

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(
        note.fields.get("Text").map(String::as_str),
        Some(cloze_text)
    );
    assert_eq!(note.tags, vec!["tagged"]);

    let plan = ProductDocument::new("io-doc")
        .with_image_occlusion("io-main")
        .add_image_occlusion_note_with_tags(
            "io-main",
            "note-1",
            "Default",
            "occlusion",
            "image.png",
            "Header",
            "back_extra",
            "comments",
            ["image-tag"],
        )
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "cloze");
    assert_eq!(
        notetype.original_stock_kind.as_deref(),
        Some("image_occlusion")
    );

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(
        note.fields.get("Header").map(String::as_str),
        Some("Header")
    );
    assert_eq!(note.tags, vec!["image-tag"]);
}

#[test]
fn image_occlusion_missing_image_emits_product_diagnostic() {
    let err = ProductDocument::new("io-doc")
        .with_image_occlusion("io-main")
        .add_image_occlusion_note_with_tags(
            "io-main",
            "note-1",
            "Default",
            "occlusion",
            "",
            "Header",
            "back_extra",
            "comments",
            std::iter::empty::<&str>(),
        )
        .lower()
        .expect_err("lower should fail");

    assert!(err
        .product_diagnostics
        .iter()
        .any(|d| d.code == "PHASE5A.IO_IMAGE_REQUIRED"));
}

#[test]
fn custom_escape_hatch_lowers_to_explicit_authoring_normal_notetype_shape() {
    let plan = ProductDocument::new("custom-doc")
        .with_custom_notetype(CustomNoteType {
            id: "custom-main".into(),
            name: Some("Custom Normal".into()),
            fields: vec![
                CustomField {
                    name: "Front".into(),
                    key: None,
                },
                CustomField {
                    name: "Back".into(),
                    key: None,
                },
            ],
            templates: vec![CustomTemplate {
                name: "Card 1".into(),
                key: None,
                question_format: "{{Front}}".into(),
                answer_format: "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}".into(),
                generation_rule: None,
            }],
            css: Some(".card { color: red; }".into()),
        })
        .add_custom_note(CustomNote {
            id: "note-1".into(),
            note_type_id: "custom-main".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                ("Front".into(), "front".into()),
                ("Back".into(), "back".into()),
            ]),
            tags: vec![],
        })
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.css.as_deref(), Some(".card { color: red; }"));

    let fields = notetype.fields.as_ref().expect("explicit custom fields");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "Front");
    assert_eq!(fields[0].ord, Some(0));
    assert_eq!(fields[1].name, "Back");
    assert_eq!(fields[1].ord, Some(1));

    let templates = notetype
        .templates
        .as_ref()
        .expect("explicit custom templates");
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].name, "Card 1");
    assert_eq!(templates[0].ord, Some(0));
    assert_eq!(templates[0].question_format, "{{Front}}");
    assert_eq!(
        templates[0].answer_format,
        "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
    );
}

#[test]
fn product_lowering_records_note_template_css_and_media_source_paths() {
    let plan = ProductDocument::new("source-map-doc")
        .with_custom_notetype(CustomNoteType {
            id: "jp-vocab".into(),
            name: Some("Japanese Vocabulary".into()),
            fields: vec![
                CustomField {
                    name: "Expression".into(),
                    key: Some("expression".into()),
                },
                CustomField {
                    name: "Audio".into(),
                    key: Some("audio".into()),
                },
            ],
            templates: vec![CustomTemplate {
                name: "Recognition".into(),
                key: Some("recognition".into()),
                question_format: "{{Expression}}".into(),
                answer_format: "{{FrontSide}}{{Audio}}".into(),
                generation_rule: None,
            }],
            css: Some(".card { font-family: sans-serif; }".into()),
        })
        .with_browser_appearance(
            "jp-vocab",
            anki_forge::product::metadata::TemplateBrowserAppearanceDeclaration {
                template_name: "Recognition".into(),
                question_format: Some("{{Expression}}".into()),
                answer_format: Some("{{Audio}}".into()),
                font_name: None,
                font_size: None,
            },
        )
        .add_custom_note(CustomNote {
            id: "jp:taberu".into(),
            note_type_id: "jp-vocab".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                ("Expression".into(), "taberu".into()),
                ("Audio".into(), "[sound:taberu.mp3]".into()),
            ]),
            tags: vec![],
        })
        .lower()
        .expect("lower should succeed");

    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.notes[\"jp:taberu\"].fields[\"Audio\"]"),
        Some("project.notes[\"jp:taberu\"].fields[\"Audio\"]")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[\"jp-vocab\"].templates[\"Recognition\"].front"
        ),
        Some("project.note_types[\"jp-vocab\"].templates[\"Recognition\"].front")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[\"jp-vocab\"].templates[\"Recognition\"].browser_back"
        ),
        Some("project.note_types[\"jp-vocab\"].templates[\"Recognition\"].browser_back")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[\"jp-vocab\"].css"),
        Some("project.note_types[\"jp-vocab\"].css")
    );
}

#[test]
fn duplicate_notetype_ids_use_index_source_paths_for_template_browser_and_css_surfaces() {
    let plan = ProductDocument::new("duplicate-notetype-source-map")
        .with_custom_notetype(CustomNoteType {
            id: "jp-vocab".into(),
            name: Some("Japanese Vocabulary".into()),
            fields: vec![CustomField {
                name: "Expression".into(),
                key: Some("expression".into()),
            }],
            templates: vec![CustomTemplate {
                name: "Recognition".into(),
                key: Some("recognition".into()),
                question_format: "{{Expression}}".into(),
                answer_format: "{{FrontSide}}".into(),
                generation_rule: None,
            }],
            css: Some(".card { color: red; }".into()),
        })
        .with_custom_notetype(CustomNoteType {
            id: "jp-vocab".into(),
            name: Some("Japanese Vocabulary Copy".into()),
            fields: vec![CustomField {
                name: "Expression".into(),
                key: Some("expression".into()),
            }],
            templates: vec![CustomTemplate {
                name: "Recall".into(),
                key: Some("recall".into()),
                question_format: "{{Expression}}".into(),
                answer_format: "{{FrontSide}}".into(),
                generation_rule: None,
            }],
            css: Some(".card { color: blue; }".into()),
        })
        .with_browser_appearance(
            "jp-vocab",
            anki_forge::product::metadata::TemplateBrowserAppearanceDeclaration {
                template_name: "Recognition".into(),
                question_format: Some("{{Expression}}".into()),
                answer_format: Some("{{Expression}}".into()),
                font_name: None,
                font_size: None,
            },
        )
        .with_browser_appearance(
            "jp-vocab",
            anki_forge::product::metadata::TemplateBrowserAppearanceDeclaration {
                template_name: "Recall".into(),
                question_format: Some("{{Expression}}".into()),
                answer_format: Some("{{Expression}}".into()),
                font_name: None,
                font_size: None,
            },
        )
        .lower()
        .expect("lower duplicate notetype ids enough to inspect source map");

    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[0].templates[\"Recognition\"].front"),
        Some("project.note_types[0].templates[\"Recognition\"].front")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[0].templates[\"Recognition\"].browser_back"
        ),
        Some("project.note_types[0].templates[\"Recognition\"].browser_back")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[0].css"),
        Some("project.note_types[0].css")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[1].templates[\"Recall\"].back"),
        Some("project.note_types[1].templates[\"Recall\"].back")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[1].css"),
        Some("project.note_types[1].css")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[\"jp-vocab\"].templates[\"Recognition\"].front"
        ),
        None
    );
}

#[test]
fn product_default_deck_does_not_overwrite_explicit_note_deck() {
    let plan = ProductDocument::new("multi-deck-doc")
        .with_default_deck("Package::Default")
        .with_basic("basic-main")
        .add_basic_note("basic-main", "note-1", "Per Note::Deck", "front", "back")
        .lower()
        .expect("lower should succeed");

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");

    assert_eq!(note.deck_name, "Per Note::Deck");
}

fn workspace_runtime_start_dir() -> PathBuf {
    let mut cursor = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if cursor.join("contracts/manifest.yaml").exists() {
            return cursor;
        }
        if !cursor.pop() {
            return std::env::current_dir().expect("current dir fallback");
        }
    }
}

fn product_v2_fixture(name: &str) -> ProductDocument {
    let path = workspace_runtime_start_dir()
        .join("contracts/fixtures/product-v2")
        .join(name);
    let raw = std::fs::read_to_string(&path).expect("read product-v2 fixture");
    serde_json::from_str(&raw).expect("parse product-v2 fixture")
}

fn product_v2_inline(raw: &str) -> ProductDocument {
    serde_json::from_str(raw).expect("parse inline product-v2 document")
}

fn build_product_document_with_workspace_writer_stack(
    document: ProductDocument,
    options: BuildOptions,
) -> anki_forge::build::BuildReport {
    let start = workspace_runtime_start_dir();
    let runtime = anki_forge::runtime::discover_workspace_runtime(&start)
        .expect("discover workspace runtime");
    let bundle = anki_forge::runtime::load_bundle_from_manifest(&runtime.manifest_path)
        .expect("load workspace runtime bundle");
    let writer_policy =
        anki_forge::runtime::load_writer_policy(&bundle, "default").expect("load writer policy");
    let build_context =
        anki_forge::runtime::load_build_context(&bundle, "default").expect("load build context");

    anki_forge::runtime::build_product_document_with_writer_stack(
        document,
        options,
        writer_policy,
        build_context,
    )
    .expect("build product document with workspace writer stack")
}

fn identity_options(temp: &Path, label: &str) -> (BuildOptions, PathBuf) {
    let lockfile = temp.join(format!("{label}.lock.json"));
    let output = temp.join(format!("{label}.apkg"));
    (
        BuildOptions::new()
            .output(output)
            .identity_lockfile(&lockfile)
            .write_identity_lockfile(true)
            .update_safety(UpdateSafetyMode::Strict),
        lockfile,
    )
}

fn active_note_identity(lockfile: &Path) -> NoteIdentityEntry {
    let lockfile = anki_forge::update_safety::lockfile::read_lockfile(lockfile)
        .expect("read identity lockfile");
    lockfile
        .identity_index
        .notes
        .into_iter()
        .find(|entry| entry.entry_lifecycle == "active")
        .expect("active note identity entry")
}

fn diagnostic_codes(document: ProductDocument) -> Vec<&'static str> {
    document
        .lower()
        .expect("product-v2 diagnostics should be carried in the lowering plan")
        .product_diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn product_v2_workspace_runtime_prerequisite_is_available() {
    let start = workspace_runtime_start_dir();
    let runtime =
        anki_forge::runtime::discover_workspace_runtime(&start).expect("workspace runtime");

    assert!(runtime.manifest_path.ends_with("contracts/manifest.yaml"));
    assert!(runtime.bundle_root.ends_with("contracts"));
}

#[test]
fn product_v2_basic_lowers_to_authoring_fields_and_stock_identity() {
    let plan = product_v2_fixture("basic-stock.json")
        .lower()
        .expect("lower product-v2 basic fixture");

    assert!(plan.product_diagnostics.is_empty());
    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("basic notetype");
    assert_eq!(notetype.id, "basic");
    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.original_stock_kind.as_deref(), Some("basic"));

    let note = plan.authoring_document.notes.first().expect("basic note");
    assert_eq!(note.id, "basic:hello");
    assert_eq!(note.fields.get("Front").map(String::as_str), Some("Hello"));
    assert_eq!(note.fields.get("Back").map(String::as_str), Some("World"));
}

#[test]
fn product_v2_basic_identity_matches_builder_stock_recipe() {
    let temp = tempfile::tempdir().expect("tempdir");
    let raw = r#"{
      "product_document_version": "product-v2",
      "document_id": "basic-identity-demo",
      "default_deck_name": "Identity",
      "note_types": [{
        "kind": "stock",
        "id": "basic",
        "name": "Basic",
        "fields": [
          {"name": "Front", "key": "front", "identity": false, "sort": true, "required": true},
          {"name": "Back", "key": "back", "identity": false, "sort": false, "required": false}
        ],
        "templates": [{"name": "Card 1", "key": "card_1", "front": "{{Front}}", "back": "{{Back}}", "generation_rule": {"kind": "anki_default"}}],
        "css": null
      }],
      "notes": [{
        "kind": "stock",
        "note_type_id": "basic",
        "deck_name": "Identity",
        "fields": {
          "front": {"kind": "text", "value": "Derived front"},
          "back": {"kind": "text", "value": "Derived back"}
        },
        "tags": [],
        "source_path": "project.notes[0]"
      }],
      "media": []
    }"#;

    let (product_options, product_lockfile) = identity_options(temp.path(), "product-basic");
    let product_report =
        build_product_document_with_workspace_writer_stack(product_v2_inline(raw), product_options);

    let mut builder = Project::new("Identity").stable_id("basic-identity-demo");
    builder
        .add_note(Note::basic("Derived front", "Derived back").deck("Identity"))
        .expect("add builder basic note");
    let (builder_options, builder_lockfile) = identity_options(temp.path(), "builder-basic");
    let builder_report = builder.build(builder_options).expect("build builder basic");

    let product_identity = active_note_identity(&product_lockfile);
    let builder_identity = active_note_identity(&builder_lockfile);
    assert_eq!(product_report.status, builder_report.status);
    assert_eq!(product_identity.stable_id, builder_identity.stable_id);
    assert_eq!(product_identity.recipe_id, "basic.core.v1");
    assert_eq!(
        product_identity.canonical_payload_hash,
        builder_identity.canonical_payload_hash
    );
}

#[test]
fn product_v2_stock_basic_without_stable_id_derives_identity() {
    let raw = r#"{
      "product_document_version": "product-v2",
      "document_id": "basic-derived-demo",
      "default_deck_name": "Identity",
      "note_types": [{"kind": "stock", "id": "basic", "name": "Basic", "fields": [], "templates": [], "css": null}],
      "notes": [{
        "kind": "stock",
        "note_type_id": "basic",
        "deck_name": "Identity",
        "fields": {
          "front": {"kind": "text", "value": "No explicit id"},
          "back": {"kind": "text", "value": "Back"}
        },
        "tags": [],
        "source_path": "project.notes[0]"
      }],
      "media": []
    }"#;
    let plan = product_v2_inline(raw)
        .lower()
        .expect("lower product-v2 basic without stable id");
    let note = plan.authoring_document.notes.first().expect("lowered note");

    assert!(note.id.starts_with("afid:v1:"));
}

#[test]
fn product_v2_source_path_mixed_notes_use_absolute_indexes() {
    let plan = product_v2_fixture("source-path-mixed-notes.json")
        .lower()
        .expect("lower mixed source paths");

    assert!(plan.product_diagnostics.is_empty());
    assert_eq!(plan.authoring_document.notes.len(), 4);
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.notes[1]"),
        Some("project.notes[1]")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.notes[3]"),
        Some("project.notes[3]")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("product_v2.notes[1]"),
        Some("project.notes[1]")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("product_v2.notes[3]"),
        Some("project.notes[3]")
    );
}

#[test]
fn product_v2_stock_notetype_order_is_semantic_not_positional() {
    let plan = product_v2_fixture("stock-order-cloze-before-basic.json")
        .lower()
        .expect("lower stock order fixture");

    assert!(plan.product_diagnostics.is_empty());
    assert_eq!(
        plan.authoring_document.notes[0]
            .fields
            .get("Front")
            .map(String::as_str),
        Some("Basic front")
    );
    assert_eq!(
        plan.authoring_document.notes[1]
            .fields
            .get("Text")
            .map(String::as_str),
        Some("A {{c1::cloze}} note")
    );
}

#[test]
fn product_v2_custom_identity_matches_builder_product_recipe() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (product_options, product_lockfile) = identity_options(temp.path(), "product-custom");
    let product_report = build_product_document_with_workspace_writer_stack(
        product_v2_fixture("custom-identity-derived.json"),
        product_options,
    );

    let mut builder = Project::new("Identity").stable_id("custom-identity-demo");
    builder
        .add_notetype(
            NoteType::custom("jp-vocab")
                .name("Japanese Vocabulary")
                .field(
                    Field::new("Expression")
                        .key("expr")
                        .identity()
                        .sort()
                        .required(),
                )
                .field(Field::new("Meaning").key("meaning"))
                .template(
                    Template::new("Recognition")
                        .key("recognition")
                        .front("{{Expression}}")
                        .back("{{Meaning}}")
                        .generate_when(anki_forge::prelude::GenerationRule::all(["expr"])),
                )
                .identity(IdentityRecipe::fields(["expr"])),
        )
        .expect("add custom notetype");
    builder
        .add_note(
            Note::new("jp-vocab")
                .deck("Identity")
                .text("expr", "食べる")
                .text("meaning", "to eat"),
        )
        .expect("add custom note");
    let (builder_options, builder_lockfile) = identity_options(temp.path(), "builder-custom");
    let builder_report = builder
        .build(builder_options)
        .expect("build builder custom");

    let product_identity = active_note_identity(&product_lockfile);
    let builder_identity = active_note_identity(&builder_lockfile);
    assert_eq!(product_report.status, builder_report.status);
    assert_eq!(product_identity.stable_id, builder_identity.stable_id);
    assert_eq!(product_identity.recipe_id, "custom.notetype.fields.v1");
    assert_eq!(
        product_identity.canonical_payload_hash,
        builder_identity.canonical_payload_hash
    );
    assert_eq!(product_identity.provenance, builder_identity.provenance);
}

#[test]
fn product_v2_empty_generation_rule_fields_are_diagnostic() {
    let codes = diagnostic_codes(product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-generation-empty",
          "default_deck_name": "Invalid",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [{"name": "Prompt", "key": "prompt"}],
            "templates": [{"name": "Card", "key": "card", "front": "{{Prompt}}", "back": "{{Prompt}}", "generation_rule": {"kind": "all", "fields": []}}],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "css": null
          }],
          "notes": [],
          "media": []
        }"#,
    ));

    assert!(codes.contains(&"PRODUCT.GENERATION_RULE_INVALID"));
}

#[test]
fn product_v2_generation_rule_unknown_field_is_diagnostic() {
    let codes = diagnostic_codes(product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-generation-field",
          "default_deck_name": "Invalid",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [{"name": "Prompt", "key": "prompt"}],
            "templates": [{"name": "Card", "key": "card", "front": "{{Prompt}}", "back": "{{Prompt}}", "generation_rule": {"kind": "any", "fields": ["missing"]}}],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "css": null
          }],
          "notes": [],
          "media": []
        }"#,
    ));

    assert!(codes.contains(&"PRODUCT.GENERATION_RULE_INVALID"));
}

#[test]
fn product_v2_custom_note_unknown_field_is_diagnostic() {
    let codes = diagnostic_codes(product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-note-field",
          "default_deck_name": "Invalid",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [{"name": "Prompt", "key": "prompt", "required": true}],
            "templates": [{"name": "Card", "key": "card", "front": "{{Prompt}}", "back": "{{Prompt}}", "generation_rule": {"kind": "anki_default"}}],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "css": null
          }],
          "notes": [{
            "kind": "custom",
            "note_type_id": "custom",
            "deck_name": "Invalid",
            "fields": {
              "prompt": {"kind": "text", "value": "ok"},
              "extra": {"kind": "text", "value": "no"}
            },
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    ));

    assert!(codes.contains(&"PRODUCT.FIELD_UNKNOWN"));
}

#[test]
fn product_v2_custom_note_missing_required_field_is_diagnostic() {
    let codes = diagnostic_codes(product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-note-required",
          "default_deck_name": "Invalid",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [{"name": "Prompt", "key": "prompt", "required": true}],
            "templates": [{"name": "Card", "key": "card", "front": "{{Prompt}}", "back": "{{Prompt}}", "generation_rule": {"kind": "anki_default"}}],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "css": null
          }],
          "notes": [{
            "kind": "custom",
            "note_type_id": "custom",
            "deck_name": "Invalid",
            "fields": {},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    ));

    assert!(codes.contains(&"PRODUCT.REQUIRED_FIELD_MISSING"));
}

#[test]
fn product_v2_stock_note_without_declaration_is_diagnostic() {
    let codes = diagnostic_codes(product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-stock-note",
          "default_deck_name": "Invalid",
          "note_types": [],
          "notes": [{
            "kind": "stock",
            "note_type_id": "basic",
            "deck_name": "Invalid",
            "fields": {"front": {"kind": "text", "value": "front"}},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    ));

    assert!(codes.contains(&"PRODUCT.STOCK_NOTE_TYPE_MISSING"));
}
