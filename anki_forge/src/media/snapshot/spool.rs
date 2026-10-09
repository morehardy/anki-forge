//! Bounded spill blocks shared only by their live immutable snapshot segments.
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex, Weak},
};

const BLOCK_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Default)]
pub(super) struct Pool {
    current: Weak<Block>,
    length: u64,
}

#[derive(Debug)]
struct Block {
    // Drop the writer before its path, including on Windows.
    writer: Mutex<Option<File>>,
    path: Option<tempfile::TempPath>,
    process: u32,
}

impl Block {
    fn path(&self) -> &std::path::Path {
        self.path.as_deref().expect("live spill block owns a path")
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        // Arc counts are copied at fork. A child must never unlink parent storage.
        // Field destruction closes an inherited writer without waiting on its
        // potentially inherited mutex. If copied Arc counts prevent this Drop,
        // the inherited descriptor remains until the child process exits.
        if self.process != std::process::id() {
            if let Some(path) = self.path.take() {
                std::mem::forget(path);
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct Segment {
    block: Arc<Block>,
    offset: u64,
    length: u64,
}

impl Pool {
    // The process-local snapshot cache serializes append operations and retains
    // only a Weak owner. The active block owns one writer; rotation, failure and
    // its final snapshot owner close it. Completed blocks retain no writer.
    pub(super) fn store(&mut self, bytes: &[u8]) -> io::Result<Segment> {
        let previous = self.current.upgrade();
        let (block, offset) = match previous {
            Some(block) if self.length + bytes.len() as u64 <= BLOCK_BYTES => (block, self.length),
            previous => {
                // A failed replacement must not leave a closed writer reusable.
                self.current = Weak::new();
                self.length = 0;
                if let Some(block) = previous {
                    block
                        .writer
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .take();
                }
                let (file, path) = tempfile::NamedTempFile::new()?.into_parts();
                (
                    Arc::new(Block {
                        writer: Mutex::new(Some(file)),
                        path: Some(path),
                        process: std::process::id(),
                    }),
                    0,
                )
            }
        };
        let mut writer = block
            .writer
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // Readers use independent descriptors. Only serialized appends advance
        // this writer's cursor, so a successful append needs no seek.
        let file = writer.as_mut().expect("active spill block has a writer");
        if let Err(error) = file.write_all(bytes) {
            // Roll back a partial append while preserving the original I/O cause.
            // Even if truncation fails, never reuse the failed block's tail.
            let _ = file.set_len(offset);
            writer.take();
            self.current = Weak::new();
            self.length = 0;
            return Err(error);
        }
        drop(writer);
        self.length = offset + bytes.len() as u64;
        self.current = Arc::downgrade(&block);
        Ok(Segment {
            block,
            offset,
            length: bytes.len() as u64,
        })
    }
}

impl Segment {
    pub(super) fn reader(&self) -> io::Result<Reader> {
        let mut file = File::open(self.block.path())?;
        file.seek(SeekFrom::Start(self.offset))?;
        Ok(Reader {
            file,
            offset: self.offset,
            length: self.length,
            position: 0,
        })
    }
}

pub(super) struct Reader {
    file: File,
    offset: u64,
    length: u64,
    position: u64,
}

impl Read for Reader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let limit = self
            .length
            .saturating_sub(self.position)
            .min(bytes.len() as u64) as usize;
        let count = self.file.read(&mut bytes[..limit])?;
        self.position += count as u64;
        Ok(count)
    }
}

impl Seek for Reader {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(position) => i128::from(position),
            SeekFrom::Current(delta) => i128::from(self.position) + i128::from(delta),
            SeekFrom::End(delta) => i128::from(self.length) + i128::from(delta),
        };
        let invalid =
            || io::Error::new(io::ErrorKind::InvalidInput, "invalid snapshot segment seek");
        let position = u64::try_from(position).map_err(|_| invalid())?;
        let absolute = self.offset.checked_add(position).ok_or_else(invalid)?;
        self.file.seek(SeekFrom::Start(absolute))?;
        self.position = position;
        Ok(position)
    }
}

/// One descriptor per preparation worker, released when that build finishes.
/// Weak identity prevents retaining a snapshot's cleanup owner or confusing
/// a newly allocated block with a previous block at the same address.
#[derive(Default)]
pub(crate) struct ReaderCache {
    current: Option<(Weak<Block>, File)>,
}

