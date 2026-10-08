# Node and TypeScript API

The `ankiforge` entry point exposes the sole Project authoring model.
Types are defined in [the public source](../../bindings/node/src/index.ts),
[report classes](../../bindings/node/src/report.ts), and
[JSON snapshot declarations](../../bindings/node/src/snapshots.ts). Start with
[the quick start](quick-start.md).

## Project and notes

| API | Behavior |
| --- | --- |
| `new Project(namespace)` | Validate a stable publication namespace |
| `.defaultDeck(name)` | Set the default destination; returns the project |
| `.add(key, note)` | Atomically collect note/model/media; duplicate keys fail |
| `.addAsset(media)` | Include an explicit raw HTML/CSS/script resource |
| `.clone()` | Independent editable project sharing immutable values |
| `.length` | Registered note count |
| `Note.basic(front, back)` / `Note.cloze(text)` | Immutable note holding its built-in model |
| `model.note().field(key, content)` | Immutable custom note holding its model |
| `note.deck(name).tag(tag).tags(iterable)` | Return a newly configured note |

Strings always become Text, including Cloze strings. Explicit markup uses
`Content.html`. `Content.sequence(iterable)` combines text, HTML and typed media.
Use explicit business keys; neither content nor display names derive identity.

## Immutable models

```js
import { NoteType, Field, Template, GenerationRule } from 'ankiforge';
const model = NoteType.builder('vocabulary')
  .name('词汇')
  .field(new Field('front', { name: '正面', required: true, sort: true }))
  .field(new Field('back', { name: '背面' }))
  .template(new Template('recognition', {
    front: '{{front}}',
    back: '{{FrontSide}}<hr>{{back}}',
    generation: GenerationRule.all(['front']),
  }))
  .build();
const note = model.note().field('front', 'cell').field('back', '细胞');
```

Builders return new values. A completed model has no field/template mutation
methods. `builder.clozeField(key)` selects custom Cloze semantics; `builder.css`
and `builder.asset` supply styling and explicit dependencies. Templates also
accept `name`, `browserFront`, `browserBack` and `targetDeck`. Template references
and generation conditions use field keys; display names are compiled for Anki.
`GenerationRule.ankiDefault()` selects Anki's default behavior; `all` and `any`
select explicit conditions.

`await NoteType.fromBundle(path, mediaLimits?)` returns the same immutable model
from `template-bundle-v2`. Imported text/assets no longer depend on the source
directory after success. Models can be reused across projects; incompatible
definitions under one model key fail atomically.

## Media and Image Occlusion

`await Media.file(path, limits?)` and
`await Media.bytes(Uint8Array, mime, limits?)` return owned snapshots. Source files
can change or disappear after the await succeeds. `withExportName(name)` returns
a renamed value without changing earlier clones or content. Read `filename`,
`mediaType` and `byteLength` as properties.

`media.image()` and `.sound()` return typed Content that automatically brings its
asset into a project. Explicit `project.addAsset` and model `builder.asset` cover
raw HTML/CSS/script references. Media limits accept `{ maxBytes }`; the default
is 256 MiB and native bytes have no 64 KiB inline restriction.

```js
import { Media, Note, Mask } from 'ankiforge';
const image = await Media.file('diagram.png');
const note = Note.imageOcclusion(image)
  .mask(Mask.rect('nucleus', 10, 10, 20, 20))
  .mask(Mask.rect('wall', 50, 50, 20, 20))
  .mode('hide_one_guess_one')
  .build().field('header', 'Cell');
```

The default mode is `hide_all_guess_one`; both modes generate separate cards for
keyed masks. `.build()` validates decoded dimensions, finite rectangles and
unique keys. `image` and `occlusion` are reserved generated fields. See
[Image Occlusion](../image-occlusion.md) for decoding and update limits.

## Build, compare and output

| API | Result |
| --- | --- |
| `await project.build(BuildOptions.to(path))` | BuildOutput owning persistent artifact |
| `await project.build(BuildOptions.temporary())` | BuildOutput owning temporary artifact |
| `await project.compare(CompareOptions.against(path))` | Completed ComparisonReport, including blocked policy |
| `options.updateFrom(path)` | Configure update from complete original distribution evidence |
| `options.inspectLimits(limits)` | Override independent inspection counters |
| `options.updatePolicy(policy)` | Configure update/compare risk policy |

A successful output exposes `.artifact`, `.report` and `.snapshot()`.
The `BuildReport` exposes `.counts`, `.baselineCounts`, `.durationMs`, `.diagnostics`,
`.comparison`, and `.snapshot()`; it has no outcome or file ownership.
`.comparison` is a `ComparisonReport` or `null`, using the same class as
`project.compare()`. Comparisons expose `.findings`,
`.highestRisk`, `.policy`, `.diagnostics`, and `.snapshot()`.
Snapshot field names mirror Rust JSON, including `allows_publication` and
`schema_version`; they are not camel-cased copies of the native report.
Use `report.comparison.highestRisk` on a comparison object, or
`report.snapshot().comparison.highest_risk` in its JSON snapshot, after checking
for `null`.

`new UpdatePolicy()` blocks High/Critical. `.failOn('medium')` changes the
threshold. `.allow('RISK.NOTE_REMOVED')` explicitly accepts that whole category
while retaining its evidence and severity. Unknown codes and hard errors cannot
be allowed. Policy on a create-only build is an error. Use the same policy and
limits in compare and build when you need matching publication decisions.

