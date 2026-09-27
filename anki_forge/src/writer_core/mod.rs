pub(crate) mod anki_proto;
mod apkg_index;
mod apkg_reader;
mod compat_schema;
pub(crate) mod deck_name;
pub(crate) mod identity;
mod inspect_limits;
pub(crate) mod note_data;
pub(crate) mod note_revision;
mod pipelined_sha1;
pub(crate) mod stream_zip;

pub mod apkg;
pub mod build;
pub mod canonical_json;
pub mod card_plan;
#[cfg(any(test, feature = "internal-tools"))]
pub mod diff;
pub mod inspect;
pub mod media;
pub mod media_refs;
pub mod model;
pub mod policy;
pub mod staging;

#[cfg(feature = "internal-tools")]
pub use build::build;
#[cfg(all(test, feature = "internal-tools"))]
pub use build::build_with_guid_plan;
pub use build::BuildArtifactTarget;
pub use canonical_json::to_canonical_json;
#[cfg(feature = "internal-tools")]
pub use diff::diff_reports;
pub use inspect::artifact_path_from_ref;
#[cfg(any(test, feature = "internal-tools"))]
pub use inspect::inspect_apkg;
#[cfg(feature = "internal-tools")]
pub use inspect::inspect_staging;
#[cfg(all(test, feature = "internal-tools"))]
pub use inspect::{inspect_apkg_with_limits, inspect_build_result};
pub use inspect_limits::InspectError;
#[cfg(all(test, feature = "internal-tools"))]
pub use inspect_limits::InspectLimits;
#[cfg(feature = "internal-tools")]
pub use media_refs::extract_media_references;
pub use model::*;
#[cfg(feature = "internal-tools")]
pub use policy::{build_context_ref, policy_ref};
#[cfg(all(test, feature = "internal-tools"))]
pub use staging::StagingPackage;

pub fn tool_contract_version() -> &'static str {
    "phase3-v1"
}
