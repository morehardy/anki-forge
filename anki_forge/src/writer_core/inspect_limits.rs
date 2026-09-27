use std::fmt;

pub use crate::build::{InspectLimitExceeded, InspectLimits};

/// Inspection failure. Resource exhaustion is distinct from malformed input or
/// ordinary I/O failures, and must never be swallowed as missing media.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InspectError {
    LimitExceeded(InspectLimitExceeded),
    Read(std::sync::Arc<dyn std::error::Error + Send + Sync>),
}

impl InspectError {
    pub fn limit_exceeded(&self) -> Option<&InspectLimitExceeded> {
        match self {
            Self::LimitExceeded(limit) => Some(limit),
            Self::Read(_) => None,
        }
    }

    pub(crate) fn from_anyhow(error: anyhow::Error) -> Self {
        for cause in error.chain() {
            if let Some(limit) = cause.downcast_ref::<InspectLimitExceeded>() {
                return Self::LimitExceeded(limit.clone());
            }
            if let Some(limit) = cause
                .downcast_ref::<std::io::Error>()
                .and_then(std::io::Error::get_ref)
                .and_then(|inner| inner.downcast_ref::<InspectLimitExceeded>())
            {
                return Self::LimitExceeded(limit.clone());
            }
        }
        Self::Read(std::sync::Arc::from(
            error.reallocate_into_boxed_dyn_error_without_backtrace(),
        ))
    }
}

impl fmt::Display for InspectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded(limit) => limit.fmt(f),
            Self::Read(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for InspectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::LimitExceeded(limit) => limit,
            Self::Read(cause) => &**cause,
        })
    }
}

// Internal observation equality, never used to classify a failure or assign codes.
impl PartialEq for InspectError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LimitExceeded(a), Self::LimitExceeded(b)) => a == b,
            (Self::Read(a), Self::Read(b)) => a.to_string() == b.to_string(),
            _ => false,
        }
    }
}
impl Eq for InspectError {}

pub(crate) fn check(
    resource: &'static str,
    entry: Option<&str>,
    limit: u64,
    observed: u64,
) -> Result<(), InspectLimitExceeded> {
    if observed > limit {
        Err(InspectLimitExceeded {
            resource,
            entry: entry.map(str::to_owned),
            limit,
            observed,
        })
    } else {
        Ok(())
    }
}
