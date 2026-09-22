//! Independent Rust producer. The observer is deliberately shared by both producers.
use anki_forge::prelude::*;
use serde_json::{json, Value};
use std::path::Path;

fn inspect(path: &Path) -> anyhow::Result<Value> {
    let report = anki_forge::writer::inspect_apkg(path)?;
    anyhow::ensure!(
        report.observation_status == "complete",
        "incomplete observation"
    );
    let identity =
        anki_forge::update_safety::baseline::load_previous_apkg_identity_index(path, None, None)?;
    Ok(
        json!({"observations": report.observations, "identity": identity,
        "missing_domains": report.missing_domains, "degradation_reasons": report.degradation_reasons}),
    )
}

fn basic(path: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("Native").stable_id("native-basic");
    project.add_note(Note::basic("Front", "Back").stable_id("note-1"))?;
    let report = project.write_apkg(path)?;
    report.ensure_success()?;
    inspect(path)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(args.len() == 3, "usage: python_parity basic|inspect PATH");
    let value = match args[1].as_str() {
        "basic" => basic(Path::new(&args[2]))?,
        "inspect" => inspect(Path::new(&args[2]))?,
        _ => anyhow::bail!("unknown operation"),
    };
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}
