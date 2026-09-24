//! Reproducible README cards. Run from a checkout; no external media is needed.
use ankiforge::schema::GenerationRule;
use ankiforge::{BuildOptions, Field, Media, Note, NoteType, Project, Template};
use std::{f32::consts::TAU, fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("readme-showcase.apkg"));
    let mut project = Project::new("readme-showcase")?
        .name("anki-forge Showcase")
        .default_deck("anki-forge::Showcase");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    project.add(
        "sound:pitch",
        Note::cloze("A sound's pitch depends on its {{c1::frequency}}.")
            .field("back_extra", "More cycles per second, higher pitch."),
    )?;

    let points = (0..=320)
        .map(|x| format!("{x},{:.2}", 52.0 - 30.0 * (x as f32 * TAU / 80.0).sin()))
        .collect::<Vec<_>>()
        .join(" ");
    let wave = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 104"><path d="M0 52H320" stroke="#bfad98"/><polyline points="{points}" fill="none" stroke="#bd562e" stroke-width="3"/></svg>"##
    );
    let picture =
        Media::bytes(wave.into_bytes(), "image/svg+xml")?.with_export_name("waveform.svg")?;
    let audio = Media::bytes(concert_a(), "audio/wav")?.with_export_name("concert-a.wav")?;
    let model = NoteType::builder("ear-training")
        .name("Ear Training")
        .field(Field::new("prompt").name("Prompt"))
        .field(Field::new("answer").name("Answer").required())
        .field(Field::new("picture").name("Picture").required())
        .field(Field::new("audio").name("Audio").required())
        .template(
            Template::new("listen")
                .name("Listen")
                .front(concat!(
                    "<div class='eyebrow'>EAR TRAINING</div>",
                    "<div class='prompt'>{{prompt}}</div>",
                    "<div class='wave'>{{picture}}</div>",
                    "{{audio}}"
                ))
                .back("{{FrontSide}}<hr id='answer'><div class='meaning'>{{answer}}</div>")
                .generate_when(GenerationRule::all(["prompt"])),
        )
        .css(concat!(
            ".card { font-family: Arial, sans-serif; text-align: center; ",
            "background: #f7f1e7; color: #372c25; margin: 0; padding: 24px 18px; }",
            ".eyebrow { font-size: 11px; letter-spacing: 2px; color: #875d46; }",
            ".prompt { font-size: 23px; margin: 16px 0 0; }",
            ".wave img { width: 100%; max-width: 320px; height: 80px; margin: 10px 0; }",
            "audio { width: 100%; max-width: 290px; height: 34px; }",
            "#answer { border: 0; border-top: 1px solid #d8c9b9; margin: 20px 0 15px; }",
            ".meaning { font-size: 20px; }",
            ".card.nightMode { background: #2b241f; color: #f5e8d7; }",
            ".nightMode .eyebrow { color: #ddb08d; }"
        ))
        .build()?;
    project.add(
        "sound:a4",
        model
            .note()
            .field("prompt", "Name this pitch.")
            .field("answer", "A4 · 440 Hz")
            .field("picture", picture.image())
            .field("audio", audio.sound()),
    )?;
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let report = project.build(BuildOptions::to(&output))?;
    println!("{}", serde_json::to_string_pretty(&report.snapshot())?);
    Ok(())
}

fn concert_a() -> Vec<u8> {
    // One second of a quiet 440 Hz sine tone, with a short fade at either end.
    let rate = 8_000u32;
    let data_len = rate * 2;
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..rate {
        let fade = (i.min(rate - 1 - i) as f32 / 160.0).min(1.0);
        let sample = (5_000.0 * fade * (TAU * 440.0 * i as f32 / rate as f32).sin()) as i16;
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}
