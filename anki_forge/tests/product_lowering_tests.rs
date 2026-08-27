#![cfg(feature = "internal-tools")]

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
fn legacy_custom_cloze_rejects_multiple_templates() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "document_id": "legacy-invalid-cloze",
        "note_types": [{
            "Custom": {
                "id": "legacy-cloze",
                "name": "Legacy Cloze",
                "fields": [{"name": "Text", "key": "text"}],
                "templates": [
                    {
                        "name": "Normal",
                        "question_format": "{{Text}}",
                        "answer_format": "{{Text}}"
                    },
                    {
                        "name": "Cloze",
                        "question_format": "{{cloze:Text}}",
                        "answer_format": "{{cloze:Text}}",
                        "generation_rule": {"cloze": {"field": "text"}}
                    }
                ]
            }
        }],
        "notes": []
    }))
    .expect("legacy product document");

    let error = document
        .lower()
        .expect_err("invalid legacy Cloze shape should fail lowering");

    assert!(error
        .product_diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.code == "TEMPLATE.CLOZE_TEMPLATE_COUNT_INVALID" }));
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
    try_build_product_document_with_workspace_writer_stack(document, options)
        .expect("build product document with workspace writer stack")
}

fn try_build_product_document_with_workspace_writer_stack(
    document: ProductDocument,
    options: BuildOptions,
) -> Result<anki_forge::build::BuildReport, anki_forge::build::BuildError> {
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

fn diagnostic_source<'a>(
    diagnostics: &'a [anki_forge::diagnostics::Diagnostic],
    code: &str,
) -> Option<&'a str> {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == code)
        .and_then(|diagnostic| diagnostic.source.as_ref())
        .map(|source| source.as_str())
}

fn build_error_diagnostic_source(document: ProductDocument, code: &str) -> Option<String> {
    let temp = tempfile::tempdir().expect("tempdir");
    let err = try_build_product_document_with_workspace_writer_stack(
        document,
        BuildOptions::new().output(temp.path().join("invalid.apkg")),
    )
    .expect_err("product diagnostics should make the build unsuccessful");

    diagnostic_source(&err.report.diagnostics, code).map(str::to_owned)
}

const PRODUCT_V2_BASIC_MISSING_FRONT: &str = r#"{
  "product_document_version": "product-v2",
  "document_id": "invalid-basic-required",
  "default_deck_name": "Invalid",
  "note_types": [{
    "kind": "stock",
    "id": "basic",
    "name": "Basic",
    "fields": [
      {"name": "Front", "key": "front", "required": true},
      {"name": "Back", "key": "back", "required": false}
    ],
    "templates": [],
    "css": null
  }],
  "notes": [{
    "kind": "stock",
    "note_type_id": "basic",
    "deck_name": "Invalid",
    "fields": {"back": {"kind": "text", "value": "Back only"}},
    "source_path": "project.notes[0]"
  }],
  "media": []
}"#;

const PRODUCT_V2_IO_STOCK: &str = r#"{
  "product_document_version": "product-v2",
  "document_id": "io-stock",
  "default_deck_name": "IO",
  "note_types": [{
    "kind": "stock",
    "id": "image_occlusion",
    "name": "Image Occlusion",
    "fields": [
      {"name": "Occlusion", "key": "occlusion", "required": true},
      {"name": "Image", "key": "image", "required": true},
      {"name": "Header", "key": "header", "required": true},
      {"name": "Back Extra", "key": "back_extra", "required": true},
      {"name": "Comments", "key": "comments", "required": false}
    ],
    "templates": [{
      "name": "Image Occlusion",
      "key": "image_occlusion",
      "front": "{{cloze:Occlusion}}",
      "back": "{{cloze:Occlusion}}<br>{{Image}}",
      "generation_rule": {"kind": "cloze", "field": "occlusion"}
    }],
    "css": null
  }],
  "notes": [{
    "kind": "stock",
    "note_type_id": "image_occlusion",
    "stable_id": "io:1",
    "deck_name": "IO",
    "fields": {
      "occlusion": {"kind": "html", "value": "{{c1::image-occlusion:rect:left=0:top=0:width=1:height=1}}<br>"},
      "image": {"kind": "image", "media_id": "media:heart"},
      "header": {"kind": "text", "value": "Heart"},
      "back_extra": {"kind": "text", "value": "Identify it"},
      "comments": {"kind": "text", "value": "Review"}
    },
    "tags": ["io"],
    "source_path": "project.notes[0]"
  }],
  "media": [{
    "id": "media:heart",
    "source": {"kind": "inline_base64", "source_label": "heart.png", "data_base64": "aGVhcnQ="},
    "export_as": "heart.png"
  }]
}"#;

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
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[\"basic\"].fields[\"Front\"]"),
        Some("project.note_types[\"basic\"].fields[\"front\"]")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.note_types[\"basic\"].templates[\"Card 1\"]"),
        Some("project.note_types[\"basic\"].templates[\"card_1\"]")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(
            "authoring.note_types[\"basic\"].templates[\"Card 1\"].front"
        ),
        Some("project.note_types[\"basic\"].templates[\"card_1\"].front")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.notes[\"a\"].fields[\"Front\"]"),
        Some("project.notes[\"a\"].fields[\"front\"]")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path("authoring.notes[\"a\"].fields[\"Back\"]"),
        Some("project.notes[\"a\"].fields[\"back\"]")
    );
}

