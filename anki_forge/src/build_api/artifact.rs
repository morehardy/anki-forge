#![deny(missing_docs)]
//! Ownership of published APKG files. Candidate files never enter this type.
use crate::build::{
    json::{Durability, PublicationSnapshot, PublicationStage},
    PersistError, PersistErrorKind,
};
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
/// With [`crate::BuildOptions::temporary`], the last handle removes the APKG.
/// Retain this handle (not just its path), or call [`Self::persist_to`] to keep a copy.
/// Explicit destinations are caller-owned and are never removed on drop.
///
/// ```no_run
/// use ankiforge::{Project, BuildOptions};
/// # fn example(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
/// let output = project.build(BuildOptions::temporary())?;
/// let artifact = output.artifact().clone();
/// drop(output); // the cloned handle still keeps the APKG alive
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

    pub(crate) fn temporary_from_candidate(candidate: &Path) -> io::Result<Self> {
        let file = tempfile::Builder::new()
            .prefix("anki-forge-artifact-")
            .suffix(".apkg")
            .tempfile()?;
        let path = file.into_temp_path();
        publish_owned_candidate(candidate, &path)?;
        Ok(Self {
            storage: Arc::new(ArtifactStorage::Temporary(path)),
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
    pub fn persist_to(&self, path: impl AsRef<Path>) -> Result<Self, PersistError> {
        let path = path.as_ref();
        let mut publication = publication(path);
        let same = self.path() == path
            || (path.exists()
                && same_file::is_same_file(self.path(), path).map_err(|cause| {
                    PersistError::new(PersistErrorKind::Io, cause, publication.clone())
                })?);
        if same {
            return match self.storage.as_ref() {
                ArtifactStorage::Persistent(_) => Ok(self.clone()),
                ArtifactStorage::Temporary(_) => {
                    publication.temporary = true;
                    Err(PersistError::new(
                        PersistErrorKind::InvalidDestination,
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "choose a destination distinct from the temporary artifact",
                        ),
                        publication,
                    ))
                }
            };
        }
        persist_copy(self.path(), path)?;
        Ok(Self::persistent(path.to_owned()))
    }

    /// Whether this handle shares ownership of deletion when its last clone drops.
    pub fn is_temporary(&self) -> bool {
        matches!(self.storage.as_ref(), ArtifactStorage::Temporary(_))
    }
}

impl PartialEq for ApkgArtifact {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}
impl Eq for ApkgArtifact {}

/// The candidate is exclusively owned by this build, unlike `persist_to`'s
/// source. Callers normally create it beside the destination so publication
/// only syncs and renames. Cross-device fallback retains copy-before-replace.
pub(crate) fn publish_owned_candidate(candidate: &Path, target: &Path) -> io::Result<()> {
    if let Some(parent) = target.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(candidate)?
        .sync_all()?;
    match std::fs::rename(candidate, target) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
            replace_output_atomically(candidate, target)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn replace_output_atomically(temp_artifact: &Path, target: &Path) -> io::Result<()> {
    persist_copy(temp_artifact, target)
        .map(|_| ())
        .map_err(PersistError::into_io)
}

fn publication(target: &Path) -> PublicationSnapshot {
    PublicationSnapshot {
        path: target.to_owned(),
        stage: PublicationStage::NotPublished,
        temporary: false,
        durability: Durability::Unconfirmed,
    }
}

fn persist_copy(source: &Path, target: &Path) -> Result<PublicationSnapshot, PersistError> {
    let mut facts = publication(target);
    if target.as_os_str().is_empty() {
        return Err(PersistError::new(
            PersistErrorKind::InvalidDestination,
            io::Error::new(io::ErrorKind::InvalidInput, "output path cannot be empty"),
            facts,
        ));
    }
    let parent = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let result = (|| -> io::Result<()> {
        std::fs::create_dir_all(parent)?;
        let mut temporary = tempfile::Builder::new()
            .prefix(".ankiforge-publish-")
            .suffix(".tmp")
            .tempfile_in(parent)?;
        io::copy(&mut std::fs::File::open(source)?, temporary.as_file_mut())?;
        temporary.as_file_mut().sync_all()?;
        temporary.persist(target).map_err(|error| error.error)?;
        facts.stage = PublicationStage::Published;
        #[cfg(unix)]
        {
            #[cfg(test)]
            directory_sync_failure::check()?;
            std::fs::File::open(parent)?.sync_all()?;
            facts.durability = Durability::Confirmed;
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(facts),
        Err(cause) => Err(PersistError::new(PersistErrorKind::Io, cause, facts)),
    }
}

#[cfg(all(test, unix))]
mod directory_sync_failure {
    use std::{cell::Cell, io};

    thread_local! {
        static ENABLED: Cell<bool> = const { Cell::new(false) };
    }

    pub(super) fn check() -> io::Result<()> {
        if ENABLED.with(Cell::get) {
            // EIO on Unix. Keep a concrete OS I/O error in the source chain.
            Err(io::Error::from_raw_os_error(5))
        } else {
            Ok(())
        }
    }

    pub(super) fn during<T>(operation: impl FnOnce() -> T) -> T {
        struct Reset(bool);
        impl Drop for Reset {
            fn drop(&mut self) {
                ENABLED.with(|enabled| enabled.set(self.0));
            }
        }
        let _reset = Reset(ENABLED.with(|enabled| enabled.replace(true)));
        operation()
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::{
        build::{json::BuildResultSnapshot, BuildErrorKind},
        BuildOptions, Note, Project,
    };
    use std::error::Error;

    fn project() -> Project {
        let mut project = Project::new("late-directory-sync").unwrap();
        project.add("one", Note::basic("front", "back")).unwrap();
        project
    }

    #[test]
    fn late_persist_failure_reports_published_file_and_retains_original_success() {
        let output = project().build(BuildOptions::temporary()).unwrap();
        let source = output.artifact().path().to_owned();
        let bytes = std::fs::read(&source).unwrap();
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("saved.apkg");
        std::fs::write(&destination, b"previous destination").unwrap();
        let error = directory_sync_failure::during(|| output.artifact().persist_to(&destination))
            .unwrap_err();

        assert_eq!(error.kind(), PersistErrorKind::Io);
        assert_eq!(error.publication().stage, PublicationStage::Published);
        assert_eq!(error.publication().durability, Durability::Unconfirmed);
        assert_eq!(error.publication().path, destination);
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<io::Error>()
                .unwrap()
                .raw_os_error(),
            Some(5)
        );
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
        assert_eq!(std::fs::read(&source).unwrap(), bytes);
        assert!(matches!(
            output.snapshot().result,
            BuildResultSnapshot::Success { .. }
        ));
        assert_eq!(output.report().counts().notes, 1);
        // Leaving the scoped fault restores normal persistence on this thread.
        output
            .artifact()
            .persist_to(root.path().join("retry.apkg"))
            .unwrap();
        drop(output);
        assert!(!source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
    }

    #[test]
    fn late_build_failure_keeps_publication_facts_and_inspected_counts() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("published.apkg");
        let error =
            directory_sync_failure::during(|| project().build(BuildOptions::to(&destination)))
                .unwrap_err();
        assert_eq!(error.kind(), BuildErrorKind::Publication);
        assert_eq!(error.report().counts().notes, 1);
        assert_eq!(error.publications()[0].stage, PublicationStage::Published);
        assert_eq!(error.publications()[0].durability, Durability::Unconfirmed);
        let cause = error
            .source()
            .unwrap()
            .downcast_ref::<PersistError>()
            .unwrap();
        assert_eq!(
            cause
                .source()
                .unwrap()
                .downcast_ref::<io::Error>()
                .unwrap()
                .raw_os_error(),
            Some(5)
        );
        let observed = crate::writer_core::inspect_apkg(&destination).unwrap();
        assert_eq!(observed.observation_status, "complete");
        match error.snapshot().result {
            BuildResultSnapshot::Failure { publications, .. } => {
                assert_eq!(publications[0].path, destination);
                assert_eq!(publications[0].stage, PublicationStage::Published);
            }
            _ => panic!("late failure must remain a failed operation"),
        }
    }
}
