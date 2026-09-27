//! Build the website previews from actual public-API packages.
use ankiforge::schema::GenerationRule;
use ankiforge::update::CompareOptions;
use ankiforge::{BuildOptions, BuildOutput, Field, Media, Note, NoteType, Project, Template};
use anyhow::ensure;
use serde_json::{json, Value};
use std::{f32::consts::TAU, fs, path::Path};

fn verified(output: &BuildOutput, media: usize) -> anyhow::Result<Value> {
    let counts = output.report().counts();
    ensure!(counts.notes == 1 && counts.cards == 1 && counts.media == media);
    Ok(json!({"notes": counts.notes, "cards": counts.cards, "media": counts.media}))
}

fn basic(output: &Path) -> anyhow::Result<Value> {
    // website:basic:start
    let (front, back) = ("hola", "hello");
    let mut project = Project::new("website-spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic(front, back))?;
    let built = project.build(BuildOptions::to(output.join("spanish.apkg")))?;
    // website:basic:end
    Ok(
        json!({"id":"basic", "label":"Basic", "deck":"Spanish", "stableId":"es:hola",
        "front":front, "back":back, "file":"spanish.apkg", "counts":verified(&built, 0)?}),
    )
}

fn cloze(output: &Path) -> anyhow::Result<Value> {
    // website:cloze:start
    let text = "A sound's pitch depends on its {{c1::frequency}}.";
    let extra = "More cycles per second, higher pitch.";
    let mut project = Project::new("website-sound")?.default_deck("Sound");
    project.add("sound:pitch", Note::cloze(text).field("back_extra", extra))?;
    let built = project.build(BuildOptions::to(output.join("sound.apkg")))?;
    // website:cloze:end
    Ok(
        json!({"id":"cloze", "label":"Cloze", "deck":"Sound", "stableId":"sound:pitch",
        "front":text, "back":extra, "file":"sound.apkg", "counts":verified(&built,0)?}),
    )
}

fn media(output: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("website-ear-training")?.default_deck("Ear Training");
    fs::write(output.join("waveform.svg"), waveform())?;
    fs::write(output.join("concert-a.wav"), concert_a())?;
    let model = NoteType::builder("ear-training")
        .name("Ear Training")
        .field(Field::new("prompt").name("Prompt"))
        .field(Field::new("answer").name("Answer").required())
        .field(Field::new("picture").name("Picture").required())
        .field(Field::new("audio").name("Audio").required())
        .template(
            Template::new("listen")
                .name("Listen")
                .front("<h2>{{prompt}}</h2><div>{{picture}}</div>{{audio}}")
                .back("{{FrontSide}}<hr id='answer'><h2>{{answer}}</h2>")
                .generate_when(GenerationRule::all(["prompt"])),
        )
        .css(concat!(
            ".card {font:20px/1.6 Arial,sans-serif;text-align:center;}",
            "h2{font-size:24px;font-weight:500} img{width:100%;max-width:320px;}",
            "hr{border:0;border-top:1px solid #a3b9b1;margin:24px 0}"
        ))
        .build()?;
    // website:media:start
    let (prompt, answer) = ("Name this pitch.", "A4, 440 Hz");
    let picture = Media::file(output.join("waveform.svg"))?.with_export_name("waveform.svg")?;
    let audio = Media::file(output.join("concert-a.wav"))?.with_export_name("concert-a.wav")?;
    project.add(
        "sound:a4",
        model
            .note()
            .field("prompt", prompt)
            .field("answer", answer)
            .field("picture", picture.image())
            .field("audio", audio.sound()),
    )?;
    let built = project.build(BuildOptions::to(output.join("ear-training.apkg")))?;
    // website:media:end
    Ok(
        json!({"id":"media", "label":"Media", "deck":"Ear Training", "stableId":"sound:a4",
        "front":prompt, "back":answer, "image":"waveform.svg", "audio":"concert-a.wav",
        "file":"ear-training.apkg", "counts":verified(&built,2)?}),
    )
}

fn spanish(back: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("website-spanish-updates")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", back))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<Value> {
    let (before, after) = ("hello", "hello; hi");
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    spanish(before)?.build(BuildOptions::to(&previous))?;
    let project = spanish(after)?;
    // website:updates:start
    let comparison = project.compare(CompareOptions::against(&previous))?;
    ensure!(comparison.policy().allows_publication());
    let built = project.build(BuildOptions::to(&next).update_from(&previous))?;
    // website:updates:end
    let counts = verified(&built, 0)?;
    ensure!(built.report().comparison().is_some());
    Ok(
        json!({"front":"hola", "before":before, "after":after, "stableId":"es:hola",
        "previous":"spanish-v1.apkg", "next":"spanish-v2.apkg", "notesPreserved":1, "counts":counts}),
    )
}

fn main() -> anyhow::Result<()> {
    let directory = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("pass an output directory"))?;
    let output = Path::new(&directory);
    fs::create_dir_all(output)?;
    let manifest = json!({"schemaVersion":1, "crateVersion":ankiforge::facade_api_version(),
        "examples":[basic(output)?,cloze(output)?,media(output)?], "update":updates(output)?});
    fs::write(
        output.join("showcase.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "Built 3 card examples and 2 update packages in {}",
        output.display()
    );
    Ok(())
}

fn waveform() -> String {
    let points = (0..=320)
        .map(|x| format!("{x},{:.2}", 52.0 - 30.0 * (x as f32 * TAU / 80.0).sin()))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 104"><style>polyline{{stroke:#116e60}}@media(prefers-color-scheme:dark){{polyline{{stroke:#91d5c1}}}}</style><title>Illustrative waveform of a synthesized tone</title><polyline points="{points}" fill="none" stroke-width="3"/></svg>"##
    )
}

fn concert_a() -> Vec<u8> {
    // A quiet, one-second 440 Hz sine wave, faded at either end.
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
