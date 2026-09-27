//! Keep the first distributed APKG as evidence when publishing a revision.
use ankiforge::{BuildOptions, Note, Project};
use std::{fs, path::PathBuf};

fn spanish(meaning: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", meaning))?;
    Ok(project)
}

fn main() -> anyhow::Result<()> {
    let directory = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("readme-updates"));
    fs::create_dir_all(&directory)?;
    let previous = directory.join("spanish-v1.apkg");
    let next = directory.join("spanish-v2.apkg");

    spanish("hello")?.build(BuildOptions::to(&previous))?;
    let report = spanish("hello; hi")?.build(BuildOptions::to(&next).update_from(&previous))?;
    println!("{}", serde_json::to_string_pretty(&report.snapshot())?);
    Ok(())
}
