use anki_forge::prelude::*;

const TINY_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

fn tiny_wav(sample: u8) -> Vec<u8> {
    vec![
        b'R', b'I', b'F', b'F', 37, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0,
        0, 0, 1, 0, 1, 0, 0x40, 0x1f, 0, 0, 0x40, 0x1f, 0, 0, 1, 0, 8, 0, b'd', b'a', b't', b'a',
        1, 0, 0, 0, sample,
    ]
}

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Spanish Media")
        .stable_id("spanish-media")
        .default_deck("Spanish::Media");

    let audio = project
        .media_mut()
        .add_bytes("hola-source.wav", tiny_wav(128))?
        .export_as("hola.wav")?;
    let picture = project
        .media_mut()
        .add_bytes("hola-picture-source.png", TINY_PNG.to_vec())?
        .export_as("hola.png")?;
    project
        .media_mut()
        .add_bytes("unused-hint-source.wav", tiny_wav(127))?
        .export_as("unused-hint.wav")?;

    let vocab = NoteType::custom("spanish-vocab")
        .name("Spanish Vocabulary")
        .field(Field::new("Expression").key("expression").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .field(Field::new("Audio").key("audio").optional())
        .field(Field::new("Picture").key("picture").optional())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front(r#"<img class="deck-logo" src="hola.png" alt=""> {{Expression}}"#)
                .back(
                    r#"{{FrontSide}}<hr id="answer">{{Meaning}}<div class="media">{{Audio}}{{Picture}}</div>"#,
                )
                .generate_when(GenerationRule::all(["expression"])),
        )
        .css(
            r#".card { font-family: Arial, sans-serif; background-image: url("hola.png"); }
.deck-logo { width: 32px; height: 32px; }
.media img { max-width: 120px; }"#,
        )
        .identity(IdentityRecipe::fields(["expression"]));

    project.add_notetype(vocab)?;
    project.add_note(
        Note::new("spanish-vocab")
            .stable_id("es:hola")
            .text("expression", "hola")
            .text("meaning", "hello")
            .sound("audio", audio)
            .image("picture", picture),
    )?;

    let report = project.write_apkg("spanish-media.apkg")?;
    println!("{}", report.pretty_report());

    report.ensure_success()?;
    assert_eq!(report.media.unused_bindings, 1);
    assert!(report
        .diagnostic_codes()
        .iter()
        .any(|code| code == "MEDIA.UNUSED_BINDING"));
    Ok(())
}
