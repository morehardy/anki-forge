#![cfg(feature = "internal-tools")]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anki_forge::authoring::stock::resolve_stock_notetype;
use anki_forge::authoring::{
    AuthoringNotetype, MediaReference, MediaReferenceResolution, NormalizedFieldMetadata,
    NormalizedIr, NormalizedNote, NormalizedNotetype,
};
use anki_forge::writer::{
    build, diff_reports, inspect_apkg, inspect_staging, BuildArtifactTarget, BuildContext,
    InspectObservations, InspectReport, WriterPolicy,
};
use serde_json::json;
use sha1::Digest;

#[test]
fn diff_reports_between_staging_and_apkg_are_complete_and_empty_for_supported_fixture() {
    let root = unique_artifact_root("diff-semantic-consistency");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/diff-semantic-consistency");
    let normalized = sample_basic_normalized_ir_with_media(&target.media_store_dir);

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let left = inspect_staging(target.staging_manifest_path()).unwrap();
    let right = inspect_apkg(root.join("package.apkg")).unwrap();
    let diff = diff_reports(&left, &right).unwrap();

    assert_eq!(diff.comparison_status, "complete");
    assert!(diff.uncompared_domains.is_empty());
    assert!(diff.comparison_limitations.is_empty());
    assert!(diff.changes.is_empty(), "{:#?}", diff.changes);
}

