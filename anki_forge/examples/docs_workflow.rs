//! Executable source for the card, bundle, occlusion, media and update guides.
use ankiforge::note::Mask;
use ankiforge::update::CompareOptions;
use ankiforge::{BuildOptions, BuildOutput, Media, Note, NoteType, Project};
use anyhow::ensure;
use std::{fs, path::Path};

fn verify(output: &BuildOutput, notes: usize, cards: usize, media: usize) -> anyhow::Result<()> {
    let counts = output.report().counts();
    ensure!(counts.notes == notes && counts.cards == cards && counts.media == media);
    ensure!(output.artifact().path().is_file());
    Ok(())
}

// docs:cards:start
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("docs-spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    project.add(
        "capital",
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("back_extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("cards.apkg")))?;
    verify(&built, 2, 3, 0)?;
    Ok(())
}
// docs:cards:end

// docs:bundle:start
fn bundle(output: &Path, template_directory: &Path) -> anyhow::Result<()> {
    let model = NoteType::from_bundle(template_directory)?;
    let mut project = Project::new("docs-languages")?;
    project.add(
        "capital",
        model
            .note()
            .field("text", "{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("template.apkg")))?;
    verify(&built, 1, 2, 0)?;
    Ok(())
}
// docs:bundle:end

// docs:occlusion:start
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let image = Media::file(diagram)?;
    let note = Note::image_occlusion(image)
        .mask(Mask::rect("region-1", 10, 10, 40, 25))
        .build()?;
    let mut project = Project::new("docs-diagram")?.default_deck("Diagram");
    project.add("diagram", note)?;
    let built = project.build(BuildOptions::to(output.join("diagram.apkg")))?;
    verify(&built, 1, 1, 1)?;
    Ok(())
}
// docs:occlusion:end

// docs:media:start
fn media(output: &Path, image: &Path, sound: &Path) -> anyhow::Result<()> {
    let image = Media::file(image)?.with_export_name("cell.png")?;
    let sound = Media::file(sound)?;
    let mut project = Project::new("docs-media")?;
    project.add("cell", Note::basic(image.image(), sound.sound()))?;
    let css = Media::bytes(b".card { color: navy; }".to_vec(), "text/css")?
        .with_export_name("course.css")?;
    project.add_asset(css)?;
    let built = project.build(BuildOptions::to(output.join("media.apkg")))?;
    verify(&built, 1, 1, 3)?;
    Ok(())
}
// docs:media:end

// docs:updates:start
fn vocabulary(answer: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("docs-spanish-updates")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", answer))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<()> {
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    vocabulary("hello")?.build(BuildOptions::to(&previous))?;
    let project = vocabulary("hello; hi")?;
    let comparison = project.compare(CompareOptions::against(&previous))?;
    ensure!(comparison.policy().allows_publication());
    let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
    verify(&built, 1, 1, 0)?;
    ensure!(built.report().comparison().is_some());
    Ok(())
}
// docs:updates:end

fn main() -> anyhow::Result<()> {
    let destination = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/docs-examples".into());
    let output = Path::new(&destination);
    fs::create_dir_all(output)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let fixtures = root.join("anki_forge/tests/fixtures/public-api");
    cards(output)?;
    bundle(
        output,
        &root.join("contracts/fixtures/template-bundle/custom-cloze"),
    )?;
    occlusion(output, &fixtures.join("occlusion.png"))?;
    media(
        output,
        &fixtures.join("pixel.png"),
        &fixtures.join("silence.wav"),
    )?;
    updates(output)?;
    println!(
        "Verified cards, bundle, media, occlusion, comparison and update in {}",
        output.display()
    );
    Ok(())
}
