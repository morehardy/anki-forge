# Rust authoring guide

Start with [your own Rust application](installation.md) or review the
[core concepts](concepts.md) before this workflow.
This guide covers editable projects, validation, media, and repeatable updates
through the supported Rust API. The snippets use `anyhow` for error propagation;
add it to your application with `cargo add anyhow`.

- [Projects and diagnostics](#projects-and-diagnostics)
- [Media troubleshooting](#media-troubleshooting)
- [Updating distributed decks](#updating-distributed-decks)
- [Stable note identity](#stable-note-identity)

## Projects and diagnostics

```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_note(Note::basic("食べる", "to eat").stable_id("jp:taberu"))?;
    project.validate().ensure_success()?;
    project.write_apkg("jp-core.apkg")?.ensure_success()?;
    Ok(())
}
```

`Project::add_note(...)` and `Project::add_notetype(...)` fail fast for errors
that are knowable at add time, such as blank or duplicate explicit stable ids,
unknown note type ids, and unknown field keys. Call
`project.validate().ensure_success()?` when you want a full Project diagnostic
checkpoint before building; build still performs normalization, media, writer,
comparison, and update-safety checks.

`BuildReport` includes an owned artifact handle, note/card/media counts, diagnostics,
warning count, inspect summary, and duration. Diagnostics expose stable codes
and structured metadata (`severity`, `domain`, `stage`, `path`,
`suggested_fix`) so callers do not need to match human-facing strings.

`Project::from(deck)` produces an editable Project, including the Deck's media
and identity evidence. When a build has no `output` or `artifacts_dir`, its APKG is
temporary: retain its report/handle while using `artifact.path()`, or call
`artifact.persist_to(path)` for a permanent copy. The final handle's drop removes
temporary output. Explicit destinations are caller-owned and survive drop.

```rust
use anki_forge::prelude::*;

fn add_note(deck: &mut Deck) -> anyhow::Result<()> {
    if let Err(err) = deck.basic().note("hola", "hello").stable_id("   ").add() {
        match err.code() {
            ErrorCode::StableIdBlank => eprintln!("choose a non-empty stable_id"),
            ErrorCode::StableIdDuplicate => eprintln!("choose a unique stable_id"),
            other => eprintln!("anki-forge error: {}", other.as_str()),
        }
        return Err(err);
    }
    Ok(())
}
```

For complete explanations, see [custom note types](custom-notetypes.md) and
[images and audio](media.md). Runnable repository examples:

```bash
cargo run -q -p anki_forge --example target_api_custom_notetype
cargo run -q -p anki_forge --example target_api_media
```

`Note::cloze(...)` intentionally stores the cloze `Text` field as explicit HTML
so Anki receives raw `{{cN::...}}` markers. Do not assume cloze text is escaped
like `Note::basic(...)` text.

For custom fields, `.text(key, value)` escapes text and `.html(key, value)`
preserves trusted HTML. Use `.sound(key, media)` and `.image(key, media)` for
Anki-compatible media references. Choose the content method explicitly so that
literal text and intended markup remain distinguishable.

## Media troubleshooting

Register media, keep source files available and unchanged until build completes,
and match local HTML/CSS references to bare export filenames. Normal builds use
path-backed media; use inline bytes only for small assets.

Follow the [media tutorial](media.md) for a complete example and the
[media troubleshooting table](troubleshooting.md#media-and-templates) for collisions,
missing files, unused bindings, changed sources and inline limits.

## Updating distributed decks

Keep stable project/note IDs and the last distributed APKG or maintained identity
lockfile. The [update tutorial](updates.md) builds two releases, checks the report,
and demonstrates both baseline strategies.

`update_safe(lockfile)` reads existing evidence; add
`write_identity_lockfile(true)` when writing the next candidate evidence.
Baseline-free `write_apkg()` is appropriate for a first export and does not
guarantee that Anki applies later edits. Import settings and newer local edits
still matter.

Keep baselines separate from outputs and writable staging. For path aliases,
blocked builds, temporary artifacts and publication guarantees, read
[build and output guarantees](build-guarantees.md).

## Stable note identity


The `Deck` API derives AFID (`afid:v1:*`) stable note IDs when no explicit ID is
provided. Basic notes default to their front field, Cloze uses its cloze
structure and text skeleton, and Image Occlusion uses image content, dimensions,
mode, and sorted mask geometry. Editing an identity input can change the derived
ID. Use `.stable_id("your-source-id")` when notes have an identity in your source
system that should survive content edits.

Explicit `stable_id` values take precedence. The `afid:v1:*` namespace is reserved
and cannot be supplied as an explicit ID. Blank IDs, duplicate payloads, hash
collisions, and duplicate stable IDs are rejected during addition or identity
rebuilding.

For custom Project note types, use stable field/template keys and the public
`IdentityRecipe::fields(...)` API. See the
[custom note type example](../anki_forge/examples/target_api_custom_notetype.rs)
and [template bundle guide](template-bundles.md).
