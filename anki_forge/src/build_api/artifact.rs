#![deny(missing_docs)]
//! Private candidate handoff and ownership of published APKG files.
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

/// A synced private file that has not been exposed through the public API.
/// Inspection reads it without mutation; consuming this non-cloneable owner
/// either exposes a temporary artifact or publishes the already-synced inode.
#[derive(Debug)]
pub(crate) struct PrivateCandidate(ApkgArtifact);

impl PrivateCandidate {
    pub(crate) fn retain(candidate: &Path) -> io::Result<Self> {
        let file = tempfile::Builder::new()
            .prefix("anki-forge-artifact-")
            .suffix(".apkg")
            .tempfile()?;
        let path = file.into_temp_path();
        publish_owned_candidate(candidate, &path)?;
        Ok(Self(ApkgArtifact {
            storage: Arc::new(ArtifactStorage::Temporary(path)),
        }))
    }

    pub(crate) fn path(&self) -> &Path {
        self.0.path()
    }

    pub(crate) fn into_artifact(self) -> ApkgArtifact {
        self.0
    }

    pub(crate) fn publish_to(self, path: impl AsRef<Path>) -> Result<ApkgArtifact, PersistError> {
        self.0.persist(path.as_ref(), true)
    }
}

impl ApkgArtifact {
    pub(crate) fn persistent(path: PathBuf) -> Self {
        Self {
            storage: Arc::new(ArtifactStorage::Persistent(path)),
        }
    }

    /// Borrow the absolute path while keeping an artifact handle alive.
    /// Changing the process working directory does not change this location.
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
    /// Relative destinations are resolved when this method is called, and the
    /// returned handle retains that absolute location.
    pub fn persist_to(&self, path: impl AsRef<Path>) -> Result<Self, PersistError> {
        self.persist(path.as_ref(), false)
    }

    // Only PrivateCandidate may request moving an already-synced source.
    // A public artifact's path can be modified by its caller, even with one
    // owner, so public persistence always copies and syncs the fresh copy.
    fn persist(&self, path: &Path, move_synced_source: bool) -> Result<Self, PersistError> {
        let mut publication = publication(path);
        if path.as_os_str().is_empty() {
            return Err(PersistError::new(
                PersistErrorKind::InvalidDestination,
                io::Error::new(io::ErrorKind::InvalidInput, "output path cannot be empty"),
                publication,
            ));
        }
        // Anchor before alias checks and publication, preserving filesystem
        // traversal through symlinks and `..`. Canonicalizing after publishing
        // could fail after replacement or follow a different destination.
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            std::path::absolute(path).map_err(|cause| {
                PersistError::new(PersistErrorKind::Io, cause, publication.clone())
            })?
        };
        publication.path = path.clone();
        let same = crate::path_alias::paths_alias(self.path(), &path)
            .map_err(|cause| PersistError::new(PersistErrorKind::Io, cause, publication.clone()))?;
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
        persist_file(self.path(), &path, move_synced_source)?;
        Ok(Self::persistent(path))
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
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(candidate)?;
    sync_file(&file, candidate)?;
    drop(file);
    match rename_candidate(candidate, target) {
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
    persist_file(source, target, false)
}

fn persist_file(
    source: &Path,
    target: &Path,
    move_synced_source: bool,
) -> Result<PublicationSnapshot, PersistError> {
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
        #[cfg(unix)]
        let directories = directories_to_sync(parent)?;
        std::fs::create_dir_all(parent)?;
        let moved = if move_synced_source {
            match rename_candidate(source, target) {
                Ok(()) => true,
                Err(error) if error.kind() == io::ErrorKind::CrossesDevices => false,
                Err(error) => return Err(error),
            }
        } else {
            false
        };
        if !moved {
            let mut temporary = tempfile::Builder::new()
                .prefix(".ankiforge-publish-")
                .suffix(".tmp")
                .tempfile_in(parent)?;
            io::copy(&mut std::fs::File::open(source)?, temporary.as_file_mut())?;
            sync_file(temporary.as_file(), temporary.path())?;
            temporary.persist(target).map_err(|error| error.error)?;
        }
        facts.stage = PublicationStage::Published;
        #[cfg(unix)]
        {
            for directory in directories {
                #[cfg(test)]
                directory_sync_failure::check(&directory)?;
                std::fs::File::open(directory)?.sync_all()?;
            }
            facts.durability = Durability::Confirmed;
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(facts),
        Err(cause) => Err(PersistError::new(PersistErrorKind::Io, cause, facts)),
    }
}

fn sync_file(file: &std::fs::File, _path: &Path) -> io::Result<()> {
    #[cfg(all(test, unix))]
    file_publication::before_sync(_path)?;
    file.sync_all()
}

fn rename_candidate(source: &Path, target: &Path) -> io::Result<()> {
    #[cfg(all(test, unix))]
    file_publication::before_rename(source, target)?;
    std::fs::rename(source, target)
}

#[cfg(unix)]
fn directories_to_sync(parent: &Path) -> io::Result<Vec<PathBuf>> {
    let mut directories = vec![parent.to_owned()];
    // Record missing directory entries before create_dir_all makes them look
    // pre-existing. Each needs its parent synced, in addition to the leaf
    // directory containing the published file. Preserve filesystem traversal
    // through symlinks and `..`; lexical normalization would skip directories
    // that must actually be created to make such a path traversable.
    for directory in parent.ancestors() {
        let directory = if directory.as_os_str().is_empty() {
            Path::new(".")
        } else {
            directory
        };
        match directory.metadata() {
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if let Some(ancestor) = directory.parent() {
                    directories.push(if ancestor.as_os_str().is_empty() {
                        PathBuf::from(".")
                    } else {
                        ancestor.to_owned()
                    });
                }
            }
            Err(error) => return Err(error),
        }
    }
    Ok(directories)
}

