use serde_json::Value;
use std::{fs, process::Command};
use tempfile::tempdir;

fn cargo_bin() -> &'static str {
    env!("CARGO_BIN_EXE_contract_tools")
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(cargo_bin())
        .args(args)
        .output()
        .expect("contract_tools binary should run")
}

#[test]
fn verify_command_succeeds_for_the_repo_contract_bundle() {
    let output = run_cli(&[
        "verify",
        "--manifest",
        contract_tools::contract_manifest_path().to_str().unwrap(),
    ]);

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("verification passed"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn verify_command_fails_when_manifest_is_missing() {
    let output = run_cli(&["verify", "--manifest", "contracts/missing.yaml"]);

    assert!(
        !output.status.success(),
        "expected failure but command succeeded"
    );
}

#[test]
fn summary_command_prints_bundle_version_and_public_axis() {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("repo manifest should load");
    let output = run_cli(&["summary", "--manifest", manifest.path.to_str().unwrap()]);

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bundle_version: 0.1.0"), "stdout: {stdout}");
    assert!(
        stdout.contains("public_axis: bundle_version"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("component_versions:"), "stdout: {stdout}");

    for (name, asset) in &manifest.data.assets {
        let entry = format!("  {name}: {asset}");
        assert!(stdout.contains(&entry), "stdout: {stdout}");
    }
}

#[test]
fn normalize_contract_json_includes_required_top_level_fields() {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("repo manifest should load");
    let input = manifest
        .contracts_root
        .join("fixtures/valid/minimal-authoring-ir.json");

    let output = run_cli(&[
        "normalize",
        "--manifest",
        manifest.path.to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    let object = value.as_object().expect("stdout should be a JSON object");

    for key in [
        "kind",
        "result_status",
        "tool_contract_version",
        "policy_refs",
        "comparison_context",
        "diagnostics",
    ] {
        assert!(object.contains_key(key), "missing key {key} in {stdout}");
    }
}

#[test]
fn normalize_command_rejects_invalid_authoring_input_shape() {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("repo manifest should load");
    let temp = tempdir().expect("tempdir");
    let input = temp.path().join("invalid-authoring.json");
    fs::write(
        &input,
        r#"{"kind":"authoring-ir","schema_version":"0.1.0","metadata":{"document_id":"doc-bad"}}"#,
    )
    .expect("write invalid input");

    let output = run_cli(&[
        "normalize",
        "--manifest",
        manifest.path.to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
