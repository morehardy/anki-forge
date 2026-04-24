use flate2::read::GzDecoder;
use std::{fs::File, path::Path};
use tar::Archive;
use tempfile::tempdir;

fn extract_artifact(artifact_path: &Path) -> std::path::PathBuf {
    let extract_dir = tempdir().expect("temp dir");
    let file = File::open(artifact_path).expect("artifact should exist");
    let mut archive = Archive::new(GzDecoder::new(file));
    archive
        .unpack(extract_dir.path())
        .expect("artifact should unpack");
    extract_dir.into_path()
}

#[test]
fn package_command_emits_a_bundle_artifact_with_manifest_and_contract_assets() {
    let manifest_path = contract_tools::contract_manifest_path();
    let out_dir = tempdir().expect("temp dir");

    let artifact_path = contract_tools::package::build_artifact(&manifest_path, out_dir.path())
        .expect("package artifact should be created");

    assert_eq!(
        artifact_path.file_name().and_then(|name| name.to_str()),
        Some("anki-forge-contract-bundle-0.1.1.tar.gz")
    );

    let extracted_root = extract_artifact(&artifact_path);
    let extracted_manifest = extracted_root.join("contracts/manifest.yaml");

    assert!(
        extracted_manifest.exists(),
        "artifact should unpack a manifest"
    );
    contract_tools::gates::run_all(&extracted_manifest).expect("extracted artifact should verify");
}
