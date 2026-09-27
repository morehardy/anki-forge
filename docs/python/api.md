# Python API

The SDK calls the default Rust public API through a native extension. It includes `py.typed` and checked public signatures. Start with the [quickstart](quick-start.md).

## Project and note values

| Entry | Behavior |
| --- | --- |
| `Project(namespace, *, name=None, default_deck=None)` | Validates the explicit stable namespace; display name defaults to namespace |
| `project.add(key, note)` | Atomically collects the note, model and owned media; duplicate note keys fail |
| `project.add_asset(media)` | Includes an explicit raw HTML/CSS/script dependency |
| `len(project)` | Number of successfully added notes |
| `Note.basic(front, back)` | Built-in Basic model; strings mean Text |
| `Note.cloze(text)` | Built-in Cloze model; strings mean Text and retain cloze syntax |
| `note.field(key, content)` | Returns a new note with the field assignment; key validation occurs on add |
| `note.deck(name)` / `.tag(tag)` / `.tags(tags)` | Returns a configured note |
| `note.note_type` | The immutable owned model |

Ordinary strings become Text. `Content.text(value)` and `Content.html(value)` are explicit constructors; `Content.sequence(iterable)` composes strings and content without rendering or losing media dependencies. A field does not accept a raw filename as a media reference.

## Models and templates

`NoteType.builder(key)` returns an immutable builder. Its `.name`, `.field`, `.template`, `.css`, `.cloze_field` and `.asset` methods return updated builders. `.build()` validates the complete definition and returns an immutable NoteType. `model.note()` creates a note owning that model. `NoteType.from_bundle(path, limits=MediaLimits())` loads the same model and its assets from the current bundle format; the optional budget applies separately to each asset before reading.

`Field(key, name=None, required=False, sort=False)` separates the stable key from the display name. `Template(key, front, back, name=None, browser_front=None, browser_back=None, target_deck=None, generation=GenerationRule())` uses field keys in every source string. `GenerationRule.all(keys)` and `.any(keys)` configure card generation. The default infers Anki's field-presence condition. A builder's `.cloze_field(key)` declares custom cloze generation.

## Media and occlusion

| Entry | Behavior |
| --- | --- |
| `Media.file(path, *, limits=MediaLimits())` | Reads an owned snapshot immediately |
| `Media.bytes(data, media_type, *, limits=MediaLimits())` | Owns bytes with an explicit MIME type |
| `media.with_export_name(name)` | Returns a new filename value sharing the snapshot; previous references retain their names |
| `media.filename`, `media.media_type`, `len(media)` | Read-only snapshot metadata |
| `media.image()` / `.sound()` | Typed content that retains the snapshot |

`MediaLimits(max_bytes=256 << 20)` controls each import before snapshot creation. Large media can spill to owned temporary storage. Naming conflicts are validated across all explicit and automatically collected assets.

`Note.image_occlusion(media)` returns an ImageOcclusionBuilder. Chain `.mask(Mask.rect(key, x, y, width, height))`, optionally `.mode(OcclusionMode.HIDE_ONE_GUESS_ONE)`, then `.build()`. The default mode is `HIDE_ALL_GUESS_ONE`. Completion decodes the image and validates stable mask keys and finite in-bounds pixel coordinates. On the returned Note, set `header`, `back_extra` or `comments` normally. The generated `image` and `occlusion` fields cannot be assigned manually.

## Operations and results

| Entry | Result |
| --- | --- |
| `project.build(BuildOptions.to(path))` | BuildOutput with a guaranteed persistent artifact |
| `project.build(BuildOptions.temporary())` | BuildOutput with an owned temporary artifact |
| `options.update_from(path)` | Returns options for updating from complete baseline evidence |
| `project.compare(CompareOptions.against(path))` | Completed ComparisonReport, including a separate publication policy decision |
| `options.update_policy(policy)` | Applies explicit risk policy to Update/Compare; invalid for Create |
| `options.inspect_limits(limits)` | Applies finite limits independently to baseline and candidate |

BuildOutput has `.artifact`, `.report` and `.snapshot()`. BuildReport exposes `.counts` (BuildCounts with notes/cards/media), `.diagnostics`, `.comparison` and `.snapshot()`. A report has no outcome or file owner. ComparisonReport exposes `.findings`, `.policy`, `.allows_publication` and `.snapshot()`. Snapshots are ordinary JSON-serializable dictionaries and contain no artifact ownership.

ApkgArtifact exposes `.path`, `.persist_to(path)` and `.close()`. Copying it with `copy.copy` retains another owner; a context manager releases that owner on exit. Temporary output disappears when its last owner is released. Persistent paths survive handle cleanup. A failed persist keeps the source handle usable.

`UpdatePolicy()` blocks High and Critical findings. `.fail_on(RiskLevel.MEDIUM)` changes the threshold. `.allow('RISK.NOTE_REMOVED')` explicitly accepts the whole registered category, preserving evidence. Unknown codes fail immediately. `InspectLimits` exposes all finite byte/count budgets, including `max_identity_bytes`; it cannot disable required inspections.

## Errors and concurrency

Failures raise SchemaError, AddError, MediaError, ImageOcclusionError, TemplateBundleError, CompareError, PolicyError, BuildError or PersistError. Each exposes `kind`, `code`, `details`, structured observations in `source_details`, and source-chain text in `causes`, and retains the native exception as `__cause__`. I/O causes retain a chained OSError where available. BuildError provides `.report` and `.snapshot()`; CompareError provides partial `.report`; PersistError details include actual publication facts.

Native Project operations release the GIL and reject simultaneous use of the same object with `BINDING.PROJECT_BUSY`; independent projects can run concurrently. After a process fork, recreate all native values and handles. Inherited handles reject operations with `BINDING.FORKED_OBJECT`, and their cleanup cannot remove parent-owned files. Child-created values operate and clean up normally. See [diagnostics](diagnostics.md).