impl Segment {
    pub(super) fn cached_reader<'a>(
        &self,
        cache: &'a mut ReaderCache,
    ) -> io::Result<std::io::Take<&'a mut File>> {
        let key = Arc::downgrade(&self.block);
        if cache
            .current
            .as_ref()
            .is_none_or(|(block, _)| !Weak::ptr_eq(block, &key))
        {
            cache.current = Some((key, File::open(self.block.path())?));
        }
        let file = &mut cache.current.as_mut().expect("cached block reader").1;
        file.seek(SeekFrom::Start(self.offset))?;
        Ok(file.take(self.length))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_readers_bound_each_segment_and_rotate_without_retaining_owners() {
        let mut pool = Pool::default();
        let first = pool.store(b"first").unwrap();
        let second = pool.store(b"second").unwrap();
        let mut cache = ReaderCache::default();
        let mut other_worker = ReaderCache::default();
        let mut bytes = Vec::new();
        first
            .cached_reader(&mut cache)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, b"first");
        let block_identity = cache.current.as_ref().unwrap().0.clone();
        bytes.clear();
        second
            .cached_reader(&mut cache)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, b"second");
        assert!(Weak::ptr_eq(
            &block_identity,
            &cache.current.as_ref().unwrap().0
        ));
        assert_eq!(
            Arc::strong_count(&first.block),
            2,
            "cache must only borrow cleanup ownership"
        );
        bytes.clear();
        first
            .cached_reader(&mut other_worker)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(
            bytes, b"first",
            "workers must have independent file offsets"
        );

        let path = first.block.path().to_owned();
        drop((first, second));
        assert!(
            !path.exists(),
            "weak reader caches must not retain the snapshot path"
        );
        assert!(block_identity.upgrade().is_none());
        let replacement = pool.store(b"new owner").unwrap();
        bytes.clear();
        replacement
            .cached_reader(&mut cache)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, b"new owner");
        assert!(!Weak::ptr_eq(
            &block_identity,
            &cache.current.as_ref().unwrap().0
        ));
    }

    #[test]
    fn readers_are_independent_and_cannot_cross_segment_boundaries() {
        let mut pool = Pool::default();
        let first = pool.store(b"first").unwrap();
        let second = pool.store(b"second").unwrap();
        assert!(Arc::ptr_eq(&first.block, &second.block));
        let path = first.block.path().to_owned();
        let mut a = first.reader().unwrap();
        let mut b = second.reader().unwrap();
        assert_eq!(a.seek(SeekFrom::End(-2)).unwrap(), 3);
        let mut bytes = Vec::new();
        a.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"st");
        assert_eq!(a.seek(SeekFrom::Current(-5)).unwrap(), 0);
        bytes.clear();
        a.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"first");
        assert_eq!(b.seek(SeekFrom::End(-3)).unwrap(), 3);
        bytes.clear();
        b.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"ond");
        assert!(b.seek(SeekFrom::Start(20)).is_ok());
        assert_eq!(b.read(&mut [0; 10]).unwrap(), 0);
        assert!(b.seek(SeekFrom::End(-7)).is_err());
        let third = pool.store(b"third").unwrap();
        bytes.clear();
        third.reader().unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"third", "reader seeks must not move the writer");
        bytes.clear();
        first.reader().unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"first", "later appends must preserve live segments");
        drop(third);
        drop((a, b, first));
        assert!(path.exists(), "another segment still owns the shared block");
        drop(second);
        assert!(!path.exists(), "weak pool must not retain dead storage");
    }

    #[test]
    fn full_blocks_rotate_and_each_block_has_its_own_cleanup_owner() {
        let mut pool = Pool::default();
        let first = pool.store(&vec![1; BLOCK_BYTES as usize]).unwrap();
        let second = pool.store(b"next block").unwrap();
        assert!(!Arc::ptr_eq(&first.block, &second.block));
        assert!(first.block.writer.lock().unwrap().is_none());
        assert!(second.block.writer.lock().unwrap().is_some());
        let mut bytes = Vec::new();
        first.reader().unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, vec![1; BLOCK_BYTES as usize]);
        let first_path = first.block.path().to_owned();
        let second_path = second.block.path().to_owned();
        drop(first);
        assert!(!first_path.exists());
        assert!(second_path.exists());
        drop(second);
        assert!(!second_path.exists());
        let fresh = pool.store(b"fresh").unwrap();
        assert_eq!(fresh.offset, 0, "expired weak block must start a new file");
    }

    #[test]
    fn inherited_block_cleanup_preserves_parent_path() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        let (writer, owned_path) = file.into_parts();
        drop(Block {
            writer: Mutex::new(Some(writer)),
            path: Some(owned_path),
            process: std::process::id().wrapping_add(1),
        });
        assert!(path.exists());
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[test]
    fn inherited_writer_closes_without_waiting_on_its_mutex() {
        use std::{sync::mpsc, time::Duration};

        let (mut writer, owned_path) = tempfile::NamedTempFile::new().unwrap().into_parts();
        let path = owned_path.to_path_buf();
        writer.write_all(b"parent snapshot").unwrap();
        writer.lock().unwrap();
        let observer = File::open(&path).unwrap();
        assert!(matches!(
            observer.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        let block = Block {
            writer: Mutex::new(Some(writer)),
            path: Some(owned_path),
            process: std::process::id().wrapping_add(1),
        };
        let (finished, received) = mpsc::channel();
        let dropper = std::thread::spawn(move || {
            // Model a mutex left locked by a different thread at fork. There is
            // no live borrower after forgetting the guard, and Drop must not
            // try to acquire it in order to close its file.
            std::mem::forget(block.writer.lock().unwrap());
            drop(block);
            finished.send(()).unwrap();
        });
        received
            .recv_timeout(Duration::from_secs(5))
            .expect("inherited block destructor waited on its mutex");
        dropper.join().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"parent snapshot");
        observer
            .try_lock()
            .expect("inherited writer descriptor was not closed");
        drop(observer);
        std::fs::remove_file(path).unwrap();
    }
}
