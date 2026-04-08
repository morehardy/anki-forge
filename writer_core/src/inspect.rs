use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use authoring_core::{
    NormalizedField, NormalizedIr, NormalizedNote, NormalizedNotetype,
    NormalizedTemplate,
};
use base64::Engine;
use prost::Message;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use sha1::Digest;
use zip::ZipArchive;
use zstd::stream::decode_all;

use crate::anki_proto::{
    decode_field_config, decode_notetype_config, decode_notetype_metadata, decode_template_config,
    NotetypeKind, OriginalStockKind,
};
use crate::canonical_json::to_canonical_json;
use crate::media_refs::extract_media_references;
use crate::model::{InspectObservations, InspectReport, PackageBuildResult};
use crate::staging::{BuildArtifactTarget, ResolvedTemplateTargetDeck};

const OBSERVATION_MODEL_VERSION: &str = "phase3-inspect-v1";
const DOMAIN_NOTETYPES: &str = "notetypes";
const DOMAIN_TEMPLATES: &str = "templates";
const DOMAIN_FIELDS: &str = "fields";
const DOMAIN_MEDIA: &str = "media";
const DOMAIN_REFERENCES: &str = "references";

#[derive(Clone, PartialEq, Message)]
struct PackageMetadata {
    #[prost(int32, tag = "1")]
    version: i32,
}

#[derive(Clone, PartialEq, Message)]
struct MediaEntries {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<ArchiveMediaEntry>,
}

#[derive(Clone, PartialEq, Message)]
struct ArchiveMediaEntry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(uint32, tag = "2")]
    size: u32,
    #[prost(bytes, tag = "3")]
    sha1: Vec<u8>,
    #[prost(string, optional, tag = "4")]
    legacy_zip_filename: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageVersion {
    Legacy1,
    Legacy2,
    Latest,
}

impl PackageVersion {
    fn expected_collection_filename(self) -> &'static str {
        match self {
            Self::Legacy1 => "collection.anki2",
            Self::Legacy2 => "collection.anki21",
            Self::Latest => "collection.anki21b",
        }
    }

    fn media_map_is_hashmap(self) -> bool {
        matches!(self, Self::Legacy1 | Self::Legacy2)
    }

    fn zstd_compressed(self) -> bool {
        matches!(self, Self::Latest)
    }
}

