use std::{error::Error, fmt};

/// The reason an image-occlusion note could not be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOcclusionErrorKind {
    /// Image content could not be decoded in a supported raster format.
    InvalidImage,
    /// A key or rectangle is invalid, duplicated or outside the displayed image.
    InvalidMask,
    /// Decoding or ordinal allocation exceeds a finite supported limit.
    ResourceLimit,
    /// Reading or normalizing the owned image failed.
    Io,
}

/// A failed local image-occlusion declaration. No project state has changed.
#[derive(Debug)]
pub struct ImageOcclusionError {
    kind: ImageOcclusionErrorKind,
    code: &'static str,
    message: String,
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl ImageOcclusionError {
    pub(super) fn new(
        kind: ImageOcclusionErrorKind,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
            cause: None,
        }
    }

    pub(super) fn caused_by(mut self, cause: impl Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Returns the structural category independently of human wording.
    pub fn kind(&self) -> ImageOcclusionErrorKind {
        self.kind
    }

    /// Returns the registered primary machine code.
    pub fn code(&self) -> &str {
        self.code
    }
}

impl fmt::Display for ImageOcclusionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}
impl Error for ImageOcclusionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref().map(|cause| cause as _)
    }
}
