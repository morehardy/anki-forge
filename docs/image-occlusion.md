# Image Occlusion

Create a question by covering a rectangular part of an image. The supported
example uses **hide-all-guess-one**: all selected regions are hidden, and each
card asks for one region.

## Run the example

From the repository root with [Rust installed](installation.md#requirements):

```sh
cargo run --locked -p anki_forge --example docs_workflow -- target/docs-examples
```

Open `target/docs-examples/diagram.apkg` in Anki. The example uses the repository's
228 × 86 PNG fixture, covers one rectangle and checks one note, one card and one
media file. The [complete program](../anki_forge/examples/docs_workflow.rs) supplies
the image path and output directory to this function:

<!-- source: anki_forge/examples/docs_workflow.rs#occlusion -->
```rust
fn occlusion(output: &Path, diagram: &Path) -> anyhow::Result<()> {
    let mut deck = Deck::new("Diagram");
    let image = deck.media().add(MediaSource::from_file(diagram))?;
    deck.image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 10, 40, 25)
        .stable_id("diagram:region-1")
        .add()?;
    let report = deck.write_apkg(output.join("diagram.apkg"))?;
    report.ensure_success()?;
    verify(&report, 1, 1, 1)?;
    Ok(())
}
```
<!-- /source -->

## Choose your regions

`rect(x, y, width, height)` uses image pixels measured from the top-left corner.
Use positive widths/heights and rectangles within the image bounds. Add another
`.rect(...)` call for another region. When using your own image, choose coordinates
for its actual dimensions rather than its scaled size in a browser preview.

The Deck API can derive note identity from image content, dimensions, mode and
mask geometry. An explicit stable ID makes source identity deliberate. Compare
with the previous APKG when changing the image or masks of a distributed deck.

## Current limitation

The shared core currently rejects **hide-one-guess-one** grouped cloze output
with `PRODUCT.CLOZE_MARKER_MALFORMED`. Use `IoMode::HideAllGuessOne` in Rust,
`hide-all-guess-one` in Node, or `hide_all_guess_one` in Python. This limitation
applies across bindings; it is not fixed by selecting a different language.

See [Node's Deck API](node/api.md#media-and-deck),
[Python's Deck API](python/api.md#artifacts-deck-and-errors), and
[compatibility](compatibility.md) for the verified scope. Package checks do not
substitute for rendering tests in the Anki versions you intend to support.
