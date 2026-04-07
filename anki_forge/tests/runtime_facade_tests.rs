use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anki_forge::runtime::{
    build_from_path, discover_workspace_runtime, inspect_apkg_path, load_build_context,
    load_bundle_from_manifest, load_writer_policy, normalize_from_path, RuntimeMode,
};
use serde_yaml::Value as YamlValue;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn temp_bundle_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anki_forge_runtime_facade_{name}_{}_{}",
        std::process::id(),
        unique
    ))
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs::create_dir_all(dst).expect("create destination directory");

    for entry in fs::read_dir(src).expect("read source directory") {
        let entry = entry.expect("read source directory entry");
        let file_type = entry.file_type().expect("inspect source entry");
        let target_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(entry.path(), target_path);
        } else {
            fs::copy(entry.path(), target_path).expect("copy source file");
        }
    }
}

fn read_manifest_value() -> YamlValue {
    let raw = fs::read_to_string(repo_root().join("contracts/manifest.yaml"))
        .expect("read workspace manifest");
    serde_yaml::from_str(&raw).expect("parse workspace manifest")
}

fn manifest_mapping(value: &mut YamlValue) -> &mut serde_yaml::Mapping {
    value.as_mapping_mut().expect("manifest must be a mapping")
}

fn assets_mapping(value: &mut YamlValue) -> &mut serde_yaml::Mapping {
    manifest_mapping(value)
        .get_mut(YamlValue::String("assets".into()))
        .and_then(YamlValue::as_mapping_mut)
        .expect("manifest assets must be a mapping")
}

fn write_bundle_fixture(name: &str, manifest: &YamlValue) -> PathBuf {
    let root = temp_bundle_root(name);
    let contracts_root = root.join("contracts");
    copy_dir_all(repo_root().join("contracts"), &contracts_root);
    fs::write(
        contracts_root.join("manifest.yaml"),
        serde_yaml::to_string(manifest).expect("serialize manifest"),
    )
    .expect("write temp manifest");
    contracts_root.join("manifest.yaml")
}

#[test]
fn workspace_runtime_discovers_manifest_bundle_root_and_bundle_version() {
    let resolved = discover_workspace_runtime(repo_root()).expect("discover workspace runtime");

    assert_eq!(resolved.mode, RuntimeMode::Workspace);
    assert!(resolved.manifest_path.ends_with("contracts/manifest.yaml"));
    assert!(resolved.bundle_root.ends_with("contracts"));
    assert_eq!(resolved.bundle_version, "0.1.0");
}

#[test]
fn runtime_loads_default_phase3_assets_from_manifest() {
    let bundle = load_bundle_from_manifest(repo_root().join("contracts/manifest.yaml"))
        .expect("load runtime bundle");

    assert!(bundle.assets.contains_key("writer_policy"));
    assert!(bundle.assets.contains_key("build_context_default"));

    let writer_policy = load_writer_policy(&bundle, "default").expect("load writer policy");
    let build_context = load_build_context(&bundle, "default").expect("load build context");

    assert_eq!(writer_policy.id, "writer-policy.default");
    assert_eq!(build_context.id, "build-context.default");
}

#[test]
fn runtime_bundle_loading_rejects_manifests_missing_component_versions() {
    let mut manifest = read_manifest_value();
    manifest_mapping(&mut manifest).remove(YamlValue::String("component_versions".into()));
    let manifest_path = write_bundle_fixture("missing_component_versions", &manifest);

    let err = load_bundle_from_manifest(&manifest_path).expect_err("bundle load should fail");

    assert!(
        err.to_string().contains("component_versions"),
        "unexpected error: {err}"
    );
}

#[test]
fn runtime_bundle_loading_rejects_invalid_asset_paths() {
    let mut manifest = read_manifest_value();
    assets_mapping(&mut manifest).insert(
        YamlValue::String("writer_policy".into()),
        YamlValue::String("policies/missing-writer-policy.yaml".into()),
    );
    let manifest_path = write_bundle_fixture("missing_asset", &manifest);

    let err = load_bundle_from_manifest(&manifest_path).expect_err("bundle load should fail");

    assert!(
        err.to_string().contains("missing-writer-policy")
            || err.to_string().contains("writer_policy"),
        "unexpected error: {err}"
    );
}

#[test]
fn runtime_normalize_and_build_from_paths_match_repository_contracts() {
    let runtime = discover_workspace_runtime(repo_root()).expect("discover workspace runtime");
    let authoring_input = repo_root().join("contracts/fixtures/valid/minimal-authoring-ir.json");
    let normalized = normalize_from_path(&runtime, &authoring_input).expect("normalize from path");
    assert_eq!(normalized.kind, "normalization-result");
    assert_eq!(normalized.result_status, "success");

    let build_input = repo_root().join("contracts/fixtures/phase3/inputs/basic-normalized-ir.json");
    let artifacts_dir = repo_root().join("tmp/phase4-runtime-facade/basic");
    let build_result = build_from_path(&runtime, &build_input, "default", "default", &artifacts_dir)
        .expect("build from path");
    assert_eq!(build_result.kind, "package-build-result");
    assert_eq!(build_result.result_status, "success");

    let apkg_report =
        inspect_apkg_path(artifacts_dir.join("package.apkg")).expect("inspect apkg from path");
    assert_eq!(apkg_report.kind, "inspect-report");
    assert_eq!(apkg_report.observation_status, "complete");
}
