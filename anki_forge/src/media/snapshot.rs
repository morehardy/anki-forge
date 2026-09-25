use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex, MutexGuard, Weak,
    },
};

use super::{MediaError, MediaErrorKind, MediaLimits};

const MEMORY_THRESHOLD: usize = 1024 * 1024;
const BUFFER_BYTES: usize = 64 * 1024;
pub(super) const SAMPLE_BYTES: usize = 8192;

pub(crate) struct Snapshot {
    pub(crate) digest: blake3::Hash,
    pub(crate) len: u64,
    storage: Storage,
    process: u32,
}

#[derive(Debug)]
enum Storage {
    Memory(Vec<u8>),
    File(tempfile::NamedTempFile),
}

enum Reader<'a> {
    Memory(Cursor<&'a [u8]>),
    File(File),
}

impl Read for Reader<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Memory(reader) => reader.read(bytes),
            Self::File(reader) => reader.read(bytes),
        }
    }
}

impl Seek for Reader<'_> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::Memory(reader) => reader.seek(position),
            Self::File(reader) => reader.seek(position),
        }
    }
}

type Cache = HashMap<(blake3::Hash, u64), Weak<Snapshot>>;
struct SnapshotCache {
    process: AtomicU32,
    entries: Mutex<Option<Cache>>,
}

impl SnapshotCache {
    const fn new() -> Self {
        Self {
            process: AtomicU32::new(0),
            entries: Mutex::new(None),
        }
    }

    fn lock(&self, process: u32) -> Option<MutexGuard<'_, Option<Cache>>> {
        // Only a mutex initialized in this process may be waited on. After
        // fork another parent's thread might own the inherited lock forever.
        let mut entries = if self.process.load(Ordering::Acquire) == process {
            self.entries
                .lock()
                .unwrap_or_else(|error| error.into_inner())
        } else {
            match self.entries.try_lock() {
                Ok(entries) => entries,
                Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => return None,
            }
        };
        if self.process.load(Ordering::Relaxed) != process {
            // Never upgrade inherited Weak pointers: their Arc accounting is
            // copied, and cannot keep the parent's temporary files alive.
            *entries = Some(HashMap::new());
            self.process.store(process, Ordering::Release);
        }
        Some(entries)
    }
}

static SNAPSHOTS: SnapshotCache = SnapshotCache::new();

impl Snapshot {
    fn shared(self) -> Arc<Self> {
        let Some(mut entries) = SNAPSHOTS.lock(self.process) else {
            return Arc::new(self);
        };
        let cache = entries.as_mut().expect("process cache is initialized");
        let key = (self.digest, self.len);
        if let Some(existing) = cache.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        let owned = Arc::new(self);
        cache.insert(key, Arc::downgrade(&owned));
        owned
    }

    pub(super) fn bytes(bytes: Vec<u8>, limits: MediaLimits) -> Result<Arc<Self>, MediaError> {
        let len = bytes.len() as u64;
        check_limit(len, limits)?;
        let digest = blake3::hash(&bytes);
        let storage = if bytes.len() > MEMORY_THRESHOLD {
            let mut file = tempfile::NamedTempFile::new()
                .map_err(|e| MediaError::io("create media snapshot", e))?;
            file.write_all(&bytes)
                .map_err(|e| MediaError::io("write media snapshot", e))?;
            Storage::File(file)
        } else {
            Storage::Memory(bytes)
        };
        Ok(Self {
            digest,
            len,
            storage,
            process: std::process::id(),
        }
        .shared())
    }

