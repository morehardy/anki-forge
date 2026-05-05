use flate2::read::GzDecoder;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
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

fn artifact_entries(artifact_path: &Path) -> Vec<PathBuf> {
    let file = File::open(artifact_path).expect("artifact should exist");
    let mut archive = Archive::new(GzDecoder::new(file));
    archive
        .entries()
        .expect("artifact entries should be readable")
        .map(|entry| {
            entry
                .expect("artifact entry should be readable")
                .path()
                .expect("artifact entry path should be readable")
                .into_owned()
        })
        .collect()
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

#[test]
fn package_command_excludes_transient_media_store_tmp_files() {
    let manifest_path = contract_tools::contract_manifest_path();
    let contracts_root = manifest_path.parent().expect("manifest should have parent");
    let tmp_file = contracts_root
        .join("fixtures/phase3/inputs/.anki-forge-media/tmp/package-regression/leak.tmp");
    fs::create_dir_all(tmp_file.parent().expect("tmp file should have parent"))
        .expect("tmp dir should be created");
    fs::write(&tmp_file, "transient").expect("tmp file should be written");

    let out_dir = tempdir().expect("temp dir");
    let result = contract_tools::package::build_artifact(&manifest_path, out_dir.path());
    fs::remove_dir_all(contracts_root.join("fixtures/phase3/inputs/.anki-forge-media/tmp"))
        .expect("tmp dir should be removed");

    let artifact_path = result.expect("package artifact should be created");
    let entries = artifact_entries(&artifact_path);
    assert!(
        !entries.iter().any(|entry| {
            entry
            == Path::new(
                "contracts/fixtures/phase3/inputs/.anki-forge-media/tmp/package-regression/leak.tmp"
            )
        }),
        "artifact should not include transient media-store tmp files"
    );
}
