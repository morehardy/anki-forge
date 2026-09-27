use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

/// The category of a failed media import or naming operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaErrorKind {
    /// Reading the source or writing owned snapshot storage failed.
    Io,
    /// The source path does not name a regular file.
    NotRegularFile,
    /// The per-asset byte budget was exceeded.
    ResourceLimit,
    /// The declared MIME type is not a concrete, syntactically valid media type.
    InvalidMediaType,
    /// Recognized file contents contradict the declared media type.
    MediaTypeMismatch,
    /// The export filename is unsafe or not portable.
    InvalidName,
}

/// Observations from a media read that exceeded its configured budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaLimitExceeded {
    /// Counted resource; currently `media_bytes`.
    pub resource: &'static str,
    /// Configured maximum number of bytes.
    pub limit: u64,
    /// Known source length or bytes observed before the stream was stopped.
    pub observed: u64,
}

/// Media could not be imported or named. Underlying I/O and MIME errors remain
/// available through [`Error::source`]. No partial snapshot is returned.
#[derive(Debug)]
pub struct MediaError {
    kind: MediaErrorKind,
    code: &'static str,
    message: String,
    path: Option<PathBuf>,
    limit: Option<MediaLimitExceeded>,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl MediaError {
    pub(crate) fn new(
        kind: MediaErrorKind,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
            path: None,
            limit: None,
            cause: None,
        }
    }

    pub(crate) fn caused_by(mut self, cause: impl Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    pub(crate) fn at_path(mut self, path: &Path) -> Self {
        self.path = Some(path.to_owned());
        self
    }

    pub(crate) fn io(action: &'static str, cause: std::io::Error) -> Self {
        Self::new(MediaErrorKind::Io, "MEDIA.SNAPSHOT_IO_FAILED", action).caused_by(cause)
    }

    pub(crate) fn exceeded(limit: u64, observed: u64) -> Self {
        let mut error = Self::new(
            MediaErrorKind::ResourceLimit,
            "MEDIA.RESOURCE_LIMIT_EXCEEDED",
            format!("media contains at least {observed} bytes; limit is {limit}"),
        );
        error.limit = Some(MediaLimitExceeded {
            resource: "media_bytes",
            limit,
            observed,
        });
        error
    }

    /// Returns the structural error category.
    pub fn kind(&self) -> MediaErrorKind {
        self.kind
    }

    /// Returns the registered machine code, independent of human wording.
    pub fn code(&self) -> &str {
        self.code
    }

    /// Returns the source path when the failed operation was a file import.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the limit and observed size for a budget failure.
    pub fn limit_exceeded(&self) -> Option<&MediaLimitExceeded> {
        self.limit.as_ref()
    }
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(path) = &self.path {
            write!(f, " ({})", path.display())?;
        }
        if let Some(cause) = &self.cause {
            write!(f, ": {cause}")?;
        }
        Ok(())
    }
}

impl Error for MediaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_ref()
            .map(|cause| &**cause as &(dyn Error + 'static))
    }
}
