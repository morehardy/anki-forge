use ankiforge::build::json::{BuildResultSnapshot, PublicationStage};
use ankiforge::{BuildOptions, Note, Project};

fn project() -> Project {
    let mut project = Project::new("artifact-lifetime").unwrap();
    project.add("note-1", Note::basic("front", "back")).unwrap();
    project
}

#[test]
fn temporary_artifact_is_removed_only_after_the_last_handle_drops() {
    let output = project().build(BuildOptions::temporary()).unwrap();
    let cloned = output.clone();
    let artifact = output.artifact().clone();
    let path = artifact.path().to_owned();
    drop(output);
    assert!(path.is_file());
    drop(artifact);
    assert!(path.is_file());
    drop(cloned);
    assert!(!path.exists());
}

#[test]
fn persistent_destinations_and_copies_survive_output_drop() {
    let root = tempfile::tempdir().unwrap();
    let output = project().build(BuildOptions::temporary()).unwrap();
    let temporary = output.artifact().path().to_owned();
    let saved = output
        .artifact()
        .persist_to(root.path().join("saved.apkg"))
        .unwrap();
    assert_eq!(
        std::fs::read(&temporary).unwrap(),
        std::fs::read(saved.path()).unwrap()
    );
    drop(output);
    assert!(!temporary.exists());
    drop(saved);
    assert!(root.path().join("saved.apkg").is_file());
    let output = project()
        .build(BuildOptions::to(root.path().join("explicit.apkg")))
        .unwrap();
    let path = output.artifact().path().to_owned();
    drop(output);
    assert!(path.is_file());
}

#[test]
fn reports_and_serialized_snapshots_do_not_retain_temporary_files() {
    let output = project().build(BuildOptions::temporary()).unwrap();
    let path = output.artifact().path().to_owned();
    let report = output.report().clone();
    let snapshot = output.snapshot();
    let serialized = serde_json::to_value(&snapshot).unwrap();
    drop(output);
    assert!(!path.exists());
    assert_eq!(report.counts().notes, 1);
    assert_eq!(serialized["result"]["status"], "success");
    assert!(matches!(
        snapshot.result,
        BuildResultSnapshot::Success {
            temporary: true,
            ..
        }
    ));
}

#[test]
fn failed_publication_preserves_existing_destination_and_complete_observations() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("existing-directory");
    std::fs::create_dir(&destination).unwrap();
    std::fs::write(destination.join("owned"), b"keep").unwrap();
    let error = project().build(BuildOptions::to(&destination)).unwrap_err();
    assert_eq!(error.report().counts().notes, 1);
    assert_eq!(
        error.publications()[0].stage,
        PublicationStage::NotPublished
    );
    assert_eq!(std::fs::read(destination.join("owned")).unwrap(), b"keep");
    assert!(matches!(
        error.snapshot().result,
        BuildResultSnapshot::Failure { .. }
    ));
}

#[test]
fn failed_persistence_keeps_temporary_artifact_usable() {
    let root = tempfile::tempdir().unwrap();
    let output = project().build(BuildOptions::temporary()).unwrap();
    let artifact = output.artifact();
    let bytes = std::fs::read(artifact.path()).unwrap();
    assert!(artifact.persist_to(root.path()).is_err());
    assert!(artifact.persist_to(artifact.path()).is_err());
    assert_eq!(std::fs::read(artifact.path()).unwrap(), bytes);
}
