# Build your first Anki deck with Rust

Canonical page: https://ankiforge.dev/blog/your-first-anki-deck/
Language: en
Description: Build your first Anki deck with a complete Rust example: create a Spanish flashcard, export and import an APKG, and assign stable note keys for future updates.
Published: 2026-09-22T00:00:00.000Z

When your notes already live in a script, a dataset, or a collection of files, generating an Anki deck can make the authoring process easier to repeat. Your source stays reviewable in Git, and each export becomes a step you can run again.

Start with one word: **hola**. We want a card that shows the Spanish greeting on the front and its English meaning on the back.

## A complete program

This is the repository’s minimal Rust example:

```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```

`Project::new("spanish")` creates the namespace. `Note::basic("hola", "hello")` creates a Basic note, `project.add("es:hola", note)` assigns its stable key, and `build(BuildOptions::to(...))` writes the package.

`build` returns `BuildOutput` only after success. The `?` operator propagates a typed failure so scripts can exit with an error.

## Install and run

Follow the [current quickstart](https://ankiforge.dev/docs/quickstart/) for the public package installation and run command. To use the same program in your own project, follow [Add to your application](https://ankiforge.dev/docs/installation/).

After exporting, import `spanish.apkg` into Anki and reveal the answer to your first card.

The [Basic example on this site](https://ankiforge.dev/examples/#basic) also has a downloadable package if you want to inspect the result first.

## Give the note an identity

The string `es:hola` is an explicit stable note key within the `spanish` namespace. It describes which note this is, independently of how you phrase the answer. You could later expand “hello” to “hello; hi” and keep the same ID.

Use a distinct ID for each logical note. If your source already has a durable key, such as a vocabulary entry ID, that is a useful starting point. Avoid deriving identity solely from text that you expect to edit.

Updating a distributed deck also needs release history. Keep the original previous package and follow the [documented update workflow](https://ankiforge.dev/docs/updates/).

## Grow from one note

For another greeting, create another Basic note and add it with a new key. For a missing word in a sentence, use a [Cloze note](https://ankiforge.dev/examples/#cloze). For richer cards, define your fields and templates with the project API and package images or audio with the notes.

The [Rust authoring guide](https://ankiforge.dev/docs/rust-guide/) takes the next step through diagnostics, media, and repeatable builds. The [repository examples](https://github.com/morehardy/anki-forge/tree/main/anki_forge/examples) provide runnable programs to build on.
