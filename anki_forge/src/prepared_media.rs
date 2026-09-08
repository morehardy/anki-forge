//! Build-owned media prepared through bounded worker queues. Encoded payloads
//! share a fixed memory budget and spill to disk when it is exhausted.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use sha1::{Digest, Sha1};

use crate::authoring_core::media::{
    prepare_media_source, validate_authoring_media_filename, MediaIngestDiagnostic,
    MediaIngestError, MediaObject, NormalizeOptions, PreparedMediaSource,
};
use crate::authoring_core::media_io::{sniff_mime, IngestedMediaBytes};
use crate::authoring_core::model::AuthoringMedia;
use crate::writer_core::stream_zip::StreamZip;

mod payload;
use payload::{EncodedPayload, PayloadPool};

const BUFFER_BYTES: usize = 64 * 1024;
const MAX_WORKERS: usize = 4;

type CandidateArchive = StreamZip<tempfile::NamedTempFile>;

pub(crate) enum RegisteredFingerprint {
    Sha1(String),
    Blake3 { digest: String, size: u64 },
}

struct RegisteredSource {
    path: PathBuf,
    fingerprint: RegisteredFingerprint,
}

pub(crate) struct PreparedMedia {
    archive: Mutex<Option<CandidateArchive>>,
    registered: BTreeMap<String, RegisteredSource>,
    objects: BTreeMap<String, (String, u64)>,
    export_order: Vec<String>,
}

impl PreparedMedia {
    pub(crate) fn new_in(directory: &Path) -> io::Result<Self> {
        let file = tempfile::NamedTempFile::new_in(directory)?;
        let mut archive = StreamZip::new(file);
        archive.bytes("meta", &[8, 3]).map_err(io::Error::other)?;
        Ok(Self {
            archive: Mutex::new(Some(archive)),
            registered: BTreeMap::new(),
            objects: BTreeMap::new(),
            export_order: Vec::new(),
        })
    }

    pub(crate) fn register_source(
        &mut self,
        id: String,
        path: PathBuf,
        fingerprint: RegisteredFingerprint,
    ) {
        self.registered
            .insert(id, RegisteredSource { path, fingerprint });
    }

