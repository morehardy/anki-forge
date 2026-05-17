pub use authoring_core::model::NormalizationResult;
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
