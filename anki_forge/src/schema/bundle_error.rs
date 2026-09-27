use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

/// The operation that prevented a template bundle from becoming a complete model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateBundleErrorKind {
    /// A directory or file could not be read; the original I/O error is retained.
    Io,
    /// The manifest cannot be parsed or contains unsupported declarations.
    InvalidManifest,
    /// The bundle declares a format other than `template-bundle-v2`.
    UnsupportedVersion,
    /// A referenced path escapes the bundle directory or is not portable.
    UnsafePath,
    /// A referenced input is not a regular UTF-8 text file.
    InvalidFile,
    /// A manifest, text file or media asset exceeds its read budget.
    ResourceLimit,
    /// The declared fields, templates or model assets fail schema validation.
    InvalidSchema,
    /// An asset's content or export filename is invalid.
    InvalidMedia,
}

/// A manifest or template text exceeded its finite byte budget.
///
/// Asset budget failures instead retain [`crate::media::MediaError`] as their
/// source, with its [`crate::media::MediaLimitExceeded`] observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateBundleLimitExceeded {
    /// Maximum bytes permitted in the input.
    pub limit: u64,
    /// Observed bytes, which may be only the prefix needed to establish failure.
    pub observed: u64,
}

impl fmt::Display for TemplateBundleLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "bundle input exceeds {} bytes (observed {})",
            self.limit, self.observed
        )
    }
}
impl Error for TemplateBundleLimitExceeded {}

/// Loading failed without returning a partial model or retaining unfinished
/// assets. Machine decisions use [`Self::kind`] and [`Self::code`].
#[derive(Debug)]
pub struct TemplateBundleError {
    kind: TemplateBundleErrorKind,
    code: String,
    message: String,
    path: Option<PathBuf>,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl TemplateBundleError {
    pub(super) fn new(
        kind: TemplateBundleErrorKind,
        code: &str,
        message: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            path: Some(path.into()),
            cause: None,
        }
    }

    pub(super) fn caused_by(mut self, error: impl Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(error));
        self
    }

    /// Returns the failed operation's structural category.
    pub fn kind(&self) -> TemplateBundleErrorKind {
        self.kind
    }

    /// Returns the primary machine code, assigned without parsing error text.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the failing input's path, including template origins when known.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the UTF-8 offset in the original template for schema failures.
    /// Non-template failures have no template offset.
    pub fn byte_offset(&self) -> Option<usize> {
        self.source()?
            .downcast_ref::<super::SchemaError>()?
            .location()
            .map(|location| location.byte_range.start)
    }
}

impl fmt::Display for TemplateBundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}
impl Error for TemplateBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref().map(|cause| cause as _)
    }
}
