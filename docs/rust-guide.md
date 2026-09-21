# Rust authoring guide

Start with the [README quick start](../README.md#quick-start) for a first APKG.
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

Custom note types, stable field/template keys, and project media are shown in:

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

Media export names must be helper-safe bare filenames such as `taberu.mp3`.
Avoid path components, absolute paths, URL escapes, and unsafe characters.
Register files with `project.media_mut().add_file(path)?.export_as("taberu.mp3")?`;
inline examples can use `project.media_mut().add_bytes(label, bytes)?.export_as(name)?`.
`write_apkg()` stages file-backed media by path by default, so large images,
audio, and video do not need to fit the inline-media limit. Use
`BuildOptions::self_contained()` only when you explicitly want a self-contained
inline authoring payload; large file-backed media should stay on the default
path-backed build path.

Common media diagnostics:

- Filename collision: the same export filename is bound to different bytes.
  Choose a unique `export_as(...)` name and update local note, template, or CSS
  references to match.
- Missing media reference: Product content refers to a local filename that is
  not registered. Register it or change the local filename in the HTML/CSS.
- CSS missing reference: CSS scanning is conservative. A local
  `url("icon.svg")` should be registered, changed to an external URL, or removed
  if the CSS rule is unused.
- CSS import reference: a local import such as `url("theme.css")` must be
  registered as packaged media, changed to an external URL, or removed if unused.
- Unused media binding: a registered file is not referenced by any note,
  template, or CSS. Remove the registration or add the intended local reference;
  this is a warning under the strict default.
- Unsafe media reference: packaged media references must be bare local
  filenames. Remove path components, absolute paths, escapes, or unsafe
  characters.
- MIME mismatch: the export filename or declared MIME does not match the
  observed source bytes. Change the export filename/declared MIME, or replace
  the source file.
- Inline too large: an explicitly self-contained build tried to inline media
  beyond the configured inline limit. Remove `self_contained()` and use the
  default path-backed build path for large assets.

`anki-forge` does not automatically rewrite filenames, HTML, or CSS because
those edits can change deck behavior and hide the authoring intent. Keep the
registered `export_as(...)` filename and local references in sync yourself.
`BuildReport::pretty_report()` is a human-facing summary. For stable
machine-readable output, use `BuildOptions::report_json(...)` or
`report.to_report_json()`; structured report JSON includes media mode details.
Automatic `report_json` requires `output` or `artifacts_dir`; JSON cannot keep a
temporary APKG alive. See [artifact ownership](../anki_forge/README.md#artifact-ownership).

## Updating distributed decks

For long-lived decks, keep stable project and note IDs and commit an identity
lockfile next to your source. Alternatively, build with
`BuildOptions::new().output("v2.apkg").compare_to("v1.apkg")`, using the latest
distributed APKG as the baseline.

Stable IDs preserve identity; updating existing Anki notes also needs revision
evidence from a previous APKG or maintained lockfile. Baseline-free
`write_apkg()` is suitable for a first export but does not guarantee that Anki
will apply later content edits. Anki's import settings still govern newer local
edits. See [revision behavior](../anki_forge/README.md#updating-distributed-decks).

First build:

```rust
use anki_forge::prelude::*;

let mut project = Project::new("Japanese Core")
    .stable_id("jp-core")
    .default_deck("Japanese::Core");
project.add_note(Note::basic("食べる", "to eat").stable_id("jp:taberu"))?;
project
    .build(
        BuildOptions::new()
            .output("dist/jp-core.apkg")
            .first_update_safe_build("anki-forge.lock.json"),
    )?
    .ensure_success()?;
```

Next build:

```rust
project
    .build(
        BuildOptions::new()
            .output("dist/jp-core.apkg")
            .update_safe("anki-forge.lock.json"),
    )?
    .ensure_success()?;
```

`update_safe(lockfile)` reads lockfile evidence but does not rewrite the file.
Use `write_identity_lockfile(true)` on release builds when you want new notes
and absent entries recorded for future updates.

When using `compare_to(previous_apkg)`, keep the baseline separate from the
output, `artifacts_dir/package.apkg`, report JSON, and any writable identity
lockfile. Builds reject existing same-file aliases (including relative paths,
symlinks, and hard links) with `PROJECT.PATH_COLLISION` before writing. This
includes the actual `staging/manifest.json` destination, whose links may point
outside the artifact directory. Baselines, outputs, retained packages, and
identity lockfiles (including read-only ones) must stay outside the writable
`staging/` tree and its media directory, including directory aliases. Staging
materialization must not overwrite these files before a risk rejection.

New destinations are rechecked after creation and before lockfile/report writes,
so filesystem-specific case folding cannot turn an APKG into JSON. A collision
detected after publication returns an error but keeps the valid published APKG.

The baseline is inspected once before building; GUID reconciliation and diff
use that same snapshot. The candidate APKG is compared and checked against
`fail_on(...)` before publishing the APKG or updating the identity lockfile.
On a blocked build, existing outputs and lockfiles remain unchanged, and the
report retains diff/risk evidence with `artifact: null`. A separate report JSON
can still be written; intermediate staging/media files may remain in an explicit
artifact directory. Successful publication uses atomic replacement per file,
not a transaction spanning the APKG, lockfile, and report.
Output-only builds copy directly from the private candidate to the requested
output; no extra package copy is made in the disposable artifact workspace.
Private candidates live inside the artifact workspace, so an explicit
`artifacts_dir(...)` also selects their filesystem; they are removed after the
build. Lockfiles use an exclusively reserved temporary file beside the target,
so temporary names cannot overwrite existing baselines or outputs.

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
