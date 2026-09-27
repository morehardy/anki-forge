use ankiforge::build::{json::BuildResultSnapshot, BuildCounts, BuildErrorKind};
use ankiforge::{BuildOptions, Content, Note, Project};

fn project() -> Project {
    let mut p = Project::new("observations").unwrap();
    p.add("one", Note::basic("front", "back")).unwrap();
    p
}

#[test]
fn completed_baseline_observations_survive_later_candidate_validation_failure() {
    let first = project().build(BuildOptions::temporary()).unwrap();
    let mut invalid = Project::new("observations").unwrap();
    invalid
        .add(
            "one",
            Note::basic("front", Content::html("<img src=\"missing.png\">")),
        )
        .unwrap();
    let error = invalid
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::Validation);
    assert_eq!(
        error.report().baseline_counts(),
        Some(&BuildCounts {
            notes: 1,
            cards: 1,
            media: 0
        })
    );
    assert!(error.report().comparison().is_none());
    assert!(error
        .report()
        .diagnostics()
        .iter()
        .any(|d| d.code == "MEDIA.MISSING_REFERENCE"));
    assert_eq!(
        serde_json::to_value(error.snapshot()).unwrap()["report"]["baseline_counts"]["notes"],
        1
    );
    assert!(matches!(
        error.snapshot().result,
        BuildResultSnapshot::Failure { .. }
    ));
}

#[test]
fn unreadable_baseline_is_distinguished_from_an_inspected_empty_baseline() {
    let directory = tempfile::tempdir().unwrap();
    let error = project()
        .build(BuildOptions::temporary().update_from(directory.path().join("missing.apkg")))
        .unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::Io);
    assert!(error.report().baseline_counts().is_none());
    assert!(error.report().comparison().is_none());
}

#[test]
fn successful_update_reports_both_completed_count_observations() {
    let first = project().build(BuildOptions::temporary()).unwrap();
    assert!(first.report().baseline_counts().is_none());
    let mut next = project();
    next.add("two", Note::basic("new question", "new answer"))
        .unwrap();
    let output = next
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(output.report().baseline_counts().unwrap().notes, 1);
    assert_eq!(output.report().counts().notes, 2);
    let snapshot = output.snapshot();
    assert!(matches!(
        snapshot.result,
        BuildResultSnapshot::Success { .. }
    ));
    assert!(
        snapshot
            .report
            .comparison
            .unwrap()
            .policy
            .allows_publication
    );
}
