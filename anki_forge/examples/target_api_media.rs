use ankiforge::{BuildOptions, Field, Media, NoteType, Project, Template};

const TINY_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

fn tiny_wav(sample: u8) -> Vec<u8> {
    vec![
        b'R', b'I', b'F', b'F', 38, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0,
        0, 0, 1, 0, 1, 0, 0x40, 0x1f, 0, 0, 0x40, 0x1f, 0, 0, 1, 0, 8, 0, b'd', b'a', b't', b'a',
        1, 0, 0, 0, sample, 0,
    ]
}

fn main() -> anyhow::Result<()> {
    let audio = Media::bytes(tiny_wav(128), "audio/wav")?.with_export_name("hola.wav")?;
    let picture = Media::bytes(TINY_PNG.to_vec(), "image/png")?.with_export_name("hola.png")?;
    let vocab = NoteType::builder("spanish-vocab")
        .field(Field::new("expression").name("Expression").required())
        .field(Field::new("meaning").name("Meaning").required())
        .field(Field::new("audio").name("Audio"))
        .field(Field::new("picture").name("Picture"))
        .template(
            Template::new("recognition")
                .front("{{expression}}")
                .back("{{FrontSide}}<hr>{{meaning}}<div>{{audio}}{{picture}}</div>"),
        )
        .asset(picture.clone())
        .css(".card { background-image: url('hola.png'); } .card img { max-width: 120px; }")
        .build()?;
    let mut project = Project::new("spanish-media")?.default_deck("Spanish::Media");
    project.add(
        "hola",
        vocab
            .note()
            .field("expression", "hola")
            .field("meaning", "hello")
            .field("audio", audio.sound())
            .field("picture", picture.image()),
    )?;
    // Explicit assets remain in the package even without a statically visible reference.
    project.add_asset(Media::bytes(tiny_wav(127), "audio/wav")?.with_export_name("hint.wav")?)?;
    let output = project.build(BuildOptions::to("spanish-media.apkg"))?;
    assert_eq!(output.report().counts().media, 3);
    println!("{}", serde_json::to_string_pretty(&output.snapshot())?);
    Ok(())
}