#[cfg(all(test, unix))]
mod file_publication {
    use std::{
        cell::RefCell,
        io,
        path::{Path, PathBuf},
    };

    #[derive(Debug)]
    pub(super) enum Operation {
        Sync(PathBuf),
        Rename { source: PathBuf, target: PathBuf },
    }

    struct Observation {
        fail_sync_at: Option<usize>,
        cross_device_target: Option<PathBuf>,
        syncs: usize,
        operations: Vec<Operation>,
    }

    thread_local! {
        static ACTIVE: RefCell<Option<Observation>> = const { RefCell::new(None) };
    }

    pub(super) fn before_sync(path: &Path) -> io::Result<()> {
        let fail = ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            let Some(observation) = active.as_mut() else {
                return false;
            };
            let index = observation.syncs;
            observation.syncs += 1;
            observation
                .operations
                .push(Operation::Sync(path.to_owned()));
            observation.fail_sync_at == Some(index)
        });
        if fail {
            Err(io::Error::from_raw_os_error(5))
        } else {
            Ok(())
        }
    }

    pub(super) fn before_rename(source: &Path, target: &Path) -> io::Result<()> {
        let cross_device = ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            let Some(observation) = active.as_mut() else {
                return false;
            };
            observation.operations.push(Operation::Rename {
                source: source.to_owned(),
                target: target.to_owned(),
            });
            observation.cross_device_target.as_deref() == Some(target)
        });
        if cross_device {
            Err(io::Error::from(io::ErrorKind::CrossesDevices))
        } else {
            Ok(())
        }
    }

    pub(super) fn observe<T>(
        fail_sync_at: Option<usize>,
        cross_device_target: Option<&Path>,
        operation: impl FnOnce() -> T,
    ) -> (T, Vec<Operation>) {
        struct Reset(Option<Observation>);
        impl Drop for Reset {
            fn drop(&mut self) {
                ACTIVE.with(|active| *active.borrow_mut() = self.0.take());
            }
        }
        let _reset = Reset(ACTIVE.with(|active| {
            active.borrow_mut().replace(Observation {
                fail_sync_at,
                cross_device_target: cross_device_target.map(Path::to_owned),
                syncs: 0,
                operations: Vec::new(),
            })
        }));
        let result = operation();
        let operations = ACTIVE
            .with(|active| std::mem::take(&mut active.borrow_mut().as_mut().unwrap().operations));
        (result, operations)
    }
}

