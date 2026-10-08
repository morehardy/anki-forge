# ankiforge

Node.js 22.13+ SDK using the default public Rust API. Install with optional dependencies enabled: `npm install --include=optional ankiforge`.

Supported native runtimes: macOS arm64/x64, Linux x64 glibc, and Windows x64.
For source builds, candidate tarballs and first-publication setup, see the
[release runbook](RELEASING.md) and the repository's release status.

A Project is one publication. Its namespace and each note key are stable identity; display names and content may change. Strings are text; use `Content.html` for intentional markup.

```js
import { Project, Note, Content, BuildOptions } from 'ankiforge';
const project = new Project('biology').defaultDeck('Biology');
project.add('cell', Note.basic('What is a <cell>?', Content.html('<b>Life’s basic unit</b>')));
const output = await project.build(BuildOptions.to('biology-v1.apkg'));
console.log(output.report.counts, output.snapshot());
await output.artifact.close(); // Persistent files remain on disk.
```

Models are validated and immutable. Fields and templates have explicit keys; templates reference keys, while Anki receives display names. A note holds its model, so no separate registration is required. Builder methods return new values.

```js
import { Project, NoteType, Field, Template, BuildOptions } from 'ankiforge';
const model = NoteType.builder('vocabulary')
  .name('词汇')
  .field(new Field('front', { name: '正面', required: true }))
  .field(new Field('back', { name: '背面' }))
  .template(new Template('recognition', { front: '{{front}}', back: '{{FrontSide}}<hr>{{back}}' }))
  .build();
const output = await new Project('vocabulary')
  .add('cell', model.note().field('front', 'cell').field('back', '细胞'))
  .build(BuildOptions.temporary());
try { console.log(output.report.snapshot()); }
finally { await output.artifact.close(); }
```

Media imports take owned snapshots immediately. Source files can subsequently change or disappear. `Media.bytes(Uint8Array, mime, limits?)` supports assets larger than 64 KiB. Typed image/sound content collects dependencies automatically; `project.addAsset(media)` and `builder.asset(media)` include raw HTML/CSS/script assets explicitly. Fixed names are validated and conflicting names fail atomically.

```js
import { Project, Note, Media, Mask, BuildOptions } from 'ankiforge';
const image = (await Media.file('diagram.png')).withExportName('diagram.png');
const note = Note.imageOcclusion(image)
  .mask(Mask.rect('nucleus', 10, 10, 20, 20))
  .mask(Mask.rect('wall', 50, 50, 20, 20))
  .mode('hide_all_guess_one')
  .build().field('header', 'Cell structure');
const output = await new Project('diagram').add('cell', note).build(BuildOptions.temporary());
await output.artifact.close();
```

`NoteType.fromBundle(path, mediaLimits?)` loads the new `anki-template.yaml` format (`template-bundle-v2`) with a complete owned asset closure. `Content.sequence` combines text, HTML and typed media. Notes and models can be reused across projects.

Updates use the previous original distribution APKG, including its complete embedded identity evidence. Compare completes even when policy blocks publication; build throws a `BuildError` for the same blocked policy. Missing/corrupt evidence is always an error. Default policy blocks High and Critical findings. `new UpdatePolicy().allow('RISK.NOTE_REMOVED')` explicitly accepts that whole category and preserves its evidence. APKG omission does not delete learners’ existing notes.

```js
import { Project, Note, BuildOptions, CompareOptions } from 'ankiforge';
const first = await new Project('update-example').add('cell', Note.basic('Cell?', 'Unit of life'))
  .build(BuildOptions.temporary());
try {
  const next = new Project('update-example').add('cell', Note.basic('Cell?', 'The basic unit of life'));
  const comparison = await next.compare(CompareOptions.against(first.artifact.path));
  console.log(comparison.findings, comparison.policy);
  const output = await next.build(BuildOptions.temporary().updateFrom(first.artifact.path));
  await output.artifact.close();
} finally { await first.artifact.close(); }
```

`BuildOutput` always owns an artifact; `BuildReport` is observations only. Success and error `.report` accessors both return `BuildReport`; its `.comparison` is a `ComparisonReport` or `null`, with the same interface as `project.compare()`. `snapshot()` produces JSON data without extending temporary-file lifetime. `artifact.clone()` creates another owner; the last `close()` deletes a temporary file. `persistTo(path)` returns a persistent owner and retains publication facts on failure. Use standard Node file streams on `artifact.path` while retaining the artifact.

Errors have `kind`, `code`, `causes` (native source-chain text), structured `sourceDetails` for recognized native causes, and operation `details`. `BuildError.snapshot()` contains the failure, report and publication facts; `CompareError.report` contains observations from incomplete analysis. No error code is inferred from human wording. Build and compare capture the project at invocation and run on a worker; later additions cannot mutate an in-flight request.

`BuildOptions.inspectLimits` and `CompareOptions.inspectLimits` accept nonnegative safe integer or `bigint` budgets (through `u64::MAX`) such as `{ maxArchiveBytes: 1_000_000, maxCollectionBytes: 100_000_000 }`. All twelve Rust inspection counters are available, independently applied to baseline and candidate. Media constructors accept `{ maxBytes }` before reading. Zero is a real zero budget. Unknown options are rejected by the native transport.

This is an intentional breaking API: no Deck authoring container, media registry, mutable NoteType, implicit note identity, lockfile options, or legacy runtime export remains. Local development: `npm run setup`, `npm run build`, `npm test`, `npm run test:installed`. Tests inspect real APKG note fields, GUIDs, card counts and media bytes, and compare an independent Rust producer.

### Addition diagnostics

Projects have a stable namespace and optional default deck, with no title.
Image and sound constructors remain infallible; adding a note checks retained
MIME categories: image/* for images, audio/* or video/* for sound references.
Other categories raise AddError with code NOTE.MEDIA_USAGE_INVALID. Raw HTML and
explicit assets retain their existing behavior; category checking does not promise
decodability. Build still performs its independent MIME/extension checks.

`AddError.details` has deeply frozen typed `context` and nullable `detail`.
The context includes `noteKey`, `modelKey` and a discriminated `target`; field
targets carry `fieldKey`, `contentPath` and `byteRange`.
Native protocol 6 is required, including for native packages also labelled 0.2.0.
A field path is null/None for the field, [] for root content, or zero-based
indices into the original sequences. Byte ranges use UTF-8 offsets in the
original text/HTML leaf. Nested source details preserve the same facts.

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