#[test]
fn product_v2_custom_note_field_source_paths_use_product_keys() {
    let plan = product_v2_fixture("custom-identity-derived.json")
        .lower()
        .expect("lower custom identity fixture");
    let note_id = &plan
        .authoring_document
        .notes
        .first()
        .expect("lowered custom note")
        .id;

    assert_eq!(
        plan.source_map.source_for_authoring_path(&format!(
            "authoring.notes[{note_id:?}].fields[\"Expression\"]"
        )),
        Some("project.notes[0].fields[\"expr\"]")
    );
    assert_eq!(
        plan.source_map.source_for_authoring_path(&format!(
            "authoring.notes[{note_id:?}].fields[\"Meaning\"]"
        )),
        Some("project.notes[0].fields[\"meaning\"]")
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
fn product_v2_stock_image_occlusion_lowers_to_authoring_fields() {
    let plan = product_v2_inline(PRODUCT_V2_IO_STOCK)
        .lower()
        .expect("lower io product-v2");

    let notetype = plan.authoring_document.notetypes.first().expect("notetype");
    assert_eq!(
        notetype.original_stock_kind.as_deref(),
        Some("image_occlusion")
    );
    assert_eq!(notetype.kind, "cloze");

    let note = plan.authoring_document.notes.first().expect("note");
    assert_eq!(note.notetype_id, "image_occlusion");
    assert_eq!(note.fields.get("Header").map(String::as_str), Some("Heart"));
    assert_eq!(
        note.fields.get("Image").map(String::as_str),
        Some("<img src=\"heart.png\">")
    );
    assert_eq!(note.tags, vec!["io"]);
}

#[test]
fn product_v2_stock_image_occlusion_typed_image_resolves_media() {
    let plan = product_v2_inline(PRODUCT_V2_IO_STOCK)
        .lower()
        .expect("lower io product-v2");
    let note = plan.authoring_document.notes.first().expect("note");

    assert_eq!(
        note.fields.get("Image").map(String::as_str),
        Some("<img src=\"heart.png\">")
    );
}

#[test]
fn product_v2_stock_image_occlusion_unknown_field_source_path() {
    let document = PRODUCT_V2_IO_STOCK.replace(
        "\"comments\": {\"kind\": \"text\", \"value\": \"Review\"}",
        "\"comments\": {\"kind\": \"text\", \"value\": \"Review\"}, \"extra\": {\"kind\": \"text\", \"value\": \"ignored\"}",
    );
    let plan = product_v2_inline(&document)
        .lower()
        .expect("lower invalid io product-v2 with diagnostics");
    let diagnostic = plan
        .product_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PRODUCT.FIELD_UNKNOWN")
        .expect("unknown field diagnostic");

    assert_eq!(
        diagnostic.source_path.as_deref(),
        Some("project.notes[0].fields[\"extra\"]")
    );
}

#[test]
fn product_v2_stock_image_occlusion_missing_required_field() {
    let document = PRODUCT_V2_IO_STOCK.replace(
        "\"image\": {\"kind\": \"image\", \"media_id\": \"media:heart\"},",
        "",
    );
    let plan = product_v2_inline(&document)
        .lower()
        .expect("lower invalid io product-v2 with diagnostics");

    assert!(plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
    assert!(plan.authoring_document.notes.is_empty());
}

#[test]
fn product_v2_stock_image_occlusion_without_stable_id_is_identity_missing() {
    let document = PRODUCT_V2_IO_STOCK.replace("\"stable_id\": \"io:1\",", "");
    let plan = product_v2_inline(&document)
        .lower()
        .expect("lower invalid io product-v2 with diagnostics");

    assert!(plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.IDENTITY_MISSING"));
    assert!(plan.authoring_document.notes.is_empty());
}

#[test]
fn product_v2_stock_basic_missing_front_is_required_field_diagnostic() {
    let plan = product_v2_inline(PRODUCT_V2_BASIC_MISSING_FRONT)
        .lower()
        .expect("invalid product-v2 basic should lower with diagnostics");

    assert!(plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
    assert!(plan.authoring_document.notes.is_empty());
}

#[test]
fn product_v2_stock_cloze_missing_text_is_required_field_diagnostic() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-cloze-required",
          "default_deck_name": "Invalid",
          "note_types": [{
            "kind": "stock",
            "id": "cloze",
            "name": "Cloze",
            "fields": [
              {"name": "Text", "key": "text", "required": true},
              {"name": "Back Extra", "key": "back_extra", "required": false}
            ],
            "templates": [],
            "css": null
          }],
          "notes": [{
            "kind": "stock",
            "note_type_id": "cloze",
            "deck_name": "Invalid",
            "fields": {"back_extra": {"kind": "text", "value": "Hint only"}},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    )
    .lower()
    .expect("invalid product-v2 cloze should lower with diagnostics");

    assert!(plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
    assert!(plan.authoring_document.notes.is_empty());
}

#[test]
fn product_v2_stock_basic_missing_optional_back_still_lowers() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "basic-optional-back",
          "default_deck_name": "Optional",
          "note_types": [{
            "kind": "stock",
            "id": "basic",
            "name": "Basic",
            "fields": [
              {"name": "Front", "key": "front", "required": true},
              {"name": "Back", "key": "back", "required": false}
            ],
            "templates": [],
            "css": null
          }],
          "notes": [{
            "kind": "stock",
            "note_type_id": "basic",
            "stable_id": "basic:optional-back",
            "deck_name": "Optional",
            "fields": {"front": {"kind": "text", "value": "Front only"}},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    )
    .lower()
    .expect("lower basic with missing optional back");

    assert!(!plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
    let note = plan.authoring_document.notes.first().expect("lowered note");
    assert_eq!(
        note.fields.get("Front").map(String::as_str),
        Some("Front only")
    );
    assert_eq!(note.fields.get("Back").map(String::as_str), Some(""));
}

#[test]
fn product_v2_stock_cloze_missing_optional_back_extra_still_lowers() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "cloze-optional-back-extra",
          "default_deck_name": "Optional",
          "note_types": [{
            "kind": "stock",
            "id": "cloze",
            "name": "Cloze",
            "fields": [
              {"name": "Text", "key": "text", "required": true},
              {"name": "Back Extra", "key": "back_extra", "required": false}
            ],
            "templates": [],
            "css": null
          }],
          "notes": [{
            "kind": "stock",
            "note_type_id": "cloze",
            "stable_id": "cloze:optional-back-extra",
            "deck_name": "Optional",
            "fields": {"text": {"kind": "html", "value": "A {{c1::cloze}} note"}},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    )
    .lower()
    .expect("lower cloze with missing optional back extra");

    assert!(!plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
    let note = plan.authoring_document.notes.first().expect("lowered note");
    assert_eq!(
        note.fields.get("Text").map(String::as_str),
        Some("A {{c1::cloze}} note")
    );
    assert_eq!(note.fields.get("Back Extra").map(String::as_str), Some(""));
}

#[test]
fn product_v2_custom_missing_optional_field_still_lowers_empty() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "custom-optional",
          "default_deck_name": "Optional",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [
              {"name": "Prompt", "key": "prompt", "identity": true, "required": true},
              {"name": "Back", "key": "back", "required": false}
            ],
            "templates": [{"name": "Card", "key": "card", "front": "{{Prompt}}", "back": "{{Back}}", "generation_rule": {"kind": "all", "fields": ["prompt"]}}],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "css": null
          }],
          "notes": [{
            "kind": "custom",
            "note_type_id": "custom",
            "deck_name": "Optional",
            "fields": {"prompt": {"kind": "text", "value": "Front only"}},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    )
    .lower()
    .expect("lower custom note with missing optional field");

    assert!(!plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.REQUIRED_FIELD_MISSING"));
    let note = plan.authoring_document.notes.first().expect("lowered note");
    assert_eq!(
        note.fields.get("Prompt").map(String::as_str),
        Some("Front only")
    );
    assert_eq!(note.fields.get("Back").map(String::as_str), Some(""));
    assert_eq!(
        plan.source_map.source_for_authoring_path(&format!(
            "authoring.notes[{:?}].fields[\"Prompt\"]",
            note.id
        )),
        Some("project.notes[0].fields[\"prompt\"]")
    );
    assert_eq!(
        plan.source_map
            .source_for_authoring_path(&format!("authoring.notes[{:?}].fields[\"Back\"]", note.id)),
        None
    );
}

#[test]
fn product_v2_image_content_renders_like_builder_media_ref() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "image-content",
          "default_deck_name": "Images",
          "note_types": [{
            "kind": "custom",
            "id": "image-card",
            "name": "Image Card",
            "fields": [{"name": "Picture", "key": "picture", "identity": true, "required": true}],
            "templates": [{"name": "Card", "key": "card", "front": "{{Picture}}", "back": "{{Picture}}", "generation_rule": {"kind": "all", "fields": ["picture"]}}],
            "identity": {"kind": "fields", "fields": ["picture"]},
            "css": null
          }],
          "notes": [{
            "kind": "custom",
            "note_type_id": "image-card",
            "stable_id": "image:one",
            "deck_name": "Images",
            "fields": {"picture": {"kind": "image", "media_id": "image:one"}},
            "source_path": "project.notes[0]"
          }],
          "media": [{
            "id": "image:one",
            "source": {"kind": "inline_base64", "source_label": "pixel", "data_base64": "AA=="},
            "export_as": "picture.png"
          }]
        }"#,
    )
    .lower()
    .expect("lower image content");

    let note = plan.authoring_document.notes.first().expect("lowered note");
    assert_eq!(
        note.fields.get("Picture").map(String::as_str),
        Some("<img src=\"picture.png\">")
    );
}

#[test]
fn product_v2_sound_content_renders_real_anki_sound_markup() {
    let plan = product_v2_fixture("custom-typed-media.json")
        .lower()
        .expect("lower typed media fixture");

    let note = plan.authoring_document.notes.first().expect("lowered note");
    assert_eq!(
        note.fields.get("Audio").map(String::as_str),
        Some("[sound:hello.wav]")
    );
}

#[test]
fn rust_product_media_registry_declares_expanded_mime_types_from_export_name() {
    let mut project = Project::new("Expanded MIME")
        .stable_id("expanded-mime")
        .default_deck("Expanded MIME");

    for export_as in [
        "diagram.webp",
        "clip.mp4",
        "movie.webm",
        "voice.ogg",
        "voice.opus",
        "song.m4a",
        "raw.aac",
        "handout.pdf",
        "theme.css",
        "script.js",
        "fragment.html",
        "font.ttf",
        "font.otf",
        "font.woff",
        "font.woff2",
    ] {
        project
            .media_mut()
            .add_bytes(
                format!("{export_as}.source"),
                format!("bytes for {export_as}").into_bytes(),
            )
            .expect("register bytes")
            .export_as(export_as)
            .expect("export media");
    }

    let plan = project.lower().expect("lower product media");
    let declared_by_name = plan
        .authoring_document
        .media
        .iter()
        .map(|media| {
            (
                media.desired_filename.as_str(),
                media.declared_mime.as_deref(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(declared_by_name["diagram.webp"], Some("image/webp"));
    assert_eq!(declared_by_name["clip.mp4"], Some("video/mp4"));
    assert_eq!(declared_by_name["movie.webm"], Some("video/webm"));
    assert_eq!(declared_by_name["voice.ogg"], Some("audio/ogg"));
    assert_eq!(declared_by_name["voice.opus"], Some("audio/opus"));
    assert_eq!(declared_by_name["song.m4a"], Some("audio/mp4"));
    assert_eq!(declared_by_name["raw.aac"], Some("audio/aac"));
    assert_eq!(declared_by_name["handout.pdf"], Some("application/pdf"));
    assert_eq!(declared_by_name["theme.css"], Some("text/css"));
    assert_eq!(declared_by_name["script.js"], Some("text/javascript"));
    assert_eq!(declared_by_name["fragment.html"], Some("text/html"));
    assert_eq!(declared_by_name["font.ttf"], Some("font/ttf"));
    assert_eq!(declared_by_name["font.otf"], Some("font/otf"));
    assert_eq!(declared_by_name["font.woff"], Some("font/woff"));
    assert_eq!(declared_by_name["font.woff2"], Some("font/woff2"));
}

#[test]
fn product_v2_normalization_uses_export_extension_mime_without_declared_mime() {
    let document = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "product-v2-extension-mime",
          "default_deck_name": "Media",
          "note_types": [{
            "kind": "stock",
            "id": "basic",
            "name": "Basic",
            "fields": [
              {"name": "Front", "key": "front", "required": true},
              {"name": "Back", "key": "back", "required": false}
            ],
            "templates": [],
            "css": null
          }],
          "notes": [{
            "kind": "stock",
            "note_type_id": "basic",
            "stable_id": "extension:mime",
            "deck_name": "Media",
            "fields": {
              "front": {"kind": "text", "value": "front"},
              "back": {"kind": "text", "value": "back"}
            }
          }],
          "media": [
            {"id": "media:css", "source": {"kind": "inline_base64", "source_label": "theme", "data_base64": "LmNhcmQgeyBjb2xvcjogcmVkOyB9Cg=="}, "export_as": "theme.css"},
            {"id": "media:webm", "source": {"kind": "inline_base64", "source_label": "clip", "data_base64": "d2VibSBwbGFjZWhvbGRlcg=="}, "export_as": "clip.webm"},
            {"id": "media:woff2", "source": {"kind": "inline_base64", "source_label": "font", "data_base64": "Zm9udCBwbGFjZWhvbGRlcg=="}, "export_as": "font.woff2"}
          ]
        }"#,
    );

    let normalized = Project::from_product_document(document)
        .normalize()
        .expect("normalize product-v2 media");
    let mime_by_binding = normalized
        .media_bindings
        .iter()
        .filter_map(|binding| {
            normalized
                .media_objects
                .iter()
                .find(|object| object.id == binding.object_id)
                .map(|object| (binding.export_filename.as_str(), object.mime.as_str()))
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(mime_by_binding["theme.css"], "text/css");
    assert_eq!(mime_by_binding["clip.webm"], "video/webm");
    assert_eq!(mime_by_binding["font.woff2"], "font/woff2");
}

#[test]
fn product_v2_custom_sort_flag_is_preserved_on_lowered_authoring_field() {
    let plan = product_v2_fixture("custom-typed-media.json")
        .lower()
        .expect("lower typed media fixture");
    let notetype = plan.authoring_document.notetypes.first().expect("notetype");
    let fields = notetype.fields.as_ref().expect("custom fields");

    assert!(fields
        .iter()
        .any(|field| field.name == "Prompt" && field.sort));
    assert!(fields
        .iter()
        .all(|field| field.name == "Prompt" || !field.sort));
}

#[test]
fn product_v2_generation_rule_lowers_to_writer_requirement_metadata() {
    let plan = product_v2_fixture("custom-typed-media.json")
        .lower()
        .expect("lower typed media fixture");
    let notetype = plan.authoring_document.notetypes.first().expect("notetype");
    let template = notetype
        .templates
        .as_ref()
        .expect("custom templates")
        .first()
        .expect("template");
    let requirement = template
        .generation_requirement
        .as_ref()
        .expect("generation requirement");

    assert_eq!(requirement.kind, "all");
    assert_eq!(requirement.field_names, vec!["Prompt"]);
}

#[test]
fn product_v2_duplicate_generation_fields_remain_compatible() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "duplicate-generation-fields",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "fields": [{"name": "Prompt", "key": "prompt"}],
            "templates": [{
              "name": "Card",
              "key": "card",
              "front": "{{Prompt}}",
              "back": "{{Prompt}}",
              "generation_rule": {"kind": "all", "fields": ["prompt", "prompt"]}
            }],
            "identity": {"kind": "fields", "fields": ["prompt"]}
          }],
          "notes": [],
          "media": []
        }"#,
    )
    .lower()
    .expect("product-v2 duplicate fields remain accepted");

    assert!(!plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.GENERATION_RULE_INVALID"));
    let requirement = plan.authoring_document.notetypes[0]
        .templates
        .as_ref()
        .unwrap()[0]
        .generation_requirement
        .as_ref()
        .expect("generation requirement");
    assert_eq!(requirement.field_names, vec!["Prompt", "Prompt"]);
}

#[test]
fn product_v2_unknown_stock_notetype_is_diagnostic_not_basic_lowering() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "future-stock",
          "default_deck_name": "Future",
          "note_types": [{
            "kind": "stock",
            "id": "future_stock",
            "name": "Future Stock",
            "fields": [],
            "templates": [],
            "source_path": "project.note_types[\"future_stock\"]"
          }],
          "notes": [],
          "media": []
        }"#,
    )
    .lower()
    .expect("unsupported stock should lower with diagnostics");

    assert!(plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.STOCK_NOTE_TYPE_INVALID"));
    assert!(plan.authoring_document.notetypes.is_empty());
}

#[test]
fn product_v2_custom_reorder_preserves_config_ids_and_updates_ords() {
    let left = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "reorder-left",
          "default_deck_name": "Reorder",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [
              {"name": "Prompt", "key": "prompt"},
              {"name": "Answer", "key": "answer"}
            ],
            "templates": [
              {"name": "Recognition", "key": "recognition", "front": "{{Prompt}}", "back": "{{Answer}}", "generation_rule": {"kind": "all", "fields": ["prompt"]}},
              {"name": "Production", "key": "production", "front": "{{Answer}}", "back": "{{Prompt}}", "generation_rule": {"kind": "all", "fields": ["answer"]}}
            ],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "css": null
          }],
          "notes": [],
          "media": []
        }"#,
    )
    .lower()
    .expect("lower left order");
    let right = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "reorder-right",
          "default_deck_name": "Reorder",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [
              {"name": "Answer", "key": "answer"},
              {"name": "Prompt", "key": "prompt"}
            ],
            "templates": [
              {"name": "Production", "key": "production", "front": "{{Answer}}", "back": "{{Prompt}}", "generation_rule": {"kind": "all", "fields": ["answer"]}},
              {"name": "Recognition", "key": "recognition", "front": "{{Prompt}}", "back": "{{Answer}}", "generation_rule": {"kind": "all", "fields": ["prompt"]}}
            ],
            "identity": {"kind": "fields", "fields": ["prompt"]},
            "css": null
          }],
          "notes": [],
          "media": []
        }"#,
    )
    .lower()
    .expect("lower right order");

    let left_notetype = left.authoring_document.notetypes.first().expect("left");
    let right_notetype = right.authoring_document.notetypes.first().expect("right");
    let left_prompt = left_notetype
        .fields
        .as_ref()
        .expect("left fields")
        .iter()
        .find(|field| field.name == "Prompt")
        .expect("left prompt");
    let right_prompt = right_notetype
        .fields
        .as_ref()
        .expect("right fields")
        .iter()
        .find(|field| field.name == "Prompt")
        .expect("right prompt");
    let left_recognition = left_notetype
        .templates
        .as_ref()
        .expect("left templates")
        .iter()
        .find(|template| template.name == "Recognition")
        .expect("left recognition");
    let right_recognition = right_notetype
        .templates
        .as_ref()
        .expect("right templates")
        .iter()
        .find(|template| template.name == "Recognition")
        .expect("right recognition");

    assert_eq!(left_prompt.config_id, right_prompt.config_id);
    assert_ne!(left_prompt.ord, right_prompt.ord);
    assert_eq!(left_recognition.config_id, right_recognition.config_id);
    assert_ne!(left_recognition.ord, right_recognition.ord);
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