#[test]
fn staging_and_apkg_agree_on_browser_template_sentinels() {
    for question in [None, Some(""), Some("{{Front}}"), Some(" \t\n")] {
        for answer in [None, Some(""), Some("{{Back}}"), Some(" \t\n")] {
            for font in [None, Some(""), Some("Arial"), Some(" \t\n")] {
                for size in [None, Some(0), Some(18)] {
                    let root = tempfile::tempdir().unwrap();
                    let target = BuildArtifactTarget::new(
                        root.path().to_path_buf(),
                        "artifacts/phase3/diff-browser-sentinels",
                    );
                    let mut normalized = sample_basic_normalized_ir();
                    let template = &mut normalized.notetypes[0].templates[0];
                    template.browser_question_format = question.map(str::to_owned);
                    template.browser_answer_format = answer.map(str::to_owned);
                    template.browser_font_name = font.map(str::to_owned);
                    template.browser_font_size = size;
                    let input = json!([question, answer, font, size]);

                    build(
                        &normalized,
                        &sample_writer_policy(),
                        &sample_build_context(true),
                        &target,
                    )
                    .unwrap();
                    let staging = inspect_staging(target.staging_manifest_path()).unwrap();
                    let apkg = inspect_apkg(root.path().join("package.apkg")).unwrap();
                    assert_eq!(
                        staging.observations.browser_templates, apkg.observations.browser_templates,
                        "{input}"
                    );
                    for (before, after) in [(&staging, &apkg), (&apkg, &staging)] {
                        let diff = diff_reports(before, after).unwrap();
                        assert_eq!(diff.comparison_status, "complete", "{input}");
                        assert!(diff.changes.is_empty(), "{input}: {:#?}", diff.changes);
                    }

                    // Empty/zero sentinels mean no override; whitespace is significant.
                    let expected = json!({
                        "browser_question_format": match question {
                            Some("") => None,
                            value => value,
                        },
                        "browser_answer_format": match answer {
                            Some("") => None,
                            value => value,
                        },
                        "browser_font_name": match font {
                            Some("") => None,
                            value => value,
                        },
                        "browser_font_size": match size {
                            Some(0) => None,
                            value => value,
                        },
                    });
                    let has_override = expected
                        .as_object()
                        .unwrap()
                        .values()
                        .any(|value| !value.is_null());
                    for report in [&staging, &apkg] {
                        let entries = &report.observations.browser_templates;
                        assert_eq!(entries.len(), usize::from(has_override), "{input}");
                        if has_override {
                            for (key, value) in expected.as_object().unwrap() {
                                assert_eq!(&entries[0][key], value, "{input}: {key}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn staging_and_apkg_agree_on_canonical_deck_names() {
    for (requested, canonical) in [
        (" Biology ", "Biology"),
        (" Cafe\u{301} :: Re\u{301}vision ", "Café::Révision"),
        (" English :::: Listening ", "English::blank::Listening"),
        ("fo\u{1f}o::ba\nr", "foo::bar"),
    ] {
        for template_override in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let target = BuildArtifactTarget::new(
                root.path().to_path_buf(),
                "artifacts/phase3/diff-canonical-decks",
            );
            let mut normalized = sample_basic_normalized_ir();
            normalized.notes[0].deck_name = requested.into();
            if template_override {
                normalized.notetypes[0].templates[0].target_deck_name = Some(requested.into());
            }

            let result = build(
                &normalized,
                &sample_writer_policy(),
                &sample_build_context(true),
                &target,
            )
            .unwrap();
            assert_eq!(result.result_status, "success");

            let staging = inspect_staging(target.staging_manifest_path()).unwrap();
            let apkg = inspect_apkg(root.path().join("package.apkg")).unwrap();
            let diff = diff_reports(&staging, &apkg).unwrap();
            assert_eq!(diff.comparison_status, "complete");
            assert!(
                diff.changes.is_empty(),
                "{requested:?}, template override {template_override}: {:#?}",
                diff.changes
            );
            for report in [&staging, &apkg] {
                for selector in ["note[id='note-1']", "card[note_id='note-1'][ord=0]"] {
                    let entry = report
                        .observations
                        .references
                        .iter()
                        .find(|entry| entry["selector"] == selector)
                        .unwrap();
                    assert_eq!(entry["deck_name"], canonical);
                }
                if template_override {
                    assert_eq!(
                        report.observations.template_target_decks[0]["target_deck_name"],
                        canonical
                    );
                }
            }
        }
    }
}

#[test]
fn staging_and_apkg_agree_on_deck_aliases_with_template_overrides() {
    let root = tempfile::tempdir().unwrap();
    let target = BuildArtifactTarget::new(
        root.path().to_path_buf(),
        "artifacts/phase3/diff-deck-aliases",
    );
    let mut normalized = sample_basic_normalized_ir();
    let note = normalized.notes[0].clone();
    normalized.notes = [
        "Foo",
        "foo::Child",
        "FOO::child",
        "default::Child",
        "DEFAULT::child",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, deck_name)| {
        let mut note = note.clone();
        note.id = format!("note-{index}");
        note.deck_name = deck_name.into();
        note
    })
    .collect();
    // Keep the first card in the note deck so APKG note-deck reconstruction is exact.
    normalized.notetypes[0].kind = "normal".into();
    normalized.notetypes[0].original_stock_kind = None;
    let mut template = normalized.notetypes[0].templates[0].clone();
    template.name = "Override".into();
    template.ord = Some(1);
    template.target_deck_name = Some("foo::Child".into());
    normalized.notetypes[0].templates.push(template);

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();
    assert_eq!(result.result_status, "success");
    let staging = inspect_staging(target.staging_manifest_path()).unwrap();
    let apkg = inspect_apkg(root.path().join("package.apkg")).unwrap();
    let diff = diff_reports(&staging, &apkg).unwrap();
    assert_eq!(diff.comparison_status, "complete");
    assert!(diff.changes.is_empty(), "{:#?}", diff.changes);
    for report in [&staging, &apkg] {
        assert_eq!(
            report.observations.template_target_decks[0]["target_deck_name"],
            "Foo::child"
        );
        let overridden_cards = report
            .observations
            .references
            .iter()
            .filter(|entry| entry["template_name"] == "Override")
            .collect::<Vec<_>>();
        assert_eq!(overridden_cards.len(), normalized.notes.len());
        assert!(overridden_cards
            .iter()
            .all(|entry| entry["deck_name"] == "Foo::child"));
    }
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

#[test]
fn legacy_staging_recovers_positional_ids_but_explicit_empty_plan_is_invalid() {
    let root = tempfile::tempdir().unwrap();
    let target = BuildArtifactTarget::new(root.path(), "artifacts");
    build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();
    let path = target.staging_manifest_path();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    manifest
        .as_object_mut()
        .unwrap()
        .remove("notetype_model_ids");
    fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let legacy = inspect_staging(&path).unwrap();
    assert_eq!(legacy.observations.notetypes[0]["anki_model_id"], 1);
    manifest["notetype_model_ids"] = json!({});
    fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let error = inspect_staging(&path).unwrap_err();
    assert!(error
        .to_string()
        .contains("UPDATE.WRITER_NOTETYPE_ID_PLAN_MISMATCH"));
}

#[test]
fn diff_reports_only_strip_media_provenance_from_media_domain() {
    let left = sample_inspect_report("Basic");
    let mut right = left.clone();
    right.observations.references[0]["object_id"] = json!("semantic-object-id");

    let diff = diff_reports(&left, &right).unwrap();

    assert_eq!(diff.changes.len(), 1);
    assert_eq!(diff.changes[0].domain, "references");
    assert_eq!(diff.changes[0].selector, "note[id='note-1']");
}

#[test]
fn diff_covers_field_metadata_browser_templates_and_target_decks() {
    let left = sample_inspect_report("Basic");
    let mut right = left.clone();
    right.observations.field_metadata.push(json!({
        "selector": "notetype[id='basic-main']::field-metadata[Front]",
        "label": "Prompt",
    }));
    right.observations.browser_templates.push(json!({
        "selector": "notetype[id='basic-main']::browser-template[Card 1]",
        "browser_question_format": "{{Front}}",
    }));
    right.observations.template_target_decks.push(json!({
        "selector": "notetype[id='basic-main']::template-target-deck[Card 1]",
        "target_deck_name": "Study",
        "resolved_target_deck_id": 42,
    }));
    for (before, after, category) in [(&left, &right, "added"), (&right, &left, "removed")] {
        let diff = diff_reports(before, after).unwrap();
        for domain in [
            "field_metadata",
            "browser_templates",
            "template_target_decks",
        ] {
            assert!(
                diff.changes
                    .iter()
                    .any(|change| change.domain == domain && change.category == category),
                "missing {domain}: {diff:#?}"
            );
        }
    }
    let mut modified = right.clone();
    modified.observations.field_metadata[0]["label"] = json!("Question");
    modified.observations.browser_templates[0]["browser_question_format"] =
        json!("<b>{{Front}}</b>");
    modified.observations.template_target_decks[0]["resolved_target_deck_id"] = json!(43);
    let diff = diff_reports(&right, &modified).unwrap();
    assert_eq!(diff.changes.len(), 3);
    assert!(diff
        .changes
        .iter()
        .all(|change| change.category == "modified"));
}

#[test]
fn diff_marks_unavailable_extended_domains_and_unknown_domains_as_partial() {
    for domain in [
        "field_metadata",
        "browser_templates",
        "template_target_decks",
        "future_domain",
    ] {
        let left = sample_inspect_report("Basic");
        let mut right = left.clone();
        right.missing_domains.push(domain.into());
        let diff = diff_reports(&left, &right).unwrap();
        assert_eq!(diff.comparison_status, "partial", "{domain}");
        assert!(diff.uncompared_domains.contains(&domain.to_string()));
        assert_ne!(diff.summary, "no compatibility-significant changes");
    }
}

#[test]
fn extended_metadata_changes_survive_apkg_roundtrip_and_reach_risk_report() {
    let root = tempfile::tempdir().unwrap();
    let mut before = sample_basic_normalized_ir();
    before.notetypes[0].field_metadata = vec![NormalizedFieldMetadata {
        field_name: "Front".into(),
        label: Some("Prompt".into()),
        role_hint: Some("question".into()),
    }];
    before.notetypes[0].templates[0].browser_question_format = Some("{{Front}}".into());
    before.notetypes[0].templates[0].target_deck_name = Some("Deck A".into());
    // Keep note and actual card deck observations aligned as well.
    before.notes[0].deck_name = "Deck A".into();
    let mut after = before.clone();
    after.notetypes[0].field_metadata[0].label = Some("Question".into());
    after.notetypes[0].templates[0].browser_question_format = Some("<b>{{Front}}</b>".into());
    after.notetypes[0].templates[0].target_deck_name = Some("Deck B".into());
    after.notes[0].deck_name = "Deck B".into();
    let mut reports = Vec::new();
    for (name, normalized) in [("before", before), ("after", after)] {
        let target = BuildArtifactTarget::new(root.path().join(name), "artifacts");
        build(
            &normalized,
            &sample_writer_policy(),
            &sample_build_context(true),
            &target,
        )
        .unwrap();
        let apkg = inspect_apkg(root.path().join(name).join("package.apkg")).unwrap();
        let staging = inspect_staging(target.staging_manifest_path()).unwrap();
        let roundtrip = diff_reports(&staging, &apkg).unwrap();
        assert!(roundtrip.changes.is_empty(), "{roundtrip:#?}");
        reports.push(apkg);
    }
    let diff = diff_reports(&reports[0], &reports[1]).unwrap();
    for domain in [
        "field_metadata",
        "browser_templates",
        "template_target_decks",
    ] {
        assert!(
            diff.changes.iter().any(|change| change.domain == domain),
            "{diff:#?}"
        );
    }
    let summary = anki_forge::diff::summarize_writer_diff(&diff);
    let risk = anki_forge::risk::rules::classify_import_risk(anki_forge::risk::rules::RiskInput {
        diagnostics: &[],
        comparison: anki_forge::build::ComparisonStatus::Complete,
        diff: Some(&summary),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });
    assert!(risk.findings.iter().any(|finding| finding.code
        == "RISK.TEMPLATE_TARGET_DECK_CHANGED"
        && finding.level == anki_forge::build::RiskLevel::Medium));
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
        media_resolution_mode: "pre-resolved".into(),
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
            mtime_secs: None,
        }],
        media_objects: vec![],
        media_bindings: vec![],
        media_references: vec![],
    }
}

fn sample_basic_normalized_ir_with_media(media_store: &Path) -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    let bytes = b"hello";
    let blake3_hex = blake3::hash(bytes).to_hex().to_string();
    let sha1_hex = hex::encode(sha1::Sha1::digest(bytes));
    let object_id = format!("obj:blake3:{blake3_hex}");
    let object_path = anki_forge::authoring::object_store_path(media_store, &blake3_hex).unwrap();
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(&object_path, bytes).unwrap();
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="sample.jpg">"#.into());
    normalized.media_objects = vec![anki_forge::authoring::MediaObject {
        id: object_id.clone(),
        object_ref: format!("media://blake3/{blake3_hex}"),
        blake3: blake3_hex,
        sha1: sha1_hex,
        size_bytes: bytes.len() as u64,
        mime: "image/jpeg".into(),
    }];
    normalized.media_bindings = vec![anki_forge::authoring::MediaBinding {
        id: "media:sample".into(),
        export_filename: "sample.jpg".into(),
        object_id,
    }];
    normalized.media_references = vec![MediaReference {
        owner_kind: "note".into(),
        owner_id: "note-1".into(),
        location_kind: "field".into(),
        location_name: "Back".into(),
        raw_ref: "sample.jpg".into(),
        ref_kind: "html_src".into(),
        resolution: MediaReferenceResolution::Resolved {
            media_id: "media:sample".into(),
        },
    }];
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
                "kind": "normal",
                "original_stock_kind": "basic",
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
            field_metadata: vec![],
            browser_templates: vec![],
            template_target_decks: vec![],
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
        original_stock_kind: None,
        original_id: None,
        fields: None,
        templates: None,
        css: None,
        field_metadata: vec![],
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
