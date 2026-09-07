use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::apkg_reader::ApkgReader;
use super::inspect_limits::{check, InspectError, InspectLimits};
use crate::authoring_core::{
    MediaReferenceResolution, NormalizedField, NormalizedGenerationRequirement, NormalizedIr,
    NormalizedNote, NormalizedNotetype, NormalizedTemplate,
};
use anyhow::{ensure, Context, Result};
use prost::Message;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use sha1::Digest;

use crate::writer_core::anki_proto::{
    decode_field_config, decode_notetype_config, decode_notetype_metadata, decode_template_config,
    CardRequirement, CardRequirementKind, NotetypeKind, OriginalStockKind,
};
use crate::writer_core::canonical_json::to_canonical_json;
use crate::writer_core::card_plan::plan_cards;
use crate::writer_core::deck_name::native_deck_name_to_human;
use crate::writer_core::model::{InspectObservations, InspectReport, PackageBuildResult};
use crate::writer_core::staging::{
    resolve_deck_registry, validated_media_output_path, BuildArtifactTarget,
    ResolvedTemplateTargetDeck,
};

const OBSERVATION_MODEL_VERSION: &str = "phase3-inspect-v2";
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
    #[prost(uint32, optional, tag = "255")]
    legacy_zip_filename: Option<u32>,
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
    binding_id: Option<String>,
    object_id: Option<String>,
    object_ref: Option<String>,
}

#[derive(Debug, Clone)]
struct CollectionData {
    notetypes: Vec<NormalizedNotetype>,
    notetype_model_ids: BTreeMap<String, i64>,
    notes: Vec<NormalizedNote>,
    note_identity_metadata: Vec<Value>,
    template_target_decks: Vec<ResolvedTemplateTargetDeck>,
    actual_card_decks: BTreeMap<(String, usize), String>,
    summary_counts: NoteCardCounts,
}

struct ApkgFacts {
    normalized_ir: NormalizedIr,
    media: Vec<ResolvedMedia>,
    template_target_decks: Vec<ResolvedTemplateTargetDeck>,
    actual_card_decks: BTreeMap<(String, usize), String>,
    note_identity_metadata: Vec<Value>,
    notetype_model_ids: BTreeMap<String, i64>,
    limitations: ReadLimitations,
    summary_counts: NoteCardCounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadProjection {
    Observations,
    Summary,
}

#[derive(Debug, Clone, Default)]
struct NoteCardCounts {
    notes: usize,
    cards: usize,
}

/// Facts consumed by a build without a comparison baseline. Unlike an
/// InspectReport, this does not promise observation JSON or a fingerprint.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ApkgInspectSummary {
    pub observation_status: String,
    pub notes: usize,
    pub cards: usize,
    pub notetypes: usize,
    pub templates: usize,
    pub fields: usize,
    pub media: usize,
}

