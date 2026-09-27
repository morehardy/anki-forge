use ankiforge::{BuildOptions, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    project.build(BuildOptions::to("spanish.apkg"))?;
    Ok(())
}
