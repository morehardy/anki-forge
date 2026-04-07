pub use authoring_core::{
    assess_risk, normalize, parse_selector, resolve_identity, resolve_selector,
    to_canonical_json as to_authoring_canonical_json, AuthoringDocument, AuthoringMedia,
    AuthoringNote, AuthoringNotetype, ComparisonContext, MergeRiskReport, NormalizationRequest,
    NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype, NormalizedTemplate, Selector,
    SelectorError, SelectorResolveError, SelectorTarget,
};
pub use authoring_core::model::NormalizationResult;
pub use writer_core::{
    build, build_context_ref, diff_reports, extract_media_references, inspect_apkg,
    inspect_build_result, inspect_staging, policy_ref, to_canonical_json as to_writer_canonical_json,
    BuildArtifactTarget, BuildContext, DiffReport, InspectReport, PackageBuildResult,
    VerificationGateRule, VerificationPolicy, WriterPolicy,
};

pub fn authoring_tool_contract_version() -> &'static str {
    authoring_core::tool_contract_version()
}

pub fn writer_tool_contract_version() -> &'static str {
    writer_core::tool_contract_version()
}

pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
