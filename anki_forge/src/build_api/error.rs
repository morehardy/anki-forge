use super::{
    json::{BuildResultSnapshot, BuildSnapshot, PublicationSnapshot},
    BuildReport,
};
use serde::Serialize;
use std::{error::Error, fmt, io};

/// Structured classification of a failed build request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildErrorKind {
    /// The request configuration is invalid.
    Configuration,
    /// Authored content or required identity evidence failed validation.
    Validation,
    /// A required operation exceeded a finite resource budget.
    ResourceLimit,
    /// A source or private working file could not be read or written.
    Io,
    /// Publishing the requested destination failed, possibly after replacement.
    Publication,
    /// The completed analysis did not satisfy the requested update policy.
    PolicyBlocked,
    /// An internal invariant failed.
    Internal,
}

/// A failed build, including collected observations and actual publication facts.
/// Source errors retain their concrete types; Display is never parsed for codes.
#[derive(Debug)]
pub struct BuildError {
    kind: BuildErrorKind,
    code: Box<str>,
    message: Box<str>,
    pub(crate) report: Box<BuildReport>,
    pub(crate) publications: Vec<PublicationSnapshot>,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl BuildError {
    pub(crate) fn new(
        kind: BuildErrorKind,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code: code.into().into_boxed_str(),
            message: message.into().into_boxed_str(),
            report: Box::default(),
            publications: Vec::new(),
            cause: None,
        }
    }

    pub(crate) fn caused_by(mut self, cause: impl Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    pub(crate) fn caused_by_anyhow(mut self, cause: anyhow::Error) -> Self {
        self.cause = Some(cause.reallocate_into_boxed_dyn_error_without_backtrace());
        self
    }

    /// Returns the error category independently of human wording.
    pub fn kind(&self) -> BuildErrorKind {
        self.kind
    }

    /// Returns the primary registered machine code assigned when the operation failed.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the exact inspection budget and observed count, if exceeded.
    pub fn limit_exceeded(&self) -> Option<&super::InspectLimitExceeded> {
        let mut source = self.source();
        while let Some(cause) = source {
            if let Some(limit) = cause.downcast_ref::<super::InspectLimitExceeded>() {
                return Some(limit);
            }
            source = cause.source();
        }
        None
    }

    /// Returns the observations actually collected before the request failed.
    pub fn report(&self) -> &BuildReport {
        &self.report
    }

    /// Returns actual publication stages, including any late durability failure.
    pub fn publications(&self) -> &[PublicationSnapshot] {
        &self.publications
    }

    /// Records this error's actual failure outcome without owning any file.
    pub fn snapshot(&self) -> BuildSnapshot {
        let mut causes = Vec::new();
        let mut source = self.source();
        while let Some(cause) = source {
            causes.push(cause.to_string());
            source = cause.source();
        }
        BuildSnapshot {
            schema_version: "ankiforge-build-v1".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            result: BuildResultSnapshot::Failure {
                kind: self.kind,
                code: self.code.to_string(),
                message: self.to_string(),
                causes,
                publications: self.publications.clone(),
            },
            report: self.report.snapshot(),
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(cause) = &self.cause {
            write!(f, ": {cause}")?;
        }
        Ok(())
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_ref()
            .map(|cause| &**cause as &(dyn Error + 'static))
    }
}

/// Classification of an artifact persistence failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistErrorKind {
    /// The destination is empty or aliases a temporary source.
    InvalidDestination,
    /// File preparation, copying, replacement or durability confirmation failed.
    Io,
}

/// Failed persistence with the original I/O cause and accurate publication facts.
/// The source artifact remains usable after this error.
#[derive(Debug)]
pub struct PersistError {
    kind: PersistErrorKind,
    cause: io::Error,
    publication: PublicationSnapshot,
}

impl PersistError {
    pub(crate) fn into_io(self) -> io::Error {
        self.cause
    }

    pub(crate) fn new(
        kind: PersistErrorKind,
        cause: io::Error,
        publication: PublicationSnapshot,
    ) -> Self {
        Self {
            kind,
            cause,
            publication,
        }
    }

    /// Returns the operation category, independent of the I/O message.
    pub fn kind(&self) -> PersistErrorKind {
        self.kind
    }

    /// Returns the registered machine code for this persistence failure.
    pub fn code(&self) -> &str {
        match self.kind {
            PersistErrorKind::InvalidDestination => "PERSIST.DESTINATION_INVALID",
            PersistErrorKind::Io => "PERSIST.IO_FAILED",
        }
    }

    /// Returns the destination's actual publication stage and durability state.
    pub fn publication(&self) -> &PublicationSnapshot {
        &self.publication
    }
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.cause)
    }
}

impl Error for PersistError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
