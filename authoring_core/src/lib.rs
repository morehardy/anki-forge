pub mod canonical_json;
pub mod identity;
pub mod model;
pub mod normalize;
pub mod risk;
pub mod selector;
pub mod stock;

pub use canonical_json::to_canonical_json;
pub use identity::{resolve_identity, DefaultNonceSource, NonceSource};
pub use model::{
    AuthoringDocument, AuthoringMedia, AuthoringNote, AuthoringNotetype, ComparisonContext,
    MergeRiskReport, NormalizationRequest, NormalizedIr, NormalizedMedia, NormalizedNote,
    NormalizedNotetype, NormalizedTemplate,
};
pub use normalize::normalize;
pub use risk::assess_risk;
pub use selector::{
    parse_selector, resolve_selector, Selector, SelectorError, SelectorResolveError, SelectorTarget,
};

pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}
