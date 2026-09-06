//! Repository-only independent Rust producer and APKG observer for SDK tests.
use std::path::Path;

use anki_forge::build::BuildReportJson;
use anki_forge::deck::{BasicNote, ClozeNote};
use anki_forge::prelude::*;
use serde_json::{json, Value};

#[path = "../src/json_numbers.rs"]
mod json_numbers;

const SVG: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"/>";

fn custom(field_name: &str, reverse: bool) -> anyhow::Result<Project> {
    let mut project = Project::new("Parity").stable_id("parity");
    let mut templates = vec![
        Template::new("Recognition")
            .key("recognition")
            .front(format!("{{{{{field_name}}}}}"))
            .back("{{FrontSide}}<hr>{{Meaning}}")
            .browser_front(format!("{{{{{field_name}}}}}"))
            .browser_back("{{Meaning}}")
            .target_deck("Parity::Target")
            .generate_when(GenerationRule::all(["expr"])),
        Template::new("Reverse")
            .key("reverse")
            .front("{{Meaning}}")
            .back(format!("{{{{{field_name}}}}}"))
            .generate_when(GenerationRule::any(["meaning"])),
    ];
    if reverse {
        templates.reverse();
    }
    let mut notetype = NoteType::custom("vocabulary")
        .name("Vocabulary")
        .field(Field::new(field_name).key("expr").identity().required())
        .field(Field::new("Meaning").key("meaning").sort().optional())
        .identity(IdentityRecipe::fields(["expr"]))
        .css(".card { color: navy; }");
    for template in templates {
        notetype = notetype.template(template);
    }
    project.add_notetype(notetype)?;
    project.add_note(
        Note::new("vocabulary")
            .text("expr", "<hola>")
            .html("meaning", "<b>hello</b>"),
    )?;
    Ok(project)
}

fn project(case: &str) -> anyhow::Result<Project> {
    let mut p = Project::new("Parity").stable_id("parity");
    match case {
        "stock" => {
            p = p.default_deck("Parity::Default");
            p.add_note(
                Note::basic("<hola>", "hello")
                    .stable_id("hello")
                    .tag("language"),
            )?;
            p.add_note(
                Note::cloze("{{c1::one}} {{c2::two}}")
                    .stable_id("numbers")
                    .extra("extra")
                    .deck("Parity::Cloze"),
            )?;
        }
        "normal" | "renamed" | "reordered" => {
            return custom(
                if case == "normal" {
                    "Expression"
                } else {
                    "Prompt"
                },
                case == "reordered",
            );
        }
        "custom-cloze" => {
            p.add_notetype(
                NoteType::custom_cloze("custom-cloze", "text")
                    .field(Field::new("Sentence").key("text").identity())
                    .field(Field::new("Extra").key("extra").optional())
                    .template(
                        Template::new("Cloze")
                            .front("{{cloze:Sentence}}")
                            .back("{{cloze:Sentence}} {{Extra}}")
                            .generate_when(GenerationRule::Cloze {
                                field: FieldKey::new("text"),
                            }),
                    ),
            )?;
            p.add_note(
                Note::new("custom-cloze")
                    .stable_id("sentence")
                    .html("text", "{{c1::one}} {{c2::two}}")
                    .text("extra", "extra"),
            )?;
        }
        "media" | "io" => {
            let image = p
                .media_mut()
                .add_bytes("diagram.svg", SVG.to_vec())?
                .export_as("diagram.svg")?;
            if case == "io" {
                p.add_note(
                    Note::image_occlusion(image)
                        .stable_id("diagram")
                        .rect(1, 2, 10, 20)
                        .rect(30, 40, 10, 20)
                        .header("Header")
                        .back_extra("Extra")
                        .comments("Comments")
                        .tag("image")
                        .build()?,
                )?;
            } else {
                let audio = p
                    .media_mut()
                    .add_bytes("voice.mp3", b"ID3\x04\0\0\0\0\0\0audio".to_vec())?
                    .export_as("voice.mp3")?;
                p.add_note(
                    Note::basic("", "")
                        .stable_id("media")
                        .image("Front", image)
                        .sound("Back", audio),
                )?;
            }
        }
        "deck" => {
            let mut deck = Deck::builder("Parity").stable_id("parity").build();
            deck.add(
                BasicNote::new("<hola>", "hello")
                    .stable_id("hello")
                    .tags(["language"]),
            )?;
            deck.add(
                ClozeNote::new("{{c1::one}} {{c2::two}}")
                    .stable_id("numbers")
                    .extra("extra"),
            )?;
            return Ok(Project::from(deck));
        }
        "bundle" => {
            let bundle = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../contracts/fixtures/template-bundle/custom-normal");
            p.import_template_bundle(bundle)?;
            p.add_note(
                Note::new("language-card")
                    .stable_id("bundle")
                    .text("prompt", "Question")
                    .text("extra", "Answer"),
            )?;
        }
        "revision-0" | "revision-1" | "revision-2" | "revision-3" | "revision-4" => {
            let answer = if case == "revision-1" { "B" } else { "A" };
            let mut note = Note::basic("Question", answer).stable_id("changed");
            if matches!(case, "revision-3" | "revision-4") {
                note = note.tag("new-tag");
            }
            p.add_note(note)?;
            p.add_note(
                Note::basic("Unchanged question", "Unchanged answer").stable_id("unchanged"),
            )?;
        }
        _ => anyhow::bail!("unknown scenario: {case}"),
    }
    Ok(p)
}

fn inspect(path: &Path) -> anyhow::Result<Value> {
    let report = anki_forge::writer::inspect_apkg(path)?;
    anyhow::ensure!(
        report.observation_status == "complete",
        "incomplete observation"
    );
    let identity =
        anki_forge::update_safety::baseline::load_previous_apkg_identity_index(path, None, None)?;
    Ok(
        json!({"observations": report.observations, "identity": identity, "missing_domains": report.missing_domains, "degradation_reasons": report.degradation_reasons}),
    )
}

fn suite(root: &Path) -> anyhow::Result<Value> {
    std::fs::create_dir_all(root)?;
    let mut result = serde_json::Map::new();
    for case in [
        "stock",
        "normal",
        "renamed",
        "reordered",
        "custom-cloze",
        "media",
        "io",
        "deck",
        "bundle",
        "revision-0",
        "revision-1",
        "revision-2",
        "revision-3",
        "revision-4",
    ] {
        let output = root.join(format!("{case}.apkg"));
        let mut options = BuildOptions::new().output(&output);
        if case == "renamed" {
            options = options.compare_to(root.join("normal.apkg"));
        }
        if case == "reordered" {
            options = options.compare_to(root.join("renamed.apkg"));
        }
        if case == "revision-0" {
            options = options.first_update_safe_build(root.join("identity.json"));
        }
        if case.starts_with("revision-") && case != "revision-0" {
            options = options
                .update_safe(root.join("identity.json"))
                .write_identity_lockfile(true);
        }
        let report = project(case)?.build(options)?;
        result.insert(
            case.into(),
            json!({"report": BuildReportJson::from_report(&report), "inspect": inspect(&output)?}),
        );
    }
    Ok(Value::Object(result))
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(args.len() == 3, "usage: sdk_parity suite|inspect PATH");
    let mut value = match args[1].as_str() {
        "suite" => suite(Path::new(&args[2]))?,
        "inspect" => inspect(Path::new(&args[2]))?,
        _ => anyhow::bail!("unknown operation"),
    };
    json_numbers::safe_json_numbers(&mut value);
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}
