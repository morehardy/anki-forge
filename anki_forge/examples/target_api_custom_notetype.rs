use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .identity(IdentityRecipe::fields(["expr"]));

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(vocab)?;
    project.add_note(
        Note::new("jp-vocab")
            .stable_id("jp-vocab:taberu")
            .text("expr", "食べる")
            .text("meaning", "to eat"),
    )?;

    project.write_apkg("jp-core.apkg")?.ensure_success()?;
    Ok(())
}