#[test]
fn product_v2_build_surfaces_unknown_media_source_product_diagnostic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let document = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-media-source",
          "default_deck_name": "Invalid",
          "note_types": [],
          "notes": [],
          "media": [{
            "id": "media:future",
            "source": {"kind": "future_source", "uri": "asset://future"},
            "export_as": "future.bin",
            "source_path": "project.media[\"future.bin\"]"
          }]
        }"#,
    );

    let normalize_err = Project::from_product_document(document.clone())
        .normalize()
        .expect_err("product diagnostics should make normalization unsuccessful");
    assert!(
        normalize_err
            .to_string()
            .contains("PRODUCT.MEDIA_SOURCE_KIND_UNSUPPORTED"),
        "unexpected normalize error: {normalize_err}"
    );

    let err = try_build_product_document_with_workspace_writer_stack(
        document,
        BuildOptions::new().output(temp.path().join("invalid.apkg")),
    )
    .expect_err("product diagnostics should make the build unsuccessful");

    assert!(err
        .report
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.code.as_str() == "PRODUCT.MEDIA_SOURCE_KIND_UNSUPPORTED" }));
    assert_eq!(
        diagnostic_source(
            &err.report.diagnostics,
            "PRODUCT.MEDIA_SOURCE_KIND_UNSUPPORTED"
        ),
        Some("project.media[\"future.bin\"]")
    );
    assert!(!err.report.status.is_success());
}

