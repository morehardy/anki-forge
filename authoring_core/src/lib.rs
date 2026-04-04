pub mod model;
pub mod normalize;

pub use model::{AuthoringDocument, ComparisonContext, NormalizationRequest};
pub use normalize::normalize;

pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}
