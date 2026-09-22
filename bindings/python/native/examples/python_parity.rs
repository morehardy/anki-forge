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

fn custom(path: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("Native")
        .stable_id("native-custom")
        .default_deck("Native::Custom");
    project.add_notetype(
        NoteType::custom("vocabulary")
            .name("Vocabulary")
            .css("\n.card {\n\tcolor: navy;\n}\n")
            .field(Field::new("Prompt").key("prompt").identity().sort())
            .field(Field::new("Answer").key("answer").required())
            .identity(IdentityRecipe::fields(["prompt"]))
            .template(
                Template::new("Forward")
                    .key("forward")
                    .front("\n{{Prompt}}\n")
                    .back("{{FrontSide}}<hr>{{Answer}}")
                    .browser_front("{{Prompt}}")
                    .browser_back("{{Answer}}")
                    .target_deck("Native::Forward")
                    .generate_when(GenerationRule::all(["prompt"])),
            )
            .template(
                Template::new("Reverse")
                    .key("reverse")
                    .front("{{Answer}}")
                    .back("{{Prompt}}"),
            ),
    )?;
    project.add_note(
        Note::new("vocabulary")
            .text("prompt", "<question>")
            .html("answer", "<b>答案</b>")
            .tag("vocab"),
    )?;
    project.write_apkg(path)?.ensure_success()?;
    inspect(path)
}

fn identity(path: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("Identity").stable_id("native-identity");
    project.add_notetype(
        NoteType::custom("identity")
            .name("Identity")
            .field(Field::new("Prompt").key("prompt").identity())
            .field(Field::new("Answer").key("answer"))
            .template(
                Template::new("Card")
                    .key("card")
                    .front("{{Prompt}}")
                    .back("{{Answer}}"),
            )
            .identity(IdentityRecipe::fields(["answer"])),
    )?;
    project.add_note(
        Note::new("identity")
            .text("prompt", "p1")
            .text("answer", "a1"),
    )?;
    project.add_note(
        Note::new("identity")
            .text("Prompt", "p2")
            .text("Answer", "a2")
            .identity(["prompt", "prompt"]),
    )?;
    project.add_note(
        Note::new("identity")
            .stable_id("explicit-3")
            .text("prompt", "p3")
            .text("answer", "a3")
            .identity(["prompt"]),
    )?;
    project.write_apkg(path)?.ensure_success()?;
    inspect(path)
}

fn cloze_io(path: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("Cloze and IO").stable_id("native-cloze-io");
    project.add_note(
        Note::cloze("中 {{c1::one}} and {{c2::two}}")
            .extra("More & less")
            .tag("stock"),
    )?;
    project.add_notetype(
        NoteType::custom_cloze("custom-cloze", "body")
            .name("Custom Cloze")
            .field(Field::new("Body").key("body").identity())
            .field(Field::new("Extra").key("extra").optional())
            .identity(IdentityRecipe::fields(["body"]))
            .template(
                Template::new("Cloze Card")
                    .key("cloze-card")
                    .front("{{cloze:Body}}")
                    .back("{{cloze:Body}}<hr>{{Extra}}")
                    .browser_front("{{Body}}")
                    .generate_when(GenerationRule::Cloze {
                        field: FieldKey::new("body"),
                    }),
            ),
    )?;
    project.add_note(
        Note::new("custom-cloze")
            .html("Body", "{{c1::custom::hint}} and {{c3::third}}")
            .text("extra", "extra"),
    )?;
    let media = project
        .media_mut()
        .add_bytes(
            "image",
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"></svg>".to_vec(),
        )?
        .export_as("image.svg")?;
    project.add_note(
        Note::image_occlusion(media)
            .stable_id("io-1")
            .rect(0, 0, 10, 10)
            .rect(10, 10, 5, 5)
            .header("Header")
            .back_extra("Extra")
            .comments("Comments")
            .tag("io")
            .build()?,
    )?;
    project.write_apkg(path)?.ensure_success()?;
    inspect(path)
}

fn media(path: &Path) -> anyhow::Result<Value> {
    let mut source = Project::new("Source");
    let reference = source
        .media_mut()
        .add_bytes("source", b"RIFF source".to_vec())?
        .export_as("voice.wav")?;
    let mut destination = Project::new("Destination").stable_id("native-media");
    destination
        .media_mut()
        .add_bytes("noise", b"RIFF noise".to_vec())?
        .export_as("noise.wav")?;
    destination
        .media_mut()
        .add_bytes("destination", b"RIFF destination".to_vec())?
        .export_as("voice.wav")?;
    destination.add_note(
        Note::basic("Question", "Answer")
            .stable_id("media-note")
            .sound("Back", reference),
    )?;
    destination.write_apkg(path)?.ensure_success()?;
    inspect(path)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(args.len() == 3, "usage: python_parity basic|inspect PATH");
    let value = match args[1].as_str() {
        "basic" => basic(Path::new(&args[2]))?,
        "custom" => custom(Path::new(&args[2]))?,
        "identity" => identity(Path::new(&args[2]))?,
        "cloze_io" => cloze_io(Path::new(&args[2]))?,
        "media" => media(Path::new(&args[2]))?,
        "inspect" => inspect(Path::new(&args[2]))?,
        _ => anyhow::bail!("unknown operation"),
    };
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}
