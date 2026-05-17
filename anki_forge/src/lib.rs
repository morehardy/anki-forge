mod deck;

pub mod authoring;
pub mod build;
pub mod diagnostics;
pub mod prelude;
pub mod product;
pub mod runtime;
pub mod writer;

pub use deck::*;

// Backward-compatible root re-exports. New user docs should use `prelude`,
// `authoring`, or `writer`.
#[deprecated(
    note = "use anki_forge::prelude for Product API or anki_forge::authoring / anki_forge::writer for advanced APIs"
)]
pub use authoring_core::model::NormalizationResult;
#[deprecated(
    note = "use anki_forge::prelude for Product API or anki_forge::authoring / anki_forge::writer for advanced APIs"
)]
pub use authoring_core::{
    assess_risk, normalize, normalize_with_options, parse_selector, resolve_identity,
    resolve_selector, to_canonical_json as to_authoring_canonical_json, AuthoringDocument,
    AuthoringField, AuthoringFieldMetadata, AuthoringMedia, AuthoringMediaSource, AuthoringNote,
    AuthoringNotetype, AuthoringTemplate, ComparisonContext, DiagnosticBehavior, MediaBinding,
    MediaObject, MediaPolicy, MediaReference, MediaReferenceResolution, MergeRiskReport,
    NormalizationRequest, NormalizeOptions, NormalizedField, NormalizedFieldMetadata, NormalizedIr,
    NormalizedNote, NormalizedNotetype, NormalizedTemplate, Selector, SelectorError,
    SelectorResolveError, SelectorTarget,
};
#[deprecated(
    note = "use anki_forge::prelude for Product API or anki_forge::authoring / anki_forge::writer for advanced APIs"
)]
pub use writer_core::{
    build as writer_build, build_context_ref, diff_reports, extract_media_references, inspect_apkg,
    inspect_build_result, inspect_staging, policy_ref,
    to_canonical_json as to_writer_canonical_json, BuildArtifactTarget, BuildContext, DiffReport,
    InspectReport, PackageBuildResult, VerificationGateRule, VerificationPolicy, WriterPolicy,
};

#[deprecated(
    note = "use anki_forge::prelude for Product API or anki_forge::authoring / anki_forge::writer for advanced APIs"
)]
#[allow(non_upper_case_globals)]
pub const build: fn(
    &authoring_core::NormalizedIr,
    &writer_core::WriterPolicy,
    &writer_core::BuildContext,
    &writer_core::BuildArtifactTarget,
) -> anyhow::Result<writer_core::PackageBuildResult> = writer_core::build;

pub fn authoring_tool_contract_version() -> &'static str {
    authoring_core::tool_contract_version()
}

pub fn writer_tool_contract_version() -> &'static str {
    writer_core::tool_contract_version()
}

pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
