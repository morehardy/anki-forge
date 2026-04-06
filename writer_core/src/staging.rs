use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{AuthoringNotetype, NormalizedIr, NormalizedNotetype};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha1::Digest;

use crate::canonical_json::to_canonical_json;
use crate::model::{
    BuildContext, BuildDiagnosticItem, BuildDiagnostics, PackageBuildResult, WriterPolicy,
};
use crate::policy::{build_context_ref, policy_ref};

#[derive(Debug, Clone)]
pub struct BuildArtifactTarget {
    pub root_dir: PathBuf,
    pub stable_ref_prefix: String,
}

impl BuildArtifactTarget {
    pub fn new(root_dir: impl Into<PathBuf>, stable_ref_prefix: impl Into<String>) -> Self {
        Self {
            root_dir: root_dir.into(),
            stable_ref_prefix: stable_ref_prefix.into(),
        }
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.root_dir.join("staging")
    }

    pub fn staging_manifest_path(&self) -> PathBuf {
        self.staging_dir().join("manifest.json")
    }

    pub fn staging_ref(&self) -> String {
        format!(
            "{}/staging/manifest.json",
            self.stable_ref_prefix.trim_end_matches('/')
        )
    }
}

#[derive(Debug, Clone)]
pub struct StagingPackage {
    manifest: StagingManifest,
    diagnostics: Vec<BuildDiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedStaging {
    pub manifest_ref: String,
    pub manifest_path: PathBuf,
    pub artifact_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StagingManifest {
    kind: String,
    tool_contract_version: String,
    writer_policy_ref: String,
    build_context_ref: String,
    normalized_ir: NormalizedIr,
}

pub(crate) fn load_normalized_ir_from_staging_manifest(path: &Path) -> Result<NormalizedIr> {
    let manifest_json = fs::read_to_string(path)
        .with_context(|| format!("read staging manifest {}", path.display()))?;
    let manifest: StagingManifest = serde_json::from_str(&manifest_json)
        .with_context(|| format!("decode staging manifest {}", path.display()))?;
    Ok(manifest.normalized_ir)
}

impl StagingPackage {
    pub fn from_normalized(
        normalized_ir: &NormalizedIr,
        writer_policy: &WriterPolicy,
        build_context: &BuildContext,
    ) -> std::result::Result<Self, Vec<BuildDiagnosticItem>> {
        let diagnostics = validate_normalized_ir(normalized_ir, build_context);
        let (errors, warnings): (Vec<_>, Vec<_>) = diagnostics
            .into_iter()
            .partition(|item| item.level == "error");
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            manifest: StagingManifest {
                kind: "staging-package".into(),
                tool_contract_version: crate::tool_contract_version().into(),
                writer_policy_ref: policy_ref(&writer_policy.id, &writer_policy.version),
                build_context_ref: resolved_build_context_ref(build_context),
                normalized_ir: normalized_ir.clone(),
            },
            diagnostics: warnings,
        })
    }

    pub fn diagnostics(&self) -> &[BuildDiagnosticItem] {
        &self.diagnostics
    }

    pub fn materialize(&self, target: &BuildArtifactTarget) -> Result<MaterializedStaging> {
        let staging_dir = target.staging_dir();
        fs::create_dir_all(&staging_dir)
            .with_context(|| format!("create staging directory {}", staging_dir.display()))?;

        if !self.manifest.normalized_ir.media.is_empty() {
            let media_dir = staging_dir.join("media");
            fs::create_dir_all(&media_dir).with_context(|| {
                format!("create staging media directory {}", media_dir.display())
            })?;
            for media in &self.manifest.normalized_ir.media {
                let payload = base64::engine::general_purpose::STANDARD
                    .decode(media.data_base64.as_bytes())
                    .with_context(|| format!("decode media payload {}", media.filename))?;
                let media_path = media_dir.join(&media.filename);
                fs::write(&media_path, payload)
                    .with_context(|| format!("write staging media {}", media_path.display()))?;
            }
        }

        let manifest_json = to_canonical_json(&self.manifest)?;
        let manifest_path = target.staging_manifest_path();
        fs::write(&manifest_path, manifest_json.as_bytes())
            .with_context(|| format!("write staging manifest {}", manifest_path.display()))?;

        Ok(MaterializedStaging {
            manifest_ref: target.staging_ref(),
            manifest_path,
            artifact_fingerprint: fingerprint(&manifest_json),
        })
    }
}

