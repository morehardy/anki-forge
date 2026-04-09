#[test]
fn repository_exposes_a_contract_bundle_entrypoint() {
    let manifest_path = contract_tools::contract_manifest_path();

    assert!(manifest_path.is_file());
    assert_eq!(
        manifest_path.file_name().and_then(|name| name.to_str()),
        Some("manifest.yaml")
    );
}

#[test]
fn workspace_exposes_authoring_core_contract_version() {
    assert_eq!(authoring_core::tool_contract_version(), "phase2-v1");
}

#[test]
fn workspace_exposes_writer_core_contract_version() {
    assert_eq!(writer_core::tool_contract_version(), "phase3-v1");
}

#[test]
fn contract_tools_manifest_does_not_default_depend_on_vendored_anki() {
    let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read contract_tools manifest");

    assert!(
        !manifest.contains("anki = { path ="),
        "contract_tools should not default-depend on vendored upstream anki"
    );
}

#[test]
fn roundtrip_oracle_script_exists() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let script_path = repo_root.join("scripts/run_roundtrip_oracle.sh");

    assert!(
        script_path.is_file(),
        "expected explicit local oracle runner at {}",
        script_path.display()
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = std::fs::metadata(&script_path)
            .expect("stat roundtrip oracle script")
            .permissions()
            .mode();
        assert!(
            mode & 0o111 != 0,
            "roundtrip oracle script should be executable: {}",
            script_path.display()
        );
    }
}
