# Images and audio

This guide builds a card with a waveform image and a real one-second A4 tone.
The example generates its own SVG and WAV, so no media download is required.

## Generate and inspect the result

From the repository root with [Rust installed](installation.md#requirements):

```sh
cargo run --locked -p anki_forge --example website_showcase -- target/media-example
```

Open `target/media-example/ear-training.apkg` in Anki. It contains one note,
one card and two media files. The front shows a waveform and an audio control;
the answer is **A4, 440 Hz**. The program also writes `waveform.svg` and
`concert-a.wav` to that directory for inspection.

## Register media and attach it to a note

This excerpt comes from the [complete example](../anki_forge/examples/website_showcase.rs).
The preceding code generates the two files and declares the `ear-training`
note type with `prompt`, `answer`, `picture` and `audio` fields.

<!-- source: anki_forge/examples/website_showcase.rs#website:media -->
```rust
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
```
<!-- /source -->

Registration returns a `MediaRef`. `.image(...)` and `.sound(...)` create the
Anki field markup that refers to its export filename. Include those fields in
your template, or the media will not appear on the card.

## Use your own files

Replace the two source paths with existing files. `export_as(...)` selects the
name inside the APKG; it must be a bare filename, such as `pronunciation.wav`,
not a directory path. Keep files available and unchanged until the build finishes.
Registration fingerprints sources and building checks for later changes.

For small in-memory assets, Project offers `add_bytes(label, bytes)?.export_as(name)?`.
The Project inline limit is 64 KiB. Use files for larger assets. Normal builds
use path-backed media; explicitly self-contained payloads have inline limits.

Templates and CSS can reference registered filenames directly, for example
`<img src="logo.png">` or `url("logo.png")`. Keep these names in sync yourself;
the library does not rewrite your HTML, CSS or media filenames automatically.

## Diagnose media problems

A successful package export does not prove a particular Anki client can play a
codec. Verify playback on your intended client. For missing files, changed
sources, collisions or unused bindings, use the
[media troubleshooting table](troubleshooting.md#media-and-templates).

Next, place shared media in a [template bundle](template-bundles.md), or create
[Image Occlusion cards](image-occlusion.md).
