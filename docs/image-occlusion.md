# Image Occlusion

Create a structured image-occlusion note from an owned image and rectangles.
Each mask needs a stable key that continues to identify the same region through
later versions. The note joins a project through the ordinary `add(key, note)`
path.

## Run the example

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Open `target/docs-examples/diagram.apkg` in Anki. The complete executable loads
the repository's occlusion image and checks one note, one card and one asset.

<!-- source: anki_forge/examples/docs_workflow.rs#occlusion -->
```rust
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let image = Media::file(diagram)?;
    let note = Note::image_occlusion(image)
        .mask(Mask::rect("region-1", 10, 10, 40, 25))
        .build()?;
    let mut project = Project::new("docs-diagram")?.default_deck("Diagram");
    project.add("diagram", note)?;
    let built = project.build(BuildOptions::to(output.join("diagram.apkg")))?;
    verify(&built, 1, 1, 1)?;
    Ok(())
}
```
<!-- /source -->

## Multiple masks and modes

This standalone example expects `fixtures/occlusion.png`, a decoded image at
least 80×80 pixels:

```rust
use ankiforge::{BuildOptions, Media, Note, Project};
use ankiforge::note::{Mask, OcclusionMode};

fn main() -> anyhow::Result<()> {
    let note = Note::image_occlusion(Media::file("fixtures/occlusion.png")?)
        .mask(Mask::rect("nucleus", 10, 10, 20, 20))
        .mask(Mask::rect("wall", 50, 50, 20, 20))
        .mode(OcclusionMode::HideOneGuessOne)
        .build()?
        .field("header", "Cell structure")
        .field("back_extra", "Review the diagram.");
    let mut project = Project::new("cell-diagram")?;
    project.add("cell", note)?;
    let output = project.build(BuildOptions::to("cell-diagram.apkg"))?;
    assert_eq!(output.report().counts().cards, 2);
    Ok(())
}
```

`HideAllGuessOne` is the default; `HideOneGuessOne` is also supported. Each keyed
mask produces its own cloze card. Coordinates are finite pixel values on the
displayed image, after EXIF orientation. The library fully decodes PNG, JPEG,
GIF, WebP and BMP; the decoded buffer budget is 256 MiB, independent of the
compressed media import budget. Rectangles must have positive dimensions, lie
inside the image and use unique nonempty keys.

The builder validates the image and masks before returning a `Note`. Use ordinary
`field` assignments for `header`, `back_extra` and `comments`; the generated
`image` and `occlusion` fields are reserved and cannot be overridden.

When updating from a previous distribution, mask keys preserve card ordinals
across ordering changes. Removed masks reserve their ordinals; restoring the
same key reuses the ordinal. New keys never inherit another mask's identity.
The supported allocation is 1–500, and exhaustion fails explicitly. Removing or
adding masks is visible in [update risk analysis](updates.md); omission from an
APKG does not delete a learner's existing card.
