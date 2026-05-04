use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "resolution_status", rename_all = "snake_case")]
pub enum MediaReferenceResolution {
    Resolved { media_id: String },
    Missing,
    Skipped { skip_reason: String },
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
