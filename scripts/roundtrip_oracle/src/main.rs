use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anki::collection::CollectionBuilder;
use anki::import_export::package::ImportAnkiPackageOptions;
use anyhow::{bail, ensure, Context};
use serde::Deserialize;
use serde_json::{json, Value};
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct RoundtripOracleInput {
    label: String,
    first_case: PathBuf,
    second_case: PathBuf,
    first_package: PreparedPackage,
    second_package: PreparedPackage,
}

#[derive(Debug, Deserialize)]
struct PreparedPackage {
    apkg_path: PathBuf,
    notetype_ids_by_name: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Phase5aRoundTripResult {
    after_first_import: Phase5aImportState,
    after_second_import: Phase5aImportState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Phase5aImportState {
    notetype_count: usize,
    conflicting_notes: usize,
    field_ords: BTreeMap<String, Vec<u32>>,
    template_ords: BTreeMap<String, Vec<u32>>,
    referenced_static_media: BTreeSet<String>,
    template_target_decks: Vec<TemplateDeckTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TemplateDeckTarget {
    template_name: String,
    deck_name: String,
    deck_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Phase5aCollectionState {
    notetype_count: usize,
    template_target_decks: Vec<TemplateDeckTarget>,
    field_ords: BTreeMap<String, Vec<u32>>,
    template_ords: BTreeMap<String, Vec<u32>>,
}

fn main() -> anyhow::Result<()> {
    let input_path = parse_args()?;
    let input = load_input(&input_path)?;
    let roundtrip = run_phase5a_roundtrip_oracle(&input)?;
    verify_roundtrip(&roundtrip)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "ok",
            "label": input.label,
            "first_case": input.first_case.display().to_string(),
            "second_case": input.second_case.display().to_string(),
            "after_first_import": summarize_state(&roundtrip.after_first_import),
            "after_second_import": summarize_state(&roundtrip.after_second_import),
        }))?
    );

    Ok(())
}

fn parse_args() -> anyhow::Result<PathBuf> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [input_path] => Ok(PathBuf::from(input_path)),
        _ => bail!(
            "usage: cargo run --manifest-path scripts/roundtrip_oracle/Cargo.toml -- <prepared-input.json>"
        ),
    }
}

fn load_input(path: &Path) -> anyhow::Result<RoundtripOracleInput> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read roundtrip oracle input {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("decode roundtrip oracle input {}", path.display()))
}

fn run_phase5a_roundtrip_oracle(
    input: &RoundtripOracleInput,
) -> anyhow::Result<Phase5aRoundTripResult> {
    let collection_root = tempdir().context("create phase5a upstream collection tempdir")?;
    let collection_path = collection_root.path().join("phase5a-roundtrip.anki2");
    let mut builder = CollectionBuilder::new(&collection_path);
    builder.with_desktop_media_paths();
    let mut collection = builder.build().context("build upstream collection")?;

    let first_import = collection
        .import_apkg(
            &input.first_package.apkg_path,
            ImportAnkiPackageOptions::default(),
        )
        .context("import first phase5a package into upstream collection")?;
    let after_first_import = summarize_phase5a_import_state(
        &mut collection,
        &collection_path,
        &input.first_package.notetype_ids_by_name,
        first_import.output.conflicting.len(),
    )?;

    let second_import = collection
        .import_apkg(
            &input.second_package.apkg_path,
            ImportAnkiPackageOptions {
                merge_notetypes: true,
                ..Default::default()
            },
        )
        .context("re-import second phase5a package into upstream collection")?;
    let after_second_import = summarize_phase5a_import_state(
        &mut collection,
        &collection_path,
        &input.second_package.notetype_ids_by_name,
        second_import.output.conflicting.len(),
    )?;

    Ok(Phase5aRoundTripResult {
        after_first_import,
        after_second_import,
    })
}

fn summarize_phase5a_import_state(
    collection: &mut anki::prelude::Collection,
    collection_path: &Path,
    notetype_ids_by_name: &BTreeMap<String, String>,
    conflicting_notes: usize,
) -> anyhow::Result<Phase5aImportState> {
    let collection_state = read_phase5a_collection_state(collection, notetype_ids_by_name)?;

    Ok(Phase5aImportState {
        notetype_count: collection_state.notetype_count,
        conflicting_notes,
        field_ords: collection_state.field_ords,
        template_ords: collection_state.template_ords,
        referenced_static_media: read_phase5a_imported_media(&collection_path.with_extension("media"))?,
        template_target_decks: collection_state.template_target_decks,
    })
}

