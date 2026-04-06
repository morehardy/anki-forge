use anyhow::{ensure, Context};
use authoring_core::NormalizedIr;
use prost::Message;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
use zip::ZipArchive;
use zstd::stream::decode_all;

use crate::{
    fixtures::load_fixture_catalog,
    manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path},
    policies::{load_build_context_asset, load_writer_policy_asset},
    schema::{load_schema, validate_value},
};

const LATEST_META_VERSION: i32 = 3;
const REQUIRED_COLLECTION_LATEST: &str = "collection.anki21b";
const REQUIRED_COLLECTION_LEGACY_DUMMY: &str = "collection.anki2";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase3WriterCase {
    kind: String,
    normalized_input: String,
    writer_policy_selector: String,
    build_context_selector: String,
    artifacts_dir: String,
    expected_build: String,
    expected_inspect: String,
    #[serde(default)]
    expected_diff: Option<String>,
}

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

pub fn run_compat_oracle_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;
    let catalog_path = resolve_asset_path(&manifest, "fixture_catalog")
        .context("fixture catalog asset must be declared in the manifest")?;
    let normalized_ir_schema = load_schema(resolve_asset_path(&manifest, "normalized_ir_schema")?)?;
    let catalog = load_fixture_catalog(&catalog_path)?;

    let writer_cases: Vec<_> = catalog
        .cases
        .iter()
        .filter(|case| case.category == "phase3-writer")
        .collect();
    ensure!(
        !writer_cases.is_empty(),
        "compat oracle requires phase3-writer fixtures in the catalog"
    );

    for case in writer_cases {
        let case_path = resolve_contract_relative_path(&manifest.contracts_root, &case.input)
            .with_context(|| format!("phase3 writer case path must resolve: {}", case.id))?;
        let case_data: Phase3WriterCase = load_yaml_model(&case_path)?;
        ensure!(
            case_data.kind == "phase3-writer-case",
            "phase3 writer fixture must declare kind=phase3-writer-case: {}",
            case.id
        );
        ensure!(
            !case_data.artifacts_dir.trim().is_empty(),
            "phase3 writer fixture artifacts_dir must not be empty: {}",
            case.id
        );
        resolve_contract_relative_path(&manifest.contracts_root, &case_data.expected_build)
            .with_context(|| {
                format!(
                    "phase3 writer expected_build must resolve for compat oracle case {}",
                    case.id
                )
            })?;
        resolve_contract_relative_path(&manifest.contracts_root, &case_data.expected_inspect)
            .with_context(|| {
                format!(
                    "phase3 writer expected_inspect must resolve for compat oracle case {}",
                    case.id
                )
            })?;
        if let Some(expected_diff) = &case_data.expected_diff {
            resolve_contract_relative_path(&manifest.contracts_root, expected_diff).with_context(
                || {
                    format!(
                        "phase3 writer expected_diff must resolve for compat oracle case {}",
                        case.id
                    )
                },
            )?;
        }

        let normalized_ir_path =
            resolve_contract_relative_path(&manifest.contracts_root, &case_data.normalized_input)
                .with_context(|| {
                    format!(
                        "phase3 normalized input must resolve for compat oracle case {}",
                        case.id
                    )
                })?;
        let normalized_ir = load_normalized_ir(&normalized_ir_schema, &normalized_ir_path)
            .with_context(|| format!("load normalized input for compat oracle case {}", case.id))?;

        let writer_policy =
            load_writer_policy_asset(&manifest, &case_data.writer_policy_selector).with_context(
                || {
                    format!(
                        "load writer policy for compat oracle case {}",
                        case.id
                    )
                },
            )?;
        let build_context =
            load_build_context_asset(&manifest, &case_data.build_context_selector).with_context(
                || {
                    format!(
                        "load build context for compat oracle case {}",
                        case.id
                    )
                },
            )?;

        let artifact_root = manifest
            .contracts_root
            .join("artifacts")
            .join("compat-oracle")
            .join(&case.id);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).with_context(|| {
                format!(
                    "remove previous compat oracle artifacts for {}",
                    case.id
                )
            })?;
        }

        let stable_ref_prefix = format!("artifacts/compat-oracle/{}", case.id);
        let artifact_target = writer_core::BuildArtifactTarget::new(artifact_root, stable_ref_prefix);
        let build_result = writer_core::build(
            &normalized_ir,
            &writer_policy,
            &build_context,
            &artifact_target,
        )
        .with_context(|| format!("build writer fixture for compat oracle case {}", case.id))?;
        ensure!(
            build_result.result_status == "success",
            "compat oracle build must succeed: {}",
            case.id
        );
        let apkg_ref = build_result.apkg_ref.as_deref().with_context(|| {
            format!(
                "compat oracle build must emit apkg_ref for case {}",
                case.id
            )
        })?;
        let apkg_path = artifact_path_from_ref(&artifact_target, apkg_ref);
        ensure!(
            apkg_path.exists(),
            "compat oracle apkg must exist for case {}: {}",
            case.id,
            apkg_path.display()
        );

        let inspect_report = writer_core::inspect_apkg(&apkg_path)
            .with_context(|| format!("inspect apkg for compat oracle case {}", case.id))?;
        validate_supported_package(&apkg_path, &inspect_report)
            .with_context(|| format!("compat oracle validation failed for {}", case.id))?;
    }

    Ok(())
}