#[test]
fn product_v2_build_surfaces_required_field_source_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let document = product_v2_inline(PRODUCT_V2_BASIC_MISSING_FRONT);

    let normalize_err = Project::from_product_document(document.clone())
        .normalize()
        .expect_err("product diagnostics should make normalization unsuccessful");
    assert!(
        normalize_err
            .to_string()
            .contains("PRODUCT.REQUIRED_FIELD_MISSING"),
        "unexpected normalize error: {normalize_err}"
    );

    let err = try_build_product_document_with_workspace_writer_stack(
        document,
        BuildOptions::new().output(temp.path().join("invalid-required.apkg")),
    )
    .expect_err("product diagnostics should make the build unsuccessful");

    assert_eq!(
        diagnostic_source(&err.report.diagnostics, "PRODUCT.REQUIRED_FIELD_MISSING"),
        Some("project.notes[0]")
    );
    assert!(!err.report.status.is_success());
}

#[test]
fn product_v2_build_surfaces_custom_unknown_field_source_path() {
    let source = build_error_diagnostic_source(
        product_v2_inline(
            r#"{
              "product_document_version": "product-v2",
              "document_id": "invalid-field-source",
              "default_deck_name": "Invalid",
              "note_types": [{
                "kind": "custom",
                "id": "custom",
                "name": "Custom",
                "fields": [{"name": "Prompt", "key": "prompt"}],
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
                "source_path": "project.notes[\"bad-field\"]"
              }],
              "media": []
            }"#,
        ),
        "PRODUCT.FIELD_UNKNOWN",
    );

    assert_eq!(source.as_deref(), Some("project.notes[\"bad-field\"]"));
}

