mod deck;

pub mod authoring;
pub mod build;
pub mod diagnostics;
pub mod diff;
pub mod prelude;
pub mod product;
pub mod risk;
pub mod runtime;
pub mod update_safety;
pub mod writer;

pub use deck::*;
pub use diagnostics::Severity;

pub fn authoring_tool_contract_version() -> &'static str {
    authoring_core::tool_contract_version()
}

pub fn writer_tool_contract_version() -> &'static str {
    writer_core::tool_contract_version()
}

pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
