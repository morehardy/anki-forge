//! Ordered package hashing that overlaps large writes, with bounded buffers.
use std::io;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::{Builder, JoinHandle};

use sha1::{Digest, Sha1};

const BLOCK_BYTES: usize = 256 * 1024;

pub(crate) struct PipelinedSha1 {
    serial: Sha1,
    worker: Option<HashWorker>,
    bytes: usize,
    attempted_worker: bool,
}

impl PipelinedSha1 {
    pub(crate) fn new() -> Self {
        Self {
            serial: Sha1::new(),
            worker: None,
            bytes: 0,
            attempted_worker: false,
        }
    }

    pub(crate) fn update(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.bytes = self.bytes.saturating_add(bytes.len());
        if !self.attempted_worker && self.bytes >= BLOCK_BYTES {
            self.attempted_worker = true;
            // Keep the serial state usable if thread creation fails.
            self.worker = HashWorker::new(self.serial.clone()).ok();
        }
        if let Some(worker) = self.worker.as_mut() {
            worker.update(bytes)
        } else {
            self.serial.update(bytes);
            Ok(())
        }
    }

    pub(crate) fn finish(self) -> io::Result<String> {
        let hash = match self.worker {
            Some(worker) => worker.finish()?,
            None => self.serial,
        };
        Ok(hex::encode(hash.finalize()))
    }
}

// One active block, one queued block, and one caller buffer: at most 768 KiB.
// Drop closes the queue and joins even when the output writer fails.
struct HashWorker {
    sender: Option<SyncSender<Vec<u8>>>,
    result: Option<JoinHandle<Sha1>>,
    pending: Vec<u8>,
}

impl HashWorker {
    fn new(mut hash: Sha1) -> io::Result<Self> {
        let (sender, receiver) = sync_channel::<Vec<u8>>(1);
        let result = Builder::new()
            .name("anki-forge-package-hash".into())
            .spawn(move || {
                while let Ok(bytes) = receiver.recv() {
                    hash.update(&bytes);
                }
                hash
            })?;
        Ok(Self {
            sender: Some(sender),
            result: Some(result),
            pending: Vec::new(),
        })
    }

    fn update(&mut self, mut bytes: &[u8]) -> io::Result<()> {
        while !bytes.is_empty() {
            if self.pending.len() == BLOCK_BYTES {
                self.send_pending()?;
            }
            if self.pending.capacity() == 0 {
                self.pending = Vec::with_capacity(BLOCK_BYTES);
            }
            let count = bytes.len().min(BLOCK_BYTES - self.pending.len());
            self.pending.extend_from_slice(&bytes[..count]);
            bytes = &bytes[count..];
        }
        Ok(())
    }

    fn send_pending(&mut self) -> io::Result<()> {
        if !self.pending.is_empty() {
            let bytes = std::mem::take(&mut self.pending);
            self.sender
                .as_ref()
                .expect("active hash sender")
                .send(bytes)
                .map_err(|_| io::Error::other("package hash worker stopped"))?;
        }
        Ok(())
    }

    fn finish(mut self) -> io::Result<Sha1> {
        self.send_pending()?;
        drop(self.sender.take());
        self.result
            .take()
            .expect("active hash worker")
            .join()
            .map_err(|_| io::Error::other("package hash worker stopped"))
    }
}

impl Drop for HashWorker {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(result) = self.result.take() {
            let _ = result.join();
        }
    }
}
