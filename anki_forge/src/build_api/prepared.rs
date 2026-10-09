use std::{fmt, path::PathBuf, time::Instant};

use super::{
    artifact::PrivateCandidate, BuildError, BuildErrorKind, BuildMode, BuildOptions, BuildOutput,
    BuildReport, OutputTarget,
};

/// An inspected private candidate, bound to its destination and update policy.
///
/// Reports are observations only. Dropping this owner deletes the unpublished
/// candidate; publishing consumes it even on failure. Reprepare to change any
/// option or retry. No authoring state is retained while a report is reviewed.
#[must_use = "review the report, then publish or drop the prepared publication"]
pub struct PreparedPublication {
    candidate: PrivateCandidate,
    report: BuildReport,
    output: OutputTarget,
    baseline: Option<BaselineLocation>,
}

impl fmt::Debug for PreparedPublication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedPublication")
            .field("report", &self.report)
            .finish_non_exhaustive()
    }
}

struct BaselineLocation {
    anchored: PathBuf,
    resolved: PathBuf,
    // Retain filesystem identity even if the original is moved after review.
    identity: same_file::Handle,
}

impl PreparedPublication {
    pub(super) fn prepare(
        project: &crate::Project,
        mut options: BuildOptions,
    ) -> Result<Self, BuildError> {
        let started = Instant::now();
        let mut result: Result<Self, BuildError> = (|| {
            if let OutputTarget::Persistent(path) = &mut options.output {
                anchor(path, "BUILD.OUTPUT_INVALID")?;
            }
            let baseline = if let BuildMode::Update(path) = &mut options.mode {
                anchor(path, "BUILD.BASELINE_INVALID")?;
                // Inspect first through the normal pipeline so baseline errors
                // retain their established classification and source chain.
                Some(path.clone())
            } else {
                None
            };
            let baseline = baseline
                .map(|anchored| {
                    let resolved = anchored.canonicalize()?;
                    let identity = same_file::Handle::from_path(&resolved)?;
                    Ok::<_, std::io::Error>(BaselineLocation {
                        anchored,
                        resolved,
                        identity,
                    })
                })
                .transpose();
            let (candidate, report) = project.prepare_candidate(&options)?;
            let baseline = baseline.map_err(|cause| {
                let mut error = path_error(cause);
                error.report = Box::new(report.clone());
                error
            })?;
            let prepared = Self {
                candidate,
                report,
                output: options.output,
                baseline,
            };
            prepared.check_destination().map_err(|mut error| {
                error.report = Box::new(prepared.report.clone());
                error
            })?;
            Ok(prepared)
        })();
        match &mut result {
            Ok(prepared) => prepared.report.duration = started.elapsed(),
            Err(error) => error.report.duration = started.elapsed(),
        }
        result
    }

    /// Borrows the complete preparation observations without exposing the file.
    pub fn report(&self) -> &BuildReport {
        &self.report
    }

    /// Publishes exactly the inspected candidate using its bound policy and path.
    /// The duration includes preparation and publication, excluding review time.
    pub fn publish(self) -> Result<BuildOutput, BuildError> {
        let started = Instant::now();
        let preparation = self.report.duration;
        let mut result = self.publish_inner();
        let duration = preparation + started.elapsed();
        match &mut result {
            Ok(output) => output.report.duration = duration,
            Err(error) => error.report.duration = duration,
        }
        result
    }

    fn check_destination(&self) -> Result<(), BuildError> {
        if let (OutputTarget::Persistent(output), Some(baseline)) = (&self.output, &self.baseline) {
            let aliases = crate::path_alias::paths_alias(output, &baseline.anchored)
                .and_then(|current| {
                    Ok(current || crate::path_alias::paths_alias(output, &baseline.resolved)?)
                })
                .map_err(path_error)?;
            let original = match same_file::Handle::from_path(output) {
                Ok(handle) => handle == baseline.identity,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => return Err(path_error(error)),
            };
            if aliases || original {
                return Err(BuildError::new(BuildErrorKind::Configuration, "BUILD.OUTPUT_INVALID",
                    "output must not replace the original update baseline; choose a separate destination"));
            }
        }
        Ok(())
    }

    fn publish_inner(self) -> Result<BuildOutput, BuildError> {
        if self
            .report
            .comparison
            .as_ref()
            .is_some_and(|c| !c.policy().allows_publication())
        {
            let mut error = BuildError::new(
                BuildErrorKind::PolicyBlocked,
                "UPDATE.POLICY_BLOCKED",
                "completed comparison contains unaccepted blocking risks",
            );
            error.report = Box::new(self.report);
            return Err(error);
        }
        self.check_destination().map_err(|mut error| {
            error.report = Box::new(self.report.clone());
            error
        })?;
        let artifact = match self.output {
            OutputTarget::Temporary => self.candidate.into_artifact(),
            OutputTarget::Persistent(path) => self.candidate.publish_to(path).map_err(|cause| {
                let publication = cause.publication().clone();
                let mut error =
                    BuildError::new(BuildErrorKind::Publication, cause.code(), "publish APKG")
                        .caused_by(cause);
                error.report = Box::new(self.report.clone());
                error.publications.push(publication);
                error
            })?,
        };
        Ok(BuildOutput {
            artifact,
            report: self.report,
        })
    }
}

fn anchor(path: &mut PathBuf, code: &str) -> Result<(), BuildError> {
    if path.as_os_str().is_empty() {
        return Err(BuildError::new(
            BuildErrorKind::Configuration,
            code,
            "path cannot be empty",
        ));
    }
    *path = std::path::absolute(&*path).map_err(|cause| {
        BuildError::new(
            BuildErrorKind::Configuration,
            code,
            "anchor publication path",
        )
        .caused_by(cause)
    })?;
    Ok(())
}

fn path_error(cause: std::io::Error) -> BuildError {
    BuildError::new(
        BuildErrorKind::Configuration,
        "BUILD.OUTPUT_INVALID",
        "could not distinguish output from update baseline",
    )
    .caused_by(cause)
}
