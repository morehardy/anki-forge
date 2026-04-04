pub mod model;
pub mod normalize;
pub mod selector;

pub use model::{AuthoringDocument, ComparisonContext, NormalizationRequest};
pub use normalize::normalize;
pub use selector::{
    parse_selector, resolve_selector, Selector, SelectorError, SelectorResolveError,
    SelectorTarget,
};

pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}
