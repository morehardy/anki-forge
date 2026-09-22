//! Executable source for the card, template, image-occlusion and update guides.
//! cargo run -p anki_forge --example docs_workflow -- target/docs-examples
use anki_forge::prelude::*;
use anyhow::ensure;
use std::{fs, path::Path};

fn verify(report: &BuildReport, notes: usize, cards: usize, media: usize) -> anyhow::Result<()> {
    report.ensure_success()?;
    let inspected = report.inspect.as_ref().expect("package inspection");
    ensure!(inspected.notes == notes && inspected.cards == cards && inspected.media == media);
    Ok(())
}

// docs:cards:start
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("Spanish")
        .stable_id("docs-spanish")
        .default_deck("Spanish");
    project.add_note(Note::basic("hola", "hello").stable_id("es:hola"))?;
    project.add_note(
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .extra("A city and its country.")
            .stable_id("es:capital"),
    )?;
    project.validate().ensure_success()?;
    let report = project.write_apkg(output.join("cards.apkg"))?;
    report.ensure_success()?;
    verify(&report, 2, 3, 0)?;
    Ok(())
}
// docs:cards:end

// docs:bundle:start
fn bundle(output: &Path, template_directory: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("Languages").stable_id("docs-languages");
    project.import_template_bundle(template_directory)?;
    project.add_note(
        Note::new("language-cloze")
            .stable_id("es:capital")
            .text("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .text("extra", "A city and its country."),
    )?;
    let report = project.write_apkg(output.join("template.apkg"))?;
    report.ensure_success()?;
    verify(&report, 1, 2, 0)?;
    Ok(())
}
// docs:bundle:end

// docs:occlusion:start
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let mut deck = Deck::new("Diagram");
    let image = deck.media().add(MediaSource::from_file(diagram))?;
    deck.image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 10, 40, 25)
        .stable_id("diagram:region-1")
        .add()?;
    let report = deck.write_apkg(output.join("diagram.apkg"))?;
    report.ensure_success()?;
    verify(&report, 1, 1, 1)?;
    Ok(())
}
// docs:occlusion:end

// docs:updates:start
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("Spanish")
        .stable_id("docs-spanish-updates")
        .default_deck("Spanish");
    project.add_note(Note::basic("hola", answer).stable_id("es:hola"))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?
        .write_apkg(&previous)?
        .ensure_success()?;
    let report =
        vocabulary("hello; hi")?.build(BuildOptions::new().output(&next).compare_to(&previous))?;
    report.ensure_success()?;
    verify(&report, 1, 1, 0)?;
    ensure!(report.diff.is_some());
    let safety = report.update_safety.as_ref().expect("baseline evidence");
    ensure!(safety.notes_preserved == 1 && safety.notes_failed == 0);
    Ok(())
}
// docs:updates:end

// docs:lockfile:start
fn lockfile_updates(output: &Path) -> anyhow::Result<()> {
    let lockfile = output.join("identity.lock.json");
    vocabulary("hello")?
        .build(
            BuildOptions::new()
                .output(output.join("locked-v1.apkg"))
                .first_update_safe_build(&lockfile),
        )?
        .ensure_success()?;
    let report = vocabulary("hello; hi")?.build(
        BuildOptions::new()
            .output(output.join("locked-v2.apkg"))
            .update_safe(&lockfile)
            .write_identity_lockfile(true),
    )?;
    report.ensure_success()?;
    verify(&report, 1, 1, 0)?;
    let safety = report.update_safety.as_ref().expect("lockfile evidence");
    ensure!(safety.notes_preserved == 1 && safety.notes_failed == 0);
    ensure!(lockfile.is_file());
    Ok(())
}
// docs:lockfile:end

fn main() -> anyhow::Result<()> {
    let destination = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/docs-examples".into());
    let output = Path::new(&destination);
    fs::create_dir_all(output)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository");
    cards(output)?;
    bundle(
        output,
        &root.join("contracts/fixtures/template-bundle/custom-cloze"),
    )?;
    occlusion(
        output,
        &root.join("contracts/fixtures/phase3/inputs/assets/occlusion.png"),
    )?;
    updates(output)?;
    lockfile_updates(output)?;
    println!(
        "Verified card, template, occlusion, APKG baseline and lockfile examples in {}",
        output.display()
    );
    Ok(())
}
