use anki_forge::prelude::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let scenario = args.next().ok_or("missing scenario id")?;
    let bundle = PathBuf::from(args.next().ok_or("missing template bundle path")?);
    let output = PathBuf::from(args.next().ok_or("missing APKG output path")?);
    if args.next().is_some() {
        return Err("unexpected extra arguments".into());
    }

    let mut project = Project::new(&scenario)
        .stable_id(format!("manual:{scenario}"))
        .default_deck("Anki Forge Manual");
    project.import_template_bundle(bundle)?;

    match scenario.as_str() {
        "S10_custom_normal_bundle" => {
            project.add_note(
                Note::new("desktop-custom-normal")
                    .stable_id("manual:custom-normal:1")
                    .text("prompt", "Capital of Spain?")
                    .text("sort_key", "Spain"),
            )?;
        }
        "S11_custom_cloze_bundle" => {
            project.add_note(
                Note::new("desktop-custom-cloze")
                    .stable_id("manual:custom-cloze:1")
                    .text("text", "{{c1::Madrid}} is in {{c2::Spain}}"),
            )?;
        }
        _ => return Err(format!("unsupported template-bundle scenario '{scenario}'").into()),
    }

    let report = project.write_apkg(output)?;
    report.ensure_success()?;
    println!(
        "generated {} note(s), {} card(s), {} media item(s)",
        report.counts.notes, report.counts.cards, report.counts.media
    );
    Ok(())
}
