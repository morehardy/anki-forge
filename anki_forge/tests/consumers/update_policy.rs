use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::BuildErrorKind;
use ankiforge::update::{CompareError, CompareOptions, ComparisonReport, PolicyError, PolicyErrorKind, RiskCode, RiskLevel, UpdatePolicy};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    media_history_survives_omitted_releases()?;
    media_names_share_asset_collision_identity()?;
    fn error<T: Error + Send + Sync + 'static>() {}
    error::<CompareError>(); error::<PolicyError>();
    let mut baseline = Project::new("policy")?;
    baseline.add("keep", Note::basic("keep", "answer"))?;
    baseline.add("remove", Note::basic("remove", "answer"))?;
    baseline.build(BuildOptions::to("v1.apkg"))?;
    let mut project = Project::new("policy")?;
    project.add("keep", Note::basic("keep", "answer"))?;
    let report: ComparisonReport = project.compare(CompareOptions::against("v1.apkg"))?;
    assert_eq!(report.highest_risk(), Some(RiskLevel::High));
    assert_eq!(report.policy().threshold(), RiskLevel::High);
    assert!(!report.policy().allows_publication());
    let finding = report.findings().iter().find(|f| f.code() == RiskCode::NoteRemoved).unwrap();
    assert!(!finding.evidence().is_empty());
    assert_eq!(finding.level(), RiskLevel::High);
    let snapshot: ankiforge::update::json::ComparisonSnapshot = report.snapshot();
    let json = serde_json::to_value(snapshot)?;
    assert!(json.get("result").is_none());
    assert_eq!(json["policy"]["allows_publication"], false);
    fs::write("v2.apkg", "previous destination")?;
    let blocked = project.build(BuildOptions::to("v2.apkg").update_from("v1.apkg")).unwrap_err();
    assert_eq!(blocked.kind(), BuildErrorKind::PolicyBlocked);
    assert_eq!(blocked.code(), "UPDATE.POLICY_BLOCKED");
    assert!(!blocked.report().comparison().unwrap().policy().allows_publication());
    assert!(blocked.publications().is_empty());
    assert_eq!(fs::read_to_string("v2.apkg")?, "previous destination");
    assert_eq!(serde_json::to_value(blocked.snapshot())?["report"]["comparison"]["highest_risk"], "high");

    let policy = UpdatePolicy::default().allow(RiskCode::NoteRemoved).allow(RiskCode::MediaChanged);
    let accepted = project.compare(CompareOptions::against("v1.apkg").update_policy(policy.clone()))?;
    assert!(accepted.policy().allows_publication());
    assert_eq!(accepted.highest_risk(), Some(RiskLevel::High), "allowing a category must not hide its risk");
    assert_eq!(accepted.policy().allowed_codes(), &[RiskCode::NoteRemoved]);
    assert_eq!(accepted.policy().unmatched_allowances(), &[RiskCode::MediaChanged]);
    assert!(accepted.diagnostics().iter().any(|d| d.code == "UPDATE.UNMATCHED_ALLOWANCE" && d.severity == ankiforge::diagnostics::Severity::Warning));
    let output = project.build(BuildOptions::to("v2.apkg").update_policy(policy.clone()).update_from("v1.apkg"))?;
    assert_eq!(output.report().comparison().unwrap().policy(), accepted.policy());
    let reversed = project.build(BuildOptions::temporary().update_from("v1.apkg").update_policy(policy))?;
    assert_eq!(reversed.report().comparison().unwrap().policy(), accepted.policy());
    assert_eq!(project.build(BuildOptions::temporary().update_policy(UpdatePolicy::default())).unwrap_err().kind(), BuildErrorKind::Configuration);

    for code in ["RISK.DOES_NOT_EXIST", "UPDATE.EVIDENCE_MISSING", "RISK.*"] {
        let error = code.parse::<RiskCode>().unwrap_err();
        assert_eq!(error.kind(), PolicyErrorKind::UnknownRiskCode);
        assert_eq!(error.code(), "UPDATE.RISK_CODE_INVALID");
        let wrapped = anyhow::Error::new(error).context("read update policy");
        assert_eq!(wrapped.downcast_ref::<PolicyError>().unwrap().code(), "UPDATE.RISK_CODE_INVALID");
    }
    assert_eq!("RISK.NOTE_REMOVED".parse::<RiskCode>()?, RiskCode::NoteRemoved);
    let strict = project.compare(CompareOptions::against("v1.apkg").update_policy(UpdatePolicy::default().fail_on(RiskLevel::Medium)))?;
    assert!(!strict.policy().allows_publication());
    let error = project.compare(CompareOptions::against("missing.apkg")).unwrap_err();
    assert!(error.source().is_some());
    assert_eq!(error.code(), "BUILD.INSPECT_FAILED");
    let wrapped = anyhow::Error::new(error).context("compare lesson");
    assert!(wrapped.downcast_ref::<CompareError>().is_some());
    Ok(())
}

fn media_project(filename: Option<&str>, bytes: &[u8]) -> Result<Project, Box<dyn Error>> {
    let mut project = Project::new("media-release-history")?;
    project.add("one", Note::basic("question", "answer"))?;
    if let Some(filename) = filename {
        project.add_asset(ankiforge::Media::bytes(bytes.to_vec(), "application/octet-stream")?
            .with_export_name(filename)?)?;
    }
    Ok(project)
}