pub fn validate_supported_package(
    apkg_path: impl AsRef<Path>,
    inspect_report: &writer_core::InspectReport,
) -> anyhow::Result<()> {
    ensure!(
        inspect_report.observation_status == "complete",
        "inspect report must be complete for compatibility oracle"
    );
    ensure!(
        inspect_report.missing_domains.is_empty(),
        "inspect report must not contain missing domains for compatibility oracle"
    );

    let apkg_path = apkg_path.as_ref();
    let file =
        File::open(apkg_path).with_context(|| format!("open apkg {}", apkg_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("open apkg archive {}", apkg_path.display()))?;

    let meta_bytes = read_required_zip_entry(&mut archive, "meta")?;
    let meta = PackageMetadata::decode(meta_bytes.as_slice()).context("decode package meta")?;
    ensure!(
        meta.version == LATEST_META_VERSION,
        "unsupported package lane in meta: expected {} but found {}",
        LATEST_META_VERSION,
        meta.version
    );

    read_required_zip_entry(&mut archive, REQUIRED_COLLECTION_LATEST)?;
    read_required_zip_entry(&mut archive, REQUIRED_COLLECTION_LEGACY_DUMMY)?;

    let latest_collection_encoded = read_required_zip_entry(&mut archive, REQUIRED_COLLECTION_LATEST)?;
    let latest_collection = decode_all(latest_collection_encoded.as_slice())
        .context("decode zstd collection.anki21b")?;
    let collection_counts = read_collection_counts(&latest_collection)?;
    let inspect_counts = read_inspect_counts(inspect_report)?;
    ensure!(
        collection_counts.notetype_count == inspect_counts.notetype_count,
        "notetype count mismatch between collection DB ({}) and inspect metadata ({})",
        collection_counts.notetype_count,
        inspect_counts.notetype_count
    );
    ensure!(
        collection_counts.note_count == inspect_counts.note_count,
        "note count mismatch between collection DB ({}) and inspect metadata ({})",
        collection_counts.note_count,
        inspect_counts.note_count
    );
    ensure!(
        collection_counts.card_count == inspect_counts.card_count,
        "card count mismatch between collection DB ({}) and inspect metadata ({})",
        collection_counts.card_count,
        inspect_counts.card_count
    );

    let media_encoded = read_required_zip_entry(&mut archive, "media")?;
    let media_bytes = decode_all(media_encoded.as_slice()).context("decode zstd media map")?;
    ensure!(
        serde_json::from_slice::<HashMap<String, String>>(&media_bytes).is_err(),
        "latest media map must not decode as legacy hashmap JSON"
    );
    let media_entries = MediaEntries::decode(media_bytes.as_slice()).context("decode media map proto")?;
    let package_media = read_media_payload_entries(&mut archive, &media_entries)?;

    let inspect_media = read_inspect_media_entries(inspect_report)?;
    ensure!(
        package_media == inspect_media,
        "media map payloads must match inspect media observations"
    );
    ensure!(
        inspect_counts.media_count == package_media.len(),
        "media count mismatch between inspect metadata ({}) and media map ({})",
        inspect_counts.media_count,
        package_media.len()
    );

    let referenced_media = read_media_references_from_inspect(inspect_report)?;
    for referenced in &referenced_media {
        ensure!(
            package_media.contains_key(referenced.as_str()),
            "media reference {} from inspect report is missing from media map",
            referenced
        );
    }

    validate_stock_lane_invariants(inspect_report)?;

    Ok(())
}

fn validate_stock_lane_invariants(inspect_report: &writer_core::InspectReport) -> anyhow::Result<()> {
    let templates_by_notetype = templates_by_notetype(inspect_report)?;
    let fields_by_notetype = fields_by_notetype(inspect_report)?;

    for notetype in &inspect_report.observations.notetypes {
        let notetype_id = required_str_field(notetype, "id")?;
        let kind = required_str_field(notetype, "kind")?;
        let name = required_str_field(notetype, "name")?;

        match kind {
            "basic" => {
                ensure!(name == "Basic", "basic notetype must be named Basic");
                let fields = fields_by_notetype
                    .get(notetype_id)
                    .with_context(|| format!("missing fields for notetype {}", notetype_id))?;
                ensure!(
                    fields == &["Back".to_string(), "Front".to_string()],
                    "basic notetype fields must be Front/Back"
                );
                let templates = templates_by_notetype
                    .get(notetype_id)
                    .with_context(|| format!("missing templates for notetype {}", notetype_id))?;
                ensure!(templates.len() == 1, "basic notetype must have exactly one template");
                let template = &templates[0];
                ensure!(
                    required_str_field(template, "name")? == "Card 1",
                    "basic template name must be Card 1"
                );
                ensure!(
                    required_str_field(template, "question_format")? == "{{Front}}",
                    "basic template question format must match stock basic front"
                );
                ensure!(
                    required_str_field(template, "answer_format")?
                        == "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}",
                    "basic template answer format must match stock basic back"
                );
            }
            "cloze" => {
                ensure!(name == "Cloze", "cloze notetype must be named Cloze");
                let fields = fields_by_notetype
                    .get(notetype_id)
                    .with_context(|| format!("missing fields for notetype {}", notetype_id))?;
                ensure!(
                    fields == &["Back Extra".to_string(), "Text".to_string()],
                    "cloze notetype fields must be Text/Back Extra"
                );
                let templates = templates_by_notetype
                    .get(notetype_id)
                    .with_context(|| format!("missing templates for notetype {}", notetype_id))?;
                ensure!(templates.len() == 1, "cloze notetype must have exactly one template");
                let template = &templates[0];
                ensure!(
                    required_str_field(template, "name")? == "Cloze",
                    "cloze template name must be Cloze"
                );
                ensure!(
                    required_str_field(template, "question_format")? == "{{cloze:Text}}",
                    "cloze template question format must match stock cloze"
                );
                ensure!(
                    required_str_field(template, "answer_format")? == "{{cloze:Text}}<br>\n{{Back Extra}}",
                    "cloze template answer format must match stock cloze"
                );
            }
            "image_occlusion" => {
                ensure!(
                    name == "Image Occlusion",
                    "image occlusion notetype must be named Image Occlusion"
                );
                let fields = fields_by_notetype
                    .get(notetype_id)
                    .with_context(|| format!("missing fields for notetype {}", notetype_id))?;
                for required in ["Back Extra", "Header", "Image", "Occlusion"] {
                    ensure!(
                        fields.contains(&required.to_string()),
                        "image occlusion notetype must include field {}",
                        required
                    );
                }
                ensure!(
                    fields.len() >= 4,
                    "image occlusion notetype must have at least four fields"
                );
                let templates = templates_by_notetype
                    .get(notetype_id)
                    .with_context(|| format!("missing templates for notetype {}", notetype_id))?;
                ensure!(
                    templates.len() == 1,
                    "image occlusion notetype must have exactly one template"
                );
                let template = &templates[0];
                ensure!(
                    required_str_field(template, "name")? == "Image Occlusion",
                    "image occlusion template name must be Image Occlusion"
                );
                let question = required_str_field(template, "question_format")?;
                let answer = required_str_field(template, "answer_format")?;
                ensure!(
                    question.contains("{{cloze:Occlusion}}")
                        && question.contains("{{Image}}")
                        && question.contains("anki.imageOcclusion.setup()"),
                    "image occlusion question format must include cloze, image, and setup script"
                );
                ensure!(
                    answer.contains("id=\"toggle\""),
                    "image occlusion answer format must include the toggle masks control"
                );
            }
            other => {
                ensure!(
                    false,
                    "compat oracle currently supports only basic/cloze/image_occlusion notetypes, found {}",
                    other
                );
            }
        }
    }

    Ok(())
}

fn read_media_payload_entries(
    archive: &mut ZipArchive<File>,
    media_entries: &MediaEntries,
) -> anyhow::Result<BTreeMap<String, (usize, String)>> {
    let mut result = BTreeMap::new();

    for (index, entry) in media_entries.entries.iter().enumerate() {
        ensure!(
            entry.legacy_zip_filename.is_none(),
            "latest media map entries must not set legacy_zip_filename"
        );
        let payload_encoded = read_required_zip_entry(archive, &index.to_string())?;
        let payload = decode_all(payload_encoded.as_slice())
            .with_context(|| format!("decode zstd media payload {}", index))?;
        let sha1 = Sha1::digest(&payload).to_vec();
        ensure!(
            payload.len() == entry.size as usize,
            "media payload size mismatch for {}: map={} payload={}",
            entry.name,
            entry.size,
            payload.len()
        );
        ensure!(
            sha1 == entry.sha1,
            "media payload sha1 mismatch for {}",
            entry.name
        );

        result.insert(entry.name.clone(), (payload.len(), hex_lower(&sha1)));
    }

    Ok(result)
}

fn read_inspect_media_entries(
    inspect_report: &writer_core::InspectReport,
) -> anyhow::Result<BTreeMap<String, (usize, String)>> {
    let mut media = BTreeMap::new();
    for entry in &inspect_report.observations.media {
        let filename = required_str_field(entry, "filename")?.to_string();
        let size = required_u64_field(entry, "size")? as usize;
        let sha1 = required_str_field(entry, "sha1")?.to_string();
        media.insert(filename, (size, sha1));
    }
    Ok(media)
}

fn read_media_references_from_inspect(
    inspect_report: &writer_core::InspectReport,
) -> anyhow::Result<BTreeSet<String>> {
    let mut references = BTreeSet::new();
    for entry in &inspect_report.observations.references {
        if let Some(reference) = entry.get("reference").and_then(Value::as_str) {
            references.insert(reference.to_string());
        }
    }
    Ok(references)
}

fn templates_by_notetype(
    inspect_report: &writer_core::InspectReport,
) -> anyhow::Result<BTreeMap<String, Vec<Value>>> {
    let mut templates: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for template in &inspect_report.observations.templates {
        let notetype_id = required_str_field(template, "notetype_id")?.to_string();
        templates
            .entry(notetype_id)
            .or_default()
            .push(template.clone());
    }

    for values in templates.values_mut() {
        values.sort_by(|left, right| {
            required_str_field(left, "name")
                .unwrap_or("")
                .cmp(required_str_field(right, "name").unwrap_or(""))
        });
    }

    Ok(templates)
}

fn fields_by_notetype(
    inspect_report: &writer_core::InspectReport,
) -> anyhow::Result<BTreeMap<String, Vec<String>>> {
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for field in &inspect_report.observations.fields {
        let notetype_id = required_str_field(field, "notetype_id")?.to_string();
        let name = required_str_field(field, "name")?.to_string();
        fields.entry(notetype_id).or_default().push(name);
    }
    for values in fields.values_mut() {
        values.sort();
    }
    Ok(fields)
}

#[derive(Debug, Clone, Copy)]
struct CollectionCounts {
    notetype_count: usize,
    note_count: usize,
    card_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct InspectCounts {
    notetype_count: usize,
    note_count: usize,
    card_count: usize,
    media_count: usize,
}

fn read_collection_counts(collection_bytes: &[u8]) -> anyhow::Result<CollectionCounts> {
    with_temp_sqlite(collection_bytes, |conn| {
        Ok(CollectionCounts {
            notetype_count: count_rows(conn, "notetypes")?,
            note_count: count_rows(conn, "notes")?,
            card_count: count_rows(conn, "cards")?,
        })
    })
}

fn read_inspect_counts(inspect_report: &writer_core::InspectReport) -> anyhow::Result<InspectCounts> {
    let counts = inspect_report
        .observations
        .metadata
        .iter()
        .find(|entry| entry.get("selector").and_then(Value::as_str) == Some("counts"))
        .context("inspect report metadata must include selector=counts")?;

    Ok(InspectCounts {
        notetype_count: required_u64_field(counts, "notetype_count")? as usize,
        note_count: required_u64_field(counts, "note_count")? as usize,
        card_count: required_u64_field(counts, "card_count")? as usize,
        media_count: required_u64_field(counts, "media_count")? as usize,
    })
}

fn count_rows(conn: &Connection, table: &str) -> anyhow::Result<usize> {
    let query = format!("select count(*) from {table}");
    let count: i64 = conn.query_row(&query, [], |row| row.get(0))?;
    Ok(count as usize)
}

fn with_temp_sqlite<T>(bytes: &[u8], f: impl FnOnce(&Connection) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let path = unique_temp_path("compat-oracle.sqlite");
    fs::write(&path, bytes).with_context(|| format!("write temp sqlite {}", path.display()))?;
    let result = (|| {
        let conn = Connection::open(&path)
            .with_context(|| format!("open temp sqlite {}", path.display()))?;
        f(&conn)
    })();
    let _ = fs::remove_file(&path);
    result
}

fn unique_temp_path(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anki-forge-{name}-{}-{nanos}",
        std::process::id()
    ))
}

fn read_required_zip_entry(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> anyhow::Result<Vec<u8>> {
    read_optional_zip_entry(archive, name)?
        .with_context(|| format!("required zip entry is missing: {}", name))
}

fn read_optional_zip_entry(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
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

fn artifact_path_from_ref(target: &writer_core::BuildArtifactTarget, reference: &str) -> PathBuf {
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

fn load_yaml_model<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML asset: {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("YAML asset must match its model: {}", path.display()))
}

fn load_normalized_ir(schema: &jsonschema::JSONSchema, path: &Path) -> anyhow::Result<NormalizedIr> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read normalized input {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&raw).with_context(|| format!("decode JSON {}", path.display()))?;
    validate_value(schema, &value)
        .with_context(|| format!("normalized input must satisfy schema: {}", path.display()))?;
    serde_json::from_value(value)
        .with_context(|| format!("normalized input must map to execution model: {}", path.display()))
}

fn required_str_field<'a>(value: &'a Value, field: &str) -> anyhow::Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("missing string field {}", field))
}

fn required_u64_field(value: &Value, field: &str) -> anyhow::Result<u64> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .with_context(|| format!("missing integer field {}", field))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:02x}", byte);
    }
    out
}
