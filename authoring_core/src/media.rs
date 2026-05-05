use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use crate::media_io::{
    decode_inline_bytes, ingest_media_read_source_to_cas, object_store_path,
    CasExistingIntegrityReason, MediaIoError, MediaReadSource, MediaSniffConfidence, SniffedMime,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthoringMediaSource {
    Path { path: String },
    InlineBytes { data_base64: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticBehavior {
    Ignore,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizeOptions {
    pub base_dir: PathBuf,
    pub media_store_dir: PathBuf,
    pub media_policy: MediaPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPolicy {
    pub inline_bytes_max: usize,
    pub max_media_object_bytes: Option<u64>,
    pub max_total_media_bytes: Option<u64>,
    pub declared_mime_mismatch_behavior: DiagnosticBehavior,
    pub unknown_mime_behavior: DiagnosticBehavior,
    pub unused_binding_behavior: DiagnosticBehavior,
}

impl MediaPolicy {
    pub fn default_strict() -> Self {
        Self {
            inline_bytes_max: 64 * 1024,
            max_media_object_bytes: None,
            max_total_media_bytes: None,
            declared_mime_mismatch_behavior: DiagnosticBehavior::Error,
            unknown_mime_behavior: DiagnosticBehavior::Warning,
            unused_binding_behavior: DiagnosticBehavior::Warning,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaObject {
    pub id: String,
    pub object_ref: String,
    pub blake3: String,
    pub sha1: String,
    pub size_bytes: u64,
    pub mime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaBinding {
    pub id: String,
    pub export_filename: String,
    pub object_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaReference {
    pub owner_kind: String,
    pub owner_id: String,
    pub location_kind: String,
    pub location_name: String,
    pub raw_ref: String,
    pub ref_kind: String,
    #[serde(flatten)]
    pub resolution: MediaReferenceResolution,
}

impl MediaReference {
    pub fn resolution_status(&self) -> &'static str {
        match self.resolution {
            MediaReferenceResolution::Resolved { .. } => "resolved",
            MediaReferenceResolution::Missing => "missing",
            MediaReferenceResolution::Skipped { .. } => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "resolution_status", rename_all = "snake_case")]
pub enum MediaReferenceResolution {
    Resolved { media_id: String },
    Missing,
    Skipped { skip_reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaIngestDiagnostic {
    pub level: String,
    pub code: String,
    pub summary: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MediaIngestError {
    pub diagnostics: Vec<MediaIngestDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct MediaIngestResult {
    pub objects: Vec<MediaObject>,
    pub bindings: Vec<MediaBinding>,
    pub diagnostics: Vec<MediaIngestDiagnostic>,
    pub media_store_dir: PathBuf,
}

impl MediaIngestResult {
    pub fn object_path(&self, object: &MediaObject) -> Option<PathBuf> {
        object_store_path(&self.media_store_dir, &object.blake3).ok()
    }
}

pub fn ingest_authoring_media(
    media: &[crate::model::AuthoringMedia],
    options: &NormalizeOptions,
) -> Result<MediaIngestResult, MediaIngestError> {
    let mut diagnostics = Vec::new();
    let mut objects_by_id = BTreeMap::<String, MediaObject>::new();
    let mut bindings = Vec::<MediaBinding>::new();
    let mut seen_media_ids = BTreeSet::<String>::new();
    let mut filename_to_object = BTreeMap::<String, String>::new();
    let mut total_unique_size = 0_u64;

    for item in media {
        if !seen_media_ids.insert(item.id.clone()) {
            diagnostics.push(error(
                "MEDIA.DUPLICATE_MEDIA_ID",
                format!("duplicate media id {}", item.id),
                Some(item.id.clone()),
            ));
            continue;
        }

        if let Err(message) = validate_bare_filename(&item.desired_filename) {
            diagnostics.push(error(
                "MEDIA.UNSAFE_FILENAME",
                message,
                Some(item.desired_filename.clone()),
            ));
            continue;
        }

        let prepared_source = match prepare_media_source(item, options) {
            Ok(source) => source,
            Err(mut err) => {
                diagnostics.append(&mut err.diagnostics);
                continue;
            }
        };

        if let Some(limit) = options.media_policy.max_media_object_bytes {
            let source_size = prepared_source.known_size_bytes();
            if source_size > limit {
                diagnostics.push(size_limit_exceeded(
                    &item.id,
                    source_size,
                    limit,
                    "max_media_object_bytes",
                ));
                continue;
            }
        }

        let ingested = match ingest_media_read_source_to_cas(
            prepared_source.as_read_source(),
            &options.media_store_dir,
        ) {
            Ok(ingested) => ingested,
            Err(err) => {
                diagnostics.push(media_io_error_to_diagnostic(err, &item.id));
                continue;
            }
        };

        let mut object_has_error = false;
        if let Some(limit) = options.media_policy.max_media_object_bytes {
            if ingested.size_bytes > limit {
                diagnostics.push(size_limit_exceeded(
                    &item.id,
                    ingested.size_bytes,
                    limit,
                    "max_media_object_bytes",
                ));
                object_has_error = true;
            }
        }

        let object_id = media_object_id(&ingested.blake3);
        let is_new_object = !objects_by_id.contains_key(&object_id);
        let mut mime_diagnostics = Vec::new();
        let mime = effective_mime(
            item.declared_mime.as_deref(),
            ingested.sniffed_mime.as_ref(),
            &options.media_policy,
            &mut mime_diagnostics,
            &item.id,
        );
        let mime_has_error = mime_diagnostics.iter().any(|item| item.level == "error");
        diagnostics.append(&mut mime_diagnostics);
        if object_has_error || mime_has_error {
            continue;
        }

        let object = MediaObject {
            id: object_id.clone(),
            object_ref: media_object_ref(&ingested.blake3),
            blake3: ingested.blake3,
            sha1: ingested.sha1,
            size_bytes: ingested.size_bytes,
            mime,
        };

        if let Some(previous_object_id) = filename_to_object.get(&item.desired_filename) {
            if previous_object_id != &object.id {
                diagnostics.push(error(
                    "MEDIA.DUPLICATE_FILENAME_CONFLICT",
                    format!(
                        "export filename {} maps to multiple objects",
                        item.desired_filename
                    ),
                    Some(item.desired_filename.clone()),
                ));
            } else {
                diagnostics.push(info(
                    "MEDIA.DEDUPED_BINDING",
                    format!(
                        "duplicate export filename {} maps to the same object and was merged",
                        item.desired_filename
                    ),
                    Some(item.desired_filename.clone()),
                ));
            }
            continue;
        }

        if is_new_object {
            if let Some(limit) = options.media_policy.max_total_media_bytes {
                if total_unique_size.saturating_add(object.size_bytes) > limit {
                    diagnostics.push(error(
                        "MEDIA.SIZE_LIMIT_EXCEEDED",
                        format!("unique media bytes would exceed max_total_media_bytes {limit}"),
                        Some(item.id.clone()),
                    ));
                    continue;
                }
            }
            total_unique_size += object.size_bytes;
        } else {
            diagnostics.push(info(
                "MEDIA.DEDUPED_OBJECT",
                format!("media {} reuses object {}", item.id, object.id),
                Some(item.id.clone()),
            ));
        }
        objects_by_id
            .entry(object.id.clone())
            .or_insert(object.clone());
        filename_to_object.insert(item.desired_filename.clone(), object.id.clone());
        bindings.push(MediaBinding {
            id: item.id.clone(),
            export_filename: item.desired_filename.clone(),
            object_id: object.id,
        });
    }

    if diagnostics.iter().any(|item| item.level == "error") {
        return Err(MediaIngestError { diagnostics });
    }

    let mut objects = objects_by_id.into_values().collect::<Vec<_>>();
    sort_media_objects(&mut objects);
    sort_media_bindings(&mut bindings);
    Ok(MediaIngestResult {
        objects,
        bindings,
        diagnostics,
        media_store_dir: options.media_store_dir.clone(),
    })
}

enum PreparedMediaSource {
    Path { path: PathBuf, size_bytes: u64 },
    InlineBytes(Vec<u8>),
}

impl PreparedMediaSource {
    fn as_read_source(&self) -> MediaReadSource<'_> {
        match self {
            Self::Path { path, .. } => MediaReadSource::File { path },
            Self::InlineBytes(bytes) => MediaReadSource::InlineBytes { bytes },
        }
    }

    fn known_size_bytes(&self) -> u64 {
        match self {
            Self::Path { size_bytes, .. } => *size_bytes,
            Self::InlineBytes(bytes) => bytes.len() as u64,
        }
    }
}

fn prepare_media_source(
    item: &crate::model::AuthoringMedia,
    options: &NormalizeOptions,
) -> Result<PreparedMediaSource, MediaIngestError> {
    match &item.source {
        AuthoringMediaSource::Path { path } => resolve_path_source(path, options)
            .map(|(path, size_bytes)| PreparedMediaSource::Path { path, size_bytes }),
        AuthoringMediaSource::InlineBytes { data_base64 } => {
            decode_inline_bytes(data_base64, options.media_policy.inline_bytes_max)
                .map(PreparedMediaSource::InlineBytes)
                .map_err(|err| MediaIngestError {
                    diagnostics: vec![media_io_error_to_diagnostic(err, &item.id)],
                })
        }
    }
}

fn resolve_path_source(
    path: &str,
    options: &NormalizeOptions,
) -> Result<(PathBuf, u64), MediaIngestError> {
    let raw_path = Path::new(path);
    if path.is_empty() || raw_path.is_absolute() || has_parent_component(raw_path) {
        return Err(one_error(
            "MEDIA.UNSAFE_SOURCE_PATH",
            format!("source.path must be relative and stay below base_dir: {path}"),
            Some(path.into()),
        ));
    }

    let base = options
        .base_dir
        .canonicalize()
        .map_err(|err| MediaIngestError {
            diagnostics: vec![error(
                "MEDIA.UNSAFE_SOURCE_PATH",
                format!(
                    "canonicalize base_dir {}: {err}",
                    options.base_dir.display()
                ),
                Some(options.base_dir.display().to_string()),
            )],
        })?;
    let candidate = options.base_dir.join(raw_path);
    let canonical = candidate.canonicalize().map_err(|err| MediaIngestError {
        diagnostics: vec![error(
            "MEDIA.SOURCE_MISSING",
            format!("read source.path {path}: {err}"),
            Some(path.into()),
        )],
    })?;

    if !canonical.starts_with(&base) {
        return Err(one_error(
            "MEDIA.UNSAFE_SOURCE_PATH",
            format!("source.path escapes base_dir: {path}"),
            Some(path.into()),
        ));
    }

    let metadata = std::fs::metadata(&canonical).map_err(|err| MediaIngestError {
        diagnostics: vec![error(
            "MEDIA.SOURCE_MISSING",
            format!("stat source.path {path}: {err}"),
            Some(path.into()),
        )],
    })?;
    if !metadata.is_file() {
        return Err(one_error(
            "MEDIA.SOURCE_NOT_REGULAR_FILE",
            format!("source.path is not a regular file: {path}"),
            Some(path.into()),
        ));
    }

    Ok((canonical, metadata.len()))
}

fn validate_bare_filename(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("media filename must not be empty".into());
    }
    if name.contains(['/', '\\'])
        || Path::new(name).is_absolute()
        || has_parent_component(Path::new(name))
    {
        return Err(format!("media filename must be a bare filename: {name}"));
    }
    let mut components = Path::new(name).components();
    let is_bare =
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
    if is_bare {
        Ok(())
    } else {
        Err(format!("media filename must be a bare filename: {name}"))
    }
}

fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
}

fn effective_mime(
    declared_mime: Option<&str>,
    sniffed: Option<&SniffedMime>,
    policy: &MediaPolicy,
    diagnostics: &mut Vec<MediaIngestDiagnostic>,
    media_id: &str,
) -> String {
    if let (Some(declared), Some(sniffed)) = (declared_mime, sniffed) {
        if sniffed.confidence == MediaSniffConfidence::High
            && !mime_type_subtype_eq(declared, &sniffed.mime)
        {
            push_policy_diagnostic(
                diagnostics,
                policy.declared_mime_mismatch_behavior,
                "MEDIA.DECLARED_MIME_MISMATCH",
                format!(
                    "declared MIME {declared} conflicts with sniffed MIME {}",
                    sniffed.mime
                ),
                Some(media_id.into()),
            );
        }
    }

    if let Some(sniffed) = sniffed {
        sniffed.mime.clone()
    } else if let Some(declared) = declared_mime {
        declared.into()
    } else {
        push_policy_diagnostic(
            diagnostics,
            policy.unknown_mime_behavior,
            "MEDIA.UNKNOWN_MIME",
            format!("could not determine MIME for {media_id}"),
            Some(media_id.into()),
        );
        "application/octet-stream".into()
    }
}

fn mime_type_subtype_eq(left: &str, right: &str) -> bool {
    let left = left.split(';').next().unwrap_or("").trim();
    let right = right.split(';').next().unwrap_or("").trim();
    left.eq_ignore_ascii_case(right)
}

fn media_io_error_to_diagnostic(err: MediaIoError, media_id: &str) -> MediaIngestDiagnostic {
    let code = err.diagnostic_code();
    let summary = match err {
        MediaIoError::SourceOpen { path, message } => {
            format!(
                "open media source for {media_id} at {}: {message}",
                path.display()
            )
        }
        MediaIoError::SourceRead { path, message } => match path {
            Some(path) => format!(
                "read media source for {media_id} at {}: {message}",
                path.display()
            ),
            None => format!("read inline media bytes for {media_id}: {message}"),
        },
        MediaIoError::InlineBase64Decode { message } => {
            format!("decode inline bytes for {media_id}: {message}")
        }
        MediaIoError::InlineBytesTooLarge { size, limit } => {
            format!("inline bytes for {media_id} decode to {size} bytes, above inline_bytes_max {limit}")
        }
        MediaIoError::CasWrite { path, message } => {
            format!(
                "write CAS object for {media_id} at {}: {message}",
                path.display()
            )
        }
        MediaIoError::CasFinalize { path, message } => format!(
            "finalize CAS object for {media_id} at {}: {message}",
            path.display()
        ),
        MediaIoError::CasExistingIntegrity { path, reason } => format!(
            "existing CAS object for {media_id} at {} failed integrity check: {}",
            path.display(),
            cas_existing_integrity_summary(reason)
        ),
    };

    error(code, summary, Some(media_id.into()))
}

fn cas_existing_integrity_summary(reason: CasExistingIntegrityReason) -> String {
    match reason {
        CasExistingIntegrityReason::OpenFailed { message } => {
            format!("could not open existing object: {message}")
        }
        CasExistingIntegrityReason::ReadFailed { message } => {
            format!("could not read existing object: {message}")
        }
        CasExistingIntegrityReason::Mismatch {
            expected_blake3,
            actual_blake3,
            expected_size,
            actual_size,
        } => format!(
            "expected blake3 {expected_blake3} and size {expected_size}, found blake3 {actual_blake3} and size {actual_size}"
        ),
    }
}

fn size_limit_exceeded(
    media_id: &str,
    size_bytes: u64,
    limit: u64,
    policy_name: &str,
) -> MediaIngestDiagnostic {
    error(
        "MEDIA.SIZE_LIMIT_EXCEEDED",
        format!("media object {media_id} has {size_bytes} bytes, above {policy_name} {limit}"),
        Some(media_id.into()),
    )
}

fn push_policy_diagnostic(
    diagnostics: &mut Vec<MediaIngestDiagnostic>,
    behavior: DiagnosticBehavior,
    code: &str,
    summary: String,
    path: Option<String>,
) {
    match behavior {
        DiagnosticBehavior::Ignore => {}
        DiagnosticBehavior::Info => diagnostics.push(diagnostic("info", code, summary, path)),
        DiagnosticBehavior::Warning => diagnostics.push(diagnostic("warning", code, summary, path)),
        DiagnosticBehavior::Error => diagnostics.push(diagnostic("error", code, summary, path)),
    }
}

fn one_error(
    code: impl Into<String>,
    summary: impl Into<String>,
    path: Option<String>,
) -> MediaIngestError {
    MediaIngestError {
        diagnostics: vec![error(code, summary, path)],
    }
}

fn info(
    code: impl Into<String>,
    summary: impl Into<String>,
    path: Option<String>,
) -> MediaIngestDiagnostic {
    diagnostic("info", code, summary, path)
}

fn error(
    code: impl Into<String>,
    summary: impl Into<String>,
    path: Option<String>,
) -> MediaIngestDiagnostic {
    diagnostic("error", code, summary, path)
}

fn diagnostic(
    level: impl Into<String>,
    code: impl Into<String>,
    summary: impl Into<String>,
    path: Option<String>,
) -> MediaIngestDiagnostic {
    MediaIngestDiagnostic {
        level: level.into(),
        code: code.into(),
        summary: summary.into(),
        path,
    }
}

pub fn media_object_id(blake3_hex: &str) -> String {
    format!("obj:blake3:{blake3_hex}")
}

pub fn media_object_ref(blake3_hex: &str) -> String {
    format!("media://blake3/{blake3_hex}")
}

pub fn sort_media_objects(objects: &mut [MediaObject]) {
    objects.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
}

pub fn sort_media_bindings(bindings: &mut [MediaBinding]) {
    bindings.sort_by(|left, right| {
        left.export_filename
            .as_bytes()
            .cmp(right.export_filename.as_bytes())
            .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
    });
}

pub fn sort_media_references(references: &mut [MediaReference]) {
    references.sort_by(compare_media_references);
}

fn compare_media_references(left: &MediaReference, right: &MediaReference) -> Ordering {
    left.owner_kind
        .as_bytes()
        .cmp(right.owner_kind.as_bytes())
        .then_with(|| left.owner_id.as_bytes().cmp(right.owner_id.as_bytes()))
        .then_with(|| {
            left.location_kind
                .as_bytes()
                .cmp(right.location_kind.as_bytes())
        })
        .then_with(|| {
            left.location_name
                .as_bytes()
                .cmp(right.location_name.as_bytes())
        })
        .then_with(|| left.raw_ref.as_bytes().cmp(right.raw_ref.as_bytes()))
        .then_with(|| left.ref_kind.as_bytes().cmp(right.ref_kind.as_bytes()))
        .then_with(|| compare_media_reference_resolutions(&left.resolution, &right.resolution))
}

fn compare_media_reference_resolutions(
    left: &MediaReferenceResolution,
    right: &MediaReferenceResolution,
) -> Ordering {
    let (left_status, left_media_id, left_skip_reason) =
        media_reference_resolution_sort_parts(left);
    let (right_status, right_media_id, right_skip_reason) =
        media_reference_resolution_sort_parts(right);

    left_status
        .as_bytes()
        .cmp(right_status.as_bytes())
        .then_with(|| left_media_id.as_bytes().cmp(right_media_id.as_bytes()))
        .then_with(|| {
            left_skip_reason
                .as_bytes()
                .cmp(right_skip_reason.as_bytes())
        })
}

fn media_reference_resolution_sort_parts(
    resolution: &MediaReferenceResolution,
) -> (&'static str, &str, &str) {
    match resolution {
        MediaReferenceResolution::Resolved { media_id } => ("resolved", media_id.as_str(), ""),
        MediaReferenceResolution::Missing => ("missing", "", ""),
        MediaReferenceResolution::Skipped { skip_reason } => ("skipped", "", skip_reason.as_str()),
    }
}
