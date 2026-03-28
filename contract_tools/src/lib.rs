use std::path::PathBuf;

pub mod manifest;
pub mod registry;
pub mod schema;
pub mod semantics;

pub fn contract_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/manifest.yaml")
}
