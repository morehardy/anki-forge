use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{
    AuthoringNotetype, NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype,
};
use serde_json::json;
use writer_core::{
    build, diff_reports, inspect_apkg, inspect_staging, BuildArtifactTarget, BuildContext,
    InspectObservations, InspectReport, WriterPolicy,
};

#[test]
fn diff_reports_between_staging_and_apkg_are_complete_and_empty_for_supported_fixture() {
    let root = unique_artifact_root("diff-semantic-consistency");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/diff-semantic-consistency");

    build(
        &sample_basic_normalized_ir_with_media(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let left = inspect_staging(&target.staging_manifest_path()).unwrap();
    let right = inspect_apkg(&root.join("package.apkg")).unwrap();
    let diff = diff_reports(&left, &right).unwrap();

    assert_eq!(diff.comparison_status, "complete");
    assert!(diff.uncompared_domains.is_empty());
    assert!(diff.comparison_limitations.is_empty());
    assert!(diff.changes.is_empty());
}

#[test]
fn diff_reports_emit_stable_selector_and_evidence_refs_for_domain_changes() {
    let left = sample_inspect_report("Basic");
    let mut right = left.clone();
    right.observations.notetypes[0]["name"] = json!("Renamed Basic");

    let diff = diff_reports(&left, &right).unwrap();

    assert_eq!(diff.comparison_status, "complete");
    let change = diff.changes.first().expect("expected one change");
    assert_eq!(change.domain, "notetypes");
    assert_eq!(change.selector, "notetype[id='basic-main']");
    assert!(!change.evidence_refs.is_empty());
}

fn sample_writer_policy() -> WriterPolicy {
    WriterPolicy {
        id: "writer-policy.default".into(),
        version: "1.0.0".into(),
        compatibility_target: "latest-only".into(),
        stock_notetype_mode: "source-grounded".into(),
        media_entry_mode: "inline".into(),
        apkg_version: "latest".into(),
    }
}

fn sample_build_context(emit_apkg: bool) -> BuildContext {
    BuildContext {
        id: "build-context.default".into(),
        version: "1.0.0".into(),
        emit_apkg,
        materialize_staging: true,
        media_resolution_mode: "inline-only".into(),
        unresolved_asset_behavior: "fail".into(),
        fingerprint_mode: "canonical".into(),
    }
}

fn sample_basic_normalized_ir() -> NormalizedIr {
    NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: "0.1.0".into(),
        document_id: "demo-doc".into(),
        resolved_identity: "document:demo-doc".into(),
        notetypes: vec![resolved_stock_notetype("basic-main", "basic", "Basic")],
        notes: vec![NormalizedNote {
            id: "note-1".into(),
            notetype_id: "basic-main".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                ("Front".into(), "front".into()),
                ("Back".into(), "back".into()),
            ]),
            tags: vec!["demo".into()],
        }],
        media: vec![],
    }
}

fn sample_basic_normalized_ir_with_media() -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="sample.jpg">"#.into());
    normalized.media.push(NormalizedMedia {
        filename: "sample.jpg".into(),
        mime: "image/jpeg".into(),
        data_base64: "aGVsbG8=".into(),
    });
    normalized
}

fn sample_inspect_report(name: &str) -> InspectReport {
    InspectReport {
        kind: "inspect-report".into(),
        observation_model_version: "phase3-inspect-v1".into(),
        source_kind: "staging".into(),
        source_ref: "artifacts/phase3/demo/staging/manifest.json".into(),
        artifact_fingerprint: "artifact:demo".into(),
        observation_status: "complete".into(),
        missing_domains: vec![],
        degradation_reasons: vec![],
        observations: InspectObservations {
            notetypes: vec![json!({
                "selector": "notetype[id='basic-main']",
                "name": name,
                "kind": "basic",
                "evidence_refs": ["staging:manifest", "collection:notetypes"],
            })],
            templates: vec![json!({
                "selector": "notetype[id='basic-main']::template[0]",
                "name": "Card 1",
                "evidence_refs": ["staging:manifest", "collection:templates"],
            })],
            fields: vec![json!({
                "selector": "notetype[id='basic-main']::field[Front]",
                "name": "Front",
                "evidence_refs": ["staging:manifest", "collection:fields"],
            })],
            media: vec![json!({
                "selector": "media[filename='sample.jpg']",
                "filename": "sample.jpg",
                "evidence_refs": ["staging:manifest", "collection:media"],
            })],
            metadata: vec![json!({
                "selector": "counts",
                "notetype_count": 1,
                "note_count": 1,
                "card_count": 1,
                "media_count": 1,
                "evidence_refs": ["manifest:counts", "collection:counts"],
            })],
            references: vec![json!({
                "selector": "note[id='note-1']",
                "kind": "note",
                "evidence_refs": ["collection:notes"],
            })],
        },
    }
}

fn resolved_stock_notetype(id: &str, kind: &str, name: &str) -> NormalizedNotetype {
    let mut notetype = resolve_stock_notetype(&AuthoringNotetype {
        id: id.into(),
        kind: kind.into(),
        name: Some(name.into()),
    })
    .expect("resolve stock notetype");
    notetype.id = id.into();
    notetype
}

fn unique_artifact_root(case: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "anki-forge-phase3-{case}-{}-{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}
