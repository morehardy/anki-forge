//! Import README example packages with Anki and retain their actual card HTML.
use anki::{
    collection::CollectionBuilder, import_export::package::ImportAnkiPackageOptions,
    search::SortMode,
};
use anyhow::{ensure, Context};
use serde_json::json;
use std::{fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() >= 2,
        "usage: readme_render OUTPUT_DIR APKG [UPDATE_APKG]"
    );
    let output = PathBuf::from(&args[0]);
    fs::create_dir_all(&output)?;
    let work = tempfile::tempdir()?;
    let collection = work.path().join("readme.anki2");
    let mut builder = CollectionBuilder::new(&collection);
    builder.with_desktop_media_paths();
    let mut col = builder.build()?;
    for package in &args[1..] {
        let imported = col.import_apkg(package, ImportAnkiPackageOptions::default())?;
        ensure!(imported.output.conflicting.is_empty(), "conflicting import");
    }
    let mut cards = vec![];
    for cid in col.search_cards("", SortMode::NoOrder)? {
        let card = col.storage.get_card(cid)?.context("card missing")?;
        let note = col
            .storage
            .get_note(card.note_id())?
            .context("note missing")?;
        let model = col
            .get_notetype(note.notetype_id)?
            .context("note type missing")?;
        let rendered = col.render_existing_card(cid, false, false)?;
        ensure!(!rendered.is_empty, "empty card");
        cards.push(json!({
            "notetype": model.name,
            "fields": note.fields(),
            "css": model.config.css,
            "question": rendered.question(),
            "answer": rendered.answer(),
        }));
    }
    cards.sort_by_key(|card| card["notetype"].as_str().unwrap().to_owned());
    for file in fs::read_dir(collection.with_extension("media"))? {
        let file = file?;
        if file.file_type()?.is_file() {
            fs::copy(file.path(), output.join(file.file_name()))?;
        }
    }
    let result = json!({"packages": args[1..], "cards": cards});
    fs::write(
        output.join("cards.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{}", output.join("cards.json").display());
    Ok(())
}
