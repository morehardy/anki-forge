# Python API

This reference describes the native Python 0.2 product API. Start with the
[Python quickstart](quick-start.md). The package includes `py.typed`; public
signatures are in [the package source](../../bindings/python/src/anki_forge/__init__.py).

## Project and notes

| API | Return / behavior | Defaults and requirements |
| --- | --- | --- |
| `Project(name, stable_id=None, default_deck=None, *, base_dir=None)` | A Rust-backed Project | base_dir captures the current directory; settings are read-only |
| `project.add_note(note)` | The Project for chaining | Validates and snapshots the input |
| `project.add_notetype(note_type)` | The Project for chaining | Declare the type before adding its notes |
| `project.validate()` | `ValidationReport` | Authoring checks; no APKG or media re-read |
| `project.import_template_bundle(path)` | The Project for chaining | Relative to base_dir; failed imports leave no partial additions |
| `Project.from_deck(deck)` | A Project snapshot | Preserves the original Deck |
| `Note.basic(front, back, stable_id=...)` | Mutable input Note | Escapes text |
| `Note.cloze(text, back_extra=..., stable_id=...)` | Mutable input Note | Preserves HTML/cloze text; extra is text-escaped |
| `Note(type_id).text/.html/.image/.sound(...)` | The input Note | Finish edits before passing it to add_note |

`project.notes` and `project.notetypes` are detached observations. Editing them or
an already-added input does not change the Project. Construct the desired new
Project with the same stable identities when updating a dataset.

## Custom note types

Use `NoteType.custom(id)` or `NoteType.custom_cloze(id, cloze_field=...)`, then
chain `.field(Field(...))`, `.template(Template(...))`, `.identity(...)` and
`.css(...)`. Consult [the exact constructors](../../bindings/python/src/anki_forge/notetype.py)
for positional/keyword details.

Fields have display names and stable keys. `required=True` and `optional=True`
are mutually exclusive. `GenerationRule.all([...])` / `.any([...])` use keys;
template HTML references display names. Explicit identities use
`IdentityRecipe.fields([...])`; retain old keys during a 0.1 migration.

## Media

| API | Return / behavior |
| --- | --- |
| `project.media.add_file(path, export_as=...)` | `MediaRef`; reads and fingerprints the file immediately |
| `project.media.add_bytes(source_label=..., data=..., export_as=...)` | `MediaRef`; snapshots non-empty bytes/bytearray up to 64 KiB |
| `MediaRegistry.inline_limit_bytes()` | Current core inline limit |
| `media.image()` / `media.sound()` | Content values using the registered export filename |

Keep file sources available and unchanged until export. A reference from another
Project only resolves if the destination registers that export filename. Deck
has its own `DeckMediaRef` and media rules.

## Build and output

| API | Result | Behavior |
| --- | --- | --- |
| `project.build(options=None)` | `BuildReport` | Default options; temporary artifact without an explicit destination |
| `project.write_apkg(path, *, options=None, ...)` | `BuildReport` | Persistent output; selected comparison/lockfile keywords are also supported |
| `project.diff_against_apkg(path, *, inspect_limits=None)` | `ProjectDiffReport` | No package publication or lockfile advancement |
| `project.to_apkg_bytes(options=None)` | `bytes` | Reads the complete built package into memory |
| `project.write_to(binary_file, options=None)` | Written byte count | Copies a completed package in bounded chunks, leaves the stream open |

Call `ensure_success()` on validation/build/diff reports. Counts are dictionary
entries such as `report.counts["notes"]`. `raw` and `to_json()` retain core fields
and Python integer precision. A deserialized report contains path metadata,
not ownership of a temporary file.

## BuildOptions

`BuildOptions` is immutable; unspecified (`None`) values keep Rust defaults.

| Fields | Purpose |
| --- | --- |
| `output`, `artifacts_dir`, `report_json` | Permanent output and optional retained evidence |
| `inspect`, `inspect_limits` | Inspection settings; `InspectLimits()` reads current core defaults |
| `compare_to`, `fail_on` | Previous APKG and optional risk threshold |
| `identity_lockfile`, `write_identity_lockfile`, `update_safety` | Identity/revision evidence and update policy |
| `self_contained`, `media_mode` | Explicit inline behavior or normal path-backed media |
| `media_policy`, `media_store_dir` | Advanced SDK media configuration |

`.first_update_safe_build(path)` returns strict options that write the first
lockfile. `.update_safe(path)` returns strict options that read it. Explicitly
request `write_identity_lockfile=True` for candidate evidence advancement.
`report_json` needs a persistent output/artifact destination. See
[diagnostics and update-safe builds](diagnostics.md).

## Artifacts, Deck and errors

`report.artifact` is an owning handle when present. Retain it to keep temporary
output alive; `copy.copy(handle)` creates another owner. `persist_to(path)`
returns a persistent handle, and `close()` releases an owner. Explicit outputs
survive cleanup. Reports and artifacts support context managers.

`Deck` offers `add_basic`, `add_cloze`, `add_image_occlusion`, validation and the
same build/output helpers. Deck text follows Rust Deck HTML semantics. See
[the complete native workflow](../../bindings/python/examples/native_workflow.py)
and [Image Occlusion limits](../image-occlusion.md).

Add/media/bundle failures raise structured exceptions; build failures are
reported through `BuildReport`, and `ensure_success()` raises `BuildError` with
the report. Read [diagnostics](diagnostics.md) for paths, codes and import errors.
Each native object permits one operation at a time; use separate objects for
concurrent work. Native objects and Artifact handles cannot be reused after fork.