#[derive(Debug, Clone, Default)]
struct ReadLimitations {
    observation_status: String,
    missing_domains: BTreeSet<String>,
    degradation_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
struct ResolvedMedia {
    filename: String,
    size: usize,
    sha1_hex: String,
}

#[derive(Debug, Clone)]
struct CollectionData {
    notetypes: Vec<NormalizedNotetype>,
    notes: Vec<NormalizedNote>,
    template_target_decks: Vec<ResolvedTemplateTargetDeck>,
}

pub fn inspect_build_result(
    build_result: &PackageBuildResult,
    artifact_target: &BuildArtifactTarget,
) -> Result<InspectReport> {
    if let Some(staging_ref) = &build_result.staging_ref {
        let staging_path = artifact_path_from_ref(artifact_target, staging_ref);
        if staging_path.exists() {
            let mut report = inspect_staging(&staging_path)?;
            report.source_ref = staging_ref.clone();
            return Ok(report);
        }
    }

    if let Some(apkg_ref) = &build_result.apkg_ref {
        let apkg_path = artifact_path_from_ref(artifact_target, apkg_ref);
        if apkg_path.exists() {
            let mut report = inspect_apkg(&apkg_path)?;
            report.source_ref = apkg_ref.clone();
            return Ok(report);
        }
    }

    if let Some(staging_ref) = &build_result.staging_ref {
        let staging_path = artifact_path_from_ref(artifact_target, staging_ref);
        let mut report = inspect_staging(&staging_path)?;
        report.source_ref = staging_ref.clone();
        return Ok(report);
    }

    if let Some(apkg_ref) = &build_result.apkg_ref {
        let apkg_path = artifact_path_from_ref(artifact_target, apkg_ref);
        let mut report = inspect_apkg(&apkg_path)?;
        report.source_ref = apkg_ref.clone();
        return Ok(report);
    }

    anyhow::bail!("package build result does not reference staging or apkg artifacts");
}

pub fn inspect_staging(path: impl AsRef<Path>) -> Result<InspectReport> {
    let path = path.as_ref();
    let raw_manifest =
        fs::read(path).with_context(|| format!("read staging manifest {}", path.display()))?;
    let manifest: StagingManifest = serde_json::from_slice(&raw_manifest)
        .with_context(|| format!("decode staging manifest {}", path.display()))?;
    let media_root = path
        .parent()
        .map(|parent| parent.join("media"))
        .unwrap_or_else(|| PathBuf::from("media"));

    let (media, mut limitations) = resolve_staging_media(&manifest.normalized_ir, &media_root)?;
    let observations =
        build_observations(&manifest.normalized_ir, &media, &manifest.template_target_decks);
    limitations.observation_status = derive_status(limitations.missing_domains.is_empty(), true);

    Ok(build_report(
        "staging",
        path.display().to_string(),
        &raw_manifest,
        observations,
        limitations,
    ))
}

pub fn inspect_apkg(path: impl AsRef<Path>) -> Result<InspectReport> {
    let path = path.as_ref();
    let file = File::open(path).with_context(|| format!("open apkg {}", path.display()))?;
    let mut archive =
        ZipArchive::new(file).with_context(|| format!("open apkg archive {}", path.display()))?;

    let (version, mut limitations) = read_package_version(&mut archive)?;
    let media = match read_media_entries(&mut archive, version) {
        Ok(media) => media,
        Err(err) => {
            limitations.missing_domains.insert(DOMAIN_MEDIA.into());
            limitations
                .degradation_reasons
                .push(format!("media map unavailable: {err}"));
            vec![]
        }
    };

    let mut normalized_ir = NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: "0.1.0".into(),
        document_id: String::new(),
        resolved_identity: String::new(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };
    let mut has_core_data = false;
    let mut template_target_decks = vec![];

    if let Some(collection_bytes) = read_expected_collection_bytes(&mut archive, version)? {
        let collection = read_collection_data(&collection_bytes)?;
        normalized_ir.notetypes = collection.notetypes;
        normalized_ir.notes = collection.notes;
        template_target_decks = collection.template_target_decks;
        has_core_data = true;
    } else {
        limitations.missing_domains.insert(DOMAIN_NOTETYPES.into());
        limitations.missing_domains.insert(DOMAIN_TEMPLATES.into());
        limitations.missing_domains.insert(DOMAIN_FIELDS.into());
        limitations.missing_domains.insert(DOMAIN_REFERENCES.into());
        limitations
            .degradation_reasons
            .push("collection database is unavailable".into());
    }

    let observations = build_observations(&normalized_ir, &media, &template_target_decks);
    limitations.observation_status =
        derive_status(limitations.missing_domains.is_empty(), has_core_data);

    Ok(build_report(
        "apkg",
        path.display().to_string(),
        b"",
        observations,
        limitations,
    ))
}

fn build_report(
    source_kind: &str,
    source_ref: String,
    source_bytes: &[u8],
    observations: InspectObservations,
    limitations: ReadLimitations,
) -> InspectReport {
    let observation_status = limitations.observation_status;
    let missing_domains = limitations.missing_domains.into_iter().collect::<Vec<_>>();
    let degradation_reasons = limitations.degradation_reasons;
    let artifact_fingerprint = fingerprint_report(
        &observation_status,
        &missing_domains,
        &degradation_reasons,
        &observations,
        source_bytes,
    );

    InspectReport {
        kind: "inspect-report".into(),
        observation_model_version: OBSERVATION_MODEL_VERSION.into(),
        source_kind: source_kind.into(),
        source_ref,
        artifact_fingerprint,
        observation_status,
        missing_domains,
        degradation_reasons,
        observations,
    }
}

fn fingerprint_report(
    observation_status: &str,
    missing_domains: &[String],
    degradation_reasons: &[String],
    observations: &InspectObservations,
    source_bytes: &[u8],
) -> String {
    let payload = json!({
        "observation_status": observation_status,
        "missing_domains": missing_domains,
        "degradation_reasons": degradation_reasons,
        "observations": strip_evidence_refs(observations),
        "source_bytes": if source_bytes.is_empty() {
            Value::Null
        } else {
            json!(hex::encode(sha1::Sha1::digest(source_bytes)))
        }
    });
    let canonical = to_canonical_json(&payload).expect("canonical inspection payload");
    format!(
        "artifact:{}",
        hex::encode(sha1::Sha1::digest(canonical.as_bytes()))
    )
}

fn strip_evidence_refs(observations: &InspectObservations) -> Value {
    json!({
        "notetypes": observations.notetypes.iter().map(strip_value).collect::<Vec<_>>(),
        "templates": observations.templates.iter().map(strip_value).collect::<Vec<_>>(),
        "fields": observations.fields.iter().map(strip_value).collect::<Vec<_>>(),
        "media": observations.media.iter().map(strip_value).collect::<Vec<_>>(),
        "field_metadata": observations.field_metadata.iter().map(strip_value).collect::<Vec<_>>(),
        "browser_templates": observations.browser_templates.iter().map(strip_value).collect::<Vec<_>>(),
        "template_target_decks": observations.template_target_decks.iter().map(strip_value).collect::<Vec<_>>(),
        "metadata": observations.metadata.iter().map(strip_value).collect::<Vec<_>>(),
        "references": observations.references.iter().map(strip_value).collect::<Vec<_>>(),
    })
}

fn strip_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut map = map.clone();
            map.remove("evidence_refs");
            Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, strip_value(&value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(strip_value).collect()),
        other => other.clone(),
    }
}