pub fn inspect_build_result(
    build_result: &PackageBuildResult,
    artifact_target: &BuildArtifactTarget,
) -> Result<InspectReport> {
    if let Some(staging_ref) = &build_result.staging_ref {
        let staging_path = artifact_path_from_ref(artifact_target, staging_ref)?;
        if staging_path.exists() {
            let mut report = inspect_staging(&staging_path)?;
            report.source_ref = staging_ref.clone();
            return Ok(report);
        }
    }

    if let Some(apkg_ref) = &build_result.apkg_ref {
        let apkg_path = artifact_path_from_ref(artifact_target, apkg_ref)?;
        if apkg_path.exists() {
            let mut report = inspect_apkg(&apkg_path)?;
            report.source_ref = apkg_ref.clone();
            return Ok(report);
        }
    }

    if let Some(staging_ref) = &build_result.staging_ref {
        let staging_path = artifact_path_from_ref(artifact_target, staging_ref)?;
        let mut report = inspect_staging(&staging_path)?;
        report.source_ref = staging_ref.clone();
        return Ok(report);
    }

    if let Some(apkg_ref) = &build_result.apkg_ref {
        let apkg_path = artifact_path_from_ref(artifact_target, apkg_ref)?;
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
    let note_identity_metadata =
        build_note_identity_metadata_from_normalized_ir(&manifest.normalized_ir);
    let observations = build_observations(
        &manifest.normalized_ir,
        &media,
        &manifest.template_target_decks,
        None,
        &note_identity_metadata,
        &crate::writer_core::staging::staging_notetype_ids(
            &manifest.normalized_ir,
            manifest.notetype_model_ids,
        )?,
    );
    limitations.observation_status = derive_status(limitations.missing_domains.is_empty(), true);

    Ok(build_report(
        "staging",
        path.display().to_string(),
        &raw_manifest,
        observations,
        limitations,
    ))
}

pub fn inspect_apkg(path: impl AsRef<Path>) -> std::result::Result<InspectReport, InspectError> {
    inspect_apkg_with_limits(path, &InspectLimits::default())
}

pub fn inspect_apkg_with_limits(
    path: impl AsRef<Path>,
    limits: &InspectLimits,
) -> std::result::Result<InspectReport, InspectError> {
    inspect_apkg_inner(path.as_ref(), limits).map_err(InspectError::from_anyhow)
}

fn inspect_apkg_inner(path: &Path, limits: &InspectLimits) -> Result<InspectReport> {
    let facts = read_apkg_facts(path, limits, ReadProjection::Observations)?;
    let observations = build_observations(
        &facts.normalized_ir,
        &facts.media,
        &facts.template_target_decks,
        Some(&facts.actual_card_decks),
        &facts.note_identity_metadata,
        &facts.notetype_model_ids,
    );

    Ok(build_report(
        "apkg",
        path.display().to_string(),
        b"",
        observations,
        facts.limitations,
    ))
}

pub(crate) fn inspect_apkg_summary_with_limits(
    path: &Path,
    limits: &InspectLimits,
) -> std::result::Result<ApkgInspectSummary, InspectError> {
    let facts = read_apkg_facts(path, limits, ReadProjection::Summary)
        .map_err(InspectError::from_anyhow)?;
    Ok(ApkgInspectSummary {
        observation_status: facts.limitations.observation_status,
        notes: facts.summary_counts.notes,
        cards: facts.summary_counts.cards,
        notetypes: facts.normalized_ir.notetypes.len(),
        templates: facts
            .normalized_ir
            .notetypes
            .iter()
            .map(|notetype| notetype.templates.len())
            .sum(),
        fields: facts
            .normalized_ir
            .notetypes
            .iter()
            .map(|notetype| notetype.fields.len())
            .sum(),
        media: facts.media.len(),
    })
}

// Both projections perform the same bounded archive reads, SQLite column
// decoding and model validation. Only retained observations differ.
fn read_apkg_facts(
    path: &Path,
    limits: &InspectLimits,
    projection: ReadProjection,
) -> Result<ApkgFacts> {
    let mut archive = ApkgReader::open(path, limits)?;

    let (version, mut limitations) = read_package_version(&mut archive)?;
    let media = match read_media_entries(&mut archive, version) {
        Ok(media) => media,
        Err(err) => {
            // Resource errors are terminal, unlike ordinary missing-media degradation.
            let error = InspectError::from_anyhow(err);
            if let InspectError::LimitExceeded(limit) = error {
                return Err(limit.into());
            }
            limitations.missing_domains.insert(DOMAIN_MEDIA.into());
            limitations
                .degradation_reasons
                .push(format!("media map unavailable: {error}"));
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
        media_objects: vec![],
        media_bindings: vec![],
        media_references: vec![],
    };
    let mut has_core_data = false;
    let mut template_target_decks = vec![];
    let mut actual_card_decks = BTreeMap::new();
    let mut note_identity_metadata = vec![];
    let mut notetype_model_ids = BTreeMap::new();
    let mut summary_counts = NoteCardCounts::default();

    let collection_root = tempfile::tempdir().context("create inspect collection directory")?;
    let collection_path = collection_root.path().join("collection.sqlite");
    let mut collection_file = std::fs::File::create(&collection_path)?;
    if archive
        .copy(
            version.expected_collection_filename(),
            version.zstd_compressed(),
            "collection_bytes",
            limits.max_collection_bytes,
            &mut collection_file,
        )?
        .is_some()
    {
        drop(collection_file);
        let collection = read_collection_data(&collection_path, projection)?;
        normalized_ir.notetypes = collection.notetypes;
        notetype_model_ids = collection.notetype_model_ids;
        normalized_ir.notes = collection.notes;
        note_identity_metadata = collection.note_identity_metadata;
        template_target_decks = collection.template_target_decks;
        actual_card_decks = collection.actual_card_decks;
        summary_counts = collection.summary_counts;
        has_core_data = true;
    } else {
        limitations.missing_domains.insert(DOMAIN_NOTETYPES.into());
        limitations.missing_domains.insert(DOMAIN_TEMPLATES.into());
        limitations.missing_domains.insert(DOMAIN_FIELDS.into());
        limitations.missing_domains.insert(DOMAIN_REFERENCES.into());
        for domain in [
            "field_metadata",
            "browser_templates",
            "template_target_decks",
            "metadata",
        ] {
            limitations.missing_domains.insert(domain.into());
        }
        limitations
            .degradation_reasons
            .push("collection database is unavailable".into());
    }

    limitations.observation_status =
        derive_status(limitations.missing_domains.is_empty(), has_core_data);

    Ok(ApkgFacts {
        normalized_ir,
        media,
        template_target_decks,
        actual_card_decks,
        note_identity_metadata,
        notetype_model_ids,
        limitations,
        summary_counts,
    })
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
    Value::Object(
        observations
            .domains()
            .into_iter()
            .map(|(domain, values)| {
                (
                    domain.to_string(),
                    Value::Array(values.iter().map(strip_value).collect()),
                )
            })
            .collect(),
    )
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

fn observed_notes(
    normalized_ir: &NormalizedIr,
) -> impl Iterator<Item = (&NormalizedNote, &NormalizedNotetype)> {
    let notetypes_by_id: BTreeMap<_, _> = normalized_ir
        .notetypes
        .iter()
        .map(|notetype| (notetype.id.as_str(), notetype))
        .collect();
    normalized_ir.notes.iter().filter_map(move |note| {
        notetypes_by_id
            .get(note.notetype_id.as_str())
            .map(|notetype| (note, *notetype))
    })
}

fn actual_cards_for_note<'a>(
    cards: &'a BTreeMap<(String, usize), String>,
    note_id: &str,
) -> impl Iterator<Item = (&'a (String, usize), &'a String)> {
    cards.range((note_id.to_owned(), 0)..=(note_id.to_owned(), usize::MAX))
}

fn build_observations(
    normalized_ir: &NormalizedIr,
    media: &[ResolvedMedia],
    template_target_decks: &[ResolvedTemplateTargetDeck],
    actual_card_decks: Option<&BTreeMap<(String, usize), String>>,
    note_identity_metadata: &[Value],
    notetype_model_ids: &BTreeMap<String, i64>,
) -> InspectObservations {
    let staging_decks = actual_card_decks
        .is_none()
        .then(|| resolve_deck_registry(normalized_ir));
    let observed_deck_name = |name: &str| {
        staging_decks
            .as_ref()
            .and_then(|registry| registry.deck_for_human_name(name))
            .map(|deck| deck.human_name())
            .unwrap_or_else(|| name.to_string())
    };
    let media_by_binding_id: BTreeMap<_, _> = media
        .iter()
        .filter_map(|media| {
            media
                .binding_id
                .as_deref()
                .map(|binding_id| (binding_id, media))
        })
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
            "anki_model_id": notetype_model_ids.get(notetype_id),
            "kind": notetype_kind,
            "original_stock_kind": notetype.original_stock_kind,
            "name": notetype_name,
            "field_count": notetype.fields.len(),
            "template_count": notetype.templates.len(),
            "css": notetype.css.as_str(),
            "evidence_refs": [format!("notetype:{}", notetype_id)],
        }));

        for (field_index, field) in notetype.fields.iter().enumerate() {
            let field_name = field.name.as_str();
            let mut field_entry = json!({
                "selector": format!("notetype[id='{}']::field[{}]", notetype_id, field_name),
                "notetype_id": notetype_id,
                "name": field_name,
                "ord": field.ord.unwrap_or(field_index as u32),
                "config_id": field.config_id,
                "tag": field.tag,
                "evidence_refs": [format!("field:{}:{}", notetype_id, field_name)],
            });
            if field.sort {
                field_entry["sort"] = json!(true);
            }
            field_entries.push(field_entry);
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

        for (template_index, template) in notetype.templates.iter().enumerate() {
            let template_name = template.name.as_str();
            let mut template_entry = json!({
                "selector": format!("notetype[id='{}']::template[{}]", notetype_id, template_name),
                "notetype_id": notetype_id,
                "name": template_name,
                "ord": template.ord.unwrap_or(template_index as u32),
                "config_id": template.config_id,
                "question_format": template.question_format.as_str(),
                "answer_format": template.answer_format.as_str(),
                "evidence_refs": [format!("template:{}:{}", notetype_id, template_name)],
            });
            if let Some(requirement) = template.generation_requirement.as_ref() {
                template_entry["generation_requirement"] = json!(requirement);
            }
            template_entries.push(template_entry);

            // Anki stores absent browser overrides as empty strings or zero.
            // Match APKG inspection before deciding whether an entry exists.
            let browser_question_format = template
                .browser_question_format
                .as_deref()
                .filter(|value| !value.is_empty());
            let browser_answer_format = template
                .browser_answer_format
                .as_deref()
                .filter(|value| !value.is_empty());
            let browser_font_name = template
                .browser_font_name
                .as_deref()
                .filter(|value| !value.is_empty());
            let browser_font_size = template.browser_font_size.filter(|value| *value != 0);
            if browser_question_format.is_some()
                || browser_answer_format.is_some()
                || browser_font_name.is_some()
                || browser_font_size.is_some()
            {
                browser_template_entries.push(json!({
                    "selector": format!("notetype[id='{}']::browser-template[{}]", notetype_id, template_name),
                    "notetype_id": notetype_id,
                    "template_name": template_name,
                    "browser_question_format": browser_question_format,
                    "browser_answer_format": browser_answer_format,
                    "browser_font_name": browser_font_name,
                    "browser_font_size": browser_font_size,
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
            "target_deck_name": observed_deck_name(&template_target_deck.target_deck_name),
            "resolved_target_deck_id": template_target_deck.resolved_target_deck_id,
            "evidence_refs": [format!(
                "template-target-deck:{}:{}",
                template_target_deck.notetype_id,
                template_target_deck.template_name
            )],
        }));
    }

    for (note, notetype) in observed_notes(normalized_ir) {
        let note_id = note.id.as_str();
        let notetype_id = note.notetype_id.as_str();
        note_entries.push(json!({
            "selector": format!("note[id='{}']", note_id),
            "id": note_id,
            "notetype_id": notetype_id,
            "deck_name": observed_deck_name(&note.deck_name),
            "tags": &note.tags,
            "fields": &note.fields,
            "revision": super::note_revision::NoteRevision::from_note(note),
            "evidence_refs": [format!("note:{}", note_id)],
        }));

        if let Some(actual_card_decks) = actual_card_decks {
            for ((actual_note_id, actual_ord), deck_name) in
                actual_cards_for_note(actual_card_decks, note_id)
            {
                let card_ord = *actual_ord as u32;
                let template_name = template_for_card_ord(notetype, card_ord)
                    .map(|template| template.name.as_str())
                    .unwrap_or("<missing template>");
                card_entries.push(json!({
                    "selector": format!("card[note_id='{}'][ord={}]", actual_note_id, card_ord),
                    "note_id": actual_note_id,
                    "ord": card_ord,
                    "template_name": template_name,
                    "deck_name": deck_name,
                    "evidence_refs": [format!("card:{}:{}", actual_note_id, card_ord)],
                }));
            }
        } else {
            for planned_card in plan_cards(note, notetype) {
                let template = &notetype.templates[planned_card.template_index];
                let template_name = template.name.as_str();
                let card_ord = planned_card.card_ord;
                let card_deck_name = template
                    .target_deck_name
                    .as_deref()
                    .unwrap_or(note.deck_name.as_str());
                card_entries.push(json!({
                    "selector": format!("card[note_id='{}'][ord={}]", note_id, card_ord),
                    "note_id": note_id,
                    "ord": card_ord,
                    "template_name": template_name,
                    "deck_name": observed_deck_name(card_deck_name),
                    "evidence_refs": [format!("card:{}:{}", note_id, card_ord)],
                }));
            }
        }
    }

    for media_ref in &normalized_ir.media_references {
        let MediaReferenceResolution::Resolved { media_id } = &media_ref.resolution else {
            continue;
        };
        let resolved_media = media_by_binding_id.get(media_id.as_str());
        let mut entry = json!({
            "selector": media_ref_selector(
                &media_ref.owner_kind,
                &media_ref.owner_id,
                &media_ref.location_kind,
                &media_ref.location_name,
                &media_ref.raw_ref,
            ),
            "owner_kind": media_ref.owner_kind.as_str(),
            "owner_id": media_ref.owner_id.as_str(),
            "location_kind": media_ref.location_kind.as_str(),
            "location_name": media_ref.location_name.as_str(),
            "reference": media_ref.raw_ref.as_str(),
            "ref_kind": media_ref.ref_kind.as_str(),
            "media_id": media_id.as_str(),
            "evidence_refs": [format!(
                "media-ref:{}:{}:{}:{}:{}",
                evidence_component(&media_ref.owner_kind),
                evidence_component(&media_ref.owner_id),
                evidence_component(&media_ref.location_kind),
                evidence_component(&media_ref.location_name),
                evidence_component(&media_ref.raw_ref)
            )],
        });
        if let Some(resolved_media) = resolved_media {
            entry["filename"] = json!(resolved_media.filename.as_str());
        }
        media_reference_entries.push(entry);
    }

    let mut metadata_entries = vec![json!({
        "selector": "counts",
        "notetype_count": normalized_ir.notetypes.len(),
        "template_count": template_entries.len(),
        "field_count": field_entries.len(),
        "note_count": note_entries.len(),
        "card_count": card_entries.len(),
        "media_count": media.len(),
        "evidence_refs": ["counts"],
    })];
    metadata_entries.extend(note_identity_metadata.iter().cloned());

    InspectObservations {
        notetypes: notetype_entries,
        templates: template_entries,
        fields: field_entries,
        media: media
            .iter()
            .map(|entry| {
                let mut value = json!({
                    "selector": format!("media[filename='{}']", entry.filename),
                    "filename": entry.filename,
                    "size": entry.size,
                    "sha1": entry.sha1_hex,
                    "evidence_refs": [format!("media:{}", entry.filename)],
                });
                if let Some(binding_id) = &entry.binding_id {
                    value["binding_id"] = json!(binding_id);
                }
                if let Some(object_id) = &entry.object_id {
                    value["object_id"] = json!(object_id);
                }
                if let Some(object_ref) = &entry.object_ref {
                    value["object_ref"] = json!(object_ref);
                }
                value
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

fn template_for_card_ord(
    notetype: &NormalizedNotetype,
    card_ord: u32,
) -> Option<&NormalizedTemplate> {
    if notetype.kind == "cloze" {
        return notetype.templates.first();
    }
    notetype
        .templates
        .iter()
        .enumerate()
        .find(|(index, template)| template.ord.unwrap_or(*index as u32) == card_ord)
        .map(|(_, template)| template)
}

fn build_note_identity_metadata_from_normalized_ir(normalized_ir: &NormalizedIr) -> Vec<Value> {
    normalized_ir
        .notes
        .iter()
        .map(|note| {
            let guid = &note.id;
            json!({
                "selector": format!("note[guid='{}']::anki_forge_identity", guid),
                "schema_version": "identity-note-v1",
                "stable_id": note.id,
                "recipe_id": "product.explicit-or-normalized.v1",
                "canonical_payload_hash": None::<String>,
                "current_guid_candidate": note.id,
                "selected_anki_guid": note.id,
                "guid_derivation_version": "guid.raw-stable-id.v1",
                "guid_source": "current_derivation",
                "recovery_method": "current_resolution",
                "provenance": "ExplicitStableId",
                "used_override": false,
                "evidence_refs": [format!("note-data:{}", guid)],
            })
        })
        .collect()
}

fn media_ref_selector(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    raw_ref: &str,
) -> String {
    format!(
        "media-ref[owner_kind={}][owner_id={}][location_kind={}][location_name={}][ref={}]",
        selector_value(owner_kind),
        selector_value(owner_id),
        selector_value(location_kind),
        selector_value(location_name),
        selector_value(raw_ref),
    )
}

fn selector_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('\'');
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '"' => escaped.push_str("\\\""),
            '[' => escaped.push_str("\\["),
            ']' => escaped.push_str("\\]"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                escaped.push_str("\\u{");
                escaped.push_str(&format!("{:x}", ch as u32));
                escaped.push('}');
            }
            _ => escaped.push(ch),
        }
    }
    escaped.push('\'');
    escaped
}

fn evidence_component(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                escaped.push_str("\\u{");
                escaped.push_str(&format!("{:x}", ch as u32));
                escaped.push('}');
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn resolve_staging_media(
    normalized_ir: &NormalizedIr,
    media_root: &Path,
) -> Result<(Vec<ResolvedMedia>, ReadLimitations)> {
    let mut limitations = ReadLimitations::default();
    let mut resolved = vec![];
    let objects_by_id = normalized_ir
        .media_objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();

    for binding in &normalized_ir.media_bindings {
        let Some(object) = objects_by_id.get(binding.object_id.as_str()) else {
            limitations.missing_domains.insert(DOMAIN_MEDIA.into());
            limitations.degradation_reasons.push(format!(
                "staging media binding {} references missing object {}",
                binding.id, binding.object_id
            ));
            continue;
        };

        let media_path = match validated_media_output_path(media_root, &binding.export_filename) {
            Ok(path) => path,
            Err(err) => {
                limitations.missing_domains.insert(DOMAIN_MEDIA.into());
                limitations.degradation_reasons.push(format!(
                    "invalid staged media filename {}: {err}",
                    binding.export_filename
                ));
                continue;
            }
        };
        let payload = match fs::read(&media_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                limitations.missing_domains.insert(DOMAIN_MEDIA.into());
                limitations.degradation_reasons.push(format!(
                    "missing staged media {}: {err}",
                    binding.export_filename
                ));
                continue;
            }
        };
        let sha1_hex = hex::encode(sha1::Sha1::digest(&payload));

        if payload.len() as u64 != object.size_bytes {
            limitations.missing_domains.insert(DOMAIN_MEDIA.into());
            limitations.degradation_reasons.push(format!(
                "staged media {} size mismatch for object {}: manifest {}, observed {}",
                binding.export_filename,
                object.id,
                object.size_bytes,
                payload.len()
            ));
        }

        if !sha1_hex.eq_ignore_ascii_case(&object.sha1) {
            limitations.missing_domains.insert(DOMAIN_MEDIA.into());
            limitations.degradation_reasons.push(format!(
                "staged media {} sha1 mismatch for object {}: manifest {}, observed {}",
                binding.export_filename, object.id, object.sha1, sha1_hex
            ));
        }

        resolved.push(ResolvedMedia {
            filename: binding.export_filename.clone(),
            size: payload.len(),
            sha1_hex,
            binding_id: Some(binding.id.clone()),
            object_id: Some(object.id.clone()),
            object_ref: Some(object.object_ref.clone()),
        });
    }

    Ok((resolved, limitations))
}

fn read_package_version(archive: &mut ApkgReader<'_>) -> Result<(PackageVersion, ReadLimitations)> {
    if let Some(meta_bytes) =
        archive.bytes("meta", false, "meta_bytes", archive.limits.max_meta_bytes)?
    {
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

fn infer_version_from_archive(archive: &ApkgReader<'_>) -> PackageVersion {
    if archive.contains("collection.anki21b") {
        PackageVersion::Latest
    } else if archive.contains("collection.anki21") {
        PackageVersion::Legacy2
    } else {
        PackageVersion::Legacy1
    }
}

fn read_media_entries(
    archive: &mut ApkgReader<'_>,
    version: PackageVersion,
) -> Result<Vec<ResolvedMedia>> {
    let decoded = archive
        .bytes(
            "media",
            version.zstd_compressed(),
            "media_map_bytes",
            archive.limits.max_media_map_bytes,
        )?
        .context("media map missing")?;
    check_media_map_count(&decoded, version, archive.limits.max_entries)?;
    let entries: Vec<(usize, String)> = if version.media_map_is_hashmap() {
        let media_map: HashMap<String, String> =
            serde_json::from_slice(&decoded).context("decode legacy media map")?;
        let mut entries = BTreeMap::new();
        for (index, name) in media_map {
            let index = index.parse::<usize>().context("parse legacy media index")?;
            ensure!(
                entries.insert(index, name).is_none(),
                "duplicate media index"
            );
        }
        entries.into_iter().collect()
    } else {
        let entries = MediaEntries::decode(decoded.as_slice()).context("decode media map")?;
        entries
            .entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| (index, entry.name))
            .collect()
    };
    let mut resolved = Vec::new();
    for (index, name) in entries {
        let mut hash = MediaHash(sha1::Sha1::new());
        let size = archive
            .copy(
                &index.to_string(),
                version.zstd_compressed(),
                "media_bytes",
                archive.limits.max_media_bytes,
                &mut hash,
            )?
            .with_context(|| format!("missing media payload {index}"))?;
        resolved.push(ResolvedMedia {
            filename: name,
            size: usize::try_from(size).context("media size exceeds address space")?,
            sha1_hex: hex::encode(hash.0.finalize()),
            binding_id: None,
            object_id: None,
            object_ref: None,
        });
    }
    Ok(resolved)
}

// Count entries without allocating their strings/messages. In particular, an
// attacker can encode millions of empty protobuf messages in a small byte map.
fn check_media_map_count(bytes: &[u8], version: PackageVersion, limit: u64) -> Result<()> {
    if version.media_map_is_hashmap() {
        use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
        use serde::Deserializer;
        struct Counter<'a> {
            count: &'a std::cell::Cell<u64>,
            limit: u64,
        }
        impl<'de> Visitor<'de> for Counter<'_> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a media map")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
                while map.next_key::<IgnoredAny>()?.is_some() {
                    self.count.set(self.count.get() + 1);
                    if self.count.get() > self.limit {
                        return Err(A::Error::custom("media entry limit exceeded"));
                    }
                    map.next_value::<IgnoredAny>()?;
                }
                Ok(())
            }
        }
        let count = std::cell::Cell::new(0);
        let mut decoder = serde_json::Deserializer::from_slice(bytes);
        let result = decoder.deserialize_map(Counter {
            count: &count,
            limit,
        });
        // Keep a typed limit error across serde's string-only custom error API.
        check("media_entries", Some("media"), limit, count.get())?;
        result.context("decode legacy media map")?;
        decoder.end()?;
    } else {
        use prost::encoding::{decode_key, skip_field, DecodeContext};
        let mut remaining = bytes;
        let mut count = 0;
        while !remaining.is_empty() {
            let (tag, wire) = decode_key(&mut remaining)?;
            if tag == 1 {
                count += 1;
                check("media_entries", Some("media"), limit, count)?;
            }
            skip_field(wire, tag, &mut remaining, DecodeContext::default())?;
        }
    }
    Ok(())
}

