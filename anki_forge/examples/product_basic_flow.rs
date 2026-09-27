use ankiforge::{BuildOptions, Content, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("example-doc")?.default_deck("Default");
    project.add("note-1", Note::basic("front", Content::html("<hr>back")))?;
    let output = project.build(BuildOptions::temporary())?;
    println!("{}", serde_json::to_string_pretty(&output.snapshot())?);
    Ok(())
}
