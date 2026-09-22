//! Independent Rust producer. The observer is deliberately shared by both producers.
use anki_forge::deck::{BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection};
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

fn names(path: &Path, scenario: &str) -> anyhow::Result<Value> {
    let (field_name, template_name, explicit_key) = match scenario {
        "unicode" => ("中文", "卡片", None),
        "spaces" => (" Prompt ", "Card\nOne", None),
        "punctuation" => ("C++  Prompt", "Card\tOne", None),
        "explicit_whitespace" => ("Prompt", "Card", Some(" key\t ")),
        "explicit_empty" => ("中文", "Card", Some("")),
        _ => anyhow::bail!("unknown names scenario"),
    };
    let mut field = Field::new(field_name).identity();
    if let Some(key) = explicit_key {
        field = field.key(key);
    }
    let key = field.key_ref().as_str().to_string();
    let mut project = Project::new("Names").stable_id("native-names");
    project.add_notetype(
        NoteType::custom("names")
            .name("  Names  ")
            .field(field)
            .identity(IdentityRecipe::fields([&key]))
            .template(
                Template::new(template_name)
                    .front("Question")
                    .back("Answer")
                    .target_deck(" Names:: Cards ")
                    .generate_when(GenerationRule::all([&key])),
            ),
    )?;
    project.add_note(Note::new("names").text(field_name, "内容").identity([&key]))?;
    project.write_apkg(path)?.ensure_success()?;
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

fn large(path: &Path, media_path: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("Large").stable_id("native-large");
    let media = project
        .media_mut()
        .add_file(media_path)?
        .export_as("large.bin")?;
    project.add_note(
        Note::basic("Question", "Answer")
            .stable_id("one")
            .sound("Back", media),
    )?;
    project.write_apkg(path)?.ensure_success()?;
    inspect(path)
}

fn bundle(path: &Path, bundle: &Path, cloze: bool) -> anyhow::Result<Value> {
    let mut project = Project::new("Bundle")
        .stable_id("native-bundle")
        .default_deck("Bundle");
    let (note_type, template, note) = if cloze {
        (
            NoteType::custom_cloze("bundle-card", "prompt"),
            Template::new("Card")
                .front("{{cloze:Prompt}}")
                .back("{{cloze:Prompt}}<hr>{{Extra}}<img src=\"icon.svg\">"),
            Note::new("bundle-card").html("prompt", "{{c1::一}} and {{c2::two}}"),
        )
    } else {
        (
            NoteType::custom("bundle-card"),
            Template::new("Card")
                .front(" \r\n<section>{{Prompt}}</section>\n")
                .back("{{Prompt}}<hr>{{Extra}}<img src=\"icon.svg\">")
                .generate_when(GenerationRule::all(["prompt"])),
            Note::new("bundle-card").text("prompt", "中 & prompt"),
        )
    };
    project.add_notetype(
        note_type.name("Bundle Card")
            .field(Field::new("Prompt").key("prompt").identity().required())
            .field(Field::new("Extra").key("extra").optional())
            .identity(IdentityRecipe::fields(["prompt"]))
            .css("\n@font-face { font-family: Bundle; src: url(font.woff2); }\n.card { background-image: url(icon.svg); }\n")
            .template(template.key("card")
                .browser_front("{{Prompt}}")
                .target_deck("Bundle::Cards")),
    )?;
    for name in ["icon.svg", "font.woff2"] {
        project
            .media_mut()
            .add_file(bundle.join("assets").join(name))?
            .export_as(name)?;
    }
    project.add_note(note.text("extra", "Extra"))?;
    project.write_apkg(path)?.ensure_success()?;
    inspect(path)
}

fn deck(path: &Path, image: &Path, convert: bool) -> anyhow::Result<Value> {
    let mut deck = Deck::builder("Native Deck")
        .stable_id("native-deck")
        .basic_identity(BasicIdentitySelection::new([BasicIdentityField::Front])?)
        .build();
    deck.basic()
        .note("<b>Front</b>", "Answer")
        .tags(["basic"])
        .add()?;
    deck.basic()
        .note("Second", "<i>Identity answer</i>")
        .identity_override(BasicIdentityOverride::new(
            [BasicIdentityField::Back],
            "test-answer-key",
        )?)
        .add()?;
    deck.cloze()
        .note("{{c1::one}} and {{c2::two}}")
        .extra("Extra")
        .tags(["cloze"])
        .add()?;
    let image = deck.media().add(MediaSource::from_file(image))?;
    deck.image_occlusion()
        .note(image)
        .rect(0, 0, 1, 1)
        .header("Header")
        .back_extra("Extra")
        .comments("Comments")
        .tags(["io"])
        .add()?;
    if convert {
        let mut project = Project::from(deck);
        project.add_notetype(
            NoteType::custom("extra")
                .name("extra")
                .field(Field::new("Question").key("q").identity())
                .identity(IdentityRecipe::fields(["q"]))
                .template(
                    Template::new("Card")
                        .key("card")
                        .front("{{Question}}")
                        .back("{{FrontSide}}"),
                ),
        )?;
        project.add_note(Note::new("extra").text("q", "custom"))?;
        project.write_apkg(path)?.ensure_success()?;
    } else {
        deck.write_apkg(path)?.ensure_success()?;
    }
    inspect(path)
}

fn evolution(path: &Path, mode: &str) -> anyhow::Result<Value> {
    fn project(changed: bool, mode: &str) -> anyhow::Result<Project> {
        let mut project = Project::new("Evolution").stable_id("native-evolution");
        let name = if changed && mode != "reorder" {
            "Renamed Prompt"
        } else {
            "Prompt"
        };
        let prompt = Field::new(name).key("prompt").identity().sort();
        let answer = Field::new("Answer").key("answer");
        let note_type = NoteType::custom("evolution")
            .name("Evolution")
            .identity(IdentityRecipe::fields(["prompt"]));
        let note_type = if changed {
            note_type.field(answer).field(prompt)
        } else {
            note_type.field(prompt).field(answer)
        };
        project.add_notetype(
            note_type.template(
                Template::new("Card")
                    .key("card")
                    .front(format!("{{{{{name}}}}}"))
                    .back("{{Answer}}"),
            ),
        )?;
        let note = Note::new("evolution")
            .text("prompt", "Question")
            .text("answer", "Answer");
        project.add_note(if mode == "explicit" {
            note.stable_id("existing")
        } else {
            note
        })?;
        Ok(project)
    }
    let baseline = path.with_extension("baseline.apkg");
    project(false, mode)?
        .write_apkg(&baseline)?
        .ensure_success()?;
    project(true, mode)?
        .build(BuildOptions::new().output(path).compare_to(&baseline))?
        .ensure_success()?;
    inspect(path)
}

fn diff(path: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("Diff").stable_id("native-diff");
    project.add_note(Note::basic("Front", "changed").stable_id("one").tag("diff"))?;
    let report = project
        .diff_against_apkg(path)
        .map_err(|error| anyhow::anyhow!("{error:?}"))?;
    let mut value = serde_json::to_value(&report)?;
    value["diagnostics"] = json!(report
        .diagnostics
        .iter()
        .map(anki_forge::build::json_report::DiagnosticJson::from)
        .collect::<Vec<_>>());
    value["failure_cause"] = Value::Null;
    Ok(value)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(
        args.len() >= 3,
        "usage: python_parity SCENARIO PATH [MEDIA]"
    );
    let value = match args[1].as_str() {
        "basic" => basic(Path::new(&args[2]))?,
        "names" => names(
            Path::new(&args[2]),
            args.get(3)
                .ok_or_else(|| anyhow::anyhow!("names scenario required"))?,
        )?,
        "custom" => custom(Path::new(&args[2]))?,
        "identity" => identity(Path::new(&args[2]))?,
        "cloze_io" => cloze_io(Path::new(&args[2]))?,
        "media" => media(Path::new(&args[2]))?,
        "bundle" | "bundle_cloze" => bundle(
            Path::new(&args[2]),
            Path::new(
                args.get(3)
                    .ok_or_else(|| anyhow::anyhow!("bundle path required"))?,
            ),
            args[1] == "bundle_cloze",
        )?,
        "large" => large(
            Path::new(&args[2]),
            Path::new(
                args.get(3)
                    .ok_or_else(|| anyhow::anyhow!("media path required"))?,
            ),
        )?,
        "inspect" => inspect(Path::new(&args[2]))?,
        "diff" => diff(Path::new(&args[2]))?,
        "evolution_rename" | "evolution_explicit" | "evolution_reorder" => evolution(
            Path::new(&args[2]),
            args[1].strip_prefix("evolution_").unwrap(),
        )?,
        "deck" | "deck_project" => deck(
            Path::new(&args[2]),
            Path::new(
                args.get(3)
                    .ok_or_else(|| anyhow::anyhow!("image path required"))?,
            ),
            args[1] == "deck_project",
        )?,
        _ => anyhow::bail!("unknown operation"),
    };
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}
