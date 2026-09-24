pub mod html_text;
pub mod identity;
pub mod media;
pub mod media_io;
pub mod media_refs;
pub mod mime;
pub mod model;
pub mod normalize;
pub mod risk;
pub mod selector;
pub mod stock;
pub mod template_parser;
pub mod template_semantics;

pub use html_text::strip_html_preserving_media_filenames;
pub use media::{
    AuthoringMediaSource, MediaObject, MediaPolicy, MediaReferenceResolution, NormalizeOptions,
};
pub use media_io::object_store_path;
pub use model::{
    AuthoringDocument, AuthoringField, AuthoringFieldMetadata, AuthoringGenerationRequirement,
    AuthoringMedia, AuthoringNote, AuthoringNotetype, AuthoringTemplate, NormalizationRequest,
    NormalizedField, NormalizedFieldMetadata, NormalizedGenerationRequirement, NormalizedIr,
    NormalizedNote, NormalizedNotetype, NormalizedTemplate,
};
#[cfg(feature = "internal-tools")]
pub use normalize::normalize;
#[cfg(feature = "internal-tools")]
pub use normalize::normalize_with_options;
pub use template_parser::{
    is_special_template_field, parse_template, TemplateParseIssueKind, TemplateToken,
};
pub use template_semantics::{infer_generation_requirement, TemplateGenerationRequirement};

#[cfg(all(test, feature = "internal-tools"))]
pub use media::{
    ingest_authoring_media, sort_media_bindings, sort_media_objects, sort_media_references,
    DiagnosticBehavior, MediaIngestResult, MediaReference,
};
#[cfg(all(test, feature = "internal-tools"))]
pub use media_io::{
    decode_inline_bytes, ingest_media_read_source_to_cas, CasExistingIntegrityReason, MediaIoError,
    MediaReadSource, MediaSniffConfidence,
};
#[cfg(all(test, feature = "internal-tools"))]
pub use media_refs::{
    extract_media_reference_candidates, MediaReferenceCandidate, MediaReferenceCandidateKind,
};
#[cfg(all(test, feature = "internal-tools"))]
pub use model::ComparisonContext;
#[cfg(feature = "internal-tools")]
pub use model::NormalizationResult;

pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}

#[cfg(test)]
pub use media::MediaBinding;