    pub(crate) fn prepare_all(
        &mut self,
        items: &[AuthoringMedia],
        options: &NormalizeOptions,
    ) -> Vec<Option<Result<IngestedMediaBytes, MediaIngestError>>> {
        let mut seen = BTreeSet::new();
        let mut jobs = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                seen.insert(item.id.as_str())
                    && validate_authoring_media_filename(&item.desired_filename).is_ok()
            })
            .collect::<Vec<_>>();
        jobs.sort_by(|(_, left), (_, right)| {
            left.desired_filename
                .as_bytes()
                .cmp(right.desired_filename.as_bytes())
                .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
        });
        let mut ingested = std::iter::repeat_with(|| None)
            .take(items.len())
            .collect::<Vec<_>>();
        if jobs.is_empty() {
            return ingested;
        }
        let pool = PayloadPool::new();
        let workers = if jobs.len() < 16 {
            1
        } else {
            std::thread::available_parallelism()
                .map_or(1, usize::from)
                .min(MAX_WORKERS)
        };
        let mut archive = self
            .archive
            .get_mut()
            .expect("unshared archive lock")
            .take()
            .expect("unconsumed candidate archive");
        let mut objects = BTreeMap::new();
        let mut names = BTreeSet::new();
        let mut export_order = Vec::new();
        let mut accept =
            |index: usize,
             result: Result<(IngestedMediaBytes, EncodedPayload), MediaIngestError>| {
                let item = &items[index];
                ingested[index] = Some(result.and_then(|(metadata, payload)| {
                    if names.insert(item.desired_filename.clone()) {
                        archive
                            .entry(
                                &export_order.len().to_string(),
                                payload.size(),
                                payload.checksum(),
                                |writer| {
                                    payload.write_to(writer)?;
                                    Ok(())
                                },
                            )
                            .map_err(|error| {
                                diagnostic(item, "MEDIA.CAS_WRITE_FAILED", error.to_string())
                            })?;
                        export_order.push(item.desired_filename.clone());
                    }
                    objects.insert(
                        metadata.blake3.clone(),
                        (metadata.sha1.clone(), metadata.size_bytes),
                    );
                    Ok(metadata)
                }));
            };
        if workers == 1 {
            let mut context = zstd::zstd_safe::CCtx::create();
            for &(index, item) in &jobs {
                accept(index, self.prepare_one(item, options, &mut context, &pool));
            }
        } else {
            std::thread::scope(|scope| {
                // Bound all submitted, unfinished and completed payloads together.
                // Per-job reply channels preserve order and disconnect if a worker
                // stops; no reply sender remains in a different waiting worker.
                type Preparation = Result<(IngestedMediaBytes, EncodedPayload), MediaIngestError>;
                let window = workers * 2 + 1;
                let (sender, receiver) = sync_channel::<(usize, SyncSender<Preparation>)>(window);
                let receiver = Arc::new(Mutex::new(receiver));
                let mut handles = Vec::new();
                for _ in 0..workers {
                    let receiver = Arc::clone(&receiver);
                    let jobs = &jobs;
                    let prepared = &*self;
                    let pool = &pool;
                    handles.push(
                        std::thread::Builder::new()
                            .name("anki-forge-media".into())
                            .spawn_scoped(scope, move || {
                                let mut context = zstd::zstd_safe::CCtx::create();
                                loop {
                                    let job = receiver.lock().expect("media job queue lock").recv();
                                    let Ok((position, reply)) = job else { break };
                                    if reply
                                        .send(prepared.prepare_one(
                                            jobs[position].1,
                                            options,
                                            &mut context,
                                            pool,
                                        ))
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            }),
                    );
                }
                drop(receiver);
                let stopped = |item| {
                    diagnostic(
                        item,
                        "MEDIA.CAS_WRITE_FAILED",
                        "media preparation worker could not complete".into(),
                    )
                };
                if handles.iter().any(Result::is_err) {
                    drop(sender);
                    for &(index, item) in &jobs {
                        accept(index, Err(stopped(item)));
                    }
                } else {
                    let mut pending = VecDeque::with_capacity(window);
                    let mut next = 0;
                    for _ in 0..jobs.len() {
                        while next < jobs.len() && pending.len() < window {
                            let (reply, receiver) = sync_channel(1);
                            // Failed submission drops the reply sender, which is
                            // reported through the same ordered receive below.
                            let _ = sender.send((next, reply));
                            pending.push_back((next, receiver));
                            next += 1;
                        }
                        let (position, receiver) = pending.pop_front().expect("pending media job");
                        let (index, item) = jobs[position];
                        let result = receiver.recv().unwrap_or_else(|_| Err(stopped(item)));
                        accept(index, result);
                    }
                    drop(sender);
                }
                for handle in handles.into_iter().flatten() {
                    let _ = handle.join();
                }
            });
        }
        self.objects = objects;
        self.export_order = export_order;
        *self.archive.get_mut().expect("unshared archive lock") = Some(archive);
        ingested
    }
    fn prepare_one(
        &self,
        item: &AuthoringMedia,
        options: &NormalizeOptions,
        context: &mut zstd::zstd_safe::CCtx<'static>,
        pool: &PayloadPool,
    ) -> Result<(IngestedMediaBytes, EncodedPayload), MediaIngestError> {
        let source = if let Some(registered) = self.registered.get(&item.id) {
            let metadata = std::fs::metadata(&registered.path)
                .map_err(|error| source_error(item, &registered.path, error))?;
            if !metadata.is_file() {
                return Err(diagnostic(
                    item,
                    "MEDIA.SOURCE_NOT_REGULAR_FILE",
                    format!(
                        "source is not a regular file: {}",
                        registered.path.display()
                    ),
                ));
            }
            PreparedMediaSource::Path {
                path: registered.path.clone(),
                size_bytes: metadata.len(),
            }
        } else {
            prepare_media_source(item, options)?
        };
        let limit = options.media_policy.max_media_object_bytes;
        if !self.registered.contains_key(&item.id)
            && limit.is_some_and(|limit| source.known_size_bytes() > limit)
        {
            return Err(diagnostic(
                item,
                "MEDIA.SIZE_LIMIT_EXCEEDED",
                "source exceeds max_media_object_bytes".into(),
            ));
        }
        let mut reader: Box<dyn Read + '_> = match &source {
            PreparedMediaSource::Path { path, .. } => {
                Box::new(File::open(path).map_err(|error| source_error(item, path, error))?)
            }
            PreparedMediaSource::InlineBytes(bytes) => Box::new(io::Cursor::new(bytes)),
        };
        let sink = pool.payload();
        let encode_error =
            |error: io::Error| diagnostic(item, "MEDIA.CAS_WRITE_FAILED", error.to_string());
        let mut encoder = zstd::stream::Encoder::with_context(sink, context);
        let mut sha1 = Sha1::new();
        let mut blake3 = blake3::Hasher::new();
        let mut size = 0u64;
        let mut sample = Vec::with_capacity(8192);
        let mut buffer = [0u8; BUFFER_BYTES];
        loop {
            let count = reader
                .read(&mut buffer)
                .map_err(|error| diagnostic(item, "MEDIA.SOURCE_READ_FAILED", error.to_string()))?;
            if count == 0 {
                break;
            }
            size = size.checked_add(count as u64).ok_or_else(|| {
                diagnostic(
                    item,
                    "MEDIA.SIZE_LIMIT_EXCEEDED",
                    "media size overflow".into(),
                )
            })?;
            if !self.registered.contains_key(&item.id) && limit.is_some_and(|limit| size > limit) {
                return Err(diagnostic(
                    item,
                    "MEDIA.SIZE_LIMIT_EXCEEDED",
                    "source exceeds max_media_object_bytes while reading".into(),
                ));
            }
            let bytes = &buffer[..count];
            let sample_count = (8192 - sample.len()).min(count);
            sample.extend_from_slice(&bytes[..sample_count]);
            sha1.update(bytes);
            blake3.update(bytes);
            if limit.is_none_or(|limit| size <= limit) {
                encoder.write_all(bytes).map_err(encode_error)?;
            }
        }
        let sha1 = hex::encode(sha1.finalize());
        let blake3 = blake3.finalize().to_hex().to_string();
        if let Some(registered) = self.registered.get(&item.id) {
            let matches = match &registered.fingerprint {
                RegisteredFingerprint::Sha1(expected) => &sha1 == expected,
                RegisteredFingerprint::Blake3 {
                    digest,
                    size: expected_size,
                } => &blake3 == digest && size == *expected_size,
            };
            if !matches {
                return Err(diagnostic(
                    item,
                    "MEDIA.SOURCE_CHANGED",
                    format!(
                        "media source {} changed after registration",
                        registered.path.display()
                    ),
                ));
            }
        }
        if limit.is_some_and(|limit| size > limit) {
            return Err(diagnostic(
                item,
                "MEDIA.SIZE_LIMIT_EXCEEDED",
                "source exceeds max_media_object_bytes".into(),
            ));
        }
        let sink = encoder.finish().map_err(encode_error)?;
        Ok((
            IngestedMediaBytes {
                blake3,
                sha1: sha1.clone(),
                size_bytes: size,
                sniffed_mime: sniff_mime(&sample),
                object_path: PathBuf::new(),
            },
            sink,
        ))
    }

    pub(crate) fn verify(&self, object: &MediaObject) -> Result<()> {
        let (sha1, size) = self
            .objects
            .get(&object.blake3)
            .context("missing build-owned media object")?;
        anyhow::ensure!(
            sha1 == &object.sha1 && size == &object.size_bytes,
            "MEDIA.CAS_INTEGRITY_MISMATCH: build-owned object {}",
            object.id
        );
        Ok(())
    }

    pub(crate) fn take_archive(
        &self,
        normalized: &crate::authoring_core::NormalizedIr,
    ) -> Result<CandidateArchive> {
        anyhow::ensure!(
            self.export_order.len() == normalized.media_bindings.len()
                && self
                    .export_order
                    .iter()
                    .zip(&normalized.media_bindings)
                    .all(|(name, binding)| name == &binding.export_filename),
            "prepared media order differs from normalized bindings"
        );
        for object in &normalized.media_objects {
            self.verify(object)?;
        }
        self.archive
            .lock()
            .map_err(|_| anyhow::anyhow!("candidate archive lock poisoned"))?
            .take()
            .context("candidate archive was already consumed")
    }
}

