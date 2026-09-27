# Images and audio

A `Media` value owns an immutable snapshot. Import files with `Media::file` or
native bytes with an explicit MIME type via `Media::bytes`. After successful
import, modifying or deleting the source file cannot change the asset. Clones
share storage and can be used across projects.

## Execute the media workflow

From the repository root:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
cargo run --locked -p ankiforge --example target_api_media
```

The workflow uses repository PNG/WAV fixtures and writes `media.apkg` with one
note, one card and three assets. The second example generates its own tiny image
and audio, includes CSS-linked and explicit media, and writes `spanish-media.apkg`.

<!-- source: anki_forge/examples/docs_workflow.rs#media -->
```rust
fn media(output: &Path, image: &Path, sound: &Path) -> anyhow::Result<()> {
    let image = Media::file(image)?.with_export_name("cell.png")?;
    let sound = Media::file(sound)?;
    let mut project = Project::new("docs-media")?;
    project.add("cell", Note::basic(image.image(), sound.sound()))?;
    let css = Media::bytes(b".card { color: navy; }".to_vec(), "text/css")?
        .with_export_name("course.css")?;
    project.add_asset(css)?;
    let built = project.build(BuildOptions::to(output.join("media.apkg")))?;
    verify(&built, 1, 1, 3)?;
    Ok(())
}
```
<!-- /source -->

## Compose content and explicit assets

This complete program uses `fixtures/pixel.png`, `fixtures/silence.wav`, and
`fixtures/labels.woff`. In your application, pass your own source paths. The
fixture font is the repository's redistributable test font.

```rust
use ankiforge::{BuildOptions, Content, Field, Media, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let image = Media::file("fixtures/pixel.png")?.with_export_name("badge.png")?;
    let sound = Media::file("fixtures/silence.wav")?;
    let font = Media::file("fixtures/labels.woff")?.with_export_name("labels.woff")?;
    let model = NoteType::builder("media-card")
        .field(Field::new("front"))
        .field(Field::new("back"))
        .template(Template::new("card")
            .front(r#"{{front}}<img src="badge.png">"#)
            .back("{{FrontSide}}<hr>{{back}}"))
        .css("@font-face { font-family: labels; src: url('labels.woff'); } .card { font-family: labels; }")
        .asset(font)
        .asset(image.clone())
        .build()?;
    let mut project = Project::new("media-guide")?;
    project.add("cell", model.note()
        .field("front", Content::sequence([Content::text("Cell "), image.image()]))
        .field("back", sound.sound()))?;
    let output = project.build(BuildOptions::to("media-guide.apkg"))?;
    assert_eq!(output.report().counts().media, 3);
    Ok(())
}
```

Typed `image()` and `sound()` values retain dependencies until export. Manual
HTML/CSS/script references need `builder.asset(media)` or `project.add_asset(media)`;
the library does not infer file contents from arbitrary strings. An explicit
asset is included even without a statically detectable reference.

Pass the literal filename to `with_export_name`. Typed sound references escape
HTML entities when rendered, so a name such as `tone&copy;.mp3` still refers to
that exact asset. Do not pre-escape the filename. Percent signs and fragment
characters in sound filenames remain literal; they are not URL paths.

## Names, MIME and budgets

Default export names are content-derived and independent of the source path.
Use `with_export_name` before creating references when a fixed filename is
needed. Renaming a clone does not rename earlier clones or content nodes.

Names cannot contain paths, controls or nonportable reserved names. Conflict
checks use Unicode NFC plus case folding: case-only or normalization-only
variants also conflict. Exactly identical name/content pairs are idempotent;
different contents under one name fail without partial registration.

The complete export filename, including its extension, is limited to 255 UTF-8
bytes. Project and template-bundle JSON Schemas check filename syntax and
character count; the loaders additionally check this byte budget. For example,
128 repetitions of `é` pass the schema's character limit but occupy 256 bytes
and return `MEDIA.EXPORT_NAME_INVALID` when loaded. Use `tools::load_project` or
`NoteType::from_bundle` to validate the complete input before building.

MIME syntax and identifiable content must agree. Native bytes have no 64 KiB
inline restriction. The default import budget is 256 MiB per asset;
`file_with_limits` and `bytes_with_limits` accept `MediaLimits { max_bytes }`
before reading. Large snapshots may spill to temporary storage, cleaned up when
the last owner disappears. Snapshot ownership does not promise an atomic
filesystem view of a source being modified concurrently during import.

Project JSON applies the same default per-asset budget while decoding an inline
`bytes` array. It stops at the first excess byte without collecting the rest of
the array, and retains a `MediaError` with the limit and observed byte count.
The count at rejection is a lower bound; it is not the unread array's total size.

A valid media package does not prove codec playback on every Anki client. Test
images, sound and fonts on your target clients. See [troubleshooting](troubleshooting.md#media-and-templates).
