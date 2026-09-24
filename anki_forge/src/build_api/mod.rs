//! Build requests, successful artifacts, structured failures and observations.

mod artifact;
mod candidate;
mod error;
pub(crate) mod identity;
pub mod json;
mod limits;
mod normalize;
mod pipeline;
mod report;

pub use artifact::ApkgArtifact;
pub use error::{BuildError, BuildErrorKind, PersistError, PersistErrorKind};
pub use limits::{InspectLimitExceeded, InspectLimits};
pub use report::{BuildCounts, BuildOutput, BuildReport};

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OutputTarget {
    Persistent(PathBuf),
    Temporary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BuildMode {
    Create,
    Update(PathBuf),
}

/// An explicit output destination and finite inspection budgets.
///
/// Use [`Self::to`] for persistent output or [`Self::temporary`] for a file owned
/// by the returned artifact handle. Constructing options does not execute a build.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "pass these options to Project::build"]
pub struct BuildOptions {
    pub(crate) output: OutputTarget,
    pub(crate) inspect_limits: InspectLimits,
    pub(crate) mode: BuildMode,
    pub(crate) update_policy: Option<crate::update::UpdatePolicy>,
}

impl BuildOptions {
    /// Requests an APKG at a caller-owned path, atomically replacing any old file.
    pub fn to(path: impl Into<PathBuf>) -> Self {
        Self {
            output: OutputTarget::Persistent(path.into()),
            inspect_limits: InspectLimits::default(),
            mode: BuildMode::Create,
            update_policy: None,
        }
    }

    /// Requests an APKG that is removed when its last artifact handle is dropped.
    pub fn temporary() -> Self {
        Self {
            output: OutputTarget::Temporary,
            inspect_limits: InspectLimits::default(),
            mode: BuildMode::Create,
            update_policy: None,
        }
    }

    /// Overrides the finite budgets for each inspected APKG independently.
    pub fn inspect_limits(mut self, limits: InspectLimits) -> Self {
        self.inspect_limits = limits;
        self
    }

    /// Updates from the previous original distribution APKG. Complete embedded
    /// identity evidence is required; an Anki re-export is not a baseline.
    pub fn update_from(mut self, path: impl Into<PathBuf>) -> Self {
        self.mode = BuildMode::Update(path.into());
        self
    }

    /// Sets publication risk policy for an Update request. Explicit use on a
    /// Create request is a configuration error, regardless of setter order.
    pub fn update_policy(mut self, policy: crate::update::UpdatePolicy) -> Self {
        self.update_policy = Some(policy);
        self
    }
}
