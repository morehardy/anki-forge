use ankiforge::build::json::{BuildResultSnapshot, PublicationStage};
use ankiforge::{BuildOptions, Note, Project};

fn project() -> Project {
    let mut project = Project::new("artifact-lifetime").unwrap();
    project.add("note-1", Note::basic("front", "back")).unwrap();
    project
}

fn check_relative_artifact_after_chdir(test_name: &str, persist: bool) {
    // Changing cwd is process-wide, so exercise it in a child that runs only
    // this test, without interfering with the parallel integration suite.
    const CHILD: &str = "ANKIFORGE_ARTIFACT_CWD_CHILD";
    if std::env::var(CHILD).as_deref() != Ok(test_name) {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name, "--nocapture"])
            .env(CHILD, test_name)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "child failed: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    let original_cwd = std::env::current_dir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let before = root.path().join("before");
    let after = root.path().join("after");
    std::fs::create_dir_all(&before).unwrap();
    std::fs::create_dir_all(&after).unwrap();
    std::env::set_current_dir(&before).unwrap();
    let artifact = if persist {
        project()
            .build(BuildOptions::temporary())
            .unwrap()
            .artifact()
            .persist_to("saved.apkg")
            .unwrap()
    } else {
        project()
            .build(BuildOptions::to("saved.apkg"))
            .unwrap()
            .artifact()
            .clone()
    };
    let original = std::fs::read(before.join("saved.apkg")).unwrap();
    std::fs::write(after.join("saved.apkg"), b"unrelated file").unwrap();
    std::env::set_current_dir(&after).unwrap();
    assert!(
        std::fs::read(artifact.path()).unwrap() == original,
        "artifact followed cwd to the unrelated file"
    );
    assert!(artifact.path().is_absolute());
    let cloned = artifact.clone();
    drop(artifact);
    let copied = cloned.persist_to("copied.apkg").unwrap();
    assert_eq!(std::fs::read(copied.path()).unwrap(), original);
    std::env::set_current_dir(root.path()).unwrap();
    assert_eq!(std::fs::read(copied.path()).unwrap(), original);
    assert_eq!(std::fs::read(cloned.path()).unwrap(), original);
    assert_eq!(
        std::fs::read(after.join("saved.apkg")).unwrap(),
        b"unrelated file"
    );
    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn relative_build_artifact_survives_chdir() {
    check_relative_artifact_after_chdir("relative_build_artifact_survives_chdir", false);
}

#[test]
fn relative_persist_artifact_survives_chdir() {
    check_relative_artifact_after_chdir("relative_persist_artifact_survives_chdir", true);
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
    assert_eq!(
        artifact.persist_to("").unwrap_err().kind(),
        ankiforge::build::PersistErrorKind::InvalidDestination
    );
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
    let cwd = std::env::current_dir().unwrap();
    let root = tempfile::tempdir_in(&cwd).unwrap();
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
    let saved = output
        .artifact()
        .persist_to(destination.strip_prefix(&cwd).unwrap())
        .unwrap();
    assert!(saved.path().is_absolute());
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

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn non_unicode_success_paths_are_lossless_json_after_publication() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let root = tempfile::tempdir().unwrap();
    let path = root
        .path()
        .join(std::ffi::OsString::from_vec(b"saved-\xff.apkg".to_vec()));
    let output = project().build(BuildOptions::to(&path)).unwrap();
    assert!(path.is_file());
    assert_eq!(output.artifact().path(), path);
    let value = serde_json::to_value(output.snapshot()).unwrap();
    assert_eq!(
        value["result"]["artifact"],
        serde_json::json!({"encoding":"unix_bytes", "bytes":path.as_os_str().as_bytes()})
    );
    let bytes =
        serde_json::from_value::<Vec<u8>>(value["result"]["artifact"]["bytes"].clone()).unwrap();
    let restored = std::path::PathBuf::from(std::ffi::OsString::from_vec(bytes));
    assert_eq!(restored, path);
    drop(output);
    assert!(restored.is_file());
}

#[cfg(unix)]
#[test]
fn non_unicode_failure_publications_are_lossless_json() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let root = tempfile::tempdir().unwrap();
    let path = root
        .path()
        .join(std::ffi::OsString::from_vec(b"blocked-\xfe".to_vec()));
    // APFS rejects invalid UTF-8 names itself; byte-oriented Unix filesystems
    // need an existing directory here to force the same pre-publication error.
    #[cfg(not(target_os = "macos"))]
    std::fs::create_dir(&path).unwrap();
    let error = project().build(BuildOptions::to(&path)).unwrap_err();
    assert_eq!(
        error.publications()[0].stage,
        PublicationStage::NotPublished
    );
    let expected =
        serde_json::json!({"encoding":"unix_bytes", "bytes":path.as_os_str().as_bytes()});
    let value = serde_json::to_value(error.snapshot()).unwrap();
    assert_eq!(value["result"]["publications"][0]["path"], expected);
    let output = project().build(BuildOptions::temporary()).unwrap();
    let error = output.artifact().persist_to(&path).unwrap_err();
    assert_eq!(
        serde_json::to_value(error.publication()).unwrap()["path"],
        expected
    );
    assert!(output.artifact().path().is_file());
}
