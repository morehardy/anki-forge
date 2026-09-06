#![cfg(feature = "internal-tools")]

use std::fs;

use anki_forge::authoring::NormalizedIr;
use anki_forge::writer::{
    apkg::emit_apkg, build, inspect_apkg, BuildArtifactTarget, BuildContext, StagingPackage,
    WriterGuidPlan, WriterPolicy,
};
use serde_json::json;

fn basic() -> NormalizedIr {
    serde_json::from_str(include_str!(
        "../../contracts/fixtures/phase3/inputs/basic-normalized-ir.json"
    ))
    .unwrap()
}

fn defaults() -> (WriterPolicy, BuildContext) {
    (
        serde_yaml::from_str(include_str!(
            "../../contracts/policies/writer-policy.default.yaml"
        ))
        .unwrap(),
        serde_yaml::from_str(include_str!(
            "../../contracts/contexts/build-context.default.yaml"
        ))
        .unwrap(),
    )
}

#[test]
fn staging_package_owns_a_snapshot_after_the_source_is_changed_and_dropped() {
    let package: StagingPackage = {
        let mut normalized = basic();
        let (policy, context) = defaults();
        let package = StagingPackage::from_normalized(&normalized, &policy, &context).unwrap();
        normalized.notes[0]
            .fields
            .insert("Front".into(), "changed after staging".into());
        normalized.notetypes.clear();
        drop(normalized);
        package
    };
    let root = tempfile::tempdir().unwrap();
    let target = BuildArtifactTarget::new(root.path(), "artifacts");
    let staging = package.materialize(&target).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&staging.manifest_path).unwrap()).unwrap();

    assert_eq!(
        manifest["normalized_ir"],
        serde_json::to_value(basic()).unwrap()
    );
    emit_apkg(&staging, &target, None).expect("the owned snapshot remains exportable");
}

#[test]
fn repeated_exports_support_native_artifact_paths() {
    let root = tempfile::tempdir().unwrap();
    let paths = vec![root.path().join("引号 ' and spaces")];
    // Linux filesystems support native names which are not valid UTF-8.
    #[cfg(target_os = "linux")]
    let paths = {
        use std::os::unix::ffi::OsStringExt;
        let mut paths = paths;
        paths.push(root.path().join(std::ffi::OsString::from_vec(
            b"native-\xff-artifacts".to_vec(),
        )));
        paths
    };
    let (policy, context) = defaults();
    for path in paths {
        let target = BuildArtifactTarget::new(&path, "artifacts");
        let mut normalized = basic();
        for front in ["first export", "updated export"] {
            normalized.notes[0]
                .fields
                .insert("Front".into(), front.into());
            let result = build(&normalized, &policy, &context, &target).unwrap();
            assert_eq!(result.result_status, "success", "{:?}", result.diagnostics);
            let report = inspect_apkg(path.join("package.apkg")).unwrap();
            let note = report
                .observations
                .references
                .iter()
                .find(|entry| entry["id"] == normalized.notes[0].id)
                .expect("exported note must remain readable");
            assert_eq!(note["fields"]["Front"], front);
        }
    }
}

#[test]
fn typed_and_materialized_writers_preserve_staging_and_package_bytes() {
    let mut custom = basic();
    let notetype = &mut custom.notetypes[0];
    notetype.original_stock_kind = None;
    let mut reverse = notetype.templates[0].clone();
    reverse.name = "Reverse".into();
    reverse.ord = Some(7);
    reverse.question_format = "{{Back}}".into();
    reverse.answer_format = "{{Front}}".into();
    notetype.templates.push(reverse);
    let cloze: NormalizedIr = serde_json::from_str(include_str!(
        "../../contracts/fixtures/phase3/inputs/cloze-normalized-ir.json"
    ))
    .unwrap();
    let (policy, context) = defaults();

    for normalized in [basic(), cloze, custom] {
        let root = tempfile::tempdir().unwrap();
        let typed_target = BuildArtifactTarget::new(root.path().join("typed"), "artifacts");
        let disk_target = BuildArtifactTarget::new(root.path().join("disk"), "artifacts");
        let typed = build(&normalized, &policy, &context, &typed_target).unwrap();
        assert_eq!(typed.result_status, "success", "{:?}", typed.diagnostics);
        let package = StagingPackage::from_normalized(&normalized, &policy, &context).unwrap();
        let staging = package.materialize(&disk_target).unwrap();
        let disk = emit_apkg(&staging, &disk_target, None).unwrap();

        assert_eq!(
            typed.staging_ref.as_deref(),
            Some(staging.manifest_ref.as_str())
        );
        assert_eq!(
            typed.artifact_fingerprint.as_deref(),
            Some(staging.artifact_fingerprint.as_str())
        );
        assert_eq!(
            typed.package_fingerprint.as_deref(),
            Some(disk.package_fingerprint.as_str())
        );
        assert_eq!(
            fs::read(typed_target.staging_manifest_path()).unwrap(),
            fs::read(staging.manifest_path).unwrap()
        );
        assert_eq!(
            fs::read(typed_target.root_dir.join("package.apkg")).unwrap(),
            fs::read(disk.apkg_path).unwrap()
        );
    }
}

