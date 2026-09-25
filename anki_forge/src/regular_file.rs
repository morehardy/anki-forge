//! Open untrusted input without blocking on a replacement FIFO.
use std::{fs::File, io, path::Path};

#[derive(Debug)]
pub(crate) enum OpenError {
    Io(io::Error),
    NotRegular,
    LimitExceeded { limit: u64, observed: u64 },
}

impl From<io::Error> for OpenError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub(crate) fn open(path: &Path, max_bytes: u64) -> Result<File, OpenError> {
    #[cfg(unix)]
    let source = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map(File::from)
    .map_err(io::Error::from)?;
    #[cfg(not(unix))]
    let source = File::open(path)?;
    let metadata = source.metadata()?;
    if !metadata.is_file() {
        return Err(OpenError::NotRegular);
    }
    if metadata.len() > max_bytes {
        return Err(OpenError::LimitExceeded {
            limit: max_bytes,
            observed: metadata.len(),
        });
    }
    #[cfg(unix)]
    {
        // Restore blocking reads only on the verified regular descriptor, never
        // reopening the pathname. Retain every other status flag.
        let flags = rustix::fs::fcntl_getfl(&source).map_err(io::Error::from)?;
        rustix::fs::fcntl_setfl(&source, flags & !rustix::fs::OFlags::NONBLOCK)
            .map_err(io::Error::from)?;
    }
    Ok(source)
}
