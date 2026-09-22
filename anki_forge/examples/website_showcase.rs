//! Build the website's examples and their shared preview fields.
//! Run: cargo run -p anki_forge --example website_showcase -- target/website-examples
use anki_forge::prelude::*;
use anyhow::ensure;
use serde_json::{json, Value};
use std::{f32::consts::TAU, fs, path::Path};

fn verified(report: &BuildReport, media: usize) -> anyhow::Result<Value> {
    report.ensure_success()?;
    ensure!(report.counts.notes == 1 && report.counts.cards == 1);
    ensure!(report.counts.media == media);
    let inspect = report
        .inspect
        .as_ref()
        .expect("successful build inspection");
    ensure!(inspect.notes == 1 && inspect.cards == 1 && inspect.media == media);
    Ok(json!({ "notes": inspect.notes, "cards": inspect.cards, "media": inspect.media }))
}

fn basic(output: &Path) -> anyhow::Result<Value> {
    // website:basic:start
    let (front, back) = ("hola", "hello");
    let mut deck = Deck::new("Spanish");
    deck.basic().note(front, back).stable_id("es:hola").add()?;
    let report = deck.write_apkg(output.join("spanish.apkg"))?;
    report.ensure_success()?;
    // website:basic:end
    Ok(json!({
        "id": "basic", "label": "Basic", "deck": "Spanish", "stableId": "es:hola",
        "front": front, "back": back, "file": "spanish.apkg", "counts": verified(&report, 0)?
    }))
}

fn cloze(output: &Path) -> anyhow::Result<Value> {
    // website:cloze:start
    let text = "A sound's pitch depends on its {{c1::frequency}}.";
    let extra = "More cycles per second, higher pitch.";
    let mut project = Project::new("Sound")
        .stable_id("website-sound")
        .default_deck("Sound");
    project.add_note(Note::cloze(text).extra(extra).stable_id("sound:pitch"))?;
    let report = project.write_apkg(output.join("sound.apkg"))?;
    report.ensure_success()?;
    // website:cloze:end
    Ok(json!({
        "id": "cloze", "label": "Cloze", "deck": "Sound", "stableId": "sound:pitch",
        "front": text, "back": extra, "file": "sound.apkg", "counts": verified(&report, 0)?
    }))
}

fn media(output: &Path) -> anyhow::Result<Value> {
    let mut project = Project::new("Ear Training")
        .stable_id("website-ear-training")
        .default_deck("Ear Training");
    let wave = waveform();
    let tone = concert_a();
    fs::write(output.join("waveform.svg"), &wave)?;
    fs::write(output.join("concert-a.wav"), &tone)?;
    project.add_notetype(
        NoteType::custom("ear-training")
            .name("Ear Training")
            .field(Field::new("Prompt").key("prompt").identity())
            .field(Field::new("Answer").key("answer").required())
            .field(Field::new("Picture").key("picture").required())
            .field(Field::new("Audio").key("audio").required())
            .template(
                Template::new("Listen")
                    .key("listen")
                    .front("<h2>{{Prompt}}</h2><div>{{Picture}}</div>{{Audio}}")
                    .back("{{FrontSide}}<hr id='answer'><h2>{{Answer}}</h2>")
                    .generate_when(GenerationRule::all(["prompt"])),
            )
            .identity(IdentityRecipe::fields(["prompt"]))
            .css(concat!(
                ".card {font:20px/1.6 Arial,sans-serif;text-align:center;}",
                "h2{font-size:24px;font-weight:500} img{width:100%;max-width:320px;}",
                "hr{border:0;border-top:1px solid #a3b9b1;margin:24px 0}"
            )),
    )?;
    // website:media:start
    let (prompt, answer) = ("Name this pitch.", "A4, 440 Hz");
    let picture = project
        .media_mut()
        .add_file(output.join("waveform.svg"))?
        .export_as("waveform.svg")?;
    let audio = project
        .media_mut()
        .add_file(output.join("concert-a.wav"))?
        .export_as("concert-a.wav")?;
    project.add_note(
        Note::new("ear-training")
            .stable_id("sound:a4")
            .text("prompt", prompt)
            .text("answer", answer)
            .image("picture", picture)
            .sound("audio", audio),
    )?;
    let report = project.write_apkg(output.join("ear-training.apkg"))?;
    report.ensure_success()?;
    // website:media:end
    Ok(json!({
        "id": "media", "label": "Media", "deck": "Ear Training", "stableId": "sound:a4",
        "front": prompt, "back": answer, "image": "waveform.svg", "audio": "concert-a.wav",
        "file": "ear-training.apkg", "counts": verified(&report, 2)?
    }))
}

fn spanish(back: &str) -> anyhow::Result<Project> {
    let mut project = Project::new("Spanish")
        .stable_id("website-spanish-updates")
        .default_deck("Spanish");
    project.add_note(Note::basic("hola", back).stable_id("es:hola"))?;
    Ok(project)
}

fn updates(output: &Path) -> anyhow::Result<Value> {
    let (before, after) = ("hello", "hello; hi");
    let previous = output.join("spanish-v1.apkg");
    let next = output.join("spanish-v2.apkg");
    spanish(before)?.write_apkg(&previous)?.ensure_success()?;
    let project = spanish(after)?;
    // website:updates:start
    let report = project.build(BuildOptions::new().output(&next).compare_to(&previous))?;
    report.ensure_success()?;
    // website:updates:end
    let counts = verified(&report, 0)?;
    let safety = report
        .update_safety
        .as_ref()
        .expect("baseline reconciliation");
    ensure!(safety.notes_preserved == 1 && safety.notes_failed == 0);
    ensure!(report.diff.is_some());
    Ok(json!({
        "front": "hola", "before": before, "after": after, "stableId": "es:hola",
        "previous": "spanish-v1.apkg", "next": "spanish-v2.apkg",
        "notesPreserved": safety.notes_preserved, "counts": counts
    }))
}

fn main() -> anyhow::Result<()> {
    let directory = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("pass an output directory"))?;
    let output = Path::new(&directory);
    fs::create_dir_all(output)?;
    let examples = [basic(output)?, cloze(output)?, media(output)?];
    let manifest = json!({
        "schemaVersion": 1,
        "crateVersion": anki_forge::facade_api_version(),
        "examples": examples,
        "update": updates(output)?
    });
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
