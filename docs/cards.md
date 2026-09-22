# Basic and Cloze cards

Build a package with two notes and three cards: one Spanish word pair and two
Cloze questions about a city and country. Start with a source checkout and
[the Rust prerequisites](installation.md#requirements).

## Run the example

From the repository root:

```sh
cargo run --locked -p anki_forge --example docs_workflow -- target/docs-examples
```

Open `target/docs-examples/cards.apkg` in Anki. The Basic card asks **hola**;
the Cloze cards separately hide **Madrid** and **Spain**. The example also
writes the packages used by the template, image-occlusion and update guides.

## Author the notes

This function is extracted from the [complete program](../anki_forge/examples/docs_workflow.rs).
`output` is the destination directory. Its `verify` helper checks the inspected
note, card and media counts after export.

<!-- source: anki_forge/examples/docs_workflow.rs#cards -->
```rust
fn cards(output: &Path) -> anyhow::Result<()> {
    let mut project = Project::new("Spanish")
        .stable_id("docs-spanish")
        .default_deck("Spanish");
    project.add_note(Note::basic("hola", "hello").stable_id("es:hola"))?;
    project.add_note(
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}}.")
            .extra("A city and its country.")
            .stable_id("es:capital"),
    )?;
    project.validate().ensure_success()?;
    let report = project.write_apkg(output.join("cards.apkg"))?;
    report.ensure_success()?;
    verify(&report, 2, 3, 0)?;
    Ok(())
}
```
<!-- /source -->

Use distinct stable IDs for the two logical notes. The Cloze numbers select the
questions within one note; they are not note IDs. Changing a Cloze's numbering
can change its card structure, so review such changes before distribution.

## Common problems

- A plain sentence without a cloze deletion does not supply a Cloze question.
  Use `{{c1::answer}}`; add `::hint` inside the deletion when a hint is useful.
- Basic text and Cloze text have different escaping behavior. Read
  [text and HTML](concepts.md#text-and-html) before interpolating source data.
- Reusing a stable ID for another note is an authoring error. Choose the ID from
  your source record, rather than a list position that changes when you reorder it.

Next, add [images and audio](media.md) or define a [custom note type](custom-notetypes.md).
