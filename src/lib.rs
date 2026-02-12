#![forbid(unsafe_code)]

pub mod api;
pub mod cardgen;
#[cfg(feature = "compat")]
pub mod compat;
pub mod db;
pub mod determinism;
pub mod domain;
pub mod error;
pub mod guid;
pub mod io;
pub mod options;
pub mod package_builder;
pub mod prelude;
pub mod scheduler;
#[cfg(test)]
pub mod test_support;
pub mod validate;

pub use crate::error::{AnkiForgeError, Result};
pub use crate::options::{BuildMode, BuildOptions, ValidationMode};
pub use crate::package_builder::PackageBuilder;
