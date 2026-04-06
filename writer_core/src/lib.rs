pub mod apkg;
pub mod build;
pub mod canonical_json;
pub mod model;
pub mod policy;
pub mod staging;

pub use build::build;
pub use build::BuildArtifactTarget;
pub use canonical_json::to_canonical_json;
pub use model::*;
pub use policy::{build_context_ref, policy_ref};
pub use staging::{MaterializedStaging, StagingPackage};

pub fn tool_contract_version() -> &'static str {
    "phase3-v1"
}