#[test]
fn materialized_writer_preserves_legacy_ids_and_rejects_invalid_identity_plans() {
    let root = tempfile::tempdir().unwrap();
    let target = BuildArtifactTarget::new(root.path(), "artifacts");
    let (policy, context) = defaults();
    let mut normalized = basic();
    let mut other = normalized.notetypes[0].clone();
    other.id = "basic-other".into();
    other.name = "Other Basic".into();
    normalized.notetypes.push(other);
    let package = StagingPackage::from_normalized(&normalized, &policy, &context).unwrap();
    let staging = package.materialize(&target).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&staging.manifest_path).unwrap()).unwrap();
    manifest
        .as_object_mut()
        .unwrap()
        .remove("notetype_model_ids");
    fs::write(
        &staging.manifest_path,
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();

    let legacy = emit_apkg(&staging, &target, None).unwrap();
    let inspected = inspect_apkg(&legacy.apkg_path).unwrap();
    let ids: Vec<_> = inspected
        .observations
        .notetypes
        .iter()
        .map(|notetype| notetype["anki_model_id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, [1, 2]);

    manifest["notetype_model_ids"] = json!({"basic-main": 501, "basic-other": 502});
    fs::write(
        &staging.manifest_path,
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let explicit = emit_apkg(&staging, &target, None).unwrap();
    let inspected = inspect_apkg(&explicit.apkg_path).unwrap();
    let ids: Vec<_> = inspected
        .observations
        .notetypes
        .iter()
        .map(|notetype| notetype["anki_model_id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, [501, 502]);
    let previous_package = fs::read(&explicit.apkg_path).unwrap();

    for (ids, expected_code) in [
        (json!({}), "UPDATE.WRITER_NOTETYPE_ID_PLAN_MISMATCH"),
        (
            json!({"basic-main": 0, "basic-other": 502}),
            "UPDATE.WRITER_NOTETYPE_ID_PLAN_MISMATCH",
        ),
        (
            json!({"basic-main": 501, "basic-other": 501}),
            "UPDATE.NOTETYPE_MODEL_ID_COLLISION",
        ),
    ] {
        manifest["notetype_model_ids"] = ids;
        fs::write(
            &staging.manifest_path,
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let error = emit_apkg(&staging, &target, None)
            .err()
            .expect("invalid plan must fail");
        assert!(error.to_string().contains(expected_code), "{error}");
        assert_eq!(fs::read(&explicit.apkg_path).unwrap(), previous_package);
    }

    let error = emit_apkg(
        &staging,
        &target,
        Some(&WriterGuidPlan {
            assignments: vec![],
        }),
    )
    .err()
    .expect("GUID mismatch must precede model-plan mismatch");
    assert!(error
        .to_string()
        .contains("UPDATE.WRITER_GUID_PLAN_MISMATCH"));
    assert_eq!(fs::read(explicit.apkg_path).unwrap(), previous_package);
}

#[test]
fn build_rolls_back_sql_failure_and_does_not_replace_an_existing_package() {
    let root = tempfile::tempdir().unwrap();
    let target = BuildArtifactTarget::new(root.path(), "artifacts");
    let output = root.path().join("package.apkg");
    fs::write(&output, b"previous package").unwrap();
    let (policy, context) = defaults();
    let mut normalized = basic();
    let mut other = normalized.notetypes[0].clone();
    other.id = "basic-other".into();
    // Distinct model IDs with the same display name reach SQLite's unique constraint
    // after deck metadata and the first notetype have already been inserted.
    normalized.notetypes.push(other);

    let result = build(&normalized, &policy, &context, &target).unwrap();

    assert_eq!(result.result_status, "error");
    assert!(result.diagnostics.items.iter().any(|item| {
        item.code == "PHASE3.APKG_EMISSION_FAILED"
            && item
                .summary
                .contains("UNIQUE constraint failed: notetypes.name")
    }));
    assert_eq!(fs::read(output).unwrap(), b"previous package");
    let conn =
        rusqlite::Connection::open(root.path().join(".collection.anki21b.sqlite.tmp")).unwrap();
    for table in ["notetypes", "decks", "deck_config"] {
        let count: i64 = conn
            .query_row(&format!("select count(*) from {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "partial writes remain in {table}");
    }
}
