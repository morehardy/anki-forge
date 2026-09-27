//! Selected operations for repository conformance tools.
//!
//! Available only with `internal-tools`. Contract IR is intentionally available
//! here for fixture and oracle work; ordinary authoring uses [`crate::Project`].
//! Implementation modules and the retired authoring containers remain private.
#![allow(missing_docs)]

mod project_input;
pub use project_input::load_project;
mod template_paths;
pub use template_paths::contract_template_bundle_paths;

use std::path::Path;

// Shared with core publication checks; not part of the supported consumer API.
pub use crate::path_alias::paths_alias;

// These are the concrete DTOs consumed by the contract fixture runner and
// compatibility oracle, including the field types of their public signatures.
pub use crate::authoring_core::media::{
    AuthoringMediaSource, DiagnosticBehavior, MediaBinding, MediaObject, MediaPolicy,
    MediaReference, MediaReferenceResolution, NormalizeOptions,
};
pub use crate::authoring_core::model::{
    AuthoringDocument, AuthoringField, AuthoringFieldMetadata, AuthoringGenerationRequirement,
    AuthoringMedia, AuthoringNote, AuthoringNotetype, AuthoringTemplate, ComparisonContext,
    DiagnosticItem, MergeRiskReport, NormalizationDiagnostics, NormalizationRequest,
    NormalizationResult, NormalizedField, NormalizedFieldMetadata, NormalizedGenerationRequirement,
    NormalizedIr, NormalizedNote, NormalizedNotetype, NormalizedTemplate, PolicyRefs,
};
pub use crate::authoring_core::stock::resolve_stock_notetype;
pub use crate::authoring_core::{normalize, normalize_with_options};
pub use crate::runtime::{
    build_from_path, diff_from_paths, discover_workspace_runtime, inspect_apkg_path,
    inspect_staging_path, normalize_from_path, ResolvedRuntime, RuntimeMode,
};
pub use crate::writer_core::model::{
    BuildContext, BuildDiagnosticItem, BuildDiagnostics, DiffChange, DiffReport,
    InspectObservations, InspectReport, PackageBuildResult, WriterPolicy,
};
pub use crate::writer_core::{
    build as build_contract, build_context_ref, diff_reports, extract_media_references,
    inspect_apkg, inspect_staging, policy_ref, BuildArtifactTarget, InspectError,
};

/// Loads and validates a contract bundle for path-based conformance operations.
pub fn load_runtime(manifest: impl AsRef<Path>) -> anyhow::Result<ResolvedRuntime> {
    Ok(crate::runtime::load_bundle_from_manifest(manifest)?.runtime)
}

/// Serializes a contract result using sorted JSON object keys.
pub fn canonical_json(value: &impl serde::Serialize) -> anyhow::Result<String> {
    crate::writer_core::to_canonical_json(value)
}

/// Returns the low-level authoring protocol version exercised by fixture gates.
pub fn authoring_contract_version() -> &'static str {
    crate::authoring_core::tool_contract_version()
}

/// Returns the low-level writer protocol version exercised by fixture gates.
pub fn writer_contract_version() -> &'static str {
    crate::writer_core::tool_contract_version()
}