Inspection options include `maxArchiveBytes`, `maxEntries`,
`maxCentralDirectoryBytes`, `maxZipEntryBytes`, `maxZipTotalBytes`, `maxMetaBytes`,
`maxMediaMapBytes`, `maxIdentityBytes`, `maxCollectionBytes`, `maxMediaBytes`,
`maxDecodedTotalBytes` and `maxZstdWindowBytes`. Values are nonnegative safe
integers or `bigint` through u64's maximum; zero is a real zero budget. Counters
apply independently to baseline and candidate. Unknown options fail.

## Artifact lifetime and concurrency

JSON snapshot paths use the exported `PathSnapshot` type. Unicode paths are
strings; non-Unicode native paths use `{ encoding: "unix_bytes", bytes: number[] }`
or `{ encoding: "windows_wide", units: number[] }`. The arrays preserve the exact
Unix bytes or Windows UTF-16 code units. This includes artifact paths in build
snapshots, publication paths in failures, and structured error-path details.
Decode an encoded path only for its original platform; on Unix,
`Buffer.from(value.bytes)` preserves a byte path for Node filesystem functions.
The Node authoring methods continue to accept string paths.

Relative media and bundle paths are anchored when the read is invoked;
build destinations and baselines are anchored when options are constructed.
On POSIX, symbolic links followed by `..` retain filesystem traversal semantics.
`persistTo` anchors relative destinations to the working directory captured by
the original build, including on cloned and persisted handles. Later working
directory changes do not redirect these paths.

Keep an artifact owner alive while consuming `.path`. `.clone()` creates an
independent owner; `await .close()` releases that owner. The last temporary owner
deletes its file. `await .persistTo(path)` returns a persistent artifact and
leaves the original usable even on failure. Standard Node file streams can read
an artifact while you retain its owner.

Snapshots are frozen JSON data and retain no native files. A persistent file
survives owner cleanup. `BuildOutput` cannot be constructed by the caller; it is
created only by successful builds.

Build and compare run asynchronously on an owned project snapshot captured at
invocation. Later edits cannot mutate that request. Media reads and bundle loads
also run on workers. Node ESM and CJS share the same native module and class
identities.

## Errors

`SchemaError`, `AddError`, `MediaError`, `TemplateBundleError`,
`ImageOcclusionError`, `PolicyError`, `ConfigurationError`, `CompareError`,
`BuildError`, and `PersistError` extend `ForgeError`. They expose `kind`, `code`,
`domain`, native source-chain text in `causes`, structured `sourceDetails` for recognized
I/O/schema/media/limit causes, and operation `details`.
The original native exception is retained as the JS `cause`.

`BuildError.snapshot()` preserves failure and publication facts;
`BuildError.report` and `CompareError.report` return `BuildReport`, just like
`BuildOutput.report`. Use `.baselineCounts` and `.comparison` on these reports;
use `.snapshot()` for the snake_case JSON fields. A failed comparison can have
partial observations and a `null` comparison.
`PersistError.publication` records whether the target was already published and
whether durability was confirmed. Media/inspection errors retain limit details.
Use these machine fields instead of parsing error messages.
`ArtifactClosedError` reports use of a closed owner; `NativeLoadError` reports
missing, incompatible or stale native packages.

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
validation still applies. See [the design](../plans/2026-09-28-rust-api-validation-and-errors-design.md) for the complete target/detail table.

## Source API details

These additions describe current source; see [source builds](../development.md). Batch media and prepared publication are available at source commit `1199196`; they are absent from public `0.2.0`.

### Prepare, review and publish

```js
import { Project, Note, BuildOptions, BuildError } from 'ankiforge';
const project = new Project('review-once').add('one', Note.basic('Question', 'Answer'));
const previous = await project.build(BuildOptions.to('previous.apkg'));
await previous.artifact.close();
const prepared = await project.preparePublication(
  BuildOptions.to('next.apkg').updateFrom('previous.apkg'),
);
try {
  const report = prepared.report; // immutable observations, no file ownership
  console.log(report.comparison?.snapshot());
  if (report.comparison?.policy.allows_publication !== false) {
    const output = await prepared.publish();
    await output.artifact.close(); // persistent next.apkg remains
  }
} catch (error) {
  if (error instanceof BuildError) console.log(error.snapshot().result);
  // Inspect publication/durability facts: a late failure can follow replacement.
  throw error;
} finally {
  await prepared.close(); // idempotent; cleans an unpublished candidate
}
```

`publish()` takes the native owner immediately and performs one worker task.
Every attempt consumes it, including policy and I/O rejection. Repeated calls
reject with `PreparedPublicationStateError`, code `BUILD.PREPARED_UNAVAILABLE`,
and `reason` `consumed` or `closed`. Closing after publish begins does not cancel
it. Reports remain readable after close. Options bind at preparation invocation;
changing policy or destination requires another preparation. Duration excludes
review waiting. Neither reports nor JSON can be converted back into an owner.

Build, compare, preparation and clones share immutable native Project versions.
Authoring changes remain synchronous and isolated. The first change while another
snapshot lives may copy the Project; later exclusive changes do not. This shifts
copying out of Promise submission and does not promise faster first edits, better
worst timer delay or lower RSS. JavaScript byte inputs are still copied before
worker submission. Use protocol 6 binaries with these wrappers.

## TypeScript

Install `typescript` and `@types/node` in your application. Use `main.mts` or
`"type": "module"` and this `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "outDir": "dist"
  }
}
```

Compile with `npx tsc`, then run `node dist/main.mjs` for an `.mts` entry.
