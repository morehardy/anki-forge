# Rust API guide

Use `anki_forge::prelude::*` for the supported consumer interface. This guide
covers authoring, build options and results. The [API overview](../anki_forge/README.md)
records compatibility and ownership guarantees.

For compiler-generated signatures and all public methods in this checkout, run:

```sh
cargo doc --locked -p anki_forge --no-deps --open
```

Use default features. `internal-tools` exposes repository internals outside the
consumer compatibility promise. Registry-hosted documentation must match an
actually published version; this source guide does not assume one is available.

## Deck and Project

| Operation | Result | Use and requirements |
| --- | --- | --- |
| `Deck::new(name)` | Deck | Short path to Basic, Cloze and Image Occlusion |
| `deck.basic().note(front, back).stable_id(id).add()` | Fallible addition | Deck inputs preserve HTML; stable_id is optional but recommended for source-owned identities |
| `deck.cloze().note(text).add()` | Fallible addition | Text must contain valid cloze markers |
| `deck.image_occlusion().note(image).rect(x, y, w, h).add()` | Fallible addition | Uses a Deck media reference; positive rectangles must fit the image |
| `Project::new(name).stable_id(id).default_deck(name)` | Project | Optional configuration builders; keep the ID across releases |
| `Project::from(deck)` | Project | Moves the Deck, preserving authoring, media and identity evidence |
| `project.add_notetype(note_type)` | `Result<&mut Project, ProjectAddError>` | Validate and add a custom declaration |
| `project.add_note(note)` | `Result<&mut Project, ProjectAddError>` | Reject invalid types/fields/identities before insertion |
| `project.validate()` | `ValidationReport` | An authoring checkpoint; call ensure_success |
| `project.import_template_bundle(path)` | Fallible import | Loads the directory's manifest, templates, CSS and assets |

Start with [installation](installation.md), [cards](cards.md) and
[custom note types](custom-notetypes.md). See the exact
[Project methods](../anki_forge/src/product/project.rs) and
[Deck builders](../anki_forge/src/deck/builders.rs) for generic bounds.

## Notes and content

| API | Behavior |
| --- | --- |
| `Note::basic(front, back)` | Creates stock Basic fields as escaped text |
| `Note::cloze(text).extra(extra)` | Keeps HTML/cloze text; extra is escaped text |
| `Note::new(type_id)` | Starts a note for a declared type |
| `.stable_id(id)`, `.deck(name)`, `.tag(tag)` | Sets explicit identity, destination and tags |
| `.identity(field_keys)` | Supplies selected identity fields for this note |
| `.text(field, value)` | Escapes text |
| `.html(field, value)` | Preserves intended HTML |
| `.image(field, media)`, `.sound(field, media)` | Inserts Anki media references |
| `Note::image_occlusion(media)` | Starts the Project IO builder; configure identity, regions and mode, then build |

Content setters return the Note for chaining. Complete the note before passing
it by value to `add_note`. Field and template keys differ from display names;
see [core concepts](concepts.md#field-keys-and-display-names).

## Note types, fields and templates

| API | Configuration |
| --- | --- |
| `NoteType::custom(id)` | Normal type with fields and one or more templates |
| `NoteType::custom_cloze(id, field_key)` | One selected cloze field and one template; additional fields are allowed |
| `.name(name).field(field).template(template).css(css)` | Note-type display name, fields, templates and styling |
| `.identity(IdentityRecipe::fields(keys))` | Explicit type-level fallback identity recipe |
| `Field::new(display_name).key(stable_key)` | Declares a field; explicit keys are recommended for long-lived decks |
| `.required()` / `.optional()` | Reject empty content / permit omitted content; mutually exclusive |
| `.identity()` / `.sort()` | Select fallback identity input / sort field |
| `Template::new(name).key(key).front(html).back(html)` | A card template referencing field display names |
| `.browser_front(html).browser_back(html).target_deck(name)` | Optional browser rendering and card destination |
| `.generate_when(GenerationRule::all(keys))` | Require all selected fields; any(keys) requires at least one |

Templates default to Anki-style inferred generation. If the inference cannot
represent the template accurately, supply an explicit rule. See the
[complete template guide](template-bundles.md) and source for
[types/fields](../anki_forge/src/product/notetype.rs) and
[templates](../anki_forge/src/product/template.rs).

## Media

Project uses `project.media_mut().add_file(path)?.export_as(filename)?` or
`add_bytes(label, bytes)?.export_as(filename)?`, returning a `MediaRef`.
Registration validates and fingerprints the source. Use files for large media;
Project inline byte input is limited to 64 KiB.

Deck uses `deck.media().add(MediaSource::from_file(path))?` and has its own media
reference type. Do not substitute Project references for Deck references.
The [media tutorial](media.md) shows a complete workflow and naming rules.

## Build options

`BuildOptions::new()` starts with no explicit output, no comparison/lockfile/report
paths, inspection enabled, default inspection limits, and lockfile writing disabled.
Omitted update-safety policy is resolved by the core from the supplied evidence.

| Builder | Purpose / default |
| --- | --- |
| `.output(path)` | Persistent destination; otherwise default output is temporary |
| `.artifacts_dir(path)` | Retains build artifacts in an explicit directory |
| `.report_json(path)` | Writes report JSON; requires output or artifacts_dir |
| `.compare_to(path)` | Reads a previous APKG as update/comparison evidence |
| `.identity_lockfile(path)` | Selects identity evidence; does not itself enable writing |
| `.write_identity_lockfile(true)` | Requests candidate evidence publication; default false |
| `.first_update_safe_build(path)` | Strict mode plus writing the first lockfile |
| `.update_safe(path)` | Strict mode using a lockfile; request writing separately |
| `.update_safety(UpdateSafetyMode::...)` | Explicit Strict, ReportOnly or Disabled |
| `.self_contained()` | Requests inline media payloads instead of normal path-backed handling |
| `.inspect(bool)` | Selects the inspection summary; final package checks still run |
| `.inspect_limits(InspectLimits)` | Overrides finite inspection budgets for candidate and baseline |

The supported prelude does not export every internal policy type. In particular,
do not enable internal-tools just to copy an internal `RiskLevel` example; use
the supported strict update helpers. Node/Python expose their own named risk
threshold options. See [BuildOptions source](../anki_forge/src/build/options.rs)
for the precise interface in this checkout.

## Reports and errors

`project.build(options)` and `project.write_apkg(path)` return a fallible
`BuildReport`; use `?` for the operation and `report.ensure_success()?` for its
outcome. Report fields include `counts`, `diagnostics`, `media`, `inspect`,
`diff`, `risk`, `update_safety` and an optional owning artifact.

`pretty_report()` is for people. `to_report_json()` or automatic report JSON
provides structured output. Match stable diagnostic codes and typed errors,
not human-facing text. Validation is an authoring check; building adds media,
normalization, writer, comparison and publication checks.

Retain the report or an artifact clone to use a temporary path. Persist it with
`artifact.persist_to(path)?` for a permanent copy. See
[build and output guarantees](build-guarantees.md) and [troubleshooting](troubleshooting.md).