fn build_observations(
    normalized_ir: &NormalizedIr,
    media: &[ResolvedMedia],
    template_target_decks: &[ResolvedTemplateTargetDeck],
) -> InspectObservations {
    let notetypes_by_id: BTreeMap<_, _> = normalized_ir
        .notetypes
        .iter()
        .map(|notetype| (notetype.id.as_str(), notetype))
        .collect();
    let media_by_filename: BTreeMap<_, _> = media
        .iter()
        .map(|media| (media.filename.as_str(), media))
        .collect();

    let mut notetype_entries = vec![];
    let mut template_entries = vec![];
    let mut field_entries = vec![];
    let mut field_metadata_entries = vec![];
    let mut browser_template_entries = vec![];
    let mut template_target_deck_entries = vec![];
    let mut note_entries = vec![];
    let mut card_entries = vec![];
    let mut media_reference_entries = vec![];

    for notetype in &normalized_ir.notetypes {
        let notetype_id = notetype.id.as_str();
        let notetype_kind = notetype.kind.as_str();
        let notetype_name = notetype.name.as_str();
        notetype_entries.push(json!({
            "selector": format!("notetype[id='{}']", notetype_id),
            "id": notetype_id,
            "kind": notetype_kind,
            "name": notetype_name,
            "field_count": notetype.fields.len(),
            "template_count": notetype.templates.len(),
            "css": notetype.css.as_str(),
            "evidence_refs": [format!("notetype:{}", notetype_id)],
        }));

        for field in &notetype.fields {
            let field_name = field.name.as_str();
            field_entries.push(json!({
                "selector": format!("notetype[id='{}']::field[{}]", notetype_id, field_name),
                "notetype_id": notetype_id,
                "name": field_name,
                "evidence_refs": [format!("field:{}:{}", notetype_id, field_name)],
            }));
        }

        for field_metadata in &notetype.field_metadata {
            let field_name = field_metadata.field_name.as_str();
            field_metadata_entries.push(json!({
                "selector": format!("notetype[id='{}']::field-metadata[{}]", notetype_id, field_name),
                "notetype_id": notetype_id,
                "field_name": field_name,
                "label": field_metadata.label,
                "role_hint": field_metadata.role_hint,
                "evidence_refs": [format!("field-metadata:{}:{}", notetype_id, field_name)],
            }));
        }

        for template in &notetype.templates {
            let template_name = template.name.as_str();
            template_entries.push(json!({
                "selector": format!("notetype[id='{}']::template[{}]", notetype_id, template_name),
                "notetype_id": notetype_id,
                "name": template_name,
                "question_format": template.question_format.as_str(),
                "answer_format": template.answer_format.as_str(),
                "evidence_refs": [format!("template:{}:{}", notetype_id, template_name)],
            }));

            if template.browser_question_format.is_some()
                || template.browser_answer_format.is_some()
                || template.browser_font_name.is_some()
                || template.browser_font_size.is_some()
            {
                browser_template_entries.push(json!({
                    "selector": format!("notetype[id='{}']::browser-template[{}]", notetype_id, template_name),
                    "notetype_id": notetype_id,
                    "template_name": template_name,
                    "browser_question_format": template.browser_question_format,
                    "browser_answer_format": template.browser_answer_format,
                    "browser_font_name": template.browser_font_name,
                    "browser_font_size": template.browser_font_size,
                    "evidence_refs": [format!("browser-template:{}:{}", notetype_id, template_name)],
                }));
            }
        }
    }

    for template_target_deck in template_target_decks {
        template_target_deck_entries.push(json!({
            "selector": format!(
                "notetype[id='{}']::template-target-deck[{}]",
                template_target_deck.notetype_id,
                template_target_deck.template_name
            ),
            "notetype_id": template_target_deck.notetype_id,
            "template_name": template_target_deck.template_name,
            "target_deck_name": template_target_deck.target_deck_name,
            "resolved_target_deck_id": template_target_deck.resolved_target_deck_id,
            "evidence_refs": [format!(
                "template-target-deck:{}:{}",
                template_target_deck.notetype_id,
                template_target_deck.template_name
            )],
        }));
    }

    for note in &normalized_ir.notes {
        let Some(notetype) = notetypes_by_id.get(note.notetype_id.as_str()) else {
            continue;
        };
        let note_id = note.id.as_str();
        let notetype_id = note.notetype_id.as_str();
        note_entries.push(json!({
            "selector": format!("note[id='{}']", note_id),
            "id": note_id,
            "notetype_id": notetype_id,
            "tags": &note.tags,
            "fields": &note.fields,
            "evidence_refs": [format!("note:{}", note_id)],
        }));

        for (ord, template) in notetype.templates.iter().enumerate() {
            let template_name = template.name.as_str();
            card_entries.push(json!({
                "selector": format!("card[note_id='{}'][ord={}]", note_id, ord),
                "note_id": note_id,
                "ord": ord,
                "template_name": template_name,
                "evidence_refs": [format!("card:{}:{}", note_id, ord)],
            }));
        }

        for (field_name, field_value) in &note.fields {
            for media_ref in extract_media_references(field_value) {
                if media_by_filename.contains_key(media_ref.as_str()) {
                    let field_name = field_name.as_str();
                    media_reference_entries.push(json!({
                        "selector": format!(
                            "media-ref[note_id='{}'][field='{}'][ref='{}']",
                            note_id, field_name, media_ref
                        ),
                        "note_id": note_id,
                        "field": field_name,
                        "reference": media_ref.as_str(),
                        "evidence_refs": [format!("media-ref:{}:{}:{}", note_id, field_name, media_ref)],
                    }));
                }
            }
        }
    }

    let metadata_entries = vec![json!({
        "selector": "counts",
        "notetype_count": normalized_ir.notetypes.len(),
        "template_count": template_entries.len(),
        "field_count": field_entries.len(),
        "note_count": note_entries.len(),
        "card_count": card_entries.len(),
        "media_count": media.len(),
        "evidence_refs": ["counts"],
    })];

    InspectObservations {
        notetypes: notetype_entries,
        templates: template_entries,
        fields: field_entries,
        media: media
            .iter()
            .map(|entry| {
                json!({
                    "selector": format!("media[filename='{}']", entry.filename),
                    "filename": entry.filename,
                    "size": entry.size,
                    "sha1": entry.sha1_hex,
                    "evidence_refs": [format!("media:{}", entry.filename)],
                })
            })
            .collect(),
        field_metadata: field_metadata_entries,
        browser_templates: browser_template_entries,
        template_target_decks: template_target_deck_entries,
        metadata: metadata_entries,
        references: note_entries
            .into_iter()
            .chain(card_entries)
            .chain(media_reference_entries)
            .collect(),
    }
}

