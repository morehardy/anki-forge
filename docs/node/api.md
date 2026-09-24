# Node and TypeScript API

The `anki-forge-node` entry point exposes the sole Project authoring model.
Types are defined in [the public source](../../bindings/node/src/index.ts) and
[JSON snapshot declarations](../../bindings/node/src/snapshots.ts). Start with
[the quick start](quick-start.md).

## Project and notes

| API | Behavior |
| --- | --- |
| `new Project(namespace)` | Validate a stable publication namespace |
| `.name(title).defaultDeck(name)` | Set project display values; returns the project |
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
import { NoteType, Field, Template, GenerationRule } from 'anki-forge-node';
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
import { Media, Note, Mask } from 'anki-forge-node';
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
The report exposes `.counts`, `.baselineCounts`, `.durationMs`, `.diagnostics`,
`.comparison`, and `.snapshot()`;
it has no outcome or file ownership. Comparisons expose `.findings`,
`.highestRisk`, `.policy`, `.diagnostics`, and `.snapshot()`.
Snapshot field names mirror Rust JSON, including `allows_publication` and
`schema_version`; they are not camel-cased copies of the native report.

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
`BuildError.report` and `CompareError.report` expose observations.
`PersistError.publication` records whether the target was already published and
whether durability was confirmed. Media/inspection errors retain limit details.
Use these machine fields instead of parsing error messages.
`ArtifactClosedError` reports use of a closed owner; `NativeLoadError` reports
missing, incompatible or stale native packages.