fn diagnostic(item: &AuthoringMedia, code: &str, summary: String) -> MediaIngestError {
    MediaIngestError {
        diagnostics: vec![MediaIngestDiagnostic {
            level: "error".into(),
            code: code.into(),
            summary,
            path: Some(item.id.clone()),
        }],
    }
}

fn source_error(
    item: &AuthoringMedia,
    path: &std::path::Path,
    error: io::Error,
) -> MediaIngestError {
    diagnostic(
        item,
        if error.kind() == io::ErrorKind::NotFound {
            "MEDIA.SOURCE_MISSING"
        } else {
            "MEDIA.SOURCE_READ_FAILED"
        },
        format!("read media source {}: {error}", path.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Seek;

    #[test]
    fn one_large_encoded_payload_uses_available_build_memory() {
        let pool = PayloadPool::new();
        let mut payload = pool.payload();
        let block = [93; BUFFER_BYTES];
        for _ in 0..16 {
            payload.write_all(&block).unwrap();
        }
        assert!(
            !payload.is_spilled(),
            "a 1 MiB payload should use otherwise idle build memory"
        );
    }

    #[test]
    fn preparation_preserves_order_and_recovers_after_mixed_source_failures() {
        use crate::authoring_core::media::AuthoringMediaSource;
        let root = tempfile::tempdir().unwrap();
        let options = NormalizeOptions {
            base_dir: root.path().into(),
            media_store_dir: root.path().join("unused-cas"),
            media_policy: crate::authoring_core::media::MediaPolicy::default_strict(),
        };
        let mut payloads = Vec::new();
        let mut items = Vec::new();
        let mut state = 0x1234_5678_u32;
        for index in 0..40 {
            let size = if index % 4 == 0 {
                1024 * 1024 + 17
            } else {
                index * 37
            };
            let bytes = (0..size)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect::<Vec<_>>();
            let filename = format!("{index:03}.bin");
            let path = root.path().join(&filename);
            std::fs::write(&path, &bytes).unwrap();
            items.push(AuthoringMedia {
                id: format!("media-{index}"),
                desired_filename: filename.clone(),
                source: AuthoringMediaSource::Path { path: filename },
                declared_mime: None,
            });
            payloads.push(bytes);
        }
        let mut failed = PreparedMedia::new_in(root.path()).unwrap();
        failed.register_source(
            items[20].id.clone(),
            root.path().join("020.bin"),
            RegisteredFingerprint::Sha1("changed".into()),
        );
        std::fs::remove_file(root.path().join("008.bin")).unwrap();
        let results = failed.prepare_all(&items, &options);
        for (index, result) in results.iter().enumerate() {
            let result = result.as_ref().unwrap();
            if index == 8 || index == 20 {
                let code = if index == 8 {
                    "MEDIA.SOURCE_MISSING"
                } else {
                    "MEDIA.SOURCE_CHANGED"
                };
                assert_eq!(result.as_ref().unwrap_err().diagnostics[0].code, code);
            } else {
                assert_eq!(
                    result.as_ref().unwrap().sha1,
                    hex::encode(Sha1::digest(&payloads[index]))
                );
            }
        }
        assert_eq!(
            failed.export_order,
            items
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != 8 && *i != 20)
                .map(|(_, item)| item.desired_filename.clone())
                .collect::<Vec<_>>()
        );
        drop(failed);
        // A subsequent build must succeed after both errors, with the same
        // canonical order even when the caller supplies the reverse order.
        std::fs::write(root.path().join("008.bin"), &payloads[8]).unwrap();
        let mut archives = Vec::new();
        for reverse in [false, true] {
            let mut selected = items.clone();
            if reverse {
                selected.reverse();
            }
            let mut prepared = PreparedMedia::new_in(root.path()).unwrap();
            let results = prepared.prepare_all(&selected, &options);
            assert!(results
                .iter()
                .all(|result| result.as_ref().unwrap().is_ok()));
            assert_eq!(
                prepared.export_order,
                items
                    .iter()
                    .map(|item| item.desired_filename.clone())
                    .collect::<Vec<_>>()
            );
            let archive = prepared.archive.get_mut().unwrap().take().unwrap();
            let (mut file, fingerprint) = archive.finish().unwrap();
            file.rewind().unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            assert_eq!(
                fingerprint,
                format!("package:{}", hex::encode(Sha1::digest(&bytes)))
            );
            let mut zip = zip::ZipArchive::new(io::Cursor::new(&bytes)).unwrap();
            for (index, expected) in payloads.iter().enumerate() {
                let entry = zip.by_name(&index.to_string()).unwrap();
                assert_eq!(zstd::stream::decode_all(entry).unwrap(), *expected);
            }
            archives.push(bytes);
        }
        assert_eq!(archives[0], archives[1]);
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), items.len());
    }

    #[test]
    fn encoded_payload_spills_before_unbounded_growth_and_preserves_bytes() {
        let pool = PayloadPool::new();
        let mut payload = pool.payload();
        let block = [93; BUFFER_BYTES];
        for _ in 0..payload::MEMORY_BYTES / BUFFER_BYTES {
            payload.write_all(&block).unwrap();
        }
        assert!(!payload.is_spilled());
        payload.write_all(&block).unwrap();
        assert!(payload.is_spilled());
        assert_eq!(
            payload.size(),
            (payload::MEMORY_BYTES + BUFFER_BYTES) as u64
        );
        let checksum = payload.checksum();
        let mut actual = Vec::new();
        payload.write_to(&mut actual).unwrap();
        assert_eq!(actual, vec![93; payload::MEMORY_BYTES + BUFFER_BYTES]);
        assert_eq!(checksum, crc32fast::hash(&actual));
    }
}
