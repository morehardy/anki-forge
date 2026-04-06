use serde_json::Value;
use std::{fs, process::Command};
use tempfile::tempdir;
use writer_core::build_context_ref;

fn cargo_bin() -> &'static str {
    env!("CARGO_BIN_EXE_contract_tools")
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(cargo_bin())
        .args(args)
        .output()
        .expect("contract_tools binary should run")
}

fn basic_normalized_ir() -> Value {
    serde_json::json!({
        "kind": "normalized-ir",
        "schema_version": "0.1.0",
        "document_id": "demo-doc",
        "resolved_identity": "document:demo-doc",
        "notetypes": [
            {
                "id": "basic-main",
                "kind": "basic",
                "name": "Basic",
                "fields": ["Front", "Back"],
                "templates": [
                    {
                        "name": "Card 1",
                        "question_format": "{{Front}}",
                        "answer_format": "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
                    }
                ],
                "css": ""
            }
        ],
        "notes": [
            {
                "id": "note-1",
                "notetype_id": "basic-main",
                "deck_name": "Default",
                "fields": {
                    "Front": "front",
                    "Back": "back <img src=\"sample.jpg\">"
                },
                "tags": ["demo"]
            }
        ],
        "media": [
            {
                "filename": "sample.jpg",
                "mime": "image/jpeg",
                "data_base64": "MQ=="
            }
        ]
    })
}

fn write_basic_normalized_ir(temp_dir: &std::path::Path) -> std::path::PathBuf {
    let input = temp_dir.join("basic-normalized-ir.json");
    fs::write(
        &input,
        serde_json::to_string_pretty(&basic_normalized_ir()).unwrap(),
    )
    .expect("write normalized ir fixture");
    input
}

fn load_declared_build_context_ref() -> String {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("repo manifest should load");
    let context_path =
        contract_tools::manifest::resolve_asset_path(&manifest, "build_context_default")
            .expect("build context asset should resolve");
    let raw = fs::read_to_string(context_path).expect("read build context asset");
    let context: writer_core::BuildContext =
        serde_yaml::from_str(&raw).expect("decode build context asset");
    build_context_ref(&context).expect("build context ref")
}

fn load_declared_writer_policy_ref() -> String {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("repo manifest should load");
    let policy_path = contract_tools::manifest::resolve_asset_path(&manifest, "writer_policy")
        .expect("writer policy asset should resolve");
    let raw = fs::read_to_string(policy_path).expect("read writer policy asset");
    let policy: writer_core::WriterPolicy =
        serde_yaml::from_str(&raw).expect("decode writer policy asset");
    writer_core::policy_ref(&policy.id, &policy.version)
}

fn build_basic_package(temp_dir: &std::path::Path) -> (Value, std::path::PathBuf, std::path::PathBuf) {
    let manifest = contract_tools::contract_manifest_path();
    let input = write_basic_normalized_ir(temp_dir);
    let artifacts_dir = temp_dir.join("artifacts");
    let output = run_cli(&[
        "build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
        "--writer-policy",
        "default",
        "--build-context",
        "default",
        "--artifacts-dir",
        artifacts_dir.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let build_result: Value = serde_json::from_slice(&output.stdout).expect("build JSON");
    (build_result, artifacts_dir.join("staging/manifest.json"), artifacts_dir.join("package.apkg"))
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

#[test]
fn build_command_emits_contract_json_with_manifest_backed_refs() {
    let temp = tempdir().expect("tempdir");
    let (build_result, staging_manifest, apkg_path) = build_basic_package(temp.path());

    assert_eq!(build_result["kind"], "package-build-result");
    assert_eq!(build_result["result_status"], "success");
    assert_eq!(
        build_result["writer_policy_ref"],
        load_declared_writer_policy_ref()
    );
    assert_eq!(
        build_result["build_context_ref"],
        load_declared_build_context_ref()
    );
    assert_eq!(build_result["staging_ref"], "artifacts/staging/manifest.json");
    assert_eq!(build_result["apkg_ref"], "artifacts/package.apkg");
    assert!(
        build_result["artifact_fingerprint"]
            .as_str()
            .expect("artifact fingerprint")
            .starts_with("artifact:")
    );
    assert!(
        build_result["package_fingerprint"]
            .as_str()
            .expect("package fingerprint")
            .starts_with("package:")
    );
    assert!(staging_manifest.exists(), "staging manifest should exist");
    assert!(apkg_path.exists(), "apkg should exist");
}

#[test]
fn build_command_requires_artifacts_dir() {
    let temp = tempdir().expect("tempdir");
    let manifest = contract_tools::contract_manifest_path();
    let input = write_basic_normalized_ir(temp.path());

    let output = run_cli(&[
        "build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
        "--writer-policy",
        "default",
        "--build-context",
        "default",
        "--output",
        "contract-json",
    ]);

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("artifacts-dir"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn inspect_and_diff_commands_emit_contract_json_for_real_fixture() {
    let temp = tempdir().expect("tempdir");
    let (_build_result, staging_manifest, apkg_path) = build_basic_package(temp.path());

    let staging_output = run_cli(&[
        "inspect",
        "--staging",
        staging_manifest.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(
        staging_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&staging_output.stdout),
        String::from_utf8_lossy(&staging_output.stderr)
    );
    let staging_report: Value =
        serde_json::from_slice(&staging_output.stdout).expect("staging inspect JSON");
    assert_eq!(staging_report["kind"], "inspect-report");
    assert_eq!(staging_report["source_kind"], "staging");
    assert_eq!(staging_report["observation_status"], "complete");

    let apkg_output = run_cli(&[
        "inspect",
        "--apkg",
        apkg_path.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(
        apkg_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&apkg_output.stdout),
        String::from_utf8_lossy(&apkg_output.stderr)
    );
    let apkg_report: Value = serde_json::from_slice(&apkg_output.stdout).expect("apkg inspect JSON");
    assert_eq!(apkg_report["kind"], "inspect-report");
    assert_eq!(apkg_report["source_kind"], "apkg");
    assert_eq!(apkg_report["observation_status"], "complete");

    let left = temp.path().join("left.inspect.json");
    let right = temp.path().join("right.inspect.json");
    fs::write(&left, serde_json::to_string_pretty(&staging_report).unwrap()).unwrap();
    fs::write(&right, serde_json::to_string_pretty(&apkg_report).unwrap()).unwrap();

    let diff_output = run_cli(&[
        "diff",
        "--left",
        left.to_str().unwrap(),
        "--right",
        right.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(
        diff_output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&diff_output.stdout),
        String::from_utf8_lossy(&diff_output.stderr)
    );
    let diff_report: Value = serde_json::from_slice(&diff_output.stdout).expect("diff JSON");
    assert_eq!(diff_report["kind"], "diff-report");
    assert_eq!(diff_report["comparison_status"], "complete");
    assert_eq!(diff_report["changes"], serde_json::json!([]));
}