    pub(super) fn file(
        path: &Path,
        limits: MediaLimits,
    ) -> Result<(Arc<Self>, Vec<u8>), MediaError> {
        // Reject invalid inputs early. open_source also checks the descriptor
        // and prevents a replacement FIFO from blocking between these steps.
        let metadata =
            std::fs::metadata(path).map_err(|e| MediaError::io("inspect media source", e))?;
        if !metadata.is_file() {
            return Err(not_regular());
        }
        check_limit(metadata.len(), limits)?;
        let mut source = open_source(path, limits)?;
        let mut memory = Vec::new();
        let mut spool = None::<tempfile::NamedTempFile>;
        let mut sample = Vec::with_capacity(SAMPLE_BYTES);
        let mut digest = blake3::Hasher::new();
        let mut len = 0_u64;
        let mut buffer = [0; BUFFER_BYTES];
        loop {
            let count = match source.read(&mut buffer) {
                Ok(count) => count,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(MediaError::io("read media source", error)),
            };
            if count == 0 {
                break;
            }
            len = len.saturating_add(count as u64);
            check_limit(len, limits)?;
            digest.update(&buffer[..count]);
            let sample_count = count.min(SAMPLE_BYTES - sample.len());
            sample.extend_from_slice(&buffer[..sample_count]);
            if spool.is_none() && memory.len() + count > MEMORY_THRESHOLD {
                let mut file = tempfile::NamedTempFile::new()
                    .map_err(|e| MediaError::io("create media snapshot", e))?;
                file.write_all(&memory)
                    .map_err(|e| MediaError::io("write media snapshot", e))?;
                memory = Vec::new();
                spool = Some(file);
            }
            if let Some(file) = &mut spool {
                file.write_all(&buffer[..count])
                    .map_err(|e| MediaError::io("write media snapshot", e))?;
            } else {
                memory.extend_from_slice(&buffer[..count]);
            }
        }
        let storage = match spool {
            Some(file) => Storage::File(file),
            None => Storage::Memory(memory),
        };
        Ok((
            Self {
                digest: digest.finalize(),
                len,
                storage,
                process: std::process::id(),
            }
            .shared(),
            sample,
        ))
    }

    pub(crate) fn reader(&self) -> Result<impl Read + Seek + Send + '_, MediaError> {
        match &self.storage {
            Storage::Memory(bytes) => Ok(Reader::Memory(Cursor::new(bytes.as_slice()))),
            Storage::File(file) => file
                .reopen()
                .map(Reader::File)
                .map_err(|e| MediaError::io("read owned media snapshot", e)),
        }
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        if self.process != std::process::id() {
            // Close the inherited descriptor, but do not unlink a file still
            // owned by the parent. Forget only TempPath's cleanup token; its
            // tiny allocation is reclaimed when this child process exits.
            if let Storage::File(file) =
                std::mem::replace(&mut self.storage, Storage::Memory(Vec::new()))
            {
                let (file, path) = file.into_parts();
                drop(file);
                std::mem::forget(path);
            }
            return;
        }
        if let Some(mut entries) = SNAPSHOTS.lock(self.process) {
            let cache = entries.as_mut().expect("process cache is initialized");
            let key = (self.digest, self.len);
            if cache
                .get(&key)
                .is_some_and(|entry| std::ptr::eq(entry.as_ptr(), self))
            {
                cache.remove(&key);
            }
        }
    }
}

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("digest", &self.digest)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

fn open_source(path: &Path, limits: MediaLimits) -> Result<File, MediaError> {
    #[cfg(unix)]
    let source = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map(File::from)
    .map_err(|error| MediaError::io("open media source", error.into()))?;
    #[cfg(not(unix))]
    let source = File::open(path).map_err(|e| MediaError::io("open media source", e))?;
    let metadata = source
        .metadata()
        .map_err(|e| MediaError::io("inspect opened media source", e))?;
    if !metadata.is_file() {
        return Err(not_regular());
    }
    check_limit(metadata.len(), limits)?;
    #[cfg(unix)]
    {
        // Only a verified regular descriptor can reach normal blocking reads.
        // Keep every other status flag; neither operation reopens the pathname.
        let flags = rustix::fs::fcntl_getfl(&source)
            .map_err(|error| MediaError::io("inspect media source flags", error.into()))?;
        rustix::fs::fcntl_setfl(&source, flags & !rustix::fs::OFlags::NONBLOCK)
            .map_err(|error| MediaError::io("restore media source reads", error.into()))?;
    }
    Ok(source)
}

fn check_limit(observed: u64, limits: MediaLimits) -> Result<(), MediaError> {
    if observed > limits.max_bytes {
        Err(MediaError::exceeded(limits.max_bytes, observed))
    } else {
        Ok(())
    }
}

fn not_regular() -> MediaError {
    MediaError::new(
        MediaErrorKind::NotRegularFile,
        "MEDIA.SOURCE_NOT_REGULAR_FILE",
        "media source must be a regular file",
    )
}

