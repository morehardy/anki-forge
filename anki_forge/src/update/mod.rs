//! Complete comparisons and explicit risk policies for updates from original APKGs.

mod analysis;
mod error;
pub mod json;
mod policy;
mod report;

pub use error::{CompareError, CompareErrorKind, PolicyError, PolicyErrorKind};
pub use policy::{RiskCode, RiskLevel, UpdatePolicy};
pub use report::{ComparisonEvidence, ComparisonReport, PolicyEvaluation, RiskFinding};

use crate::{build::InspectLimits, Project};
use std::path::PathBuf;

/// A comparison against a complete previous distribution, with per-package budgets.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "pass these options to Project::compare"]
pub struct CompareOptions {
    baseline: PathBuf,
    limits: InspectLimits,
    policy: UpdatePolicy,
}

impl CompareOptions {
    /// Selects the original previous APKG, retaining its full identity evidence.
    pub fn against(path: impl Into<PathBuf>) -> Self {
        Self {
            baseline: path.into(),
            limits: InspectLimits::default(),
            policy: UpdatePolicy::default(),
        }
    }

    /// Applies these finite budgets independently to baseline and candidate.
    pub fn inspect_limits(mut self, limits: InspectLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Evaluates publishability using this policy without publishing an artifact.
    pub fn update_policy(mut self, policy: UpdatePolicy) -> Self {
        self.policy = policy;
        self
    }
}

impl Project {
    /// Completes an update analysis without publishing. A blocked policy is a
    /// successful comparison; unreadable or invalid evidence is an error.
    pub fn compare(&self, options: CompareOptions) -> Result<ComparisonReport, CompareError> {
        let request = crate::BuildOptions::temporary()
            .update_from(options.baseline)
            .inspect_limits(options.limits)
            .update_policy(options.policy);
        let (_, report) = self
            .prepare_candidate(&request)
            .map_err(CompareError::from_build)?;
        Ok(report
            .comparison
            .expect("an update candidate includes a completed comparison"))
    }
}
