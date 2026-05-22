use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::update_safety::{
    classify_project_stable_id_missing, effective_mode, validate_writer_policy_ref,
    EvidenceCondition, EffectiveMode,
};

#[test]
fn effective_mode_uses_explicit_disabled_even_with_baselines() {
    let options = BuildOptions::new()
        .compare_to("previous.apkg")
        .identity_lockfile("anki-forge.lock.json")
        .write_identity_lockfile(true)
        .update_safety(UpdateSafetyMode::Disabled);

    assert_eq!(effective_mode(&options).unwrap(), EffectiveMode::Disabled);
}

#[test]
fn effective_mode_requires_lockfile_path_when_writing() {
    let err = effective_mode(&BuildOptions::new().write_identity_lockfile(true))
        .expect_err("missing lockfile path should fail option validation");

    assert_eq!(err.code.as_str(), "UPDATE.LOCKFILE_PATH_REQUIRED");
}

#[test]
fn effective_mode_upgrades_identity_lockfile_to_strict() {
    let options = BuildOptions::new()
        .identity_lockfile("anki-forge.lock.json")
        .write_identity_lockfile(true);

    assert_eq!(effective_mode(&options).unwrap(), EffectiveMode::Strict);
}

#[test]
fn classifier_returns_limitation_and_diagnostic() {
    let classified = classify_project_stable_id_missing(EvidenceCondition::StrictCompareOnly);

    assert_eq!(classified.limitation.as_deref(), Some("project_stable_id_missing"));
    assert_eq!(classified.diagnostic_code.as_deref(), Some("UPDATE.PROJECT_STABLE_ID_MISSING"));
    assert_eq!(classified.severity, anki_forge::diagnostics::Severity::Warning);

    let classified = classify_project_stable_id_missing(EvidenceCondition::LockfileRequired);
    assert_eq!(classified.diagnostic_code.as_deref(), Some("UPDATE.PROJECT_STABLE_ID_MISSING"));
    assert_eq!(classified.severity, anki_forge::diagnostics::Severity::Error);
}

#[test]
fn writer_policy_ref_rejects_at_sign_and_control_characters() {
    let err = validate_writer_policy_ref("writer@policy", "1.0.0")
        .expect_err("policy id containing @ is invalid");

    assert_eq!(err.code.as_str(), "UPDATE.WRITER_POLICY_REF_INVALID");

    let err = validate_writer_policy_ref("writer-policy.default", "1.0\n0")
        .expect_err("policy version containing control character is invalid");

    assert_eq!(err.code.as_str(), "UPDATE.WRITER_POLICY_REF_INVALID");
}
