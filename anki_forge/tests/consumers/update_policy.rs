use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::BuildErrorKind;
use ankiforge::update::{CompareError, CompareOptions, ComparisonReport, PolicyError, PolicyErrorKind, RiskCode, RiskLevel, UpdatePolicy};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
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
