#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[allow(missing_docs)]
mod authoring_core;
#[allow(missing_docs)]
#[path = "diagnostics/mod.rs"]
mod diagnostics_backend;
mod parallel_io;
mod prepared_media;
#[allow(missing_docs)]
mod product;
#[allow(missing_docs)]
mod runtime;
#[allow(missing_docs)]
mod writer_core;

#[path = "build_api/mod.rs"]
pub mod build;
#[path = "diagnostics_api.rs"]
pub mod diagnostics;
pub mod media;
pub mod note;
pub mod schema;
pub mod update;

mod project;
pub use build::{BuildOptions, BuildOutput};
pub use media::Media;
pub use note::{Content, Note};
pub use project::Project;
pub use schema::{Field, NoteType, Template};

/// Operations and transport values used by this repository's contract tools.
/// This optional interface is outside the supported consumer API.
#[cfg(feature = "internal-tools")]
#[doc(hidden)]
pub mod tools;

/// Returns the SemVer version of this public crate.
pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the compatibility version of the embedded contract resources.
/// Normal [`Project`] builds load these resources automatically.
pub const fn embedded_contract_version() -> &'static str {
    crate::runtime::embedded_bundle_version()
}

#[cfg(all(test, feature = "internal-tools"))]
#[path = "../tests/internal/mod.rs"]
mod internal_tests;
