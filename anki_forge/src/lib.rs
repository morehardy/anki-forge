#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "internal-tools"), allow(dead_code, unused_imports))]

#[allow(missing_docs)]
mod authoring_core;
#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod deck;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod deck;
#[allow(missing_docs)]
mod writer_core;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod authoring;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod authoring;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod build;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod build;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod diagnostics;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod diagnostics;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod diff;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod diff;

/// The supported 0.1 consumer interface.
///
/// Importing this module provides the types needed for normal deck and project
/// construction without exposing the crate's contract, writer, or persistence
/// implementation.
pub mod prelude;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod product;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod product;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod risk;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod risk;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod runtime;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod runtime;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod update_safety;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod update_safety;

#[cfg(feature = "internal-tools")]
#[doc(hidden)]
#[allow(missing_docs)]
pub mod writer;
#[cfg(not(feature = "internal-tools"))]
#[allow(missing_docs)]
mod writer;

/// A convenient builder-oriented facade for creating one Anki deck.
pub use deck::Deck;
/// The severity of a structured diagnostic.
pub use diagnostics::Severity;
/// A typed project facade for custom note types, media, and update-safe builds.
pub use product::Project;

/// Returns the compatibility version of the authoring contract surface.
#[cfg(feature = "internal-tools")]
#[doc(hidden)]
pub fn authoring_tool_contract_version() -> &'static str {
    crate::authoring_core::tool_contract_version()
}

/// Returns the compatibility version of the writer contract surface.
#[cfg(feature = "internal-tools")]
#[doc(hidden)]
pub fn writer_tool_contract_version() -> &'static str {
    crate::writer_core::tool_contract_version()
}

/// Returns the SemVer version of this public crate.
pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the compatibility version of the contracts embedded in this crate.
///
/// Contract compatibility and the Rust crate's semantic version are tracked
/// independently. Normal consumers do not need to load these contracts
/// manually; [`Deck`] and [`Project`] use them automatically.
pub const fn embedded_contract_version() -> &'static str {
    crate::runtime::embedded_bundle_version()
}
