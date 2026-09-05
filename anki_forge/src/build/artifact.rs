//! Ownership of published APKG files. Candidate files never enter this type.
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug)]
enum ArtifactStorage {
    Persistent(PathBuf),
    Temporary(tempfile::TempPath),
}

/// A published APKG. Clones share ownership of temporary output.
///
/// Without `output` or `artifacts_dir`, the last handle removes the APKG.
/// Retain this handle (not just its path), or call `persist_to` to keep a copy.
/// Explicit destinations are caller-owned and are never removed on drop.
///
/// ```no_run
/// use anki_forge::prelude::*;
/// # fn example(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
/// let report = project.build(BuildOptions::new())?;
/// let artifact = report.artifact.as_ref().ok_or("missing artifact")?.clone();
/// drop(report); // the cloned handle still keeps the APKG alive
/// let saved = artifact.persist_to("deck.apkg")?;
/// drop(artifact); // removes only the temporary original
/// assert!(saved.path().is_file());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ApkgArtifact {
    storage: Arc<ArtifactStorage>,
}

impl ApkgArtifact {
    pub(crate) fn persistent(path: PathBuf) -> Self {
        Self {
            storage: Arc::new(ArtifactStorage::Persistent(path)),
        }
    }

    pub(crate) fn temporary_copy(candidate: &Path) -> io::Result<Self> {
        let mut file = tempfile::Builder::new()
            .prefix("anki-forge-artifact-")
            .suffix(".apkg")
            .tempfile()?;
        io::copy(&mut std::fs::File::open(candidate)?, &mut file)?;
        file.as_file().sync_all()?;
        Ok(Self {
            storage: Arc::new(ArtifactStorage::Temporary(file.into_temp_path())),
        })
    }

    /// Borrow the path while keeping an artifact handle alive.
    pub fn path(&self) -> &Path {
        match self.storage.as_ref() {
            ArtifactStorage::Persistent(path) => path,
            ArtifactStorage::Temporary(path) => path.as_ref(),
        }
    }

    /// Atomically copy this APKG to a caller-owned destination.
    ///
    /// Existing destination contents are replaced only after the copy succeeds.
    /// On failure this handle and its original file remain usable. Other clones
    /// retain their original lifetime; the returned handle owns no cleanup.
    pub fn persist_to(&self, path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        // A temporary file cannot become persistent by copying onto itself:
        // its existing clones still own deletion of that pathname.
        if self.path() == path || (path.exists() && same_file::is_same_file(self.path(), path)?) {
            return match self.storage.as_ref() {
                ArtifactStorage::Persistent(_) => Ok(self.clone()),
                ArtifactStorage::Temporary(_) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "choose a destination distinct from the temporary artifact",
                )),
            };
        }
        replace_output_atomically(self.path(), path, false)?;
        Ok(Self::persistent(path.to_path_buf()))
    }

    /// Construct reporting fixtures through the unsupported tools interface.
    #[cfg(feature = "internal-tools")]
    #[doc(hidden)]
    pub fn from_persistent_path(path: PathBuf) -> Self {
        Self::persistent(path)
    }
}

impl PartialEq for ApkgArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}
impl Eq for ApkgArtifact {}

pub(crate) fn replace_output_atomically(
    temp_artifact: &Path,
    target: &Path,
    force_failure_for_test: bool,
) -> std::io::Result<()> {
    let parent = target.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output path has no parent",
        )
    })?;
    std::fs::create_dir_all(parent)?;
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("deck.apkg");
    let mut temp_target = tempfile::Builder::new()
        .prefix(&format!(".{file_name}."))
        .suffix(".tmp")
        .tempfile_in(parent)?;
    std::io::copy(
        &mut std::fs::File::open(temp_artifact)?,
        temp_target.as_file_mut(),
    )?;
    temp_target.as_file_mut().sync_all()?;
    if force_failure_for_test {
        return Err(std::io::Error::other("forced output replace failure"));
    }
    temp_target.persist(target).map_err(|err| err.error)?;
    Ok(())
}
