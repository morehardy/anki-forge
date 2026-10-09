use std::path::{Path, PathBuf};

use super::{Media, MediaError, MediaLimits};

impl Media {
    /// Imports owned file snapshots in input order, using at most four workers.
    /// Batches smaller than 16 files run serially. Each asset uses the default
    /// byte limit; retained snapshots share the process-wide memory budget.
    ///
    /// All started work is joined before returning. On failure, returns the
    /// first error in input order and drops every snapshot owned by this batch.
    /// Existing media owners remain valid. Files are snapshotted independently,
    /// so this is not an atomic view of a concurrently changing directory.
    pub fn files<P: AsRef<Path>>(
        paths: impl IntoIterator<Item = P>,
    ) -> Result<Vec<Self>, MediaError> {
        Self::files_with_limits(paths, MediaLimits::default())
    }

    /// Like [`Media::files`], with a byte limit applied to each individual file.
    pub fn files_with_limits<P: AsRef<Path>>(
        paths: impl IntoIterator<Item = P>,
        limits: MediaLimits,
    ) -> Result<Vec<Self>, MediaError> {
        let paths: Vec<PathBuf> = paths.into_iter().map(|p| p.as_ref().to_owned()).collect();
        if paths.len() < 16 {
            return paths
                .iter()
                .map(|path| Self::file_with_limits(path, limits))
                .collect();
        }
        crate::parallel_io::ordered(paths.len(), |index| {
            Self::file_with_limits(&paths[index], limits)
        })
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            result.unwrap_or_else(|()| {
                Err(MediaError::io(
                    "media import worker panicked",
                    std::io::Error::other("media import worker panicked"),
                )
                .at_path(&paths[index]))
            })
        })
        .collect()
    }
}
