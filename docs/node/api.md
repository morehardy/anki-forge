# Node and TypeScript API

This reference covers the public product entry point in `anki-forge-node`.
Begin with the [Node quickstart](quick-start.md). Full TypeScript signatures ship
with the package and are defined in the [public source](../../bindings/node/src/index.ts)
and [option types](../../bindings/node/src/types.ts).

## Project and notes

| API | Return / behavior | Defaults and requirements |
| --- | --- | --- |
| `new Project(name, { stableId, defaultDeck, baseDir })` | Owns a Rust Project | Options are optional; baseDir captures the current directory |
| `project.addNote(note)` | `void`; validates and adds synchronously | Note type and field references must be valid |
| `project.addNoteType(type)` | `void`; validates and adds synchronously | Declare custom types before adding their notes |
| `await project.validate()` | `ValidationReport` | Authoring checks; does not publish an APKG |
| `await project.importTemplateBundle(directory)` | `void` | Resolves relative to baseDir; failed imports are atomic |
| `Note.basic(front, back, options)` | Immutable Note | Project Basic content is text-escaped |
| `Note.cloze(text, options)` | Immutable Note | Preserves HTML/cloze markers; options include backExtra and stableId |
| `Note.custom(typeId, options)` | Immutable Note | Use field setters before adding it |
| `.text/.html/.image/.sound(field, value)` | A new Note value | Choose content handling explicitly |

Use `NoteType`, `Field`, `Template`, `GenerationRule` and `IdentityRecipe` for
custom types. [The SDK's authoring examples](../../bindings/node/README.md#authoring)
cover full declarations. Template HTML references display names; rules use keys.

## Media and Deck

`await project.media.addFile(path, { exportAs })` returns a `MediaRef` and records
the source fingerprint. `addBytes(label, bytes, options)` accepts Buffer or
Uint8Array with the core Project 64 KiB limit. `addBuffer` spools larger payloads
into private temporary files. Keep registered files available and unchanged
through build. `media.image()` and `.sound()` return Content values.

`new Deck(name, options)` provides Basic, Cloze and Image Occlusion conveniences.
Deck's media references and registration rules are separate from Project media.
Use [the Deck example](../../bindings/node/README.md#deck-and-image-occlusion) and
[Image Occlusion limits](../image-occlusion.md) for its exact behavior.

## Build, compare and output

| API | Result | Important behavior |
| --- | --- | --- |
| `await project.build(options = {})` | `BuildReport` | No destination means a temporary artifact |
| `await project.writeApkg(path, options = {})` | `BuildReport` | Explicit persistent output |
| `await project.diffAgainstApkg(path, { inspectLimits })` | `ProjectDiffReport` | Compares without publishing a package or advancing lockfiles |
| `await project.toApkgBuffer()` | Complete `Buffer` | First-build defaults; no publication options |
| `await project.writeTo(writable)` | `void` | Copies a completed APKG; keeps the caller's stream open |

Use `report.ensureSuccess()` before consuming the result. Report fields include
counts, diagnostics, media, inspection, diff, risk, policy and update safety.
`report.raw` preserves core fields; unsafe JavaScript integer values become
decimal strings instead of being rounded.

## Build options

| Option | Default / purpose |
| --- | --- |
| `output`, `artifactsDir`, `reportJson` | Unset; choose permanent output, retained artifacts or a report path |
| `inspect`, `inspectLimits` | Core defaults; `defaultInspectLimits()` returns all current budgets |
| `compareTo` | Unset; read the last distributed APKG as a baseline |
| `failOn` | Unset; choose info/low/medium/high/critical as a risk threshold |
| `identityLockfile`, `writeIdentityLockfile` | Unset / core false; reading does not automatically advance the file |
| `updateSafety` | Core-selected when omitted; strict/report-only/disabled when explicit |
| `selfContained`, `mediaMode` | Normal path-backed behavior unless explicitly changed |
| `mediaStoreDir`, `mediaPolicy` | Optional advanced SDK media configuration |

`firstUpdateSafeBuild(path)` returns strict options that write initial identity
evidence; `updateSafe(path)` reads existing evidence. Add
`writeIdentityLockfile: true` when the next candidate should write a lockfile.
Unknown options and invalid combinations fail early.

## Errors, lifetime and concurrency

Synchronous authoring failures use `ProjectAddError`; media and bundle failures
use `MediaError` and `TemplateBundleError`. Build failures retain their report
in `BuildError`. Inspect stable codes and paths rather than message wording.

Retain the owning artifact handle for temporary files and persist outputs that
must survive cleanup. See the current [SDK output contract](../../bindings/node/README.md#build-compare-and-output)
and public declarations for the artifact API supported by your checkout.

One asynchronous operation may use an object at a time. Competing operations
fail with `ProjectBusyError`; unrecoverable native failures retire the object.
There is no cancellation guarantee. The old CLI wrapper is a distinct
`anki-forge-node/legacy` entry point; its runtime options do not apply here.