#[cfg(all(test, unix))]
mod directory_sync_failure {
    use std::{
        cell::RefCell,
        io,
        path::{Path, PathBuf},
    };

    struct Observation {
        fail_at: Option<usize>,
        paths: Vec<PathBuf>,
    }

    thread_local! {
        static ACTIVE: RefCell<Option<Observation>> = const { RefCell::new(None) };
    }

    pub(super) fn check(path: &Path) -> io::Result<()> {
        let fail = ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            let Some(observation) = active.as_mut() else {
                return false;
            };
            let index = observation.paths.len();
            observation.paths.push(path.to_owned());
            observation.fail_at == Some(index)
        });
        if fail {
            // EIO on Unix. Keep a concrete OS I/O error in the source chain.
            Err(io::Error::from_raw_os_error(5))
        } else {
            Ok(())
        }
    }

    pub(super) fn during<T>(operation: impl FnOnce() -> T) -> T {
        observe(Some(0), operation).0
    }

    pub(super) fn observe<T>(
        fail_at: Option<usize>,
        operation: impl FnOnce() -> T,
    ) -> (T, Vec<PathBuf>) {
        struct Reset(Option<Observation>);
        impl Drop for Reset {
            fn drop(&mut self) {
                ACTIVE.with(|active| {
                    active.replace(self.0.take());
                });
            }
        }
        let _reset = Reset(ACTIVE.with(|active| {
            active.replace(Some(Observation {
                fail_at,
                paths: Vec::new(),
            }))
        }));
        let result = operation();
        let paths = ACTIVE.with(|active| active.borrow().as_ref().unwrap().paths.clone());
        (result, paths)
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
    fn exclusive_build_publication_moves_the_owned_file() {
        use std::os::unix::fs::MetadataExt;
        let root = tempfile::tempdir().unwrap();
        let (candidate, _) = project()
            .prepare_candidate(&BuildOptions::temporary())
            .unwrap();
        let original = candidate.path().to_owned();
        let inode = original.metadata().unwrap().ino();
        let (saved, operations) = file_publication::observe(None, None, || {
            candidate.publish_to(root.path().join("saved.apkg"))
        });
        let saved = saved.unwrap();
        assert_eq!(saved.path().metadata().unwrap().ino(), inode);
        assert!(!original.exists());
        assert!(matches!(
            operations.as_slice(),
            [file_publication::Operation::Rename { .. }]
        ));
    }

    #[test]
    fn public_persistence_keeps_the_original_handle_usable() {
        let root = tempfile::tempdir().unwrap();
        let output = project().build(BuildOptions::temporary()).unwrap();
        let original = output.artifact();
        let saved = original.persist_to(root.path().join("saved.apkg")).unwrap();
        assert_eq!(
            std::fs::read(original.path()).unwrap(),
            std::fs::read(saved.path()).unwrap()
        );
        drop(output);
        assert!(saved.path().is_file());
    }

    #[test]
    fn temporary_and_persistent_builds_sync_the_candidate_once_before_any_rename() {
        use file_publication::Operation;
        let root = tempfile::tempdir().unwrap();
        for options in [
            BuildOptions::temporary(),
            BuildOptions::to(root.path().join("saved.apkg")),
        ] {
            let (result, operations) =
                file_publication::observe(None, None, || project().build(options));
            let output = result.unwrap();
            let expected_renames = if output.artifact().is_temporary() {
                1
            } else {
                2
            };
            assert_eq!(operations.len(), 1 + expected_renames);
            let Operation::Sync(source) = &operations[0] else {
                panic!("candidate must be synchronized before it is retained: {operations:?}");
            };
            assert!(matches!(
                &operations[1], Operation::Rename { source: renamed, .. } if renamed == source
            ));
            assert!(operations[1..]
                .iter()
                .all(|operation| matches!(operation, Operation::Rename { .. })));
            assert_eq!(
                crate::writer_core::inspect_apkg(output.artifact().path())
                    .unwrap()
                    .observation_status,
                "complete"
            );
        }
    }

    #[test]
    fn initial_sync_failure_keeps_its_classification_and_precedes_inspection() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("saved.apkg");
        std::fs::write(&destination, b"previous destination").unwrap();
        for options in [BuildOptions::temporary(), BuildOptions::to(&destination)] {
            let limits = super::super::InspectLimits {
                max_archive_bytes: 1,
                ..Default::default()
            };
            let (result, operations) = file_publication::observe(Some(0), None, || {
                project().build(options.inspect_limits(limits))
            });
            let error = result.unwrap_err();
            assert_eq!(error.kind(), BuildErrorKind::Io);
            assert_eq!(error.code(), "BUILD.WORKSPACE_FAILED");
            assert!(error.publications().is_empty());
            assert_eq!(error.report().counts().notes, 1);
            assert_eq!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<io::Error>()
                    .unwrap()
                    .raw_os_error(),
                Some(5)
            );
            assert!(matches!(
                operations.as_slice(),
                [file_publication::Operation::Sync(_)]
            ));
            assert_eq!(
                std::fs::read(&destination).unwrap(),
                b"previous destination"
            );
        }
    }

    #[test]
    fn comparison_and_blocked_build_release_the_private_candidate() {
        use file_publication::Operation;
        let mut baseline = project();
        baseline
            .add("removed", Note::basic("old", "answer"))
            .unwrap();
        let baseline = baseline.build(BuildOptions::temporary()).unwrap();
        let baseline = baseline.artifact().path();
        let (result, operations) = file_publication::observe(None, None, || {
            project().compare(crate::update::CompareOptions::against(baseline))
        });
        assert!(!result.unwrap().policy().allows_publication());
        let [Operation::Sync(_), Operation::Rename { target, .. }] = operations.as_slice() else {
            panic!("comparison must only retain its private candidate: {operations:?}");
        };
        assert!(!target.exists());

        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("saved.apkg");
        std::fs::write(&destination, b"previous destination").unwrap();
        let (result, operations) = file_publication::observe(None, None, || {
            project().build(BuildOptions::to(&destination).update_from(baseline))
        });
        let error = result.unwrap_err();
        assert_eq!(error.kind(), BuildErrorKind::PolicyBlocked);
        assert_eq!(error.code(), "UPDATE.POLICY_BLOCKED");
        assert!(error.publications().is_empty());
        let [Operation::Sync(_), Operation::Rename { target, .. }] = operations.as_slice() else {
            panic!("blocked build must only retain its private candidate: {operations:?}");
        };
        assert!(!target.exists());
        assert_eq!(
            std::fs::read(&destination).unwrap(),
            b"previous destination"
        );
    }

    #[test]
    fn cross_device_publication_syncs_the_new_copy_before_replacing_the_target() {
        use file_publication::Operation;
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("saved.apkg");
        std::fs::write(&destination, b"previous destination").unwrap();
        let (result, operations) = file_publication::observe(None, Some(&destination), || {
            project().build(BuildOptions::to(&destination))
        });
        let output = result.unwrap();
        let [Operation::Sync(_), Operation::Rename { .. }, Operation::Rename { source, target }, Operation::Sync(copy)] =
            operations.as_slice()
        else {
            panic!("cross-device publication must sync its new copy: {operations:?}");
        };
        assert_eq!(target, &destination);
        assert_eq!(copy.parent(), destination.parent());
        assert!(
            !source.exists(),
            "private source must be released after copying"
        );
        assert!(
            !copy.exists(),
            "copy scratch path must be consumed by replacement"
        );
        assert_eq!(output.artifact().path(), destination);
        assert_eq!(
            crate::writer_core::inspect_apkg(&destination)
                .unwrap()
                .observation_status,
            "complete"
        );
    }

    #[test]
    fn cross_device_copy_sync_failure_preserves_target_and_publication_facts() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("saved.apkg");
        std::fs::write(&destination, b"previous destination").unwrap();
        let (result, operations) = file_publication::observe(Some(1), Some(&destination), || {
            project().build(BuildOptions::to(&destination))
        });
        let error = result.unwrap_err();
        assert_eq!(error.kind(), BuildErrorKind::Publication);
        assert_eq!(error.code(), "PERSIST.IO_FAILED");
        assert_eq!(error.report().counts().notes, 1);
        assert_eq!(
            error.publications()[0].stage,
            PublicationStage::NotPublished
        );
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
        assert_eq!(
            std::fs::read(&destination).unwrap(),
            b"previous destination"
        );
        for operation in operations {
            match operation {
                file_publication::Operation::Sync(path) => assert!(!path.exists()),
                file_publication::Operation::Rename { source, .. } => assert!(!source.exists()),
            }
        }
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[test]
    fn public_persistence_syncs_a_fresh_copy_even_after_the_caller_changes_the_source() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("saved.apkg");
        std::fs::write(&destination, b"previous destination").unwrap();
        let output = project().build(BuildOptions::temporary()).unwrap();
        let retained = output.artifact().clone();
        std::fs::write(retained.path(), b"caller changed the source").unwrap();
        let (result, operations) =
            file_publication::observe(Some(0), None, || retained.persist_to(&destination));
        let error = result.unwrap_err();
        assert_eq!(error.publication().stage, PublicationStage::NotPublished);
        assert_eq!(
            std::fs::read(&destination).unwrap(),
            b"previous destination"
        );
        assert!(matches!(
            operations.as_slice(),
            [file_publication::Operation::Sync(_)]
        ));
        let (result, operations) =
            file_publication::observe(None, None, || retained.persist_to(&destination));
        let saved = result.unwrap();
        assert!(matches!(
            operations.as_slice(),
            [file_publication::Operation::Sync(_)]
        ));
        assert_eq!(
            std::fs::read(saved.path()).unwrap(),
            b"caller changed the source"
        );
        drop(output);
        assert_eq!(
            std::fs::read(retained.path()).unwrap(),
            b"caller changed the source"
        );
    }

    #[test]
    fn nested_persistence_syncs_every_new_directory_entry_before_confirming_durability() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source.apkg");
        std::fs::write(&source, b"owned artifact").unwrap();
        let destination = root.path().join("new/nested/saved.apkg");
        let (result, synced) =
            directory_sync_failure::observe(None, || persist_copy(&source, &destination));
        let facts = result.unwrap();
        assert_eq!(facts.stage, PublicationStage::Published);
        assert_eq!(facts.durability, Durability::Confirmed);
        assert_eq!(
            synced,
            vec![
                root.path().join("new/nested"),
                root.path().join("new"),
                root.path().to_owned()
            ]
        );
        assert_eq!(std::fs::read(destination).unwrap(), b"owned artifact");
    }

    #[test]
    fn ancestor_sync_failure_preserves_published_bytes_and_original_artifact() {
        let output = project().build(BuildOptions::temporary()).unwrap();
        let source = output.artifact().path().to_owned();
        let bytes = std::fs::read(&source).unwrap();
        for fail_at in [1, 2] {
            let root = tempfile::tempdir().unwrap();
            let destination = root.path().join("new/nested/saved.apkg");
            let (result, synced) = directory_sync_failure::observe(Some(fail_at), || {
                output.artifact().persist_to(&destination)
            });
            let error = result.unwrap_err();
            assert_eq!(synced.len(), fail_at + 1);
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
        }
    }

    #[test]
    fn ancestor_sync_failure_during_build_retains_inspected_counts_and_publication() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("new/nested/saved.apkg");
        let (result, synced) = directory_sync_failure::observe(Some(2), || {
            project().build(BuildOptions::to(&destination))
        });
        let error = result.unwrap_err();
        assert_eq!(synced.len(), 3);
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
        assert_eq!(
            crate::writer_core::inspect_apkg(&destination)
                .unwrap()
                .observation_status,
            "complete"
        );
    }

    #[test]
    fn persistence_syncs_relative_directory_chains_and_empty_parents() {
        let cwd = std::env::current_dir().unwrap();
        let root = tempfile::tempdir_in(&cwd).unwrap();
        let source = root.path().join("source.apkg");
        std::fs::write(&source, b"owned artifact").unwrap();
        let relative_root = root.path().strip_prefix(&cwd).unwrap();
        let destination = relative_root.join("new/nested/saved.apkg");
        let (result, synced) =
            directory_sync_failure::observe(None, || persist_copy(&source, &destination));
        assert_eq!(result.unwrap().durability, Durability::Confirmed);
        assert_eq!(
            synced,
            vec![
                relative_root.join("new/nested"),
                relative_root.join("new"),
                relative_root.to_owned()
            ]
        );
        assert_eq!(std::fs::read(destination).unwrap(), b"owned artifact");

        // Keep ownership of this scratch path so the replaced file is cleaned
        // up without changing the process-wide working directory in tests.
        let scratch = tempfile::NamedTempFile::new_in(&cwd).unwrap();
        let destination = Path::new(scratch.path().file_name().unwrap());
        assert_eq!(destination.parent(), Some(Path::new("")));
        let (result, synced) =
            directory_sync_failure::observe(None, || persist_copy(&source, destination));
        assert_eq!(result.unwrap().durability, Durability::Confirmed);
        assert_eq!(synced, vec![PathBuf::from(".")]);
        assert_eq!(std::fs::read(destination).unwrap(), b"owned artifact");
    }

    #[test]
    fn persistence_syncs_created_entries_reached_through_symlinks_and_parent_components() {
        let root = tempfile::tempdir().unwrap();
        let elsewhere = root.path().join("elsewhere");
        std::fs::create_dir_all(elsewhere.join("child")).unwrap();
        std::os::unix::fs::symlink(elsewhere.join("child"), root.path().join("link")).unwrap();
        let source = root.path().join("source.apkg");
        std::fs::write(&source, b"owned artifact").unwrap();
        let destination = root.path().join("missing/../link/../new/nested/saved.apkg");
        let (result, synced) =
            directory_sync_failure::observe(None, || persist_copy(&source, &destination));
        assert_eq!(result.unwrap().durability, Durability::Confirmed);
        let synced: std::collections::BTreeSet<_> = synced
            .into_iter()
            .map(|path| path.canonicalize().unwrap())
            .collect();
        for directory in [
            elsewhere.join("new/nested"),
            elsewhere.join("new"),
            elsewhere,
            root.path().join("missing"),
            root.path().to_owned(),
        ] {
            assert!(
                synced.contains(&directory.canonicalize().unwrap()),
                "directory entry was not synced: {}",
                directory.display()
            );
        }
        assert_eq!(std::fs::read(&destination).unwrap(), b"owned artifact");
        assert_eq!(std::fs::read(source).unwrap(), b"owned artifact");
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

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn late_failure_with_non_unicode_path_keeps_serializable_published_facts() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join(std::ffi::OsString::from_vec(
            b"published-\xff.apkg".to_vec(),
        ));
        let error =
            directory_sync_failure::during(|| project().build(BuildOptions::to(&destination)))
                .unwrap_err();
        assert_eq!(error.publications()[0].stage, PublicationStage::Published);
        assert!(destination.is_file());
        let value = serde_json::to_value(error.snapshot()).unwrap();
        assert_eq!(
            value["result"]["publications"][0]["path"],
            serde_json::json!({"encoding":"unix_bytes", "bytes":destination.as_os_str().as_bytes()})
        );
        assert_eq!(
            value["result"]["publications"][0]["durability"],
            "unconfirmed"
        );
    }
}
