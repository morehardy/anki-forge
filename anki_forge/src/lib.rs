#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[allow(missing_docs)]
mod authoring_core;
#[allow(missing_docs)]
mod deck;
#[allow(missing_docs)]
mod writer_core;

/// Low-level authoring and normalization data structures.
#[allow(missing_docs)]
pub mod authoring;
/// Build configuration, reports, statuses, and policy results.
#[allow(missing_docs)]
pub mod build;
/// Stable diagnostic codes and severity values.
#[allow(missing_docs)]
pub mod diagnostics;
/// Semantic and artifact diff summaries.
#[allow(missing_docs)]
pub mod diff;
/// Common high-level types for typical consumers.
#[allow(missing_docs)]
pub mod prelude;
/// Typed long-lived project, note type, template, and media APIs.
#[allow(missing_docs)]
pub mod product;
/// Import-risk classification and policy APIs.
#[allow(missing_docs)]
pub mod risk;
/// Embedded contracts and lower-level runtime entry points.
#[allow(missing_docs)]
pub mod runtime;
/// Identity lockfile and update-safe build support.
#[allow(missing_docs)]
pub mod update_safety;
/// Low-level writer and artifact-inspection APIs.
#[allow(missing_docs)]
pub mod writer;

pub use deck::*;
pub use diagnostics::Severity;

/// Returns the compatibility version of the authoring contract surface.
pub fn authoring_tool_contract_version() -> &'static str {
    crate::authoring_core::tool_contract_version()
}

/// Returns the compatibility version of the writer contract surface.
pub fn writer_tool_contract_version() -> &'static str {
    crate::writer_core::tool_contract_version()
}

/// Returns the SemVer version of this public crate.
pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
