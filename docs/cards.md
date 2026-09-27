# Basic and Cloze cards

A Basic note has a front and back. A Cloze note generates one card for each
supported deletion number. This example creates two notes and three cards.

From the repository root:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Open `target/docs-examples/cards.apkg` in Anki. The Basic question is **hola**;
the Cloze cards separately hide **Madrid** and **Spain**.

## Author the notes

This function is extracted from the [complete executable](../anki_forge/examples/docs_workflow.rs).
Its `verify` helper checks the successful output's note/card/media counts.

<!-- source: anki_forge/examples/docs_workflow.rs#cards -->
```rust
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("docs-spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    project.add(
        "capital",
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .field("back_extra", "A city and its country."),
    )?;
    let built = project.build(BuildOptions::to(output.join("cards.apkg")))?;
    verify(&built, 2, 3, 0)?;
    Ok(())
}
```
<!-- /source -->

The first argument to `Project::new` is a stable namespace. Each `add` call needs
a stable source-record key. Reusing one key for another note fails; no automatic
content-based key is assigned. A changed answer under the same key remains the
same logical note when building an update from its previous distribution.

## Text, fields and card identity

Both Basic and Cloze strings are Text. Use `Content::html` for intentional HTML;
Cloze deletion syntax is still recognized inside Text. `back_extra` is the
built-in Cloze extra field key. Use the same `Note::field` method for text, HTML,
images and sounds.

Use `{{c1::answer}}` or `{{c1::answer::hint}}`. A plain sentence supplies no
Cloze question. Supported deletion numbers are 1–500; changing numbers can alter
card identities. Review such changes through [comparison and update policy](updates.md).

Next, add [media](media.md), a [custom model](custom-notetypes.md), or
[image occlusion](image-occlusion.md).
