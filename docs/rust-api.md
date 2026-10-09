# Rust API

Common values are available at the crate root: `Project`, `Note`, `NoteType`,
`Field`, `Template`, `Content`, `Media`, `BuildOptions`, and `BuildOutput`.
The public domain modules are `note`, `schema`, `media`, `build`, `update`, and
`diagnostics`. Repository tools are a separate, feature-gated surface; consumer
applications do not need them.

## Authoring

| Entry | Responsibility |
| --- | --- |
| `Project::new(namespace)` | Validate a stable publication namespace |
| `project.default_deck(name)` | Set the default destination |
| `project.add(key, note)` | Atomically collect the note, model and media |
| `project.add_asset(media)` | Include an explicit raw HTML/CSS/script asset |
| `Note::basic(front, back)` / `Note::cloze(text)` | Create a note carrying its built-in model |
| `model.note().field(key, content)` | Create a custom note carrying its immutable model |
| `note.deck(name).tag(tag)` / `.tags(iterable)` | Configure a note before adding it |
| `NoteType::builder(key)...build()` | Validate and complete an immutable model |
| `NoteType::from_bundle(path)` | Load a model and its owned assets from bundle-v2 |

`Field::new(key).name(display)` and `Template::new(key).name(display)` separate
identity from presentation. Templates bind field keys, including filters,
conditional sections and browser templates. `Field::required()` checks nonempty
content; `Field::sort()` selects the sort field. `schema::GenerationRule::all`
and `any` consume field keys. `builder.cloze_field(key)` declares a Cloze model.

`schema::FieldKey` and `TemplateKey` are distinct types. String literals, owned
strings, borrowed strings, `Cow<str>`, and borrowed or owned keys are composable.
Conversions preserve the original value; schema completion validates keys.
A template key is not implicitly accepted as a field key.

## Content and media

All ordinary strings become `Content::text`. HTML requires `Content::html`.
`Content::sequence` combines semantic values until rendering at build time.
`Media::file(path)` and `Media::bytes(bytes, mime)` own snapshots; their
`with_limits` variants apply a `media::MediaLimits { max_bytes }` budget before
reading. The default is 256 MiB per asset. `with_export_name(name)` returns a
renamed value sharing the same snapshot; existing clones retain their names.

`media.image()` and `media.sound()` keep resource dependencies. Fixed filenames
must be portable and collision-free. Explicit assets are included even without
a statically visible reference. See [media](media.md).

`Note::image_occlusion(media)` returns `note::ImageOcclusionBuilder`.
`.mask(Mask::rect(key, x, y, width, height)).mode(mode).build()` validates decoded
image dimensions, rectangles and unique stable keys. Both `HideAllGuessOne` and
`HideOneGuessOne` are supported. See [Image Occlusion](image-occlusion.md).

## Build options

`project.build(BuildOptions::to(path))` publishes a persistent APKG.
`BuildOptions::temporary()` returns owned temporary output. Both accept
`.update_from(previous_apkg)`, `.inspect_limits(limits)` and
`.update_policy(policy)`. A policy without a baseline is a configuration error.
Setter order does not affect this rule.

`build::InspectLimits::default()` gives finite budgets; its public fields can be
modified. All values are `u64`; zero means zero, not unlimited.

| Field | Default |
| --- | --- |
| `max_archive_bytes` | 2 GiB |
| `max_entries` | 100,000 |
| `max_central_directory_bytes` | 16 MiB |
| `max_zip_entry_bytes` / `max_zip_total_bytes` | 1 GiB / 4 GiB |
| `max_meta_bytes` / `max_media_map_bytes` | 64 KiB / 16 MiB |
| `max_identity_bytes` | 64 MiB |
| `max_collection_bytes` / `max_media_bytes` | 512 MiB / 256 MiB |
| `max_decoded_total_bytes` / `max_zstd_window_bytes` | 4 GiB / 64 MiB |

Limits apply independently to baseline and candidate inspection. A failed check
reports the resource, limit, observed amount and related entry where available.
Raising a limit is an explicit caller action; media import has its own earlier
budget.

## Build, compare and output

`BuildOutput::artifact()` returns an owned `build::ApkgArtifact` reference.
`BuildOutput::report()` returns observations. Artifact clones share temporary-file
lifetime; `persist_to` returns a persistent owner or `build::PersistError` with
publication facts and the underlying I/O source.

