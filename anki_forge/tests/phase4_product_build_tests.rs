use anki_forge::build::{
    BuildFailureCause, BuildOptions, BuildPolicyStatus, BuildStatus, ComparisonStatus, RiskLevel,
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

#[test]
fn product_build_report_json_writes_success_report() {
    let temp = tempdir().expect("tempdir");
    let apkg = temp.path().join("deck.apkg");
    let report_json = temp.path().join("build-report.json");

    let report = basic_project("front")
        .build(BuildOptions::new().output(&apkg).report_json(&report_json))
        .expect("build succeeds");

    assert_eq!(report.status, BuildStatus::Success);
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_json).expect("read report"))
            .expect("report JSON");
    assert_eq!(written["kind"], "anki-forge-build-report");
    assert_eq!(written["status"], "success");
}

#[test]
fn product_build_report_json_writes_invalid_baseline_report() {
    let temp = tempdir().expect("tempdir");
    let apkg = temp.path().join("deck.apkg");
    let report_json = temp.path().join("build-report.json");
    let missing = temp.path().join("missing.apkg");

    let err = basic_project("front")
        .build(
            BuildOptions::new()
                .output(&apkg)
                .compare_to(&missing)
                .fail_on(RiskLevel::High)
                .report_json(&report_json),
        )
        .expect_err("invalid baseline should fail");

    assert_eq!(err.report.status, BuildStatus::Invalid);
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_json).expect("read report"))
            .expect("report JSON");
    assert_eq!(written["status"], "invalid");
    assert_eq!(written["comparison"], "unavailable");
    assert_eq!(written["policy"]["status"], "blocked");
}

#[test]
fn product_build_report_json_write_failure_returns_io_error_report() {
    let temp = tempdir().expect("tempdir");
    let apkg = temp.path().join("deck.apkg");
    let report_json = temp.path().join("report-target");
    std::fs::create_dir(&report_json).expect("create directory at report target");

    let err = basic_project("front")
        .build(BuildOptions::new().output(&apkg).report_json(&report_json))
        .expect_err("directory report_json target should fail");

    assert_eq!(err.cause, BuildFailureCause::Io);
    assert_eq!(err.report.status, BuildStatus::Error);
    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"REPORT.JSON_WRITE_FAILED".to_string()));
    let diagnostic = err
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "REPORT.JSON_WRITE_FAILED")
        .expect("json write diagnostic");
    let expected_source = report_json.display().to_string();
    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some(expected_source.as_str())
    );
}

#[test]
fn project_diff_against_apkg_matches_build_comparison_sections() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let project = basic_project("changed front");
    let build_report = project
        .build(BuildOptions::new().output(&current).compare_to(&previous))
        .expect("build comparison");
    let diff_report = project
        .diff_against_apkg(&previous)
        .expect("standalone diff");

    assert_eq!(diff_report.comparison, build_report.comparison);
    assert_eq!(diff_report.diff, build_report.diff);
    assert_eq!(diff_report.risk, build_report.risk);
    assert!(diff_report.current_inspect.is_some());
    assert!(diff_report.previous_inspect.is_some());
}

#[test]
fn project_diff_against_apkg_unreadable_baseline_returns_invalid_report() {
    let temp = tempdir().expect("tempdir");
    let missing = temp.path().join("missing.apkg");
    let err = basic_project("front")
        .diff_against_apkg(&missing)
        .expect_err("missing baseline should fail");

    assert_eq!(err.report.status, BuildStatus::Invalid);
    assert_eq!(err.report.comparison, ComparisonStatus::Unavailable);
    assert!(err
        .report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE"));
}

fn custom_template_project(template_names: &[&str]) -> Project {
    let mut note_type = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity())
        .field(Field::new("Meaning").key("meaning"))
        .identity(IdentityRecipe::fields(["expr"]));

    for template_name in template_names {
        note_type = note_type.template(
            Template::new(*template_name)
                .key(template_name.to_ascii_lowercase().replace(' ', "-"))
                .front("{{Expression}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        );
    }

    let mut project = Project::new("Phase4 Template Oracle")
        .stable_id("phase4-template-oracle")
        .default_deck("Phase4");
    project.add_notetype(note_type).expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp-vocab:taberu")
                .text("Expression", "食べる")
                .text("Meaning", "to eat"),
        )
        .expect("add note");
    project
}

#[test]
fn oracle_template_removed_emits_critical_risk_with_evidence() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    custom_template_project(&["Recognition", "Production"])
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let err = custom_template_project(&["Recognition"])
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&previous)
                .fail_on(RiskLevel::Critical),
        )
        .expect_err("template removal should block at critical");

    assert_eq!(err.report.policy.status, BuildPolicyStatus::Blocked);
    let finding = err
        .report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.TEMPLATE_REMOVED")
        .expect("template removed finding");
    assert_eq!(finding.level, RiskLevel::Critical);
    assert!(
        finding
            .evidence_refs
            .iter()
            .any(|evidence| evidence.ref_id.contains("semantic")),
        "finding should link to semantic diff evidence"
    );
}

#[test]
fn oracle_template_reorder_emits_high_risk_with_evidence() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    custom_template_project(&["Recognition", "Production"])
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let report = custom_template_project(&["Production", "Recognition"])
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&previous)
                .fail_on(RiskLevel::Critical),
        )
        .expect("template reorder is high risk but below critical threshold");

    assert_eq!(report.policy.status, BuildPolicyStatus::Passed);
    let finding = report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .find(|finding| {
            finding.code == "RISK.TEMPLATE_REORDER"
                && finding.evidence_refs.iter().any(|evidence| {
                    evidence.ref_id.contains("semantic")
                        || evidence.ref_id.contains("diff:templates")
                })
        })
        .expect("template reorder diff-backed finding");
    assert_eq!(finding.level, RiskLevel::High);
    assert!(
        finding.evidence_refs.iter().any(|evidence| {
            evidence.ref_id.contains("semantic") || evidence.ref_id.contains("diff:templates")
        }),
        "finding should link to semantic or template diff evidence"
    );
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
