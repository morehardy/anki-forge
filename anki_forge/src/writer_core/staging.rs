use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use crate::authoring_core::stock::resolve_stock_notetype;
use crate::authoring_core::{
    AuthoringNotetype, MediaReferenceResolution, NormalizedIr, NormalizedNotetype,
};
use anyhow::{Context, Result};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};
use sha1::Digest;

#[cfg(test)]
use crate::writer_core::canonical_json::to_canonical_json;
use crate::writer_core::card_plan::{has_malformed_cloze, plan_cards};
use crate::writer_core::deck_name::DeckRegistry;
use crate::writer_core::media_refs::extract_media_references;
use crate::writer_core::model::{
    BuildContext, BuildDiagnosticItem, BuildDiagnostics, PackageBuildResult, WriterPolicy,
};
use crate::writer_core::policy::{build_context_ref, policy_ref};

#[derive(Debug, Clone)]
pub struct BuildArtifactTarget {
    pub root_dir: PathBuf,
    pub stable_ref_prefix: String,
    pub media_store_dir: PathBuf,
}

impl BuildArtifactTarget {
    pub fn new(root_dir: impl Into<PathBuf>, stable_ref_prefix: impl Into<String>) -> Self {
        let root_dir = root_dir.into();
        Self {
            media_store_dir: root_dir.join(".anki-forge-media"),
            root_dir,
            stable_ref_prefix: stable_ref_prefix.into(),
        }
    }

    pub fn with_media_store_dir(mut self, media_store_dir: impl Into<PathBuf>) -> Self {
        self.media_store_dir = media_store_dir.into();
        self
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
    inner: StagingPackageData<NormalizedIr>,
}

pub(crate) type BorrowedStagingPackage<'a> = StagingPackageData<&'a NormalizedIr>;

#[derive(Debug, Clone)]
pub(crate) struct StagingPackageData<T> {
    manifest: StagingManifest<T>,
    diagnostics: Vec<BuildDiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedStaging {
    pub manifest_ref: String,
    pub manifest_path: PathBuf,
    pub artifact_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StagingManifest<T> {
    kind: String,
    tool_contract_version: String,
    writer_policy_ref: String,
    build_context_ref: String,
    normalized_ir: T,
    #[serde(default)]
    notetype_model_ids: Option<BTreeMap<String, i64>>,
    template_target_decks: Vec<ResolvedTemplateTargetDeck>,
}

// Keep the canonical wire shape while serializing one record at a time. The
// previous Value tree and final String duplicated all note fields at once.
struct CanonicalManifest<'a, T>(&'a StagingManifest<T>);

impl<T: Borrow<NormalizedIr>> Serialize for CanonicalManifest<'_, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let manifest = self.0;
        let mut map = serializer.serialize_map(Some(7))?;
        map.serialize_entry("build_context_ref", &manifest.build_context_ref)?;
        map.serialize_entry("kind", &manifest.kind)?;
        map.serialize_entry(
            "normalized_ir",
            &CanonicalNormalized(manifest.normalized_ir.borrow()),
        )?;
        map.serialize_entry("notetype_model_ids", &manifest.notetype_model_ids)?;
        map.serialize_entry(
            "template_target_decks",
            &CanonicalItems(&manifest.template_target_decks),
        )?;
        map.serialize_entry("tool_contract_version", &manifest.tool_contract_version)?;
        map.serialize_entry("writer_policy_ref", &manifest.writer_policy_ref)?;
        map.end()
    }
}

struct CanonicalNormalized<'a>(&'a NormalizedIr);

