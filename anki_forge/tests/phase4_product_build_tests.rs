use anki_forge::build::{
    BuildFailureCause, BuildOptions, BuildStatus, ComparisonStatus, RiskLevel,
};
use anki_forge::prelude::*;
use tempfile::tempdir;

fn basic_project(front: &str) -> Project {
    let mut project = Project::new("Phase4 Basic")
        .stable_id("phase4-basic")
        .default_deck("Phase4");
    project
        .add_note(Note::basic(front, "back").stable_id("note-1"))
        .expect("add note");
    project
}

#[test]
fn product_build_compare_to_attaches_diff_risk_and_policy() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let report = basic_project("changed front")
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&previous)
                .fail_on(RiskLevel::Critical),
        )
        .expect("current build");

    assert_eq!(report.status, BuildStatus::Success);
    assert_eq!(report.comparison, ComparisonStatus::Complete);
    assert!(report.diff.is_some(), "diff summary exists");
    assert!(report.risk.is_some(), "risk report exists");
    assert_eq!(report.policy.threshold, Some(RiskLevel::Critical));
}

#[test]
fn product_build_unreadable_baseline_returns_invalid_report_with_risk() {
    let temp = tempdir().expect("tempdir");
    let current = temp.path().join("current.apkg");
    let missing = temp.path().join("missing.apkg");

    let err = basic_project("front")
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&missing)
                .fail_on(RiskLevel::High),
        )
        .expect_err("unreadable baseline should return an invalid report");

    assert_eq!(err.cause, BuildFailureCause::Diagnostics);
    assert_eq!(err.report.status, BuildStatus::Invalid);
    assert_eq!(err.report.comparison, ComparisonStatus::Unavailable);
    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"COMPARE.BASELINE_UNAVAILABLE".into()));
    assert!(err
        .report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| {
            finding.code == "RISK.BASELINE_UNAVAILABLE" && finding.level == RiskLevel::High
        }));
    assert!(err
        .report
        .policy
        .blocking_findings
        .contains(&"RISK.BASELINE_UNAVAILABLE".into()));
}

#[test]
fn product_build_strict_update_diagnostics_attach_risk_and_policy() {
    let temp = tempdir().expect("tempdir");
    let current = temp.path().join("current.apkg");
    let lockfile_path = temp.path().join("identity-lockfile.json");

    write_field_config_drift_lockfile(&lockfile_path);

    let mut project = Project::new("Japanese").stable_id("jp-core");
    project
        .add_notetype(vocab_notetype_with_expression_key("expr"))
        .expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&current)
                .identity_lockfile(&lockfile_path)
                .fail_on(RiskLevel::High),
        )
        .expect_err("strict update diagnostics should return an invalid report");

    assert_eq!(err.cause, BuildFailureCause::Diagnostics);
    assert_eq!(err.report.status, BuildStatus::Invalid);
    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.FIELD_MERGE_ID_CHANGED".into()));
    assert!(err
        .report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.NOTETYPE_CONFIG_ID_DRIFT"));
    assert!(err
        .report
        .policy
        .blocking_findings
        .contains(&"RISK.NOTETYPE_CONFIG_ID_DRIFT".into()));
}

fn vocab_notetype_with_expression_key(expression_key: &str) -> NoteType {
    NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key(expression_key))
        .field(Field::new("Meaning").key("meaning"))
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all([expression_key])),
        )
}

fn write_field_config_drift_lockfile(path: &std::path::Path) {
    let current_config_id = anki_forge::product::stable_config_id("field", "jp-vocab", "expr");
    let field_key = format!("field:config:{current_config_id}");
    let mut index = anki_forge::update_safety::model::IdentityIndex::empty_lockfile(
        "jp-core",
        "writer-policy.default@1.0.0",
    );
    index
        .notetypes
        .push(anki_forge::update_safety::model::NotetypeIdentityEntry {
            note_type_id: "jp-vocab".into(),
            anki_model_id: None,
            name: "jp-vocab".into(),
            fields: vec![anki_forge::update_safety::model::FieldMergeEntry {
                field_key,
                field_name: "Expression".into(),
                ord: 0,
                config_id: current_config_id + 1,
                tag: 0,
            }],
            templates: vec![],
        });
    let lockfile = anki_forge::update_safety::model::IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "jp-core".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: index,
        generated_by: anki_forge::update_safety::model::GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    anki_forge::update_safety::lockfile::write_lockfile_atomic(path, &lockfile)
        .expect("write drift lockfile");
}
