use std::path::PathBuf;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum AnkiForgeError {
    InvalidSpec(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    CompatFeatureRequired,
}

pub type Result<T, E = AnkiForgeError> = std::result::Result<T, E>;

impl fmt::Display for AnkiForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpec(message) => write!(f, "invalid package specification: {message}"),
            Self::Io { path, source } => write!(f, "io error at {}: {source}", path.display()),
            Self::CompatFeatureRequired => {
                write!(f, "compat feature is required for this operation")
            }
        }
    }
}

impl Error for AnkiForgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidSpec(_) | Self::CompatFeatureRequired => None,
        }
    }
}