impl Serialize for CanonicalNormalized<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ir = self.0;
        let mut map = serializer.serialize_map(Some(9))?;
        map.serialize_entry("document_id", &ir.document_id)?;
        map.serialize_entry("kind", &ir.kind)?;
        map.serialize_entry("media_bindings", &CanonicalItems(&ir.media_bindings))?;
        map.serialize_entry("media_objects", &CanonicalItems(&ir.media_objects))?;
        map.serialize_entry("media_references", &CanonicalItems(&ir.media_references))?;
        map.serialize_entry("notes", &CanonicalNotes(&ir.notes))?;
        map.serialize_entry("notetypes", &CanonicalItems(&ir.notetypes))?;
        map.serialize_entry("resolved_identity", &ir.resolved_identity)?;
        map.serialize_entry("schema_version", &ir.schema_version)?;
        map.end()
    }
}

struct CanonicalItems<'a, T>(&'a [T]);

impl<T: Serialize> Serialize for CanonicalItems<'_, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            let mut value = serde_json::to_value(item).map_err(serde::ser::Error::custom)?;
            value.sort_all_objects();
            sequence.serialize_element(&value)?;
        }
        sequence.end()
    }
}

struct CanonicalNotes<'a>(&'a [crate::authoring_core::NormalizedNote]);
struct CanonicalNote<'a>(&'a crate::authoring_core::NormalizedNote);

impl Serialize for CanonicalNotes<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for note in self.0 {
            sequence.serialize_element(&CanonicalNote(note))?;
        }
        sequence.end()
    }
}

impl Serialize for CanonicalNote<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let note = self.0;
        let mut map = serializer.serialize_map(Some(5 + usize::from(note.mtime_secs.is_some())))?;
        map.serialize_entry("deck_name", &note.deck_name)?;
        // Normalized fields are a BTreeMap<String, String>, already canonical.
        map.serialize_entry("fields", &note.fields)?;
        map.serialize_entry("id", &note.id)?;
        if let Some(mtime) = note.mtime_secs {
            map.serialize_entry("mtime_secs", &mtime)?;
        }
        map.serialize_entry("notetype_id", &note.notetype_id)?;
        map.serialize_entry("tags", &note.tags)?;
        map.end()
    }
}

struct FingerprintingWriter<W> {
    writer: W,
    hash: sha1::Sha1,
}