#[test]
fn product_v2_build_surfaces_stock_missing_notetype_source_path() {
    let source = build_error_diagnostic_source(
        product_v2_inline(
            r#"{
              "product_document_version": "product-v2",
              "document_id": "invalid-stock-source",
              "default_deck_name": "Invalid",
              "note_types": [],
              "notes": [{
                "kind": "stock",
                "note_type_id": "basic",
                "deck_name": "Invalid",
                "fields": {"front": {"kind": "text", "value": "front"}},
                "source_path": "project.notes[\"missing-stock\"]"
              }],
              "media": []
            }"#,
        ),
        "PRODUCT.STOCK_NOTE_TYPE_MISSING",
    );

    assert_eq!(source.as_deref(), Some("project.notes[\"missing-stock\"]"));
}

#[test]
fn product_v2_build_surfaces_missing_media_field_source_path() {
    let source = build_error_diagnostic_source(
        product_v2_inline(
            r#"{
              "product_document_version": "product-v2",
              "document_id": "invalid-missing-media-source",
              "default_deck_name": "Invalid",
              "note_types": [{
                "kind": "custom",
                "id": "image-card",
                "name": "Image Card",
                "fields": [{"name": "Picture", "key": "picture", "required": true}],
                "templates": [{"name": "Card", "key": "card", "front": "{{Picture}}", "back": "{{Picture}}", "generation_rule": {"kind": "anki_default"}}],
                "identity": {"kind": "fields", "fields": ["picture"]},
                "css": null
              }],
              "notes": [{
                "kind": "custom",
                "note_type_id": "image-card",
                "stable_id": "image:missing",
                "deck_name": "Invalid",
                "fields": {"picture": {"kind": "image", "media_id": "media:missing"}},
                "source_path": "project.notes[0]"
              }],
              "media": []
            }"#,
        ),
        "PRODUCT.MEDIA_MISSING",
    );

    assert_eq!(
        source.as_deref(),
        Some("project.notes[0].fields[\"picture\"]")
    );
}

