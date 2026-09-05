use anki_forge::prelude::*;

fn project() -> Project {
    let mut project = Project::new("Artifact lifetime").stable_id("artifact-lifetime");
    project
        .add_note(Note::basic("front", "back").stable_id("note-1"))
        .unwrap();
    project
}

#[test]
fn temporary_artifact_is_removed_only_after_the_last_owner_drops() {
    let report = project().build(BuildOptions::new()).unwrap();
    let report_clone = report.clone();
    let artifact = report.artifact.as_ref().unwrap().clone();
    let path = artifact.path().to_path_buf();
    drop(report);
    assert!(path.is_file());
    drop(artifact);
    assert!(path.is_file(), "a cloned report also owns the artifact");
    drop(report_clone);
    assert!(
        !path.exists(),
        "the final owner must clean up temporary output"
    );
}

#[test]
fn explicit_destinations_and_persisted_copies_survive_report_drop() {
    let root = tempfile::tempdir().unwrap();
    let report = project().build(BuildOptions::new()).unwrap();
    let temporary = report.artifact.as_ref().unwrap().path().to_path_buf();
    let saved = report
        .artifact
        .as_ref()
        .unwrap()
        .persist_to(root.path().join("saved.apkg"))
        .unwrap();
    assert_eq!(
        std::fs::read(&temporary).unwrap(),
        std::fs::read(saved.path()).unwrap()
    );
    drop(report);
    assert!(!temporary.exists());
    drop(saved);
    assert!(root.path().join("saved.apkg").is_file());
    for (artifacts, output) in [(true, false), (false, true), (true, true)] {
        let mut options = BuildOptions::new();
        if artifacts {
            options = options.artifacts_dir(root.path().join("artifacts"));
        }
        if output {
            options = options.output(root.path().join("explicit.apkg"));
        }
        let report = project().build(options).unwrap();
        let path = report.artifact.as_ref().unwrap().path().to_path_buf();
        drop(report);
        assert!(path.is_file());
        if artifacts {
            assert!(root.path().join("artifacts/package.apkg").is_file());
        }
    }
}

#[test]
fn automatic_json_report_requires_a_persistent_artifact_destination() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("report.json");
    let error = project()
        .build(BuildOptions::new().report_json(&path))
        .expect_err("JSON cannot own a temporary APKG lifetime");
    assert!(error.report.artifact.is_none());
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.REPORT_JSON_WRITE_FAILED".into()));
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert!(json["artifact"].is_null());
}

#[test]
fn published_temporary_artifact_is_owned_by_late_failure_reports() {
    let root = tempfile::tempdir().unwrap();
    let error = project()
        .build(
            BuildOptions::new()
                .identity_lockfile(root.path())
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::Disabled),
        )
        .unwrap_err();
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.LOCKFILE_WRITE_FAILED".into()));
    let path = error.report.artifact.as_ref().unwrap().path().to_path_buf();
    let cloned_error = error.clone();
    drop(error);
    assert!(path.is_file());
    drop(cloned_error);
    assert!(!path.exists());
}

#[test]
fn failed_persistence_keeps_temporary_artifact_usable() {
    let root = tempfile::tempdir().unwrap();
    let report = project().build(BuildOptions::new()).unwrap();
    let artifact = report.artifact.as_ref().unwrap();
    let bytes = std::fs::read(artifact.path()).unwrap();
    assert!(artifact.persist_to(root.path()).is_err());
    assert!(artifact.persist_to(artifact.path()).is_err());
    assert_eq!(std::fs::read(artifact.path()).unwrap(), bytes);
}