fn resolve_staging_media(
    normalized_ir: &NormalizedIr,
    media_root: &Path,
) -> Result<(Vec<ResolvedMedia>, ReadLimitations)> {
    let mut limitations = ReadLimitations::default();
    let mut resolved = vec![];

    for media in &normalized_ir.media {
        let media_path = media_root.join(&media.filename);
        let payload = match fs::read(&media_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                limitations.missing_domains.insert(DOMAIN_MEDIA.into());
                limitations
                    .degradation_reasons
                    .push(format!("missing staged media {}: {err}", media.filename));
                continue;
            }
        };

        let expected = base64::engine::general_purpose::STANDARD
            .decode(media.data_base64.as_bytes())
            .with_context(|| format!("decode staged media payload {}", media.filename))?;
        if payload != expected {
            limitations.missing_domains.insert(DOMAIN_MEDIA.into());
            limitations.degradation_reasons.push(format!(
                "staged media payload mismatch for {}",
                media.filename
            ));
            continue;
        }

        resolved.push(ResolvedMedia {
            filename: media.filename.clone(),
            size: payload.len(),
            sha1_hex: hex::encode(sha1::Sha1::digest(&payload)),
        });
    }

    Ok((resolved, limitations))
}

fn read_package_version(
    archive: &mut ZipArchive<File>,
) -> Result<(PackageVersion, ReadLimitations)> {
    if let Some(meta_bytes) = read_zip_entry_bytes(archive, "meta")? {
        let meta = PackageMetadata::decode(meta_bytes.as_slice()).context("decode package meta")?;
        Ok((
            match meta.version {
                3 => PackageVersion::Latest,
                2 => PackageVersion::Legacy2,
                _ => PackageVersion::Legacy1,
            },
            ReadLimitations::default(),
        ))
    } else {
        Ok((
            infer_version_from_archive(archive),
            ReadLimitations::default(),
        ))
    }
}

