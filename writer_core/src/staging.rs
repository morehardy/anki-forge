use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use authoring_core::NormalizedIr;
use serde::{Deserialize, Serialize};
use sha1::Digest;

use crate::canonical_json::to_canonical_json;
use crate::model::{
    BuildDiagnosticItem, BuildDiagnostics, BuildContext, PackageBuildResult, WriterPolicy,
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

impl StagingPackage {
    pub fn from_normalized(
        normalized_ir: &NormalizedIr,
        writer_policy: &WriterPolicy,
        build_context: &BuildContext,
    ) -> std::result::Result<Self, Vec<BuildDiagnosticItem>> {
        let diagnostics = validate_normalized_ir(normalized_ir);
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        Ok(Self {
            manifest: StagingManifest {
                kind: "staging-package".into(),
                tool_contract_version: crate::tool_contract_version().into(),
                writer_policy_ref: policy_ref(&writer_policy.id, &writer_policy.version),
                build_context_ref: build_context_ref(build_context).unwrap_or_default(),
                normalized_ir: normalized_ir.clone(),
            },
        })
    }

    pub fn materialize(&self, target: &BuildArtifactTarget) -> Result<MaterializedStaging> {
        let staging_dir = target.staging_dir();
        fs::create_dir_all(&staging_dir)
            .with_context(|| format!("create staging directory {}", staging_dir.display()))?;

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
        build_context_ref: build_context_ref(build_context).unwrap_or_default(),
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
) -> PackageBuildResult {
    PackageBuildResult {
        kind: "package-build-result".into(),
        result_status: "success".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        writer_policy_ref: policy_ref(&writer_policy.id, &writer_policy.version),
        build_context_ref: build_context_ref(build_context).unwrap_or_default(),
        staging_ref: Some(staging.manifest_ref),
        artifact_fingerprint: Some(staging.artifact_fingerprint),
        package_fingerprint: None,
        apkg_ref: None,
        diagnostics: BuildDiagnostics {
            kind: "build-diagnostics".into(),
            items: vec![],
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
        build_context_ref: build_context_ref(build_context).unwrap_or_default(),
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

fn validate_normalized_ir(normalized_ir: &NormalizedIr) -> Vec<BuildDiagnosticItem> {
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
                target_selector: Some(format!("notetype_id={}", notetype.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.DUPLICATE_NOTETYPE_ID".into(),
                summary: format!("first seen at notetypes[{previous}]"),
                domain: Some("notetypes".into()),
                path: Some(format!("notetypes[{previous}].id")),
                target_selector: Some(format!("notetype_id={}", notetype.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
            continue;
        }

        if !matches!(notetype.kind.as_str(), "basic" | "cloze") {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.UNSUPPORTED_NOTETYPE_KIND".into(),
                summary: format!("unsupported notetype kind {}", notetype.kind),
                domain: Some("notetypes".into()),
                path: Some(format!("notetypes[{index}].kind")),
                target_selector: Some(format!("notetype_id={}", notetype.id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
        }
    }

    for (index, note) in normalized_ir.notes.iter().enumerate() {
        let Some((_, notetype)) = notetype_map.get(note.notetype_id.as_str()) else {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PHASE3.UNKNOWN_NOTETYPE_ID".into(),
                summary: format!("unknown notetype id {}", note.notetype_id),
                domain: Some("notes".into()),
                path: Some(format!("notes[{index}].notetype_id")),
                target_selector: Some(format!("notetype_id={}", note.notetype_id)),
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
                target_selector: Some(format!("notetype_id={}", note.notetype_id)),
                stage: Some("validate".into()),
                operation: Some("normalize-lane".into()),
            });
        }
    }

    diagnostics
}

fn fingerprint(canonical_json: &str) -> String {
    let digest = sha1::Sha1::digest(canonical_json.as_bytes());
    format!("artifact:{}", hex::encode(digest))
}
