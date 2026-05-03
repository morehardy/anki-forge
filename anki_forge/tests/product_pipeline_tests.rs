use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use anki_forge::product::{
    FieldMetadataDeclaration, ProductDocument, TemplateBrowserAppearanceDeclaration,
    TemplateTargetDeckDeclaration,
};
use anki_forge::{
    build, inspect_staging, normalize, BuildArtifactTarget, BuildContext, NormalizationRequest,
    WriterPolicy,
};

fn unique_artifact_root(case: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "anki-forge-phase5a-{case}-{}-{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create artifact root");
    root
}

#[test]
fn browser_appearance_and_template_target_deck_survive_lower_normalize_and_build() {
    let lowering = ProductDocument::new("demo-doc")
        .with_default_deck("Default")
        .with_basic("basic-main")
        .with_field_metadata(
            "basic-main",
            FieldMetadataDeclaration {
                field_name: "Front".into(),
                label: Some("Prompt".into()),
                role_hint: Some("question".into()),
            },
        )
        .with_browser_appearance(
            "basic-main",
            TemplateBrowserAppearanceDeclaration {
                template_name: "Card 1".into(),
                question_format: Some("<span class=\"browser-front\">{{Front}}</span>".into()),
                answer_format: Some("<span class=\"browser-back\">{{Back}}</span>".into()),
                font_name: Some("Arial".into()),
                font_size: Some(18),
            },
        )
        .with_template_target_deck(
            "basic-main",
            TemplateTargetDeckDeclaration {
                template_name: "Card 1".into(),
                deck_name: "Custom::Deck".into(),
            },
        )
        .add_basic_note("basic-main", "note-1", "Default", "front", "back")
        .lower()
        .expect("lower product document");

    assert_eq!(lowering.authoring_document.notes[0].deck_name, "Default");
    let lowered_notetype = &lowering.authoring_document.notetypes[0];
    assert_eq!(lowered_notetype.field_metadata.len(), 1);
    assert_eq!(
        lowered_notetype.field_metadata[0].role_hint.as_deref(),
        Some("question")
    );
    let lowered_template = lowered_notetype
        .templates
        .as_ref()
        .expect("lowered templates")
        .first()
        .expect("template");
    assert_eq!(
        lowered_template.browser_question_format.as_deref(),
        Some("<span class=\"browser-front\">{{Front}}</span>")
    );
    assert_eq!(
        lowered_template.target_deck_name.as_deref(),
        Some("Custom::Deck")
    );

    let normalized = normalize(NormalizationRequest::new(lowering.authoring_document));
    let normalized = normalized.normalized_ir.expect("normalized ir");
    let normalized_notetype = &normalized.notetypes[0];
    assert_eq!(normalized_notetype.field_metadata.len(), 1);
    assert_eq!(
        normalized_notetype.field_metadata[0].label.as_deref(),
        Some("Prompt")
    );
    let normalized_template = &normalized_notetype.templates[0];
    assert_eq!(
        normalized_template.browser_question_format.as_deref(),
        Some("<span class=\"browser-front\">{{Front}}</span>")
    );
    assert_eq!(
        normalized_template.target_deck_name.as_deref(),
        Some("Custom::Deck")
    );

    let root = unique_artifact_root("product-pipeline");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts");
    let build_result = build(
        &normalized,
        &WriterPolicy {
            id: "writer-policy.default".into(),
            version: "1.0.0".into(),
            compatibility_target: "latest-only".into(),
            stock_notetype_mode: "source-grounded".into(),
            media_entry_mode: "inline".into(),
            apkg_version: "latest".into(),
        },
        &BuildContext {
            id: "build-context.default".into(),
            version: "1.0.0".into(),
            emit_apkg: false,
            materialize_staging: true,
            media_resolution_mode: "inline-only".into(),
            unresolved_asset_behavior: "fail".into(),
            fingerprint_mode: "canonical".into(),
        },
        &target,
    )
    .expect("build should succeed");

    assert_eq!(
        build_result.result_status, "success",
        "build diagnostics: {:#?}",
        build_result.diagnostics
    );
    let report = inspect_staging(target.staging_manifest_path()).expect("inspect staging");
    assert!(report
        .observations
        .field_metadata
        .iter()
        .any(|value| value["field_name"] == "Front" && value["label"] == "Prompt"));
    assert!(report
        .observations
        .browser_templates
        .iter()
        .any(|value| value["template_name"] == "Card 1"
            && value["browser_font_name"] == "Arial"
            && value["browser_font_size"] == 18));
    assert!(report
        .observations
        .template_target_decks
        .iter()
        .any(|value| value["template_name"] == "Card 1"
            && value["target_deck_name"] == "Custom::Deck"
            && value["resolved_target_deck_id"].is_number()));
}