struct MediaHash(sha1::Sha1);

impl Write for MediaHash {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn read_collection_data(path: &Path, projection: ReadProjection) -> Result<CollectionData> {
    with_readonly_sqlite(path, |conn| {
        let mut deck_rows = conn.prepare("select id, name from decks order by id")?;
        let deck_values = deck_rows
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let native_name: String = row.get(1)?;
                Ok((id, native_deck_name_to_human(&native_name)))
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

        let mut fields_by_row_id = BTreeMap::<
            i64,
            Vec<(i64, String, crate::writer_core::anki_proto::NoteFieldConfig)>,
        >::new();
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

        let mut templates_by_row_id = BTreeMap::<
            i64,
            Vec<(i64, String, crate::writer_core::anki_proto::TemplateConfig)>,
        >::new();
        for (ntid, ord, name, config) in template_values {
            templates_by_row_id
                .entry(ntid)
                .or_default()
                .push((ord, name, config));
        }

        let mut notetypes_by_row_id = BTreeMap::new();
        let mut notetype_model_ids = BTreeMap::new();
        let mut notetype_values = vec![];
        let mut template_target_decks = vec![];
        for (row_id, name, config_bytes) in raw_notetypes {
            let config = decode_notetype_config(&config_bytes)?;
            let metadata = decode_notetype_metadata(&config.other)?;
            let field_metadata = metadata
                .as_ref()
                .map(|metadata| metadata.field_metadata.clone())
                .unwrap_or_default();
            let has_exact_forge_semantics = metadata
                .as_ref()
                .is_some_and(|metadata| !metadata.field_sort.is_empty());
            let field_sort = metadata
                .as_ref()
                .map(|metadata| &metadata.field_sort)
                .filter(|field_sort| !field_sort.is_empty());
            let fields = fields_by_row_id
                .remove(&row_id)
                .unwrap_or_default()
                .into_iter()
                .enumerate()
                .map(|(field_index, (ord, name, field_config))| {
                    let sort = field_sort
                        .and_then(|field_sort| field_sort.get(name.as_str()).copied())
                        .unwrap_or(field_index == config.sort_field_idx as usize);
                    NormalizedField {
                        name,
                        ord: Some(ord as u32),
                        config_id: field_config.id,
                        tag: field_config.tag,
                        prevent_deletion: field_config.prevent_deletion,
                        sort,
                    }
                })
                .collect::<Vec<_>>();
            let field_names_by_ord = fields
                .iter()
                .enumerate()
                .map(|(field_index, field)| {
                    (field.ord.unwrap_or(field_index as u32), field.name.clone())
                })
                .collect::<BTreeMap<_, _>>();
            let requirements_by_ord = config
                .reqs
                .iter()
                .filter_map(|requirement| {
                    generation_requirement_from_card_requirement(requirement, &field_names_by_ord)
                        .map(|normalized| (requirement.card_ord, normalized))
                })
                .collect::<BTreeMap<_, _>>();
            let metadata_requirements = metadata
                .as_ref()
                .map(|metadata| &metadata.template_generation_requirements)
                .filter(|requirements| !requirements.is_empty());
            let templates = templates_by_row_id
                .remove(&row_id)
                .unwrap_or_default()
                .into_iter()
                .map(|(ord, name, template)| {
                    let target_deck_name = if template.target_deck_id == 0 {
                        None
                    } else {
                        deck_names_by_id
                            .get(&template.target_deck_id)
                            .cloned()
                            .or_else(|| Some(format!("deck-{}", template.target_deck_id)))
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
                    let generation_requirement = if has_exact_forge_semantics {
                        metadata_requirements
                            .and_then(|requirements| requirements.get(name.as_str()).cloned())
                    } else {
                        metadata_requirements
                            .and_then(|requirements| requirements.get(name.as_str()).cloned())
                            .or_else(|| requirements_by_ord.get(&(ord as u32)).cloned())
                    };
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
                        generation_requirement,
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
            notetype_model_ids.insert(notetype.id.clone(), row_id);
            notetypes_by_row_id.insert(row_id, notetype.clone());
            notetype_values.push(notetype);
        }

        let mut note_decks_by_row_id = BTreeMap::<i64, String>::new();
        let mut note_deck_rows = conn.prepare(
            "select cards.nid, decks.name
             from cards
             left join decks on decks.id = cards.did
             where cards.ord = (
                 select min(inner_cards.ord)
                 from cards inner_cards
                 where inner_cards.nid = cards.nid
             )
             order by cards.nid",
        )?;
        for row in note_deck_rows.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        })? {
            let (note_id, deck_name) = row?;
            if projection == ReadProjection::Observations {
                note_decks_by_row_id.insert(
                    note_id,
                    deck_name
                        .map(|name| native_deck_name_to_human(&name))
                        .unwrap_or_else(|| "Default".into()),
                );
            }
        }

        let mut actual_card_decks = BTreeMap::<(String, usize), String>::new();
        let mut summary_card_ordinals = BTreeMap::<String, BTreeSet<usize>>::new();
        let mut card_deck_rows = conn.prepare(
            "select notes.guid, cards.ord, decks.name
             from cards
             join notes on notes.id = cards.nid
             left join decks on decks.id = cards.did
             order by notes.guid, cards.ord",
        )?;
        for row in card_deck_rows.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })? {
            let (note_guid, ord, deck_name) = row?;
            // Both projections collapse identical (GUID, ord) pairs, including
            // the existing signed-to-usize ordinal conversion. Summary does
            // not retain deck names, but they were still decoded above.
            let ord = ord as usize;
            match projection {
                ReadProjection::Observations => {
                    actual_card_decks.insert(
                        (note_guid, ord),
                        deck_name
                            .map(|name| native_deck_name_to_human(&name))
                            .unwrap_or_else(|| "Default".into()),
                    );
                }
                ReadProjection::Summary => {
                    summary_card_ordinals
                        .entry(note_guid)
                        .or_default()
                        .insert(ord);
                }
            }
        }

        let mut notes = Vec::new();
        let mut note_identity_metadata = Vec::new();
        let mut summary_counts = NoteCardCounts::default();
        let mut note_rows =
            conn.prepare("select id, guid, mid, mod, tags, flds, data from notes order by id")?;
        let mut rows = note_rows.query([])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let guid: String = row.get(1)?;
            let mid: i64 = row.get(2)?;
            let mtime_secs: i64 = row.get(3)?;
            let tags: String = row.get(4)?;
            let flds: String = row.get(5)?;
            let data: String = row.get(6)?;
            let notetype = notetypes_by_row_id
                .get(&mid)
                .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            if projection == ReadProjection::Summary {
                // A known row-level notetype guarantees the note is eligible
                // for observed_notes(). Repeated GUIDs each observe the same
                // set of actual ordinals, matching full inspection.
                summary_counts.notes += 1;
                summary_counts.cards += summary_card_ordinals.get(&guid).map_or(0, BTreeSet::len);
                continue;
            }
            // Empty storage is one empty field, not an absent field list.
            // split also retains empty values between/trailing separators.
            let field_values = flds.split('\u{1f}');
            let mut fields = BTreeMap::new();
            for (field, value) in notetype.fields.iter().zip(field_values) {
                fields.insert(field.name.clone(), value.to_string());
            }
            let note = NormalizedNote {
                id: guid.clone(),
                notetype_id: notetype.id.clone(),
                deck_name: note_decks_by_row_id
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| "Default".into()),
                fields,
                tags: if tags.is_empty() {
                    vec![]
                } else {
                    tags.split(' ').map(|tag| tag.to_string()).collect()
                },
                mtime_secs: Some(mtime_secs),
            };
            let identity_metadata = serde_json::from_str::<Value>(&data)
                .ok()
                .and_then(|value| value.get("anki_forge_identity").cloned())
                .map(|mut observed| {
                    if let Some(object) = observed.as_object_mut() {
                        object.insert(
                            "selector".into(),
                            Value::String(format!("note[guid='{}']::anki_forge_identity", guid)),
                        );
                        object.insert(
                            "evidence_refs".into(),
                            json!([format!("note-data:{}", guid)]),
                        );
                    }
                    observed
                });
            notes.push(note);
            if let Some(identity) = identity_metadata {
                note_identity_metadata.push(identity);
            }
        }