#[cfg(test)]
mod process_tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn replaced_source_fifo_does_not_block() {
        use std::{
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        const CHILD: &str = "ANKIFORGE_FIFO_REPLACEMENT_TEST";
        if let Some(root) = std::env::var_os(CHILD) {
            let path = std::path::PathBuf::from(root).join("source");
            std::fs::write(&path, b"regular file before open").unwrap();
            let metadata = std::fs::metadata(&path).unwrap();
            assert!(metadata.is_file());
            check_limit(metadata.len(), MediaLimits::default()).unwrap();
            // Deterministically replace the checked file at the preflight/open
            // boundary. There is deliberately no writer to release a FIFO open.
            std::fs::remove_file(&path).unwrap();
            assert!(Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success());
            let error = open_source(&path, MediaLimits::default()).unwrap_err();
            assert_eq!(error.kind(), MediaErrorKind::NotRegularFile);
            assert_eq!(error.code(), "MEDIA.SOURCE_NOT_REGULAR_FILE");
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "media::snapshot::process_tests::replaced_source_fifo_does_not_block",
                "--nocapture",
            ])
            .env(CHILD, root.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let started = Instant::now();
        while child.try_wait().unwrap().is_none() {
            if started.elapsed() > Duration::from_secs(5) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("opening the replacement FIFO blocked without a writer");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
            "the isolated replacement test must actually run"
        );
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn opened_source_retains_budget_and_original_io_errors() {
        use std::error::Error;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source");
        std::fs::write(&path, b"12345").unwrap();
        let mut source = open_source(&path, MediaLimits { max_bytes: 5 }).unwrap();
        #[cfg(unix)]
        {
            assert!(!rustix::fs::fcntl_getfl(&source)
                .unwrap()
                .contains(rustix::fs::OFlags::NONBLOCK));
            let alias = root.path().join("alias");
            std::os::unix::fs::symlink(&path, &alias).unwrap();
            let mut through_alias = open_source(&alias, MediaLimits { max_bytes: 5 }).unwrap();
            let mut bytes = Vec::new();
            through_alias.read_to_end(&mut bytes).unwrap();
            assert_eq!(bytes, b"12345");
        }
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"12345");
        let error = open_source(&path, MediaLimits { max_bytes: 4 }).unwrap_err();
        assert_eq!(error.kind(), MediaErrorKind::ResourceLimit);
        let exceeded = error.limit_exceeded().unwrap();
        assert_eq!((exceeded.limit, exceeded.observed), (4, 5));
        let error = open_source(&root.path().join("missing"), MediaLimits::default()).unwrap_err();
        assert_eq!(error.kind(), MediaErrorKind::Io);
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::NotFound
        );
    }

    #[test]
    fn inherited_cache_lock_is_never_waited_on() {
        let cache = SnapshotCache::new();
        let parent = cache.lock(1).unwrap();
        // Simulates the lock inherited while a different parent thread holds it.
        // The integration runner gives actual fork consumers a process timeout.
        std::thread::scope(|scope| {
            let (sent, received) = std::sync::mpsc::channel();
            let cache = &cache;
            scope.spawn(move || sent.send(cache.lock(2).is_none()).unwrap());
            let result = received.recv_timeout(std::time::Duration::from_secs(1));
            drop(parent);
            assert_eq!(result, Ok(true), "child waited on an inherited cache lock");
        });
        assert!(cache.lock(2).unwrap().as_ref().unwrap().is_empty());
    }

    #[test]
    fn child_cache_discards_inherited_weak_owners() {
        let cache = SnapshotCache::new();
        let parent =
            Snapshot::bytes(vec![1; MEMORY_THRESHOLD + 1], MediaLimits::default()).unwrap();
        cache
            .lock(1)
            .unwrap()
            .as_mut()
            .unwrap()
            .insert((parent.digest, parent.len), Arc::downgrade(&parent));
        assert!(cache.lock(2).unwrap().as_ref().unwrap().is_empty());
        // The parent's snapshot remains readable after discarding the child cache.
        let mut bytes = Vec::new();
        parent.reader().unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes.len(), MEMORY_THRESHOLD + 1);
    }

    #[test]
    fn inherited_snapshot_drop_preserves_parent_file_and_local_drop_cleans() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        let inherited = Snapshot {
            digest: blake3::hash(b"parent"),
            len: 6,
            storage: Storage::File(file),
            process: std::process::id().wrapping_add(1),
        };
        drop(inherited);
        assert!(
            path.exists(),
            "child destructor unlinked the parent snapshot"
        );
        std::fs::remove_file(path).unwrap();

        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        drop(Snapshot {
            digest: blake3::hash(b"child"),
            len: 5,
            storage: Storage::File(file),
            process: std::process::id(),
        });
        assert!(!path.exists(), "local snapshot leaked its temporary file");
    }
}