fn infer_version_from_archive(archive: &mut ZipArchive<File>) -> PackageVersion {
    if archive.by_name("collection.anki21b").is_ok() {
        PackageVersion::Latest
    } else if archive.by_name("collection.anki21").is_ok() {
        PackageVersion::Legacy2
    } else {
        PackageVersion::Legacy1
    }
}

fn read_expected_collection_bytes(
    archive: &mut ZipArchive<File>,
    version: PackageVersion,
) -> Result<Option<Vec<u8>>> {
    let collection_name = version.expected_collection_filename();
    let Some(raw_bytes) = read_zip_entry_bytes(archive, collection_name)? else {
        return Ok(None);
    };
    if version.zstd_compressed() {
        Ok(Some(
            decode_all(raw_bytes.as_slice()).context("decode zstd collection")?,
        ))
    } else {
        Ok(Some(raw_bytes))
    }
}

fn read_media_entries(
    archive: &mut ZipArchive<File>,
    version: PackageVersion,
) -> Result<Vec<ResolvedMedia>> {
    let Some(raw_bytes) = read_zip_entry_bytes(archive, "media")? else {
        return Err(anyhow::anyhow!("media map missing"));
    };

    let decoded = if version.zstd_compressed() {
        decode_all(raw_bytes.as_slice()).context("decode zstd media map")?
    } else {
        raw_bytes
    };

    if version.media_map_is_hashmap() {
        let media_map: HashMap<String, String> =
            serde_json::from_slice(&decoded).context("decode legacy media map")?;
        let mut resolved = vec![];
        let mut entries: BTreeMap<usize, String> = BTreeMap::new();
        for (index, name) in media_map {
            let parsed_index = index
                .parse::<usize>()
                .with_context(|| format!("parse legacy media index {index}"))?;
            entries.insert(parsed_index, name);
        }
        for (index, name) in entries {
            let payload = read_zip_entry_bytes(archive, &index.to_string())?
                .ok_or_else(|| anyhow::anyhow!("missing legacy media payload {}", index))?;
            let payload = if version.zstd_compressed() {
                decode_all(payload.as_slice()).context("decode compressed media payload")?
            } else {
                payload
            };
            resolved.push(ResolvedMedia {
                filename: name,
                size: payload.len(),
                sha1_hex: hex::encode(sha1::Sha1::digest(&payload)),
            });
        }
        Ok(resolved)
    } else {
        let entries = MediaEntries::decode(decoded.as_slice()).context("decode media map")?;
        let mut resolved = vec![];
        for (index, entry) in entries.entries.into_iter().enumerate() {
            let payload = read_zip_entry_bytes(archive, &index.to_string())?
                .ok_or_else(|| anyhow::anyhow!("missing media payload {}", index))?;
            let payload = if version.zstd_compressed() {
                decode_all(payload.as_slice()).context("decode compressed media payload")?
            } else {
                payload
            };
            let sha1_hex = hex::encode(sha1::Sha1::digest(&payload));
            resolved.push(ResolvedMedia {
                filename: entry.name,
                size: payload.len(),
                sha1_hex,
            });
        }
        Ok(resolved)
    }
}

