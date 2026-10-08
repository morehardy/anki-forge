use ankiforge::{BuildOptions, Note, Project};

fn project() -> Project {
    let mut project = Project::new("prepared-publication").unwrap();
    project
        .add("one", Note::basic("reviewed", "answer"))
        .unwrap();
    project
}

#[test]
fn preparation_preserves_destination_and_publishes_the_reviewed_project() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("published.apkg");
    std::fs::write(&destination, b"previous publication").unwrap();
    let mut project = project();
    let prepared = project
        .prepare_publication(BuildOptions::to(&destination))
        .unwrap();
    let report = prepared.report().clone();
    assert_eq!(report.counts().notes, 1);
    assert_eq!(
        std::fs::read(&destination).unwrap(),
        b"previous publication"
    );
    project
        .add("later", Note::basic("not reviewed", "later"))
        .unwrap();
    drop(project);
    let output = prepared.publish().unwrap();
    assert_eq!(output.report().counts().notes, 1);
    assert!(output.report().duration() >= report.duration());
    let comparison = super_project_comparison(&destination);
    assert!(comparison.findings().is_empty());
    drop(output);
    assert!(destination.is_file());
}

fn super_project_comparison(path: &std::path::Path) -> ankiforge::update::ComparisonReport {
    project()
        .compare(ankiforge::update::CompareOptions::against(path))
        .unwrap()
}

#[test]
fn blocked_preparation_returns_full_evidence_and_consuming_publish_refuses() {
    let original = project().build(BuildOptions::temporary()).unwrap();
    let mut empty = Project::new("prepared-publication").unwrap();
    empty
        .add("replacement", Note::basic("different", "answer"))
        .unwrap();
    let options = BuildOptions::temporary().update_from(original.artifact().path());
    let comparison = empty
        .compare(ankiforge::update::CompareOptions::against(
            original.artifact().path(),
        ))
        .unwrap();
    let prepared = empty.prepare_publication(options).unwrap();
    let evidence =
        serde_json::to_value(prepared.report().comparison().unwrap().snapshot()).unwrap();
    assert_eq!(
        evidence,
        serde_json::to_value(comparison.snapshot()).unwrap()
    );
    assert!(!prepared
        .report()
        .comparison()
        .unwrap()
        .policy()
        .allows_publication());
    let error = prepared.publish().unwrap_err();
    assert_eq!(error.code(), "UPDATE.POLICY_BLOCKED");
    assert_eq!(
        serde_json::to_value(error.report().comparison().unwrap().snapshot()).unwrap(),
        evidence
    );
    assert!(error.publications().is_empty());
}

#[test]
fn temporary_publication_and_reports_have_independent_lifetimes() {
    let prepared = project()
        .prepare_publication(BuildOptions::temporary())
        .unwrap();
    let report = prepared.report().clone();
    let output = prepared.publish().unwrap();
    let path = output.artifact().path().to_owned();
    assert!(path.exists());
    drop(output);
    assert!(!path.exists());
    assert_eq!(report.counts().notes, 1);
}

#[test]
fn publication_failure_retains_observations_and_old_destination() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("destination");
    let prepared = project()
        .prepare_publication(BuildOptions::to(&destination))
        .unwrap();
    std::fs::create_dir(&destination).unwrap();
    std::fs::write(destination.join("keep"), b"old").unwrap();
    let error = prepared.publish().unwrap_err();
    assert_eq!(error.report().counts().notes, 1);
    assert_eq!(
        error.publications()[0].stage,
        ankiforge::build::json::PublicationStage::NotPublished
    );
    assert_eq!(std::fs::read(destination.join("keep")).unwrap(), b"old");
}

#[cfg(unix)]
#[test]
fn delayed_publication_rejects_changed_symlinks_hardlinks_and_parent_directories() {
    use std::os::unix::fs::symlink;
    for case in [
        "symlink",
        "hardlink",
        "parent",
        "baseline-link",
        "moved-original",
    ] {
        let root = tempfile::tempdir().unwrap();
        let original = root.path().join("original.apkg");
        project().build(BuildOptions::to(&original)).unwrap();
        let bytes = std::fs::read(&original).unwrap();
        let baseline = root.path().join("baseline.apkg");
        symlink(&original, &baseline).unwrap();
        let directory = root.path().join("out");
        std::fs::create_dir(&directory).unwrap();
        let destination = directory.join("original.apkg");
        let prepared = project()
            .prepare_publication(BuildOptions::to(&destination).update_from(&baseline))
            .unwrap();
        match case {
            "symlink" => symlink(&original, &destination).unwrap(),
            "hardlink" => std::fs::hard_link(&original, &destination).unwrap(),
            "parent" => {
                std::fs::remove_dir(&directory).unwrap();
                symlink(root.path(), &directory).unwrap();
            }
            "baseline-link" => {
                std::fs::copy(&original, &destination).unwrap();
                std::fs::remove_file(&baseline).unwrap();
                symlink(&destination, &baseline).unwrap();
            }
            "moved-original" => {
                std::fs::rename(&original, &destination).unwrap();
                std::fs::write(&original, b"replacement").unwrap();
            }
            _ => unreachable!(),
        }
        let error = prepared.publish().unwrap_err();
        assert_eq!(error.code(), "BUILD.OUTPUT_INVALID", "{case}");
        assert_eq!(error.report().counts().notes, 1);
        assert!(error.publications().is_empty());
        assert_eq!(
            std::fs::read(if case == "moved-original" {
                &destination
            } else {
                &original
            })
            .unwrap(),
            bytes
        );
    }
}

#[test]
fn delayed_publication_anchors_paths_and_excludes_review_wait_from_duration() {
    const CHILD: &str = "ANKIFORGE_PREPARED_CWD_CHILD";
    let test_name = "delayed_publication_anchors_paths_and_excludes_review_wait_from_duration";
    if std::env::var_os(CHILD).is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    let original = std::env::current_dir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let invoked = root.path().join("invoked");
    let later = root.path().join("later");
    std::fs::create_dir(&invoked).unwrap();
    std::fs::create_dir(&later).unwrap();
    std::env::set_current_dir(&invoked).unwrap();
    project().build(BuildOptions::to("baseline.apkg")).unwrap();
    let prepared = project()
        .prepare_publication(BuildOptions::to("result.apkg").update_from("baseline.apkg"))
        .unwrap();
    let duration = prepared.report().duration();
    std::env::set_current_dir(&later).unwrap();
    let wait = std::time::Duration::from_millis(500);
    std::thread::sleep(wait);
    let start = std::time::Instant::now();
    let output = prepared.publish().unwrap();
    // Bound against the actual publication wall time, not a hardware threshold.
    assert!(output.report().duration() <= duration + start.elapsed());
    assert_eq!(
        output.artifact().path(),
        invoked.canonicalize().unwrap().join("result.apkg")
    );
    assert!(!later.join("result.apkg").exists());
    std::env::set_current_dir(original).unwrap();
}
