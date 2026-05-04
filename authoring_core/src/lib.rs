pub mod canonical_json;
pub mod identity;
pub mod media;
pub mod model;
pub mod normalize;
pub mod risk;
pub mod selector;
pub mod stock;

pub use canonical_json::to_canonical_json;
pub use identity::{resolve_identity, DefaultNonceSource, NonceSource};
pub use media::{
    media_object_id, media_object_ref, sort_media_bindings, sort_media_objects,
    sort_media_references, AuthoringMediaSource, DiagnosticBehavior, MediaBinding, MediaObject,
    MediaPolicy, MediaReference, MediaReferenceResolution, NormalizeOptions,
};
pub use model::{
    AuthoringDocument, AuthoringField, AuthoringFieldMetadata, AuthoringMedia, AuthoringNote,
    AuthoringNotetype, AuthoringTemplate, ComparisonContext, MergeRiskReport, NormalizationRequest,
    NormalizedField, NormalizedFieldMetadata, NormalizedIr, NormalizedNote, NormalizedNotetype,
    NormalizedTemplate,
};
pub use normalize::normalize;
pub use risk::assess_risk;
pub use selector::{
    parse_selector, resolve_selector, Selector, SelectorError, SelectorResolveError, SelectorTarget,
};

pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}
