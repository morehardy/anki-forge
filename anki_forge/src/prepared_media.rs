//! Build-owned media prepared through bounded worker queues. Small encoded
//! payloads go straight into the candidate ZIP; oversized ones spill to disk.
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use sha1::{Digest, Sha1};

use crate::authoring_core::media::{
    prepare_media_source, validate_authoring_media_filename, MediaIngestDiagnostic,
    MediaIngestError, MediaObject, NormalizeOptions, PreparedMediaSource,
};
use crate::authoring_core::media_io::{sniff_mime, IngestedMediaBytes};
use crate::authoring_core::model::AuthoringMedia;
use crate::writer_core::stream_zip::StreamZip;

const BUFFER_BYTES: usize = 64 * 1024;
const MAX_WORKERS: usize = 4;
// Each worker can hold one queued result plus one being prepared. The consumer
// holds one more. SpooledTempFile rolls before writes cross this length; Vec
// capacity and codec contexts add bounded overhead beyond the payload limit.
const PER_PAYLOAD_MEMORY: usize = 512 * 1024;

type CandidateArchive = StreamZip<tempfile::NamedTempFile>;

pub(crate) enum RegisteredFingerprint {
    Sha1(String),
    Blake3 { digest: String, size: u64 },
}

struct RegisteredSource {
    path: PathBuf,
    fingerprint: RegisteredFingerprint,
}

struct EncodedPayload {
    data: tempfile::SpooledTempFile,
    size: u64,
    crc: crc32fast::Hasher,
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
                ingested[index] = Some(result.and_then(|(metadata, mut payload)| {
                    if names.insert(item.desired_filename.clone()) {
                        payload.data.seek(SeekFrom::Start(0)).map_err(|error| {
                            diagnostic(item, "MEDIA.CAS_WRITE_FAILED", error.to_string())
                        })?;
                        let crc = payload.crc.finalize();
                        archive
                            .entry(
                                &export_order.len().to_string(),
                                payload.size,
                                crc,
                                |writer| {
                                    io::copy(&mut payload.data, writer)?;
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
                accept(index, self.prepare_one(item, options, &mut context));
            }
        } else {
            std::thread::scope(|scope| {
                let mut receivers = Vec::new();
                let mut handles = Vec::new();
                for worker in 0..workers {
                    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
                    receivers.push(receiver);
                    let jobs = &jobs;
                    let prepared = &*self;
                    handles.push(
                        std::thread::Builder::new()
                            .name("anki-forge-media".into())
                            .spawn_scoped(scope, move || {
                                let mut context = zstd::zstd_safe::CCtx::create();
                                for &(_, item) in jobs.iter().skip(worker).step_by(workers) {
                                    if sender
                                        .send(prepared.prepare_one(item, options, &mut context))
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            }),
                    );
                }
                for (job_index, &(index, item)) in jobs.iter().enumerate() {
                    let result = receivers[job_index % workers].recv().unwrap_or_else(|_| {
                        Err(diagnostic(
                            item,
                            "MEDIA.CAS_WRITE_FAILED",
                            "media preparation worker could not complete".into(),
                        ))
                    });
                    accept(index, result);
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
        let sink = EncodedPayload {
            data: tempfile::spooled_tempfile(PER_PAYLOAD_MEMORY),
            size: 0,
            crc: crc32fast::Hasher::new(),
        };
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

impl Write for EncodedPayload {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.data.write(bytes)?;
        self.size += written as u64;
        self.crc.update(&bytes[..written]);
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
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
    #[test]
    fn encoded_payload_spills_before_unbounded_growth_and_preserves_bytes() {
        let mut payload = EncodedPayload {
            data: tempfile::spooled_tempfile(PER_PAYLOAD_MEMORY),
            size: 0,
            crc: crc32fast::Hasher::new(),
        };
        let block = [93; BUFFER_BYTES];
        for _ in 0..PER_PAYLOAD_MEMORY / BUFFER_BYTES {
            payload.write_all(&block).unwrap();
        }
        assert!(!payload.data.is_rolled());
        payload.write_all(&block).unwrap();
        assert!(payload.data.is_rolled());
        assert_eq!(payload.size, (PER_PAYLOAD_MEMORY + BUFFER_BYTES) as u64);
        payload.data.rewind().unwrap();
        let mut actual = Vec::new();
        payload.data.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, vec![93; PER_PAYLOAD_MEMORY + BUFFER_BYTES]);
        assert_eq!(payload.crc.finalize(), crc32fast::hash(&actual));
    }
}
