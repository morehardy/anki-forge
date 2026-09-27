# Rust authoring guide

One `Project` describes a publication. Create it with a stable namespace, add
notes under stable keys, and build an APKG. Common types are imported directly
from `ankiforge`; advanced options live in the public domain modules.

## Projects and diagnostics

```rust
use ankiforge::{BuildOptions, Content, Note, Project};

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("jp-core")?
        .name("Japanese Core")
        .default_deck("Japanese::Core");
    project.add("taberu", Note::basic("食べる", Content::html("<b>to eat</b>")))?;
    let output = project.build(BuildOptions::to("jp-core.apkg"))?;
    println!("{} notes", output.report().counts().notes);
    println!("{}", serde_json::to_string_pretty(&output.snapshot())?);
    Ok(())
}
```

A successful build returns `BuildOutput` with a guaranteed artifact.
`BuildReport` contains observations: counts, diagnostics, elapsed time and any
completed comparison. It contains no outcome or artifact ownership. Warnings do
not turn a successful output into a failure.

Schema errors are checked when a model is completed. `Project::add` checks the
note's key, field references, required fields, deck, tags, and dependency
conflicts atomically. Failure leaves the project unchanged. The build performs
remaining card, package and update checks. There is no separate mutable registry
or required validation checkpoint.

Errors expose inherent `kind()` and `code()` methods and implement
`std::error::Error`. Use machine codes rather than parsing the displayed message.
`BuildError::snapshot()` includes the real failure, observations, source-chain
text and publication facts. Standard error wrappers can preserve the original
error as their source.

## Models and assets

A completed `NoteType` is immutable and shareable. Call `model.note()`, assign
fields by stable key, and add the result to a project. The note holds its model;
adding it collects the model and media automatically. Use
[custom note types](custom-notetypes.md) for declarations and template rules.

`Media::file` and `Media::bytes` acquire owned snapshots. Source files need not
remain available after import succeeds. `image.image()` and `audio.sound()`
produce structured content; `Content::sequence` combines them with text or HTML.
Raw HTML/CSS/script assets must be explicitly included using `builder.asset` or
`project.add_asset`. See [media](media.md) and [template bundles](template-bundles.md).

## Output ownership

Choose `BuildOptions::to(path)` for a persistent output or
`BuildOptions::temporary()` for an owned temporary file. Retain the output or a
clone of `output.artifact()` while accessing a temporary path. Its last owner
removes the file. `persist_to(path)` returns a persistent artifact; on failure the
original owner remains usable. JSON snapshots contain paths but do not own files.

## Stable note identity

Use a durable source-record key with `project.add(key, note)`. The project
namespace, model key, field keys and template keys are independent of display
names. Content changes do not create a new logical note. Duplicate keys fail
instead of overwriting. Save and reuse your chosen keys when source records are
reordered or edited.

## Updating distributed decks

Use the previous original distribution APKG with `update_from`. Its complete
embedded evidence preserves identity and revision mappings. Independent
`project.compare(CompareOptions::against(path))` analyzes the candidate and
reports whether policy would permit publication. See the [two-version workflow](updates.md)
for high-risk changes, client import limitations and explicit allowances.

From a checkout, these examples execute the same public API:

```sh
cargo run --locked -p ankiforge --example target_api_basic
cargo run --locked -p ankiforge --example target_api_custom_notetype
cargo run --locked -p ankiforge --example target_api_media
cargo run --locked -p ankiforge --example docs_workflow -- target/docs-examples
```