#[test]
fn product_v2_custom_required_render_failure_does_not_report_required_missing() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "required-render-failure",
          "default_deck_name": "Invalid",
          "note_types": [{
            "kind": "custom",
            "id": "audio-card",
            "name": "Audio Card",
            "fields": [{"name": "Audio", "key": "audio", "required": true}],
            "templates": [{"name": "Card", "key": "card", "front": "{{Audio}}", "back": "{{Audio}}", "generation_rule": {"kind": "anki_default"}}],
            "identity": {"kind": "fields", "fields": ["audio"]},
            "css": null
          }],
          "notes": [{
            "kind": "custom",
            "note_type_id": "audio-card",
            "stable_id": "audio:missing",
            "deck_name": "Invalid",
            "fields": {"audio": {"kind": "sound", "media_id": "media:missing"}},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    )
    .lower()
    .expect("product-v2 diagnostics should be carried in the lowering plan");
    let codes = plan
        .product_diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"PRODUCT.MEDIA_MISSING"));
    assert!(!codes.contains(&"PRODUCT.REQUIRED_FIELD_MISSING"));
}

#[test]
fn product_v2_build_surfaces_unknown_basic_stock_field_source_path() {
    let source = build_error_diagnostic_source(
        product_v2_inline(
            r#"{
              "product_document_version": "product-v2",
              "document_id": "invalid-basic-extra-field",
              "default_deck_name": "Invalid",
              "note_types": [{
                "kind": "stock",
                "id": "basic",
                "name": "Basic",
                "fields": [
                  {"name": "Front", "key": "front", "required": true},
                  {"name": "Back", "key": "back", "required": false}
                ],
                "templates": [],
                "css": null
              }],
              "notes": [{
                "kind": "stock",
                "note_type_id": "basic",
                "stable_id": "basic:extra",
                "deck_name": "Invalid",
                "fields": {
                  "front": {"kind": "text", "value": "front"},
                  "extra": {"kind": "text", "value": "ignored"}
                },
                "source_path": "project.notes[0]"
              }],
              "media": []
            }"#,
        ),
        "PRODUCT.FIELD_UNKNOWN",
    );

    assert_eq!(
        source.as_deref(),
        Some("project.notes[0].fields[\"extra\"]")
    );
}

