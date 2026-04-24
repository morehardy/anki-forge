use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
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
                "kind": "normal",
                "name": "Basic",
                "original_stock_kind": "basic",
                "fields": [
                    { "name": "Front", "ord": 0, "prevent_deletion": false },
                    { "name": "Back", "ord": 1, "prevent_deletion": false }
                ],
                "templates": [
                    {
                        "name": "Card 1",
                        "ord": 0,
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

fn build_basic_package(
    temp_dir: &std::path::Path,
) -> (Value, std::path::PathBuf, std::path::PathBuf) {
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
    (
        build_result,
        artifacts_dir.join("staging/manifest.json"),
        artifacts_dir.join("package.apkg"),
    )
}

fn build_basic_package_with_default_selectors(
    temp_dir: &std::path::Path,
) -> (Value, std::path::PathBuf, std::path::PathBuf) {
    let manifest = contract_tools::contract_manifest_path();
    let input = write_basic_normalized_ir(temp_dir);
    let artifacts_dir = temp_dir.join("artifacts-defaults");
    let output = run_cli(&[
        "build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
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
    (
        build_result,
        artifacts_dir.join("staging/manifest.json"),
        artifacts_dir.join("package.apkg"),
    )
}

#[test]
fn verify_command_succeeds_for_the_repo_contract_bundle() {
    let manifest_path = copied_bundled_manifest_path("cli-verify");
    let output = run_cli(&["verify", "--manifest", manifest_path.to_str().unwrap()]);

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
    assert!(stdout.contains("bundle_version: 0.1.1"), "stdout: {stdout}");
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
fn normalize_command_matches_anki_forge_runtime_output() {
    let manifest = contract_tools::contract_manifest_path();
    let repo_root = manifest.parent().unwrap().parent().unwrap();
    let input = repo_root.join("contracts/fixtures/valid/minimal-authoring-ir.json");

    let runtime = anki_forge::runtime::load_bundle_from_manifest(&manifest)
        .unwrap()
        .runtime;
    let runtime_result = anki_forge::runtime::normalize_from_path(&runtime, &input).unwrap();

    let cli_output = run_cli(&[
        "normalize",
        "--manifest",
        manifest.to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(cli_output.status.success());
    let cli_json: serde_json::Value = serde_json::from_slice(&cli_output.stdout).unwrap();
    let runtime_json = serde_json::to_value(runtime_result).unwrap();
    assert_eq!(cli_json, runtime_json);
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
    assert_eq!(
        build_result["staging_ref"],
        "artifacts/staging/manifest.json"
    );
    assert_eq!(build_result["apkg_ref"], "artifacts/package.apkg");
    assert!(build_result["artifact_fingerprint"]
        .as_str()
        .expect("artifact fingerprint")
        .starts_with("artifact:"));
    assert!(build_result["package_fingerprint"]
        .as_str()
        .expect("package fingerprint")
        .starts_with("package:"));
    assert!(staging_manifest.exists(), "staging manifest should exist");
    assert!(apkg_path.exists(), "apkg should exist");
}

#[test]
fn build_command_defaults_writer_policy_and_build_context() {
    let temp = tempdir().expect("tempdir");
    let (build_result, staging_manifest, apkg_path) =
        build_basic_package_with_default_selectors(temp.path());

    assert_eq!(build_result["kind"], "package-build-result");
    assert_eq!(build_result["result_status"], "success");
    assert_eq!(
        build_result["staging_ref"],
        "artifacts/staging/manifest.json"
    );
    assert_eq!(build_result["apkg_ref"], "artifacts/package.apkg");
    assert!(build_result["writer_policy_ref"]
        .as_str()
        .unwrap()
        .starts_with("writer-policy.default@"));
    assert!(build_result["build_context_ref"]
        .as_str()
        .unwrap()
        .starts_with("build-context:"));
    assert!(staging_manifest.exists(), "staging manifest should exist");
    assert!(apkg_path.exists(), "apkg should exist");
}

#[test]
fn build_command_matches_anki_forge_runtime_output() {
    let manifest = contract_tools::contract_manifest_path();
    let repo_root = manifest.parent().unwrap().parent().unwrap();
    let build_input = repo_root.join("contracts/fixtures/phase3/inputs/basic-normalized-ir.json");
    let artifacts_dir = tempdir().unwrap();

    let runtime = anki_forge::runtime::load_bundle_from_manifest(&manifest)
        .unwrap()
        .runtime;
    let runtime_result = anki_forge::runtime::build_from_path(
        &runtime,
        &build_input,
        "default",
        "default",
        artifacts_dir.path(),
    )
    .unwrap();

    let cli_output = run_cli(&[
        "build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--input",
        build_input.to_str().unwrap(),
        "--writer-policy",
        "default",
        "--build-context",
        "default",
        "--artifacts-dir",
        artifacts_dir.path().to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(cli_output.status.success());
    let cli_json: serde_json::Value = serde_json::from_slice(&cli_output.stdout).unwrap();
    let runtime_json = serde_json::to_value(runtime_result).unwrap();
    assert_eq!(cli_json, runtime_json);
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
    assert!(staging_report["observations"]["field_metadata"].is_array());
    assert!(staging_report["observations"]["browser_templates"].is_array());
    assert!(staging_report["observations"]["template_target_decks"].is_array());

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
    let apkg_report: Value =
        serde_json::from_slice(&apkg_output.stdout).expect("apkg inspect JSON");
    assert_eq!(apkg_report["kind"], "inspect-report");
    assert_eq!(apkg_report["source_kind"], "apkg");
    assert_eq!(apkg_report["observation_status"], "complete");
    assert!(apkg_report["observations"]["field_metadata"].is_array());
    assert!(apkg_report["observations"]["browser_templates"].is_array());
    assert!(apkg_report["observations"]["template_target_decks"].is_array());

    let left = temp.path().join("left.inspect.json");
    let right = temp.path().join("right.inspect.json");
    fs::write(
        &left,
        serde_json::to_string_pretty(&staging_report).unwrap(),
    )
    .unwrap();
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

#[test]
fn inspect_and_diff_commands_match_anki_forge_runtime_output() {
    let manifest = contract_tools::contract_manifest_path();
    let repo_root = manifest.parent().unwrap().parent().unwrap();
    let build_input = repo_root.join("contracts/fixtures/phase3/inputs/basic-normalized-ir.json");
    let artifacts_dir = tempdir().unwrap();

    let runtime = anki_forge::runtime::load_bundle_from_manifest(&manifest)
        .unwrap()
        .runtime;
    let _build_result = anki_forge::runtime::build_from_path(
        &runtime,
        &build_input,
        "default",
        "default",
        artifacts_dir.path(),
    )
    .unwrap();

    let staging_path = artifacts_dir.path().join("staging/manifest.json");
    let apkg_path = artifacts_dir.path().join("package.apkg");

    let runtime_staging = anki_forge::runtime::inspect_staging_path(&staging_path).unwrap();
    let runtime_apkg = anki_forge::runtime::inspect_apkg_path(&apkg_path).unwrap();

    let cli_staging = run_cli(&[
        "inspect",
        "--staging",
        staging_path.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(cli_staging.status.success());
    let cli_staging_json: serde_json::Value = serde_json::from_slice(&cli_staging.stdout).unwrap();
    assert_eq!(
        cli_staging_json,
        serde_json::to_value(&runtime_staging).unwrap()
    );

    let cli_apkg = run_cli(&[
        "inspect",
        "--apkg",
        apkg_path.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(cli_apkg.status.success());
    let cli_apkg_json: serde_json::Value = serde_json::from_slice(&cli_apkg.stdout).unwrap();
    assert_eq!(cli_apkg_json, serde_json::to_value(&runtime_apkg).unwrap());

    let left = artifacts_dir.path().join("left.inspect.json");
    let right = artifacts_dir.path().join("right.inspect.json");
    fs::write(
        &left,
        serde_json::to_string_pretty(&runtime_staging).unwrap(),
    )
    .unwrap();
    fs::write(&right, serde_json::to_string_pretty(&runtime_apkg).unwrap()).unwrap();

    let runtime_diff = anki_forge::runtime::diff_from_paths(&left, &right).unwrap();
    let cli_diff = run_cli(&[
        "diff",
        "--left",
        left.to_str().unwrap(),
        "--right",
        right.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(cli_diff.status.success());
    let cli_diff_json: serde_json::Value = serde_json::from_slice(&cli_diff.stdout).unwrap();
    assert_eq!(cli_diff_json, serde_json::to_value(runtime_diff).unwrap());
}

fn temp_contract_root(label: &str) -> PathBuf {
    static NEXT_TEMP_ROOT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT_TEMP_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anki-forge-cli-tests-{}-{}-{}",
        label,
        std::process::id(),
        unique
    ))
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination tree");
    for entry in fs::read_dir(src).expect("read source tree") {
        let entry = entry.expect("read source entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy source file");
        }
    }
}

fn copied_bundled_manifest_path(label: &str) -> PathBuf {
    let root = temp_contract_root(label);
    copy_tree(
        contract_tools::contract_manifest_path()
            .parent()
            .expect("contracts root for bundled manifest"),
        &root,
    );

    let artifacts_root = root.join("artifacts");
    if artifacts_root.exists() {
        fs::remove_dir_all(&artifacts_root).expect("remove generated artifact tree");
    }

    root.join("manifest.yaml")
}