fn media_history_survives_omitted_releases() -> Result<(), Box<dyn Error>> {
    let first = media_project(Some("picture.bin"), b"original")?
        .build(BuildOptions::temporary())?;
    let omitted = media_project(None, b"")?.build(BuildOptions::temporary()
        .update_from(first.artifact().path()))?;
    assert_eq!(omitted.report().counts().media, 0);
    let changed = media_project(Some("picture.bin"), b"replacement")?;
    let policy = UpdatePolicy::default().fail_on(RiskLevel::Medium);
    let report = changed.compare(CompareOptions::against(omitted.artifact().path())
        .update_policy(policy.clone()))?;
    assert!(report.findings().iter().any(|finding| finding.code() == RiskCode::MediaChanged),
        "omitting media must not erase the risk of replacing a learner's retained bytes: {report:?}");
    assert!(!report.findings().iter().any(|finding| finding.code() == RiskCode::MediaAdded));
    fs::write("media-protected.apkg", b"existing destination")?;
    let error = changed.build(BuildOptions::to("media-protected.apkg")
        .update_from(omitted.artifact().path()).update_policy(policy)).unwrap_err();
    assert_eq!(error.code(), "UPDATE.POLICY_BLOCKED");
    assert!(error.publications().is_empty());
    assert_eq!(fs::read("media-protected.apkg")?, b"existing destination");
    let restored = media_project(Some("picture.bin"), b"original")?
        .compare(CompareOptions::against(omitted.artifact().path()))?;
    assert_eq!(restored.findings().iter().map(|f| f.code()).collect::<Vec<_>>(), [RiskCode::MediaAdded]);
    let accepted = changed.build(BuildOptions::temporary()
        .update_from(omitted.artifact().path())
        .update_policy(UpdatePolicy::default().fail_on(RiskLevel::Medium).allow(RiskCode::MediaChanged)))?;
    assert_eq!(accepted.report().comparison().unwrap().highest_risk(), Some(RiskLevel::Medium));
    assert_eq!(accepted.report().comparison().unwrap().policy().allowed_codes(), [RiskCode::MediaChanged]);
    let repeat = changed.build(BuildOptions::temporary().update_from(accepted.artifact().path()))?;
    assert!(repeat.report().comparison().unwrap().findings().is_empty(), "equal successive releases stay unchanged");
    // The last published descriptor advances to B, so restoring old A after
    // two omitted releases must still be a change, not an informational add.
    let omitted_again = media_project(None, b"")?.build(BuildOptions::temporary()
        .update_from(repeat.artifact().path()))?;
    let omitted_twice = media_project(None, b"")?.build(BuildOptions::temporary()
        .update_from(omitted_again.artifact().path()))?;
    assert!(omitted_twice.report().comparison().unwrap().findings().is_empty());
    let original = media_project(Some("picture.bin"), b"original")?
        .compare(CompareOptions::against(omitted_twice.artifact().path()))?;
    assert_eq!(original.findings().iter().map(|f| f.code()).collect::<Vec<_>>(), [RiskCode::MediaChanged]);
    let fresh = media_project(Some("another.bin"), b"replacement")?
        .compare(CompareOptions::against(omitted_twice.artifact().path()))?;
    assert_eq!(fresh.findings().iter().map(|f| f.code()).collect::<Vec<_>>(), [RiskCode::MediaAdded]);
    Ok(())
}

fn media_names_share_asset_collision_identity() -> Result<(), Box<dyn Error>> {
    for (before, after) in [("Logo.bin", "logo.bin"), ("café.bin", "cafe\u{301}.bin"), ("Straße.bin", "STRASSE.bin")] {
        let original = media_project(Some(before), b"old-content")?;
        let baseline = original.build(BuildOptions::temporary())?;
        let equal = media_project(Some(after), b"old-content")?;
        let equal_output = equal.build(BuildOptions::temporary().update_from(baseline.artifact().path()))?;
        assert!(equal_output.report().comparison().unwrap().findings().is_empty(), "spelling alone is not a byte change: {before} -> {after}");
        assert!(equal.compare(CompareOptions::against(equal_output.artifact().path()))?.findings().is_empty());
        let changed = media_project(Some(after), b"new-content")?;
        let strict = UpdatePolicy::default().fail_on(RiskLevel::Medium);
        let report = changed.compare(CompareOptions::against(baseline.artifact().path()).update_policy(strict.clone()))?;
        assert_eq!(report.findings().iter().map(|f| f.code()).collect::<Vec<_>>(), [RiskCode::MediaChanged], "{before} -> {after}");
        assert!(!report.policy().allows_publication());
        let facts = &report.findings()[0].evidence()[0];
        assert_eq!(facts.before.as_ref().unwrap()["filename"], before);
        assert_eq!(facts.after.as_ref().unwrap()["filename"], after);
        assert_eq!(facts.before.as_ref().unwrap()["content"]["size"], 11);
        assert_ne!(facts.before.as_ref().unwrap()["content"]["sha1"], facts.after.as_ref().unwrap()["content"]["sha1"]);
        fs::write("alias-protected.apkg", b"existing destination")?;
        let error = changed.build(BuildOptions::to("alias-protected.apkg")
            .update_from(baseline.artifact().path()).update_policy(strict)).unwrap_err();
        assert_eq!(error.kind(), BuildErrorKind::PolicyBlocked);
        assert!(error.publications().is_empty());
        assert_eq!(fs::read("alias-protected.apkg")?, b"existing destination");
        let omitted = media_project(None, b"")?.build(BuildOptions::temporary().update_from(baseline.artifact().path()))?;
        let recovered = changed.compare(CompareOptions::against(omitted.artifact().path()))?;
        assert_eq!(recovered.findings().iter().map(|f| f.code()).collect::<Vec<_>>(), [RiskCode::MediaChanged]);
        let mut collision = original.clone();
        let error = collision.add_asset(ankiforge::Media::bytes(b"old-content".to_vec(), "application/octet-stream")?
            .with_export_name(after)?).unwrap_err();
        assert_eq!(error.code(), "MEDIA.EXPORT_NAME_COLLISION", "same identity governs authoring and updates");
    }
    Ok(())
}
