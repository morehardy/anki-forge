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

#[test]
fn update_output_cannot_replace_baseline_or_its_file_aliases() {
    let cwd = std::env::current_dir().unwrap();
    let root = tempfile::tempdir_in(&cwd).unwrap();
    let baseline = root.path().join("baseline.apkg");
    let original_project = project();
    original_project.build(BuildOptions::to(&baseline)).unwrap();
    let original = std::fs::read(&baseline).unwrap();
    let hardlink = root.path().join("hardlink.apkg");
    std::fs::hard_link(&baseline, &hardlink).unwrap();
    let mut aliases = vec![
        baseline.clone(),
        baseline.strip_prefix(&cwd).unwrap().to_owned(),
        root.path().join(".").join("baseline.apkg"),
        hardlink,
        root.path().join("missing").join("..").join("baseline.apkg"),
    ];
    #[cfg(unix)]
    {
        let symlink = root.path().join("symlink.apkg");
        std::os::unix::fs::symlink(&baseline, &symlink).unwrap();
        aliases.push(symlink);
    }
    let mut changed = original_project;
    changed
        .add("new-note", Note::basic("new", "answer"))
        .unwrap();
    for alias in aliases {
        let error = changed
            .build(BuildOptions::to(&alias).update_from(&baseline))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            ankiforge::build::BuildErrorKind::Configuration
        );
        assert_eq!(error.code(), "BUILD.OUTPUT_INVALID");
        assert!(error.publications().is_empty());
        assert_eq!(
            error.report().counts().notes,
            0,
            "reject before generating a candidate"
        );
        assert_eq!(std::fs::read(&baseline).unwrap(), original);
        if alias.exists() {
            assert_eq!(std::fs::read(&alias).unwrap(), original);
        }
    }
    assert!(!root.path().join("missing").exists());
    let next = changed
        .build(BuildOptions::to(root.path().join("next.apkg")).update_from(&baseline))
        .unwrap();
    assert_eq!(next.report().counts().notes, 2);
    assert_eq!(std::fs::read(&baseline).unwrap(), original);
}

#[test]
fn temporary_persistence_rejects_aliases_before_creating_directories() {
    let root = tempfile::tempdir().unwrap();
    let output = project().build(BuildOptions::temporary()).unwrap();
    let artifact = output.artifact();
    let original = std::fs::read(artifact.path()).unwrap();
    let missing = artifact.path().with_extension("missing-directory");
    let prospective = missing
        .join("..")
        .join(artifact.path().file_name().unwrap());
    let hardlink = root.path().join("hardlink.apkg");
    std::fs::hard_link(artifact.path(), &hardlink).unwrap();
    let mut aliases = vec![
        artifact.path().to_owned(),
        prospective,
        hardlink,
        root.path().join("missing").join("..").join("hardlink.apkg"),
    ];
    #[cfg(unix)]
    {
        let symlink = root.path().join("symlink.apkg");
        std::os::unix::fs::symlink(artifact.path(), &symlink).unwrap();
        aliases.push(symlink);
    }
    for alias in aliases {
        let error = artifact.persist_to(&alias).unwrap_err();
        assert_eq!(
            error.kind(),
            ankiforge::build::PersistErrorKind::InvalidDestination
        );
        assert_eq!(error.publication().stage, PublicationStage::NotPublished);
        assert!(
            !missing.exists(),
            "alias rejection must have no filesystem side effects"
        );
        assert!(!root.path().join("missing").exists());
        assert_eq!(std::fs::read(artifact.path()).unwrap(), original);
    }
    // A new, genuinely distinct directory remains a valid destination and the
    // persistent artifact outlives the temporary source.
    let saved = artifact
        .persist_to(root.path().join("new").join("saved.apkg"))
        .unwrap();
    drop(output);
    assert_eq!(std::fs::read(saved.path()).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn persistence_resolves_symlinks_before_parent_components() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = root.path().join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("child")).unwrap();
    std::os::unix::fs::symlink(elsewhere.join("child"), root.path().join("link")).unwrap();
    let output = project()
        .build(BuildOptions::to(root.path().join("source.apkg")))
        .unwrap();
    let original = std::fs::read(output.artifact().path()).unwrap();
    let destination = root
        .path()
        .join("missing")
        .join("..")
        .join("link")
        .join("..")
        .join("source.apkg");
    let saved = output.artifact().persist_to(&destination).unwrap();
    drop(output);
    assert_eq!(
        std::fs::read(elsewhere.join("source.apkg")).unwrap(),
        original
    );
    assert_eq!(std::fs::read(saved.path()).unwrap(), original);
    assert_eq!(
        std::fs::read(root.path().join("source.apkg")).unwrap(),
        original
    );
}
