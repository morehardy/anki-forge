use ankiforge::{BuildOptions, NoteType, Project};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let scenario = args.next().ok_or("missing scenario id")?;
    let bundle = PathBuf::from(args.next().ok_or("missing template bundle path")?);
    let output = PathBuf::from(args.next().ok_or("missing APKG output path")?);
    if args.next().is_some() {
        return Err("unexpected extra arguments".into());
    }

    let mut project = Project::new(format!("manual:{scenario}"))?.default_deck("Anki Forge Manual");
    let model = NoteType::from_bundle(bundle)?;

    match scenario.as_str() {
        "S10_custom_normal_bundle" => {
            project.add(
                "manual:custom-normal:1",
                model
                    .note()
                    .field("prompt", "Capital of Spain?")
                    .field("sort_key", "Spain"),
            )?;
        }
        "S11_custom_cloze_bundle" => {
            project.add(
                "manual:custom-cloze:1",
                model
                    .note()
                    .field("text", "{{c1::Madrid}} is in {{c2::Spain}}"),
            )?;
        }
        _ => return Err(format!("unsupported template-bundle scenario '{scenario}'").into()),
    }

    let report = project.build(BuildOptions::to(output))?;
    let counts = report.report().counts();
    println!(
        "generated {} note(s), {} card(s), {} media item(s)",
        counts.notes, counts.cards, counts.media
    );
    Ok(())
}
