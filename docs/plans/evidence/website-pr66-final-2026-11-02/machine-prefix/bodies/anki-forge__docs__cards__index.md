# Generate Anki Basic and Cloze flashcards

Canonical page: https://morehardy.github.io/anki-forge/docs/cards/
Language: en
Description: Generate Anki Basic question-and-answer and Cloze fill-in-the-blank cards from code. A verified Rust example builds two notes and three cards.

Generate Anki Basic (question-and-answer) and Cloze (fill-in-the-blank) cards from code instead of adding them one at a time in Anki. A Basic note stores front and back fields, which its template uses to generate a question-and-answer card. A Cloze note generates one card for each supported deletion number. This Rust example creates two notes and three cards. Python and Node.js provide the same `Note.basic` and `Note.cloze` constructors; see the [Python API](https://morehardy.github.io/anki-forge/docs/python-api/#project-and-note-values) and [Node API](https://morehardy.github.io/anki-forge/docs/node-api/#project-and-notes).

From the repository root:

```sh
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```

Open `target/docs-examples/cards.apkg` in Anki. The Basic question is **hola**; the Cloze cards separately hide **Madrid** and **Spain**.

## Author the notes

This function is extracted from the [complete executable](https://github.com/morehardy/anki-forge/blob/main/anki_forge/examples/docs_workflow.rs). Its `verify` helper checks the successful output’s note/card/media counts.

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

The first argument to `Project::new` is a stable namespace. Each `add` call needs a stable source-record key. Reusing one key for another note fails; no automatic content-based key is assigned. A changed answer under the same key remains the same logical note when building an update from its previous distribution.

## Text, fields and card identity

Both Basic and Cloze strings are Text. Use `Content::html` for intentional HTML; Cloze deletion syntax is still recognized inside Text. `back_extra` is the built-in Cloze extra field key. Use the same `Note::field` method for text, HTML, images and sounds.

Use `{{c1::answer}}` or `{{c1::answer::hint}}`. A plain sentence supplies no Cloze question. Supported deletion numbers are 1–500; changing numbers can alter card identities. Review such changes through [comparison and update policy](https://morehardy.github.io/anki-forge/docs/updates/).

Next, add [media](https://morehardy.github.io/anki-forge/docs/media/), a [custom model](https://morehardy.github.io/anki-forge/docs/custom-notetypes/), or [image occlusion](https://morehardy.github.io/anki-forge/docs/image-occlusion/).