pub(crate) fn invalid_result(
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    diagnostics: Vec<BuildDiagnosticItem>,
) -> PackageBuildResult {
    PackageBuildResult {
        kind: "package-build-result".into(),
        result_status: "invalid".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        writer_policy_ref: policy_ref(&writer_policy.id, &writer_policy.version),
        build_context_ref: resolved_build_context_ref(build_context),
        staging_ref: None,
        artifact_fingerprint: None,
        package_fingerprint: None,
        apkg_ref: None,
        diagnostics: BuildDiagnostics {
            kind: "build-diagnostics".into(),
            items: diagnostics,
        },
    }
}

pub(crate) fn success_result(
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    staging: MaterializedStaging,
    diagnostics: Vec<BuildDiagnosticItem>,
) -> PackageBuildResult {
    PackageBuildResult {
        kind: "package-build-result".into(),
        result_status: "success".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        writer_policy_ref: policy_ref(&writer_policy.id, &writer_policy.version),
        build_context_ref: resolved_build_context_ref(build_context),
        staging_ref: Some(staging.manifest_ref),
        artifact_fingerprint: Some(staging.artifact_fingerprint),
        package_fingerprint: None,
        apkg_ref: None,
        diagnostics: BuildDiagnostics {
            kind: "build-diagnostics".into(),
            items: diagnostics,
        },
    }
}

pub(crate) fn error_result(
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    code: &str,
    summary: impl Into<String>,
    stage: &str,
    operation: &str,
    path: Option<String>,
) -> PackageBuildResult {
    PackageBuildResult {
        kind: "package-build-result".into(),
        result_status: "error".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        writer_policy_ref: policy_ref(&writer_policy.id, &writer_policy.version),
        build_context_ref: resolved_build_context_ref(build_context),
        staging_ref: None,
        artifact_fingerprint: None,
        package_fingerprint: None,
        apkg_ref: None,
        diagnostics: BuildDiagnostics {
            kind: "build-diagnostics".into(),
            items: vec![BuildDiagnosticItem {
                level: "error".into(),
                code: code.into(),
                summary: summary.into(),
                domain: Some("staging".into()),
                path,
                target_selector: None,
                stage: Some(stage.into()),
                operation: Some(operation.into()),
            }],
        },
    }
}