`project.compare(update::CompareOptions::against(path))` returns a completed
`ComparisonReport` even if high-risk findings block its policy. It exposes
`findings()`, `highest_risk()`, `policy()`, `diagnostics()` and `snapshot()`.
`UpdatePolicy::default()` blocks High/Critical; `.fail_on(level)` changes the
threshold and `.allow(RiskCode)` accepts a complete registered risk category.
It cannot accept missing/corrupt identity evidence or other hard errors.

## Reports and errors

`BuildReport::snapshot()` returns `build::json::ReportSnapshot`, observations
without success/failure or file ownership. `BuildOutput::snapshot()` and
`BuildError::snapshot()` return `build::json::BuildSnapshot`, including the actual
outcome and report. `update::json::ComparisonSnapshot` separates original findings
from policy evaluation. All snapshot DTOs implement `serde::Serialize` and own no
files. No extension trait is required.

Snapshot path fields retain their native `PathBuf` values in Rust. Their JSON
form is a string for valid Unicode, or a lossless native encoding:
`{"encoding":"unix_bytes","bytes":[...]}` for non-UTF-8 Unix names, and
`{"encoding":"windows_wide","units":[...]}` for Windows names containing
unpaired UTF-16 surrogates. This applies both to successful artifact paths and
failure publication paths. To restore a JSON path on its original platform,
deserialize it as `build::json::PathSnapshot` and call `into_path_buf()`; encoded
paths from a different platform are rejected. `PathSnapshot::new(path)` provides
the same representation when serializing an error's native path yourself.

Errors live with their operations: schema and bundle errors in `schema`, add/IO
errors in `note`, media errors in `media`, build/persist errors in `build`, and
compare/policy errors in `update`. Each provides `kind()` and `code()`, implements
`std::error::Error + Send + Sync + 'static`, and retains real source errors.

## Addition context

Failed additions leave the project unchanged. Structured context records the
original note/model keys and the target, with optional conflict or media usage
details. Rust exposes `note::{AddContext, AddTarget, AddDetail}` through
`AddError::context()` / `detail()`; SDKs expose `AddError.details` with typed
context/detail. Node fields use camelCase and are deeply frozen; Python uses
snake_case TypedDicts. Tags retain their insertion index; field locations retain
original sequence indices (null/None for the field, [] for its root) and optional
UTF-8 byte ranges. Default-deck errors from build have no note/model keys and
remain available in the source chain.

Typed images require image/* MIME at addition; sound references accept audio/*
or video/*. Other categories raise `NOTE.MEDIA_USAGE_INVALID`. Constructors stay
infallible; explicit assets and raw HTML are unaffected. The check uses retained
import MIME, and does not certify playback. Build's independent MIME/extension
validation still applies. See [the design](plans/2026-09-28-rust-api-validation-and-errors-design.md) for the complete target/detail table.

## Review once, publish once

**API source commit `1199196`**; not included in public `0.2.0`. See [source builds](source-builds.md).


```rust,no_run
use ankiforge::{BuildOptions, Project};
# fn publish(project: &Project) -> Result<(), Box<dyn std::error::Error>> {
let prepared = project.prepare_publication(
    BuildOptions::to("next.apkg").update_from("previous.apkg"),
)?;
let report = prepared.report().clone(); // observations; owns no file
if report.comparison().is_some_and(|c| !c.policy().allows_publication()) {
    drop(prepared); // deletes the unpublished candidate; reprepare with a new policy
    return Ok(());
}
match prepared.publish() { // consumes the owner on every outcome
    Ok(output) => println!("Published {}", output.artifact().path().display()),
    Err(error) => {
        // Replacement may have succeeded even if durability confirmation failed.
        eprintln!("{:?}", error.publications());
        return Err(error.into());
    }
}
# Ok(())
# }
```

Preparation performs full bounded inspection and comparison. No destination is
published until `publish`, which uses the same private candidate and rechecks
baseline aliases. Relative paths bind at preparation invocation. Project edits
cannot change the candidate. Report duration excludes time spent reviewing.
Dropping an unpublished owner cleans it; temporary published output follows the
usual last-artifact-owner cleanup rule. There is no restore-from-JSON or retry on
an already consumed owner. Native builds omit the unused staging manifest but
retain embedded identity evidence and media validation. Internal staging tools
still receive their complete manifest and fingerprint.
