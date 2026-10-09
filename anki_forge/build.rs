use std::{env, fs, path::PathBuf};

#[path = "src/runtime/default_models.rs"]
mod default_models;
#[path = "src/runtime/embedded_bundle.rs"]
mod embedded_bundle;
#[path = "src/runtime/manifest.rs"]
mod manifest;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let contracts_dir = manifest_dir.join("assets/contracts");
    let mut bundles = fs::read_dir(&contracts_dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", contracts_dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("anki-forge-contract-bundle-") && name.ends_with(".tar.gz")
                })
        })
        .collect::<Vec<_>>();
    bundles.sort();
    assert_eq!(
        bundles.len(),
        1,
        "expected exactly one embedded contract bundle in {}",
        contracts_dir.display()
    );

    let bundle_path = &bundles[0];
    let filename = bundle_path.file_name().unwrap().to_str().unwrap();
    let version = filename
        .strip_prefix("anki-forge-contract-bundle-")
        .and_then(|value| value.strip_suffix(".tar.gz"))
        .filter(|value| !value.is_empty())
        .expect("embedded contract bundle filename must carry a version");

    println!("cargo:rerun-if-changed={}", contracts_dir.display());
    for source in [
        "src/runtime/default_models.rs",
        "src/runtime/embedded_bundle.rs",
        "src/runtime/manifest.rs",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }
    let bytes = fs::read(bundle_path).expect("read embedded contract bundle");
    let defaults = embedded_bundle::decode_writer_defaults(&bytes, version)
        .unwrap_or_else(|error| panic!("validate embedded contract bundle: {error:#}"));
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("writer-defaults.json");
    fs::write(
        output,
        serde_json::to_vec(&defaults).expect("serialize embedded writer defaults"),
    )
    .expect("write generated writer defaults");

    println!(
        "cargo:rustc-env=ANKI_FORGE_EMBEDDED_BUNDLE_PATH={}",
        bundle_path.display()
    );
    println!("cargo:rustc-env=ANKI_FORGE_EMBEDDED_BUNDLE_VERSION={version}");
}