fn validate_normalized_ir(
    normalized_ir: &NormalizedIr,
    build_context: &BuildContext,
) -> Vec<BuildDiagnosticItem> {
    let notetype_map: BTreeMap<_, _> = normalized_ir
        .notetypes
        .iter()
        .enumerate()
        .map(|(index, notetype)| (notetype.id.as_str(), (index, notetype)))
        .collect();

    let mut diagnostics = vec![];
    let mut seen_notetype_ids = BTreeMap::new();

    for (index, notetype) in normalized_ir.notetypes.iter().enumerate() {
        if let Some(previous) = seen_notetype_ids.insert(notetype.id.as_str(), index) {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.DUPLICATE_NOTETYPE_ID".into(),
                summary: format!("duplicate notetype id {}", notetype.id),
                domain: Some("notetypes".into()),
                path: Some(format!("notetypes[{index}].id")),
                target_selector: Some(format!("notetype[id='{}']", notetype.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.DUPLICATE_NOTETYPE_ID".into(),
                summary: format!("first seen at notetypes[{previous}]"),
                domain: Some("notetypes".into()),
                path: Some(format!("notetypes[{previous}].id")),
                target_selector: Some(format!("notetype[id='{}']", notetype.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
            continue;
        }

        diagnostics.extend(validate_stock_notetype_shape(index, notetype));

        if !matches!(
            notetype.kind.as_str(),
            "basic" | "cloze" | "image_occlusion"
        ) {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.UNSUPPORTED_NOTETYPE_KIND".into(),
                summary: format!("unsupported notetype kind {}", notetype.kind),
                domain: Some("notetypes".into()),
                path: Some(format!("notetypes[{index}].kind")),
                target_selector: Some(format!("notetype[id='{}']", notetype.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
        }
    }

    let media_filenames: BTreeSet<_> = normalized_ir
        .media
        .iter()
        .map(|media| media.filename.as_str())
        .collect();

    for (index, note) in normalized_ir.notes.iter().enumerate() {
        let Some((_, notetype)) = notetype_map.get(note.notetype_id.as_str()) else {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.UNKNOWN_NOTETYPE_ID".into(),
                summary: format!("unknown notetype id {}", note.notetype_id),
                domain: Some("notes".into()),
                path: Some(format!("notes[{index}].notetype_id")),
                target_selector: Some(format!("note[id='{}']", note.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
            continue;
        };

        let mut expected_fields = notetype.fields.clone();
        let mut actual_fields: Vec<_> = note.fields.keys().cloned().collect();
        expected_fields.sort();
        actual_fields.sort();
        if actual_fields != expected_fields {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.NOTE_FIELD_MISMATCH".into(),
                summary: format!(
                    "note fields {:?} do not match notetype fields {:?}",
                    actual_fields, expected_fields
                ),
                domain: Some("notes".into()),
                path: Some(format!("notes[{index}].fields")),
                target_selector: Some(format!("note[id='{}']", note.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
        }

        if build_context.media_resolution_mode == "inline-only" {
            for (field_name, field_value) in &note.fields {
                for media_ref in extract_media_references(field_value) {
                    if media_ref.starts_with("data:")
                        || media_filenames.contains(media_ref.as_str())
                    {
                        continue;
                    }

                    diagnostics.push(BuildDiagnosticItem {
                        level: if build_context.unresolved_asset_behavior == "warn" {
                            "warning".into()
                        } else {
                            "error".into()
                        },
                        code: "PHASE3.UNRESOLVED_MEDIA_REFERENCE".into(),
                        summary: format!(
                            "field {} references missing media {}",
                            field_name, media_ref
                        ),
                        domain: Some("notes".into()),
                        path: Some(format!(r#"notes[{index}].fields["{}"]"#, field_name)),
                        target_selector: Some(format!("note[id='{}']", note.id)),
                        stage: Some("validate".into()),
                        operation: Some("resolve-media".into()),
                    });
                }
            }
        }
    }

    diagnostics
}

fn validate_stock_notetype_shape(
    index: usize,
    notetype: &NormalizedNotetype,
) -> Vec<BuildDiagnosticItem> {
    let Ok(expected) = resolve_stock_notetype(&AuthoringNotetype {
        id: notetype.id.clone(),
        kind: notetype.kind.clone(),
        name: Some(notetype.name.clone()),
    }) else {
        return vec![];
    };

    let mut diagnostics = vec![];
    if notetype.fields != expected.fields {
        diagnostics.push(stock_shape_mismatch(
            index,
            notetype,
            "fields",
            format!(
                "notetype fields {:?} do not match source-grounded fields {:?}",
                notetype.fields, expected.fields
            ),
        ));
    }

    if notetype.templates.len() != expected.templates.len() {
        diagnostics.push(stock_shape_mismatch(
            index,
            notetype,
            "templates",
            format!(
                "notetype template count {} does not match source-grounded template count {}",
                notetype.templates.len(),
                expected.templates.len()
            ),
        ));
        return diagnostics;
    }

    for (template_index, (actual, expected)) in notetype
        .templates
        .iter()
        .zip(expected.templates.iter())
        .enumerate()
    {
        if actual.name != expected.name {
            diagnostics.push(stock_shape_mismatch(
                index,
                notetype,
                &format!("templates[{template_index}].name"),
                format!(
                    "template name {:?} does not match source-grounded name {:?}",
                    actual.name, expected.name
                ),
            ));
        }
        if actual.question_format != expected.question_format {
            diagnostics.push(stock_shape_mismatch(
                index,
                notetype,
                &format!("templates[{template_index}].question_format"),
                format!(
                    "template question_format {:?} does not match source-grounded question_format {:?}",
                    actual.question_format, expected.question_format
                ),
            ));
        }
        if actual.answer_format != expected.answer_format {
            diagnostics.push(stock_shape_mismatch(
                index,
                notetype,
                &format!("templates[{template_index}].answer_format"),
                format!(
                    "template answer_format {:?} does not match source-grounded answer_format {:?}",
                    actual.answer_format, expected.answer_format
                ),
            ));
        }
    }

    if notetype.css != expected.css {
        diagnostics.push(stock_shape_mismatch(
            index,
            notetype,
            "css",
            "notetype css does not match source-grounded css".into(),
        ));
    }

    diagnostics
}

fn stock_shape_mismatch(
    index: usize,
    notetype: &NormalizedNotetype,
    path_suffix: &str,
    summary: String,
) -> BuildDiagnosticItem {
    BuildDiagnosticItem {
        level: "error".into(),
        code: "PHASE3.STOCK_NOTETYPE_SHAPE_MISMATCH".into(),
        summary,
        domain: Some("notetypes".into()),
        path: Some(format!("notetypes[{index}].{path_suffix}")),
        target_selector: Some(format!("notetype[id='{}']", notetype.id)),
        stage: Some("validate".into()),
        operation: Some("normalize-lane".into()),
    }
}

fn extract_media_references(field: &str) -> Vec<String> {
    let mut refs = extract_sound_media_references(field);
    refs.extend(extract_html_media_references(field, "src"));
    refs.extend(extract_html_media_references(field, "data"));
    refs
}

fn extract_sound_media_references(field: &str) -> Vec<String> {
    let mut refs = vec![];
    let mut remaining = field;

    while let Some(start) = remaining.find("[sound:") {
        let after = &remaining[start + "[sound:".len()..];
        if let Some(end) = after.find(']') {
            refs.push(decode_html_entities(&after[..end]));
            remaining = &after[end + 1..];
        } else {
            break;
        }
    }

    refs
}

fn extract_html_media_references(field: &str, attribute: &str) -> Vec<String> {
    let mut refs = vec![];
    let marker = format!("{attribute}=");
    let mut remaining = field;

    while let Some(start) = remaining.find(&marker) {
        let after = &remaining[start + marker.len()..];
        let Some(first_char) = after.chars().next() else {
            break;
        };

        let (raw_ref, rest) = match first_char {
            '"' | '\'' => {
                let content = &after[first_char.len_utf8()..];
                if let Some(end) = content.find(first_char) {
                    (&content[..end], &content[end + first_char.len_utf8()..])
                } else {
                    break;
                }
            }
            _ => {
                let end = after
                    .find(|ch: char| ch.is_whitespace() || ch == '>')
                    .unwrap_or(after.len());
                (&after[..end], &after[end..])
            }
        };

        refs.push(decode_html_entities(raw_ref));
        remaining = rest;
    }

    refs
}

fn decode_html_entities(value: &str) -> String {
    if !value.contains('&') {
        return value.to_string();
    }

    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

fn resolved_build_context_ref(build_context: &BuildContext) -> String {
    build_context_ref(build_context).expect("build context ref should serialize")
}

fn fingerprint(canonical_json: &str) -> String {
    let digest = sha1::Sha1::digest(canonical_json.as_bytes());
    format!("artifact:{}", hex::encode(digest))
}
