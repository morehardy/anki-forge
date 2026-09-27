use ankiforge::schema::GenerationRule;
use ankiforge::{BuildOptions, Field, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::builder("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("expr").name("Expression").sort().required())
        .field(Field::new("meaning").name("Meaning").required())
        .template(
            Template::new("recognition")
                .name("Recognition")
                .front("{{expr}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .build()?;
    let mut project = Project::new("jp-core")?.default_deck("Japanese::Core");
    project.add(
        "taberu",
        vocab
            .note()
            .field("expr", "食べる")
            .field("meaning", "to eat"),
    )?;
    project.build(BuildOptions::to("jp-core.apkg"))?;
    Ok(())
}
