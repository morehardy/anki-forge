# ankiforge

[Website](https://ankiforge.dev/) · [GitHub](https://github.com/morehardy/anki-forge)

Build Anki packages with owned notes, validated models and media snapshots.
The crate embeds its contract resources and works outside the repository.
Rust 1.92.0 or later is required.

```rust,no_run
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    assert_eq!(output.report().counts().notes, 1);
    Ok(())
}
```

Use a stable project namespace and note key from the first release. Display names,
content and deck names can change independently. A project can contain several decks;
set the default with `Project::default_deck` or use `Note::deck` for individual notes.

The common values are available at the crate root. Additional APIs live in
[`note`], [`schema`], [`media`], [`build`], [`update`] and [`diagnostics`].
The optional `internal-tools` feature exposes selected repository verification
operations under `tools`; applications should use the default features.

## Models and content

```rust,no_run
use ankiforge::{Field, NoteType, Template, Project, Content};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = NoteType::builder("vocabulary")
        .name("Vocabulary")
        .field(Field::new("front").name("Question").required())
        .field(Field::new("back").name("Answer"))
        .template(Template::new("recognition")
            .front("{{front}}")
            .back("{{FrontSide}}<hr>{{back}}"))
        .build()?;
    let mut project = Project::new("language-course")?;
    project.add("hola", model.note()
        .field("front", "hola")
        .field("back", Content::html("<b>hello</b>")))?;
    Ok(())
}
```

A completed model is immutable and cheaply cloneable. Notes own their model;
adding a note automatically collects the model and media. Templates and note field
assignments refer to stable field keys. Display names are compiled to Anki field
references during output. Plain strings are escaped text, including in Cloze notes;
use `Content::html` for explicit markup.

## Media

```rust,no_run
use ankiforge::{BuildOptions, Media, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image = Media::file("cell.png")?;
    let mut project = Project::new("biology")?;
    project.add("cell", Note::basic(image.image(), "A cell"))?;
    project.build(BuildOptions::to("biology.apkg"))?;
    Ok(())
}
```

`Media::file` acquires an owned snapshot before returning. Deleting or changing the
source later does not change this value. `Media::bytes` takes ownership of bytes and
requires a MIME type. Large snapshots use temporary storage shared between clones;
the last owner cleans it up. Import budgets are configurable through `MediaLimits`.

Content automatically retains typed image and sound references. Declare files used
in handwritten HTML, CSS or scripts with `NoteTypeBuilder::asset` or
`Project::add_asset`. `Media::with_export_name` sets a portable fixed name before
creating references; names that collide after Unicode normalization and case folding
are rejected. The `template-bundle-v2` loader returns the same immutable NoteType.

## Updates and comparison

```rust,no_run
use ankiforge::{BuildOptions, Note, Project};
use ankiforge::update::CompareOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?;
    project.add("hola", Note::basic("hola", "hello"))?;
    project.build(BuildOptions::to("v1.apkg"))?;

    let mut next = Project::new("spanish")?;
    next.add("hola", Note::basic("hola", "hello; hi"))?;
    let comparison = next.compare(CompareOptions::against("v1.apkg"))?;
    assert!(comparison.policy().allows_publication());
    next.build(BuildOptions::to("v2.apkg").update_from("v1.apkg"))?;
    Ok(())
}
```

Keep the original distributed APKG as the next baseline. Packages carry complete
identity evidence; an Anki re-export is not a valid replacement. Update builds preserve
known note, model and card identities. Both baseline and candidate inspection have
independent finite resource budgets.

Comparison returns a complete report even when policy would block publication.
Builds block High and Critical risks by default, before replacing the destination.
Use typed `UpdatePolicy` and `RiskCode` values to accept specific risk categories;
evidence and original risk levels remain in the report. Hard validation errors cannot
be accepted. Configuring update policy on a first-release build is an error.

Anki import behavior also depends on learner edits and import settings. Structural
changes can require Anki's model merge option; imported packages do not delete omitted
learner notes or cards. Read comparison evidence before accepting these changes.

## Artifacts, reports and errors

A successful build always returns `BuildOutput` with an `ApkgArtifact` and observations.
`BuildOptions::temporary()` creates an artifact removed after its last handle drops;
copying its path or serializing a snapshot does not retain the file. `BuildOptions::to`
and `ApkgArtifact::persist_to` atomically publish a persistent file.

`BuildReport` contains observations only. Obtain the actual outcome from
`BuildOutput::snapshot` or `BuildError::snapshot`. Publication failures retain the
publication stage and any already-published path. Saving JSON is a separate operation;
it does not affect artifact ownership.

Public errors implement `std::error::Error + Send + Sync + 'static`, expose stable
`kind` and `code`, and retain underlying causes. Match these instead of display text.
Owned values may be moved between threads; synchronize mutation of a shared project.

This checkout embeds contract bundle `1.0.0`.
The crate version and embedded contract version are separate compatibility axes,
reported by `facade_api_version()` and `embedded_contract_version()`.

Licensed under MIT.