        Ok(CollectionData {
            notetypes: notetype_values,
            notetype_model_ids,
            notes,
            note_identity_metadata,
            template_target_decks,
            actual_card_decks,
            summary_counts,
        })
    })
}

fn normalized_notetype_kind(config: &crate::writer_core::anki_proto::NotetypeConfig) -> String {
    match OriginalStockKind::try_from(config.original_stock_kind).ok() {
        Some(OriginalStockKind::Basic) => "normal".into(),
        Some(OriginalStockKind::Cloze) => "cloze".into(),
        Some(OriginalStockKind::ImageOcclusion) => "cloze".into(),
        _ => match NotetypeKind::try_from(config.kind).ok() {
            Some(NotetypeKind::Cloze) => "cloze".into(),
            _ => "normal".into(),
        },
    }
}

fn original_stock_kind(config: &crate::writer_core::anki_proto::NotetypeConfig) -> Option<String> {
    match OriginalStockKind::try_from(config.original_stock_kind).ok() {
        Some(OriginalStockKind::Basic) => Some("basic".into()),
        Some(OriginalStockKind::Cloze) => Some("cloze".into()),
        Some(OriginalStockKind::ImageOcclusion) => Some("image_occlusion".into()),
        _ => None,
    }
}

fn generation_requirement_from_card_requirement(
    requirement: &CardRequirement,
    field_names_by_ord: &BTreeMap<u32, String>,
) -> Option<NormalizedGenerationRequirement> {
    let kind = match CardRequirementKind::try_from(requirement.kind).ok()? {
        CardRequirementKind::All => "all",
        CardRequirementKind::Any => "any",
        CardRequirementKind::None => return None,
    };
    let field_names = requirement
        .field_ords
        .iter()
        .filter_map(|ord| field_names_by_ord.get(ord).cloned())
        .collect::<Vec<_>>();
    if field_names.is_empty() {
        return None;
    }

    Some(NormalizedGenerationRequirement {
        kind: kind.into(),
        field_names,
    })
}

