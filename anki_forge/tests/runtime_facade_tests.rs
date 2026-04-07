use std::path::PathBuf;

use anki_forge::runtime::{
    discover_workspace_runtime, load_build_context, load_bundle_from_manifest, load_writer_policy,
    RuntimeMode,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
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
