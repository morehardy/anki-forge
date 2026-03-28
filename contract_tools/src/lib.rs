use std::path::PathBuf;

pub mod gates;
pub mod manifest;
pub mod fixtures;
pub mod registry;
pub mod schema;
pub mod semantics;
pub mod summary;
pub mod versioning;

pub fn contract_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/manifest.yaml")
}