fn with_readonly_sqlite<T>(path: &Path, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .context("open inspected collection read-only")?;
    f(&conn)
}

pub fn artifact_path_from_ref(target: &BuildArtifactTarget, reference: &str) -> Result<PathBuf> {
    let prefix = target.stable_ref_prefix.trim_end_matches('/');
    let reference_path = Path::new(reference);
    let prefix_path = Path::new(prefix);

    ensure!(
        reference_path.is_relative(),
        "artifact reference must be relative: {}",
        reference
    );
    ensure!(
        !contains_parent_dir(reference_path),
        "artifact reference must not traverse upward: {}",
        reference
    );

    let remainder = if reference_has_component_prefix(reference_path, prefix_path) {
        reference_path
            .strip_prefix(prefix_path)
            .unwrap_or(reference_path)
            .to_path_buf()
    } else {
        reference_path.to_path_buf()
    };

    ensure!(
        !contains_parent_dir(&remainder),
        "artifact reference must not traverse upward: {}",
        reference
    );

    Ok(if remainder.as_os_str().is_empty() {
        target.root_dir.clone()
    } else {
        target.root_dir.join(remainder)
    })
}

fn reference_has_component_prefix(reference: &Path, prefix: &Path) -> bool {
    let mut reference_components = reference.components();
    for prefix_component in prefix.components() {
        match reference_components.next() {
            Some(reference_component) if reference_component == prefix_component => {}
            _ => return false,
        }
    }
    true
}

