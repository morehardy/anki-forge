use anki_forge::prelude::*;

const TINY_WAV: &[u8] = &[
    b'R', b'I', b'F', b'F', 37, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0, 0,
    0, 1, 0, 1, 0, 0x40, 0x1f, 0, 0, 0x40, 0x1f, 0, 0, 1, 0, 8, 0, b'd', b'a', b't', b'a', 1, 0, 0,
    0, 128,
];

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Spanish Media")
        .stable_id("spanish-media")
        .default_deck("Spanish::Media");
    let audio = project
        .media_mut()
        .add_bytes("hola-source.wav", TINY_WAV.to_vec())
        .export_as("hola.wav")?;

    project.add_note(
        Note::basic("hola", "hello")
            .stable_id("es:hola")
            .sound("Back", audio),
    )?;

    project.write_apkg("spanish-media.apkg")?.ensure_success()?;
    Ok(())
}
