# Custom note types

Complete a model once, then create notes from it. The returned `NoteType` is
immutable and can be reused in functions or multiple projects. A note holds its
model, so it cannot accidentally refer to an unregistered model name.

## Executable example

The example defines separate stable keys and display names, assigns by key,
and uses those same keys inside templates.

<!-- source: anki_forge/examples/target_api_custom_notetype.rs -->
```rust
use ankiforge::schema::GenerationRule;
use ankiforge::{BuildOptions, Field, NoteType, Project, Template};

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::builder("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("expr").name("Expression").sort().required())
        .field(Field::new("meaning").name("Meaning").required())
        .template(
            Template::new("recognition")
                .name("Recognition")
                .front("{{expr}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .build()?;
    let mut project = Project::new("jp-core")?.default_deck("Japanese::Core");
    project.add(
        "taberu",
        vocab
            .note()
            .field("expr", "食べる")
            .field("meaning", "to eat"),
    )?;
    project.build(BuildOptions::to("jp-core.apkg"))?;
    Ok(())
}
```
<!-- /source -->

Run from the checkout with
`cargo run --locked -p ankiforge --example target_api_custom_notetype`.
Open `jp-core.apkg` in Anki.

## Keys and names

`Field::new` and `Template::new` take stable keys. `.name` supplies a display name;
otherwise the name equals the key. No slug is derived from a name. Empty,
whitespace and conflicting/reserved keys fail model completion. Names can be
Chinese or other Unicode text without changing identity keys.

Templates bind keys and compile to Anki field names. This covers front/back,
browser templates, condition sections, filters and Cloze. Other HTML, whitespace
and script text are preserved. Unknown references fail with a source byte range.
Field assignment accepts only keys, with no silent trimming or case folding.

## Validation and generation

`required()` rejects missing or empty note content. Fields without it may be
omitted and render empty. `sort()` selects the sort field; at most one field can
select it. Generation uses Anki's default rule unless an explicit
`GenerationRule::all` or `any` is set on a template. These rules reference field
keys. `builder.cloze_field(key)` selects Cloze semantics for a custom model.

`builder.asset(media)` declares an owned CSS/font/script/raw-HTML dependency.
Typed image and sound content on notes collect assets automatically. Explicit
assets remain in the output even if no static reference is found.

The same model key can be reused within a project only with the same definition.
A conflicting definition fails atomically when adding a note. Display-name,
field or template changes in later releases need [update analysis](updates.md)
even though their stable keys remain unchanged.

For reusable files and assets, use a [template bundle](template-bundles.md).
