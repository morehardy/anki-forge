pub mod canonical_json;
pub mod identity;
pub mod media;
pub mod media_io;
pub mod model;
pub mod normalize;
pub mod risk;
pub mod selector;
pub mod stock;

pub use canonical_json::to_canonical_json;
pub use identity::{resolve_identity, DefaultNonceSource, NonceSource};
pub use media::{
    ingest_authoring_media, media_object_id, media_object_ref, sort_media_bindings,
    sort_media_objects, sort_media_references, AuthoringMediaSource, DiagnosticBehavior,
    MediaBinding, MediaIngestDiagnostic, MediaIngestError, MediaIngestResult, MediaObject,
    MediaPolicy, MediaReference, MediaReferenceResolution, NormalizeOptions,
};
pub use media_io::{
    decode_inline_bytes, ingest_media_read_source_to_cas, object_store_path,
    CasExistingIntegrityReason, IngestedMediaBytes, MediaIoError, MediaReadSource,
    MediaSniffConfidence, SniffedMime,
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
