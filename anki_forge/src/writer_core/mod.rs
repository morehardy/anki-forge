mod anki_proto;
mod compat_schema;
mod deck_name;
pub(crate) mod identity;
pub(crate) mod note_revision;

pub mod apkg;
pub mod build;
pub mod canonical_json;
pub mod card_plan;
pub mod diff;
pub mod inspect;
pub mod media;
pub mod media_refs;
pub mod model;
pub mod policy;
pub mod staging;

pub(crate) use build::build_with_identity_plan;
pub use build::BuildArtifactTarget;
pub use build::{build, build_with_guid_plan};
pub use canonical_json::to_canonical_json;
pub use diff::diff_reports;
pub use inspect::{artifact_path_from_ref, inspect_apkg, inspect_build_result, inspect_staging};
pub use media_refs::extract_media_references;
pub use model::*;
pub use policy::{build_context_ref, policy_ref};
pub use staging::{MaterializedStaging, StagingPackage};

pub fn tool_contract_version() -> &'static str {
    "phase3-v1"
}