#[test]
fn product_v2_build_surfaces_unknown_cloze_stock_field_source_path() {
    let source = build_error_diagnostic_source(
        product_v2_inline(
            r#"{
              "product_document_version": "product-v2",
              "document_id": "invalid-cloze-extra-field",
              "default_deck_name": "Invalid",
              "note_types": [{
                "kind": "stock",
                "id": "cloze",
                "name": "Cloze",
                "fields": [
                  {"name": "Text", "key": "text", "required": true},
                  {"name": "Back Extra", "key": "back_extra", "required": false}
                ],
                "templates": [],
                "css": null
              }],
              "notes": [{
                "kind": "stock",
                "note_type_id": "cloze",
                "stable_id": "cloze:extra",
                "deck_name": "Invalid",
                "fields": {
                  "text": {"kind": "html", "value": "A {{c1::cloze}} note"},
                  "extra": {"kind": "text", "value": "ignored"}
                },
                "source_path": "project.notes[0]"
              }],
              "media": []
            }"#,
        ),
        "PRODUCT.FIELD_UNKNOWN",
    );

    assert_eq!(
        source.as_deref(),
        Some("project.notes[0].fields[\"extra\"]")
    );
}

#[test]
fn product_v2_custom_identity_unknown_field_is_diagnostic() {
    let plan = product_v2_inline(
        r#"{
          "product_document_version": "product-v2",
          "document_id": "invalid-identity-field",
          "default_deck_name": "Invalid",
          "note_types": [{
            "kind": "custom",
            "id": "custom",
            "name": "Custom",
            "fields": [{"name": "Prompt", "key": "prompt", "required": true}],
            "templates": [{"name": "Card", "key": "card", "front": "{{Prompt}}", "back": "{{Prompt}}", "generation_rule": {"kind": "anki_default"}}],
            "identity": {"kind": "fields", "fields": ["missing"]},
            "css": null
          }],
          "notes": [{
            "kind": "custom",
            "note_type_id": "custom",
            "deck_name": "Invalid",
            "fields": {"prompt": {"kind": "text", "value": "ok"}},
            "source_path": "project.notes[0]"
          }],
          "media": []
        }"#,
    )
    .lower()
    .expect("invalid product-v2 identity should lower with diagnostics");

    assert!(plan
        .product_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "PRODUCT.IDENTITY_FIELD_UNKNOWN"));
    assert!(plan.authoring_document.notes.is_empty());
}