fn contains_parent_dir(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

#[cfg(test)]
#[path = "inspect_summary_tests.rs"]
mod summary_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apkg_observations_enumerate_actual_cards_even_when_the_current_plan_is_empty() {
        let normalized_ir = NormalizedIr {
            kind: "normalized-ir".into(),
            schema_version: "phase2-v1".into(),
            document_id: "inspect-actual-cards".into(),
            resolved_identity: "inspect-actual-cards".into(),
            notetypes: vec![NormalizedNotetype {
                id: "basic".into(),
                kind: "normal".into(),
                name: "Basic".into(),
                original_stock_kind: None,
                original_id: None,
                fields: vec![NormalizedField {
                    name: "Front".into(),
                    ord: Some(0),
                    config_id: None,
                    tag: None,
                    prevent_deletion: false,
                    sort: true,
                }],
                templates: vec![NormalizedTemplate {
                    name: "Card".into(),
                    ord: Some(0),
                    config_id: None,
                    question_format: "{{Front}}".into(),
                    answer_format: "{{Front}}".into(),
                    browser_question_format: None,
                    browser_answer_format: None,
                    target_deck_name: None,
                    browser_font_name: None,
                    browser_font_size: None,
                    generation_requirement: None,
                }],
                css: String::new(),
                field_metadata: Vec::new(),
            }],
            notes: vec![NormalizedNote {
                id: "note-1".into(),
                notetype_id: "basic".into(),
                deck_name: "Deck".into(),
                fields: BTreeMap::from([("Front".into(), String::new())]),
                tags: Vec::new(),
                mtime_secs: None,
            }],
            media_objects: Vec::new(),
            media_bindings: Vec::new(),
            media_references: Vec::new(),
        };
        let actual_cards = BTreeMap::from([(("note-1".into(), 0), "Deck".into())]);

        let observations = build_observations(
            &normalized_ir,
            &[],
            &[],
            Some(&actual_cards),
            &[],
            &BTreeMap::new(),
        );

        assert!(observations
            .references
            .iter()
            .any(|value| value["selector"] == "card[note_id='note-1'][ord=0]"));
        assert_eq!(observations.metadata[0]["card_count"], 1);
    }

    #[test]
    fn artifact_path_from_ref_does_not_strip_prefix_collisions() {
        let target = BuildArtifactTarget::new("/tmp/root", "artifacts/phase3/inspect");
        let resolved =
            artifact_path_from_ref(&target, "artifacts/phase3/inspect-apkg/package.apkg")
                .expect("resolve path");

        assert_eq!(
            resolved,
            PathBuf::from("/tmp/root/artifacts/phase3/inspect-apkg/package.apkg")
        );
    }

    #[test]
    fn artifact_path_from_ref_rejects_path_traversal() {
        let target = BuildArtifactTarget::new("/tmp/root", "artifacts/phase3/inspect");
        let err = artifact_path_from_ref(&target, "artifacts/phase3/inspect/../escape.apkg")
            .expect_err("path traversal must fail");

        assert!(err.to_string().contains("traverse upward"));
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

#[derive(Debug, Deserialize)]
struct StagingManifest {
    normalized_ir: NormalizedIr,
    #[serde(default)]
    notetype_model_ids: Option<BTreeMap<String, i64>>,
    #[serde(default)]
    template_target_decks: Vec<ResolvedTemplateTargetDeck>,
}