impl<W: Write> Write for FingerprintingWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let written = self.writer.write(bytes)?;
        self.hash.update(&bytes[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResolvedTemplateTargetDeck {
    pub(crate) notetype_id: String,
    pub(crate) template_name: String,
    pub(crate) target_deck_name: String,
    pub(crate) resolved_target_deck_id: i64,
}

pub(crate) fn load_staging_manifest(
    path: &Path,
) -> Result<(NormalizedIr, Option<BTreeMap<String, i64>>)> {
    let manifest_json = fs::read_to_string(path)
        .with_context(|| format!("read staging manifest {}", path.display()))?;
    let manifest: StagingManifest<NormalizedIr> = serde_json::from_str(&manifest_json)
        .with_context(|| format!("decode staging manifest {}", path.display()))?;
    Ok((manifest.normalized_ir, manifest.notetype_model_ids))
}

pub(crate) fn staging_notetype_ids(
    normalized: &NormalizedIr,
    ids: Option<BTreeMap<String, i64>>,
) -> Result<BTreeMap<String, i64>> {
    let ids = ids.unwrap_or_else(|| {
        // Pre-0.4 staging manifests used positional IDs. Reading them must retain
        // the IDs of the corresponding old APKG, not infer the new derivation.
        normalized
            .notetypes
            .iter()
            .enumerate()
            .map(|(index, notetype)| (notetype.id.clone(), (index + 1) as i64))
            .collect()
    });
    crate::writer_core::identity::resolve_notetype_ids(normalized, Some(&ids))
}

impl StagingPackage {
    pub fn from_normalized(
        normalized_ir: &NormalizedIr,
        writer_policy: &WriterPolicy,
        build_context: &BuildContext,
    ) -> std::result::Result<Self, Vec<BuildDiagnosticItem>> {
        StagingPackageData::from_normalized_with_ids(
            normalized_ir.clone(),
            writer_policy,
            build_context,
            None,
        )
        .map(|inner| Self { inner })
    }

    pub fn diagnostics(&self) -> &[BuildDiagnosticItem] {
        self.inner.diagnostics()
    }

    pub fn materialize(&self, target: &BuildArtifactTarget) -> Result<MaterializedStaging> {
        self.inner.materialize(target)
    }
}

impl<T: Borrow<NormalizedIr> + Serialize> StagingPackageData<T> {
    pub(crate) fn from_normalized_with_ids(
        normalized_ir: T,
        writer_policy: &WriterPolicy,
        build_context: &BuildContext,
        selected_ids: Option<&BTreeMap<String, i64>>,
    ) -> std::result::Result<Self, Vec<BuildDiagnosticItem>> {
        let normalized = normalized_ir.borrow();
        let diagnostics = validate_normalized_ir(normalized, build_context);
        let (errors, warnings): (Vec<_>, Vec<_>) = diagnostics
            .into_iter()
            .partition(|item| item.level == "error");
        if !errors.is_empty() {
            return Err(errors);
        }

        let notetype_model_ids =
            crate::writer_core::identity::resolve_notetype_ids(normalized, selected_ids).map_err(
                |error| {
                    vec![BuildDiagnosticItem {
                        level: "error".into(),
                        code: if error
                            .to_string()
                            .starts_with("UPDATE.NOTETYPE_MODEL_ID_COLLISION")
                        {
                            "UPDATE.NOTETYPE_MODEL_ID_COLLISION"
                        } else {
                            "UPDATE.WRITER_NOTETYPE_ID_PLAN_MISMATCH"
                        }
                        .into(),
                        summary: error.to_string(),
                        domain: Some("identity".into()),
                        path: Some("notetypes".into()),
                        target_selector: None,
                        stage: Some("validate".into()),
                        operation: Some("validate_notetype_ids".into()),
                    }]
                },
            )?;

        let template_target_decks = resolve_template_target_decks(normalized);
        Ok(Self {
            manifest: StagingManifest {
                kind: "staging-package".into(),
                tool_contract_version: crate::writer_core::tool_contract_version().into(),
                writer_policy_ref: policy_ref(&writer_policy.id, &writer_policy.version),
                build_context_ref: resolved_build_context_ref(build_context),
                normalized_ir,
                notetype_model_ids: Some(notetype_model_ids),
                template_target_decks,
            },
            diagnostics: warnings,
        })
    }

    pub(crate) fn diagnostics(&self) -> &[BuildDiagnosticItem] {
        &self.diagnostics
    }

    pub(crate) fn notetype_ids(&self) -> &BTreeMap<String, i64> {
        self.manifest
            .notetype_model_ids
            .as_ref()
            .expect("new staging packages always include validated model IDs")
    }

    pub(crate) fn materialize(&self, target: &BuildArtifactTarget) -> Result<MaterializedStaging> {
        let staging_dir = target.staging_dir();
        fs::create_dir_all(&staging_dir)
            .with_context(|| format!("create staging directory {}", staging_dir.display()))?;

        let normalized = self.manifest.normalized_ir.borrow();
        if !normalized.media_bindings.is_empty() {
            let media_dir = staging_dir.join("media");
            fs::create_dir_all(&media_dir).with_context(|| {
                format!("create staging media directory {}", media_dir.display())
            })?;
            let objects_by_id = normalized
                .media_objects
                .iter()
                .map(|object| (object.id.as_str(), object))
                .collect::<BTreeMap<_, _>>();
            for binding in &normalized.media_bindings {
                let object = objects_by_id
                    .get(binding.object_id.as_str())
                    .with_context(|| {
                        format!(
                            "binding {} references missing object {}",
                            binding.id, binding.object_id
                        )
                    })?;
                let media_path = validated_media_output_path(&media_dir, &binding.export_filename)?;
                crate::writer_core::media::copy_verified_cas_object_to_path(
                    &target.media_store_dir,
                    object,
                    &media_path,
                )?;
            }
        }

        let manifest_path = target.staging_manifest_path();
        let artifact_fingerprint = (|| -> Result<String> {
            let file = fs::File::create(&manifest_path)?;
            let mut writer = BufWriter::new(FingerprintingWriter {
                writer: file,
                hash: sha1::Sha1::new(),
            });
            serde_json::to_writer(&mut writer, &CanonicalManifest(&self.manifest))?;
            writer.flush()?;
            let writer = writer.into_inner().map_err(|error| error.into_error())?;
            Ok(format!("artifact:{}", hex::encode(writer.hash.finalize())))
        })()
        .with_context(|| format!("write staging manifest {}", manifest_path.display()))?;

        Ok(MaterializedStaging {
            manifest_ref: target.staging_ref(),
            manifest_path,
            artifact_fingerprint,
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
        tool_contract_version: crate::writer_core::tool_contract_version().into(),
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
        tool_contract_version: crate::writer_core::tool_contract_version().into(),
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
    error_result_with_domain(
        writer_policy,
        build_context,
        ErrorResultDetails {
            code: code.into(),
            summary: summary.into(),
            domain: "staging".into(),
            stage: stage.into(),
            operation: operation.into(),
            path,
        },
    )
}

pub(crate) struct ErrorResultDetails {
    pub(crate) code: String,
    pub(crate) summary: String,
    pub(crate) domain: String,
    pub(crate) stage: String,
    pub(crate) operation: String,
    pub(crate) path: Option<String>,
}

pub(crate) fn error_result_with_domain(
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    details: ErrorResultDetails,
) -> PackageBuildResult {
    PackageBuildResult {
        kind: "package-build-result".into(),
        result_status: "error".into(),
        tool_contract_version: crate::writer_core::tool_contract_version().into(),
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
                code: details.code,
                summary: details.summary,
                domain: Some(details.domain),
                path: details.path,
                target_selector: None,
                stage: Some(details.stage),
                operation: Some(details.operation),
            }],
        },
    }
}

fn validate_media_invariants(normalized_ir: &NormalizedIr) -> Vec<BuildDiagnosticItem> {
    let mut diagnostics = Vec::new();
    let mut object_ids = BTreeSet::new();
    for (index, object) in normalized_ir.media_objects.iter().enumerate() {
        if !object_ids.insert(object.id.as_str()) {
            diagnostics.push(media_error(
                "MEDIA.DUPLICATE_MEDIA_ID",
                format!("duplicate media object id {}", object.id),
                format!("media_objects[{index}].id"),
            ));
        }
        if object.id != format!("obj:blake3:{}", object.blake3)
            || object.object_ref != format!("media://blake3/{}", object.blake3)
            || !is_lower_hex(&object.blake3, 64)
            || !is_lower_hex(&object.sha1, 40)
            || object.mime.trim().is_empty()
        {
            diagnostics.push(media_error(
                "MEDIA.INVALID_MEDIA_OBJECT_INVARIANT",
                format!(
                    "invalid object invariant {}: blake3 must be 64 lowercase hex, sha1 must be 40 lowercase hex, object id/ref must match blake3, mime must be nonempty",
                    object.id
                ),
                format!("media_objects[{index}]"),
            ));
        }
    }
    let object_id_set = normalized_ir
        .media_objects
        .iter()
        .map(|object| object.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut binding_ids = BTreeSet::new();
    let mut filenames = BTreeSet::new();
    for (index, binding) in normalized_ir.media_bindings.iter().enumerate() {
        if !binding_ids.insert(binding.id.as_str()) {
            diagnostics.push(media_error(
                "MEDIA.DUPLICATE_MEDIA_ID",
                format!("duplicate media binding id {}", binding.id),
                format!("media_bindings[{index}].id"),
            ));
        }
        if !filenames.insert(binding.export_filename.as_str()) {
            diagnostics.push(media_error(
                "MEDIA.DUPLICATE_EXPORT_FILENAME",
                format!("duplicate export filename {}", binding.export_filename),
                format!("media_bindings[{index}].export_filename"),
            ));
        }
        if !object_id_set.contains(binding.object_id.as_str()) {
            diagnostics.push(media_error(
                "MEDIA.MEDIA_OBJECT_MISSING",
                format!(
                    "binding {} references missing object {}",
                    binding.id, binding.object_id
                ),
                format!("media_bindings[{index}].object_id"),
            ));
        }
        if !is_valid_media_object_id(&binding.object_id) {
            diagnostics.push(media_error(
                "MEDIA.INVALID_MEDIA_BINDING_INVARIANT",
                format!(
                    "binding {} object_id must be obj:blake3:<64 lowercase hex>",
                    binding.id
                ),
                format!("media_bindings[{index}].object_id"),
            ));
        }
        if let Err(err) = validated_media_output_path(Path::new("media"), &binding.export_filename)
        {
            diagnostics.push(media_error(
                "MEDIA.UNSAFE_FILENAME",
                err.to_string(),
                format!("media_bindings[{index}].export_filename"),
            ));
        }
    }
    for (index, media_ref) in normalized_ir.media_references.iter().enumerate() {
        match &media_ref.resolution {
            MediaReferenceResolution::Resolved { media_id } => {
                if !binding_ids.contains(media_id.as_str()) {
                    diagnostics.push(media_error(
                        "MEDIA.MEDIA_BINDING_MISSING",
                        format!(
                            "media reference {} resolves to missing binding {}",
                            media_ref.raw_ref, media_id
                        ),
                        format!("media_references[{index}].media_id"),
                    ));
                }
            }
            MediaReferenceResolution::Missing => diagnostics.push(media_error(
                "MEDIA.MISSING_REFERENCE",
                format!(
                    "writer-ready input contains unresolved missing media reference {}",
                    media_ref.raw_ref
                ),
                format!("media_references[{index}]"),
            )),
            MediaReferenceResolution::Skipped { skip_reason } => {
                if is_unsafe_media_reference_skip_reason(skip_reason) {
                    diagnostics.push(media_error(
                        "MEDIA.UNSAFE_REFERENCE",
                        format!(
                            "writer-ready input contains unsafe skipped media reference {}: {}",
                            media_ref.raw_ref, skip_reason
                        ),
                        format!("media_references[{index}]"),
                    ));
                }
            }
        }
    }
    diagnostics
}

fn is_unsafe_media_reference_skip_reason(skip_reason: &str) -> bool {
    matches!(
        skip_reason,
        "decoded-dot-path"
            | "decoded-path-separator"
            | "helper-unsafe-character"
            | "invalid-percent-encoding"
    )
}

fn is_valid_media_object_id(object_id: &str) -> bool {
    let Some(hash) = object_id.strip_prefix("obj:blake3:") else {
        return false;
    };
    is_lower_hex(hash, 64)
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn media_error(code: &str, summary: String, path: String) -> BuildDiagnosticItem {
    BuildDiagnosticItem {
        level: "error".into(),
        code: code.into(),
        summary,
        domain: Some("media".into()),
        path: Some(path),
        target_selector: None,
        stage: Some("validate".into()),
        operation: Some("writer-invariant".into()),
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
    diagnostics.extend(validate_media_resolution_mode(build_context));
    diagnostics.extend(validate_media_invariants(normalized_ir));
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

        if notetype.original_stock_kind.is_some() {
            diagnostics.extend(validate_stock_notetype_shape(index, notetype));
        }

        if !matches!(notetype.kind.as_str(), "normal" | "cloze") {
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
        .media_bindings
        .iter()
        .map(|binding| binding.export_filename.as_str())
        .collect();

    for (index, note) in normalized_ir.notes.iter().enumerate() {
        if let Some(mtime_secs) = note.mtime_secs {
            if mtime_secs < 1 {
                diagnostics.push(BuildDiagnosticItem {
                    level: "error".into(),
                    code: "PHASE3.INVALID_NOTE_MTIME".into(),
                    summary: format!("note mtime_secs must be positive (>= 1), found {mtime_secs}"),
                    domain: Some("notes".into()),
                    path: Some(format!("notes[{index}].mtime_secs")),
                    target_selector: Some(format!("note[id='{}']", note.id)),
                    stage: Some("validate".into()),
                    operation: Some("normalize-lane".into()),
                });
            }
        }

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
        if has_malformed_cloze(note, notetype) {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PRODUCT.CLOZE_MARKER_MALFORMED".into(),
                summary: format!("cloze note '{}' contains malformed cloze markup", note.id),
                domain: Some("notes".into()),
                path: Some(format!("notes[{index}].fields")),
                target_selector: Some(format!("note[id='{}']", note.id)),
                stage: Some("validate".into()),
                operation: Some("plan-cards".into()),
            });
        } else if notetype.kind == "cloze" && plan_cards(note, notetype).is_empty() {
            diagnostics.push(BuildDiagnosticItem {
                level: "error".into(),
                code: "PRODUCT.CLOZE_MARKER_MISSING".into(),
                summary: format!(
                    "cloze note '{}' must contain at least one valid cloze marker",
                    note.id
                ),
                domain: Some("notes".into()),
                path: Some(format!("notes[{index}].fields")),
                target_selector: Some(format!("note[id='{}']", note.id)),
                stage: Some("validate".into()),
                operation: Some("plan-cards".into()),
            });
        }

        let mut expected_fields: Vec<_> = notetype
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect();
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

        match build_context.media_resolution_mode.as_str() {
            "legacy-inline-scan" => {
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
            "pre-resolved" => {}
            _ => {}
        }
    }

    diagnostics
}

fn validate_media_resolution_mode(build_context: &BuildContext) -> Vec<BuildDiagnosticItem> {
    match build_context.media_resolution_mode.as_str() {
        "pre-resolved" | "legacy-inline-scan" => vec![],
        unsupported => vec![BuildDiagnosticItem {
            level: "error".into(),
            code: "PHASE3.UNSUPPORTED_MEDIA_RESOLUTION_MODE".into(),
            summary: format!("unsupported media_resolution_mode {unsupported}"),
            domain: Some("media".into()),
            path: Some("build_context.media_resolution_mode".into()),
            target_selector: None,
            stage: Some("validate".into()),
            operation: Some("resolve-media".into()),
        }],
    }
}

fn validate_stock_notetype_shape(
    index: usize,
    notetype: &NormalizedNotetype,
) -> Vec<BuildDiagnosticItem> {
    let Ok(expected) = resolve_stock_notetype(&AuthoringNotetype {
        id: notetype.id.clone(),
        kind: notetype.kind.clone(),
        name: Some(notetype.name.clone()),
        original_stock_kind: notetype.original_stock_kind.clone(),
        original_id: notetype.original_id,
        fields: None,
        templates: None,
        css: None,
        field_metadata: vec![],
    }) else {
        return vec![];
    };

    let mut diagnostics = vec![];
    let actual_fields = notetype
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    let expected_fields = expected
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    if actual_fields != expected_fields {
        diagnostics.push(stock_shape_mismatch(
            index,
            notetype,
            "fields",
            format!(
                "notetype fields {:?} do not match source-grounded fields {:?}",
                actual_fields, expected_fields
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

fn resolved_build_context_ref(build_context: &BuildContext) -> String {
    build_context_ref(build_context).expect("build context ref should serialize")
}

pub(crate) fn resolve_deck_registry(normalized_ir: &NormalizedIr) -> DeckRegistry {
    let names = normalized_ir
        .notes
        .iter()
        .map(|note| note.deck_name.clone())
        .chain(normalized_ir.notetypes.iter().flat_map(|notetype| {
            notetype
                .templates
                .iter()
                .filter_map(|template| template.target_deck_name.clone())
        }));

    DeckRegistry::from_human_names(names)
}

pub(crate) fn resolve_template_target_decks(
    normalized_ir: &NormalizedIr,
) -> Vec<ResolvedTemplateTargetDeck> {
    let deck_registry = resolve_deck_registry(normalized_ir);
    let mut resolved = vec![];

    for notetype in &normalized_ir.notetypes {
        for template in &notetype.templates {
            let Some(target_deck_name) = template.target_deck_name.as_ref() else {
                continue;
            };
            let deck = deck_registry
                .deck_for_human_name(target_deck_name)
                .expect("template target deck should be registered");
            resolved.push(ResolvedTemplateTargetDeck {
                notetype_id: notetype.id.clone(),
                template_name: template.name.clone(),
                target_deck_name: deck.human_name(),
                resolved_target_deck_id: deck.id,
            });
        }
    }

    resolved
}

#[cfg(test)]
fn fingerprint(canonical_json: &str) -> String {
    let digest = sha1::Sha1::digest(canonical_json.as_bytes());
    format!("artifact:{}", hex::encode(digest))
}

pub(crate) fn validated_media_output_path(media_dir: &Path, filename: &str) -> Result<PathBuf> {
    anyhow::ensure!(!filename.is_empty(), "media filename must not be empty");
    anyhow::ensure!(
        !filename.contains(['/', '\\']),
        "media filename must be a bare filename without path separators: {}",
        filename
    );

    let mut components = Path::new(filename).components();
    let is_bare_filename = matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none()
        && !Path::new(filename).is_absolute();

    anyhow::ensure!(
        is_bare_filename,
        "media filename must be a bare filename without path traversal: {}",
        filename
    );

    Ok(media_dir.join(filename))
}

#[cfg(test)]
mod canonical_stream_tests {
    use super::*;

    #[test]
    fn streamed_manifest_matches_the_previous_canonical_bytes_and_hash() {
        let bundle = crate::runtime::load_embedded_bundle().unwrap();
        let inputs = ["basic", "cloze", "image-occlusion"].map(|name| {
            std::fs::read_to_string(
                bundle
                    .runtime
                    .bundle_root
                    .join(format!("fixtures/phase3/inputs/{name}-normalized-ir.json")),
            )
            .unwrap()
        });
        for input in inputs {
            for mtime in [None, Some(1), Some(i64::MAX)] {
                let mut ir: NormalizedIr = serde_json::from_str(&input).unwrap();
                for note in &mut ir.notes {
                    note.mtime_secs = mtime;
                    note.fields.insert("z\"中".into(), "<&>\n\t\\".repeat(300));
                    note.fields.insert("a".into(), String::new());
                    note.tags = vec!["中".into(), "with space".into(), "".into()];
                }
                let manifest = StagingManifest {
                    kind: "staging-package".into(),
                    tool_contract_version: "phase3-v1".into(),
                    writer_policy_ref: "policy".into(),
                    build_context_ref: "context".into(),
                    normalized_ir: ir,
                    notetype_model_ids: Some(BTreeMap::from([("z".into(), 9), ("a".into(), 1)])),
                    template_target_decks: vec![ResolvedTemplateTargetDeck {
                        notetype_id: "note type".into(),
                        template_name: "Reverse".into(),
                        target_deck_name: "Parent::子".into(),
                        resolved_target_deck_id: 3,
                    }],
                };
                let expected = to_canonical_json(&manifest).unwrap();
                let mut writer = FingerprintingWriter {
                    writer: Vec::new(),
                    hash: sha1::Sha1::new(),
                };
                serde_json::to_writer(&mut writer, &CanonicalManifest(&manifest)).unwrap();
                assert_eq!(writer.writer, expected.as_bytes());
                assert_eq!(
                    format!("artifact:{}", hex::encode(writer.hash.finalize())),
                    fingerprint(&expected)
                );
            }
        }
    }
}
