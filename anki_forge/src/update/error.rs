use crate::build::{BuildError, BuildErrorKind, BuildReport, InspectLimitExceeded};
use std::{error::Error, fmt};

/// Classification of policy parsing errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyErrorKind {
    /// The text is not a registered, explicitly acceptable risk category.
    UnknownRiskCode,
}
/// An invalid policy value. Display wording does not determine its machine code.
#[derive(Debug)]
pub struct PolicyError {
    input: String,
}
impl PolicyError {
    pub(super) fn unknown(input: &str) -> Self {
        Self {
            input: input.into(),
        }
    }
    /// Returns the structured classification.
    pub fn kind(&self) -> PolicyErrorKind {
        PolicyErrorKind::UnknownRiskCode
    }
    /// Returns the registered machine code.
    pub fn code(&self) -> &'static str {
        "UPDATE.RISK_CODE_INVALID"
    }
}
impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:?} is not an acceptable risk category",
            self.code(),
            self.input
        )
    }
}
impl Error for PolicyError {}

/// Why an analysis could not complete. A blocking policy is not a comparison error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareErrorKind {
    /// The comparison request configuration was invalid.
    Configuration,
    /// Content or complete identity evidence could not be validated.
    Validation,
    /// A required stage exceeded an explicit resource budget.
    ResourceLimit,
    /// A source or working file could not be read or written.
    Io,
    /// An internal generation invariant failed.
    Internal,
}
/// An incomplete comparison, retaining actual observations and original sources.
#[derive(Debug)]
pub struct CompareError {
    cause: Box<BuildError>,
}
impl CompareError {
    pub(super) fn from_build(cause: BuildError) -> Self {
        Self {
            cause: Box::new(cause),
        }
    }
    /// Returns the operation classification independently of Display.
    pub fn kind(&self) -> CompareErrorKind {
        match self.cause.kind() {
            BuildErrorKind::Configuration => CompareErrorKind::Configuration,
            BuildErrorKind::Validation => CompareErrorKind::Validation,
            BuildErrorKind::ResourceLimit => CompareErrorKind::ResourceLimit,
            BuildErrorKind::Io => CompareErrorKind::Io,
            _ => CompareErrorKind::Internal,
        }
    }
    /// Returns the original registered failure code.
    pub fn code(&self) -> &str {
        self.cause.code()
    }
    /// Observations completed before failure; no full comparison is fabricated.
    pub fn report(&self) -> &BuildReport {
        self.cause.report()
    }
    /// Exact exceeded inspection budget, if one caused the failure.
    pub fn limit_exceeded(&self) -> Option<&InspectLimitExceeded> {
        self.cause.limit_exceeded()
    }
}
impl fmt::Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "comparison did not complete: {}", self.cause)
    }
}
impl Error for CompareError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.cause)
    }
}
