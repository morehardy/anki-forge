//! Keep the first distributed APKG as evidence when publishing a revision.
use anki_forge::prelude::*;
use std::{fs, path::PathBuf};

fn spanish(meaning: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("Spanish")
        .stable_id("spanish")
        .default_deck("Spanish");
    project.add_note(Note::basic("hola", meaning).stable_id("es:hola"))?;
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

    spanish("hello")?.write_apkg(&previous)?.ensure_success()?;
    let report =
        spanish("hello; hi")?.build(BuildOptions::new().output(&next).compare_to(&previous))?;
    report.ensure_success()?;
    println!("{}", report.pretty_report());
    Ok(())
}