fn read_collection_data(bytes: &[u8]) -> Result<CollectionData> {
    with_temp_sqlite(bytes, |conn| {
        let mut deck_rows = conn.prepare("select id, name from decks order by id")?;
        let deck_values = deck_rows
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((id, name))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let deck_names_by_id: BTreeMap<i64, String> = deck_values.into_iter().collect();

        let mut notetype_rows =
            conn.prepare("select id, name, config from notetypes order by id")?;
        let raw_notetypes = notetype_rows
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let config: Vec<u8> = row.get(2)?;
                Ok((id, name, config))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut field_rows =
            conn.prepare("select ntid, ord, name, config from fields order by ntid, ord")?;
        let field_values = field_rows
            .query_map([], |row| {
                let ntid: i64 = row.get(0)?;
                let ord: i64 = row.get(1)?;
                let name: String = row.get(2)?;
                let config: Vec<u8> = row.get(3)?;
                let config = decode_field_config(&config).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        config.len(),
                        rusqlite::types::Type::Blob,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            err.to_string(),
                        )),
                    )
                })?;
                Ok((ntid, ord, name, config))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut fields_by_row_id =
            BTreeMap::<i64, Vec<(i64, String, crate::anki_proto::NoteFieldConfig)>>::new();
        for (ntid, ord, name, config) in field_values {
            fields_by_row_id
                .entry(ntid)
                .or_default()
                .push((ord, name, config));
        }

        let mut template_rows =
            conn.prepare("select ntid, ord, name, config from templates order by ntid, ord")?;
        let template_values = template_rows
            .query_map([], |row| {
                let ntid: i64 = row.get(0)?;
                let ord: i64 = row.get(1)?;
                let name: String = row.get(2)?;
                let config: Vec<u8> = row.get(3)?;
                let config = decode_template_config(&config).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        config.len(),
                        rusqlite::types::Type::Blob,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            err.to_string(),
                        )),
                    )
                })?;
                Ok((ntid, ord, name, config))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut templates_by_row_id =
            BTreeMap::<i64, Vec<(i64, String, crate::anki_proto::TemplateConfig)>>::new();
        for (ntid, ord, name, config) in template_values {
            templates_by_row_id
                .entry(ntid)
                .or_default()
                .push((ord, name, config));
        }

        let mut notetypes_by_row_id = BTreeMap::new();
        let mut notetype_values = vec![];
        let mut template_target_decks = vec![];
        for (row_id, name, config_bytes) in raw_notetypes {
            let config = decode_notetype_config(&config_bytes)?;
            let metadata = decode_notetype_metadata(&config.other)?;
            let field_metadata = metadata
                .as_ref()
                .map(|metadata| metadata.field_metadata.clone())
                .unwrap_or_default();
            let fields = fields_by_row_id
                .remove(&row_id)
                .unwrap_or_default()
                .into_iter()
                .map(|(ord, name, config)| NormalizedField {
                    name,
                    ord: Some(ord as u32),
                    config_id: config.id,
                    tag: config.tag,
                    prevent_deletion: config.prevent_deletion,
                })
                .collect::<Vec<_>>();
            let templates = templates_by_row_id
                .remove(&row_id)
                .unwrap_or_default()
                .into_iter()
                .map(|(ord, name, template)| {
                    let target_deck_name = if template.target_deck_id == 0 {
                        None
                    } else {
                        deck_names_by_id.get(&template.target_deck_id).cloned().or_else(|| {
                            Some(format!("deck-{}", template.target_deck_id))
                        })
                    };
                    if let Some(target_deck_name) = target_deck_name.as_ref() {
                        template_target_decks.push(ResolvedTemplateTargetDeck {
                            notetype_id: metadata
                                .as_ref()
                                .map(|metadata| metadata.anki_forge_notetype_id.clone())
                                .unwrap_or_else(|| format!("notetype-{row_id}")),
                            template_name: name.clone(),
                            target_deck_name: target_deck_name.clone(),
                            resolved_target_deck_id: template.target_deck_id,
                        });
                    }
                    NormalizedTemplate {
                        name,
                        ord: Some(ord as u32),
                        config_id: template.id,
                        question_format: template.q_format,
                        answer_format: template.a_format,
                        browser_question_format: if template.q_format_browser.is_empty() {
                            None
                        } else {
                            Some(template.q_format_browser)
                        },
                        browser_answer_format: if template.a_format_browser.is_empty() {
                            None
                        } else {
                            Some(template.a_format_browser)
                        },
                        target_deck_name,
                        browser_font_name: if template.browser_font_name.is_empty() {
                            None
                        } else {
                            Some(template.browser_font_name)
                        },
                        browser_font_size: if template.browser_font_size == 0 {
                            None
                        } else {
                            Some(template.browser_font_size)
                        },
                    }
                })
                .collect::<Vec<_>>();
            let notetype = NormalizedNotetype {
                id: metadata
                    .as_ref()
                    .map(|metadata| metadata.anki_forge_notetype_id.clone())
                    .unwrap_or_else(|| format!("notetype-{row_id}")),
                kind: normalized_notetype_kind(&config),
                name,
                original_stock_kind: original_stock_kind(&config),
                original_id: config.original_id,
                fields,
                templates,
                css: config.css,
                field_metadata,
            };
            notetypes_by_row_id.insert(row_id, notetype.clone());
            notetype_values.push(notetype);
        }

        let mut note_rows =
            conn.prepare("select id, guid, mid, tags, flds from notes order by id")?;
        let notes = note_rows
            .query_map([], |row| {
                let _id: i64 = row.get(0)?;
                let guid: String = row.get(1)?;
                let mid: i64 = row.get(2)?;
                let tags: String = row.get(3)?;
                let flds: String = row.get(4)?;
                let notetype = notetypes_by_row_id
                    .get(&mid)
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                let field_values: Vec<_> = if flds.is_empty() {
                    vec![]
                } else {
                    flds.split('\u{1f}').map(|s| s.to_string()).collect()
                };
                let mut fields = BTreeMap::new();
                for (field, value) in notetype.fields.iter().zip(field_values) {
                    fields.insert(field.name.clone(), value);
                }
                Ok(NormalizedNote {
                    id: guid,
                    notetype_id: notetype.id.clone(),
                    deck_name: "Default".into(),
                    fields,
                    tags: if tags.is_empty() {
                        vec![]
                    } else {
                        tags.split(' ').map(|tag| tag.to_string()).collect()
                    },
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(CollectionData {
            notetypes: notetype_values,
            notes,
            template_target_decks,
        })
    })
}

fn normalized_notetype_kind(config: &crate::anki_proto::NotetypeConfig) -> String {
    match OriginalStockKind::try_from(config.original_stock_kind).ok() {
        Some(OriginalStockKind::Basic) => "basic".into(),
        Some(OriginalStockKind::Cloze) => "cloze".into(),
        Some(OriginalStockKind::ImageOcclusion) => "image_occlusion".into(),
        _ => match NotetypeKind::try_from(config.kind).ok() {
            Some(NotetypeKind::Cloze) => "cloze".into(),
            _ => "normal".into(),
        },
    }
}

fn original_stock_kind(config: &crate::anki_proto::NotetypeConfig) -> Option<String> {
    match OriginalStockKind::try_from(config.original_stock_kind).ok() {
        Some(OriginalStockKind::Basic) => Some("basic".into()),
        Some(OriginalStockKind::Cloze) => Some("cloze".into()),
        Some(OriginalStockKind::ImageOcclusion) => Some("image_occlusion".into()),
        _ => None,
    }
}

fn with_temp_sqlite<T>(bytes: &[u8], f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
    let path = unique_temp_path("writer-core-inspect.sqlite");
    fs::write(&path, bytes).with_context(|| format!("write temp sqlite {}", path.display()))?;
    let result = (|| {
        let conn = Connection::open(&path)
            .with_context(|| format!("open temp sqlite {}", path.display()))?;
        f(&conn)
    })();
    let _ = fs::remove_file(&path);
    result
}

fn read_zip_entry_bytes(archive: &mut ZipArchive<File>, name: &str) -> Result<Option<Vec<u8>>> {
    match archive.by_name(name) {
        Ok(mut file) => {
            let mut buf = vec![];
            file.read_to_end(&mut buf)
                .with_context(|| format!("read zip entry {}", name))?;
            Ok(Some(buf))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn artifact_path_from_ref(target: &BuildArtifactTarget, reference: &str) -> PathBuf {
    let prefix = target.stable_ref_prefix.trim_end_matches('/');
    let trimmed = reference
        .strip_prefix(prefix)
        .unwrap_or(reference)
        .trim_start_matches('/');
    if trimmed.is_empty() {
        target.root_dir.clone()
    } else {
        target.root_dir.join(trimmed)
    }
}

fn derive_status(all_domains_present: bool, has_core_data: bool) -> String {
    if all_domains_present {
        "complete".into()
    } else if !has_core_data {
        "unavailable".into()
    } else {
        "degraded".into()
    }
}

fn unique_temp_path(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("anki-forge-{name}-{}-{nanos}", std::process::id()))
}

#[derive(Debug, Deserialize)]
struct StagingManifest {
    normalized_ir: NormalizedIr,
    #[serde(default)]
    template_target_decks: Vec<ResolvedTemplateTargetDeck>,
}