fn read_phase5a_collection_state(
    collection: &mut anki::prelude::Collection,
    notetype_ids_by_name: &BTreeMap<String, String>,
) -> anyhow::Result<Phase5aCollectionState> {
    let mut field_ords = BTreeMap::new();
    let mut template_ords = BTreeMap::new();
    let mut template_target_decks = vec![];
    let notetypes = collection.get_all_notetypes()?;

    for notetype in &notetypes {
        let product_notetype_id = notetype_ids_by_name
            .get(&notetype.name)
            .cloned()
            .unwrap_or_else(|| format!("notetype-{}", notetype.id.0));
        field_ords.insert(
            product_notetype_id.clone(),
            notetype
                .fields
                .iter()
                .map(|field| {
                    field.ord.with_context(|| {
                        format!(
                            "missing field ord after upstream import for {}",
                            notetype.name
                        )
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
        );
        template_ords.insert(
            product_notetype_id,
            notetype
                .templates
                .iter()
                .map(|template| {
                    template.ord.with_context(|| {
                        format!(
                            "missing template ord after upstream import for {}::{}",
                            notetype.name, template.name
                        )
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
        );

        for template in &notetype.templates {
            if template.config.target_deck_id > 0 {
                let deck_id = anki::decks::DeckId(template.config.target_deck_id);
                let deck_name = collection
                    .get_deck(deck_id)?
                    .map(|deck| deck.human_name())
                    .unwrap_or_else(|| format!("deck-{}", deck_id.0));
                template_target_decks.push(TemplateDeckTarget {
                    template_name: template.name.clone(),
                    deck_name,
                    deck_id: deck_id.0,
                });
            }
        }
    }

    template_target_decks.sort_by(|left, right| {
        (&left.template_name, &left.deck_name, left.deck_id).cmp(&(
            &right.template_name,
            &right.deck_name,
            right.deck_id,
        ))
    });

    Ok(Phase5aCollectionState {
        notetype_count: notetypes.len(),
        template_target_decks,
        field_ords,
        template_ords,
    })
}

fn read_phase5a_imported_media(media_root: &Path) -> anyhow::Result<BTreeSet<String>> {
    if !media_root.exists() {
        return Ok(BTreeSet::new());
    }

    let mut filenames = BTreeSet::new();
    for entry in fs::read_dir(media_root)
        .with_context(|| format!("read media dir {}", media_root.display()))?
    {
        let entry =
            entry.with_context(|| format!("read media entry in {}", media_root.display()))?;
        if entry
            .file_type()
            .with_context(|| format!("stat media entry {}", entry.path().display()))?
            .is_file()
        {
            filenames.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(filenames)
}

fn verify_roundtrip(roundtrip: &Phase5aRoundTripResult) -> anyhow::Result<()> {
    ensure!(
        roundtrip.after_first_import.notetype_count == roundtrip.after_second_import.notetype_count,
        "notetype count changed across imports"
    );
    ensure!(
        roundtrip.after_first_import.conflicting_notes == 0,
        "first import produced conflicting notes"
    );
    ensure!(
        roundtrip.after_second_import.conflicting_notes == 0,
        "second import produced conflicting notes"
    );
    ensure!(
        roundtrip.after_first_import.field_ords.get("io-main") == Some(&vec![0, 1, 2, 3, 4]),
        "unexpected io field ords after first import"
    );
    ensure!(
        roundtrip.after_second_import.field_ords.get("io-main") == Some(&vec![0, 1, 2, 3, 4]),
        "unexpected io field ords after second import"
    );
    ensure!(
        roundtrip.after_first_import.field_ords == roundtrip.after_second_import.field_ords,
        "field ords changed across imports"
    );
    ensure!(
        roundtrip.after_first_import.template_ords.get("io-main") == Some(&vec![0]),
        "unexpected io template ords after first import"
    );
    ensure!(
        roundtrip.after_second_import.template_ords.get("io-main") == Some(&vec![0]),
        "unexpected io template ords after second import"
    );
    ensure!(
        roundtrip.after_first_import.template_ords == roundtrip.after_second_import.template_ords,
        "template ords changed across imports"
    );
    ensure!(
        roundtrip.after_first_import.template_target_decks
            == roundtrip.after_second_import.template_target_decks,
        "template target decks changed across imports"
    );
    ensure!(
        roundtrip.after_first_import.referenced_static_media
            != roundtrip.after_second_import.referenced_static_media,
        "static media should change after the second import"
    );
    ensure!(
        roundtrip
            .after_second_import
            .referenced_static_media
            .iter()
            .any(|name| name.starts_with("_io-main_") && name.ends_with(".woff2")),
        "second import should include the namespaced io font asset"
    );
    ensure!(
        roundtrip
            .after_second_import
            .template_target_decks
            .iter()
            .any(|item| item.template_name == "Image Occlusion" && item.deck_id > 0),
        "second import should resolve a positive template target deck id"
    );
    Ok(())
}

fn summarize_state(state: &Phase5aImportState) -> Value {
    json!({
        "notetype_count": state.notetype_count,
        "conflicting_notes": state.conflicting_notes,
        "field_ords": state.field_ords,
        "template_ords": state.template_ords,
        "referenced_static_media": state.referenced_static_media,
        "template_target_decks": state
            .template_target_decks
            .iter()
            .map(|item| json!({
                "template_name": item.template_name,
                "deck_name": item.deck_name,
                "deck_id": item.deck_id,
            }))
            .collect::<Vec<_>>(),
    })
}
