# anki-forge Node SDK

The 0.2 candidate exposes the Rust product API through TypeScript and Node-API.
It owns real Rust `Project` and `Deck` objects. APKG generation, validation,
identity, media fingerprints, templates and update safety run in Rust.

**Release status:** implemented and tested locally on macOS arm64. npm publication
and the complete platform matrix are pending. The package layout and isolated npm
installation are executable now; a public `npm install anki-forge-node` is the
intended released installation, not a claim that this version is already available.

## Use the product API

After the packages are released:

```sh
npm install anki-forge-node
```

No Rust compiler, Cargo, repository checkout, CLI, manifest path or installation
script is required by consumers. Node 22.13+ is the minimum declared runtime;
Node 22/24 are the required lines and Node 26 is the compatibility test line.

```js
import { Project, Note } from 'anki-forge-node';

const project = new Project('Spanish', {
  stableId: 'spanish-a1',
  defaultDeck: 'Spanish::A1',
  baseDir: process.cwd(),
});
project.addNote(Note.basic('hola', 'hello', { stableId: 'es:hola' }));
project.addNote(Note.cloze('{{c1::uno}} / {{c2::dos}}', {
  stableId: 'es:numbers', backExtra: 'One / two', tags: ['numbers'],
}));
(await project.validate()).ensureSuccess();
const report = await project.writeApkg('spanish.apkg');
report.ensureSuccess();
console.log(report.counts, report.artifact.path);
```

CommonJS uses `const { Project, Note } = require('anki-forge-node')` and exports
the same class objects. Type declarations cover both import modes.
TypeScript applications should have `typescript` and `@types/node` installed as
development dependencies.

## Authoring

`Note.basic`, `Note.cloze`, `Note.custom` and `Note.imageOcclusion` create immutable
input values. Chain `.text(field, value)`, `.html(field, value)`, `.image(field,
media)`, `.sound(field, media)` or `.field(field, Content)` to get a new value.
Pass the result to `project.addNote()`; Rust validates the addition synchronously.
A rejected addition leaves the project unchanged. Options include `stableId`,
`deckName`, `tags` and `identity` field keys; Cloze also accepts `backExtra`.

```js
import { Project, Note, NoteType, Field, Template, GenerationRule } from 'anki-forge-node';

const project = new Project('Vocabulary', { stableId: 'vocabulary' });
project.addNoteType(NoteType.custom('vocabulary-card', {
  name: 'Vocabulary Card',
  fields: [
    new Field('Expression', { key: 'expr', identity: true, required: true }),
    new Field('Meaning', { key: 'meaning', sort: true }),
    new Field('Audio', { key: 'audio', optional: true }),
  ],
  templates: [new Template('Recognition', {
    key: 'recognition', front: '{{Expression}} {{Audio}}',
    back: '{{FrontSide}}<hr id="answer">{{Meaning}}',
    generateWhen: GenerationRule.all(['expr']),
  })],
  css: '.card { font-family: sans-serif; }',
}));
const sound = await project.media.addFile('./hola.wav', { exportAs: 'hola.wav' });
project.addNote(Note.custom('vocabulary-card', { stableId: 'es:hola' })
  .text('expr', 'hola').text('meaning', 'hello').sound('audio', sound));
(await project.writeApkg('vocabulary.apkg')).ensureSuccess();
```

`NoteType.customCloze(id, clozeFieldKey, options)` creates custom Cloze types.
Templates support `key`, `browserFront`, `browserBack`, `targetDeck` and
`generateWhen`. Generation rules are `ankiDefault()`, `all(keys)`, `any(keys)`
and `cloze(key)`. `IdentityRecipe.fields(keys)` supplies an explicit note-type
recipe; core field identity flags also remain available.

`await project.importTemplateBundle(directory)` imports `anki-template.yaml`,
templates, CSS and assets through Rust. Errors retain their code, path and byte
offset; an import is atomic. `validateTemplate(source, declaredFieldNames)` returns
semantic diagnostics without building. `project.validate()` aggregates the same
checks as Rust `Project.validate()`; it does not create an APKG or certify file
availability, complete normalization or update safety. Those checks run at build.

## Deck and image occlusion

```js
import { Deck } from 'anki-forge-node';
const deck = new Deck('Quick deck', { stableId: 'quick', basicIdentity: ['front'] });
deck.basic('hola', 'hello');
deck.cloze('{{c1::uno}}', { extra: 'one' });
const image = await deck.media.addFile('./diagram.png');
deck.imageOcclusion(image, {
  rects: [{ x: 10, y: 20, width: 50, height: 30 }],
  mode: 'hide-all-guess-one', header: 'Diagram', backExtra: 'Explanation',
});
(await deck.writeApkg('quick.apkg')).ensureSuccess();
```

Deck retains its own Rust identity rules, inferred identity and rectangle bounds
validation. Basic identity can select `front`/`back`; per-note `identityOverride`
takes `{ fields, reasonCode }`. `DeckMediaRef` is distinct from Project `MediaRef`.
Deck media supports `addFile(path)`, `addBytes(name, bytes)` and synchronous
`get(filename)` using Rust Deck semantics; file exports use the source basename. Project media additionally
supports renamed exports and large Buffer spooling.

Project image occlusion uses `Note.imageOcclusion(projectMedia, { stableId, rects,
mode, header, backExtra, comments, tags })`. Its stable ID requirement and checks
follow Rust Project. **Known core limitation:** the current `hide-one-guess-one`
renderer emits grouped `c1,2` markup that the Rust writer rejects with
`PRODUCT.CLOZE_MARKER_MALFORMED`. The SDK preserves this failure. Use the working
`hide-all-guess-one` path until the core is updated; the complete plan remains
open for this limitation.

## Media and paths

Project media methods are asynchronous: `addFile(path, { exportAs })`,
`addBytes(sourceLabel, bytes, { exportAs })`, and
`addBuffer(sourceLabel, bytes, { exportAs })`. `bytes` accepts Buffer or Uint8Array
and is snapshotted before asynchronous work starts. `addBytes` preserves Rust's
64 KiB limit; `addBuffer` stores larger data in private temporary files owned by
the native project and its clones until their last owner is released. No data is written into node_modules.

Registered files retain their original fingerprints. Changing a source before
build produces `MEDIA.SOURCE_CHANGED`. Duplicate names and invalid filenames use
Rust errors. A MediaRef denotes its export filename, so cross-project use is
allowed, but the destination must register the media that its notes reference.
`media.image()`/`.sound()` yield Content values; text escaping happens in Rust.

`baseDir` is captured when a Project or Deck is constructed. Relative media,
template, output, comparison, report and lockfile paths always use that directory,
even after `process.chdir()`. Temporary staging is kept in the build workspace.

## Build, compare and output

`build(options?)` and `writeApkg(output, options)` return a full,
deeply frozen BuildReport. `BuildError` retains `code`, `failureCause` and `report`.
Diagnostics preserve code, severity, domain, stage, path, byte span, message and
suggested fix. Report properties include counts, media entries, metrics, current
and previous inspection summaries, diff, risk, policy, comparison and update
safety. `.raw` preserves core field names; large integers outside JavaScript's
safe range become decimal strings instead of being rounded. Unknown options fail
early. No generic CLI flags are exposed on this interface.

| Option | Behavior |
| --- | --- |
| `artifactsDir`, `reportJson` | Explicit retained staging or JSON report |
| `inspect`, `inspectLimits` | Current and baseline inspection with 11 finite budgets |
| `mediaMode`, `selfContained` | Core path-backed/self-contained media behavior |
| `mediaStoreDir`, `mediaPolicy` | Media storage and existing diagnostic policies |
| `compareTo`, `failOn` | Baseline APKG and risk threshold |
| `identityLockfile`, `writeIdentityLockfile`, `updateSafety` | Core update safety and publication protection |

All 11 `InspectLimits` fields accept non-negative safe `number` values or exact
`bigint` values through `18446744073709551615n`. Unsafe numbers, negative values,
fractions and overflow are rejected before work starts. The same rules apply to
build and diff; small budgets still fail inspection. `defaultInspectLimits()`
reads safe-number defaults from Rust. `firstUpdateSafeBuild(path)`
returns strict options that write the initial lockfile. `updateSafe(path)` returns
strict options that read the lockfile; explicitly set `writeIdentityLockfile:
true` when publishing an updated lockfile. A stable project ID is required for
strict proof. Modes are `strict`, `report-only` and `disabled`.

`await project.diffAgainstApkg(path, { inspectLimits })` creates a temporary
candidate and returns a ProjectDiffReport without publishing an APKG or advancing
an identity lockfile. Policy-blocked builds preserve existing output and baseline
files using Rust's path, alias and atomic publication checks.

`toApkgBuffer()` returns a complete Buffer using first-build defaults, with private
temporary output cleaned up after reading. It accepts no publication options.
`writeTo(writable)` completes the build first, then reads the owned temporary APKG
in chunks of at most 64 KiB. It awaits write callbacks and backpressure, propagates
stream errors, releases the source and temporary file, and leaves the target open.
This bounds SDK transfer buffers; it does not bound Rust build memory or storage
inside the caller's Writable. APKG generation is not incrementally streamed.

### Artifact ownership

`build()` creates a temporary APKG. `build({ output: 'file.apkg' })` publishes a
persistent file; `build({ artifactsDir: 'retained' })` retains `package.apkg` in
that directory. JSON reporting requires a persistent destination as in Rust.

`report.artifact` and `report.raw.artifact` remain `{ path }` snapshots. The new
`report.artifactHandle` owns the real Rust artifact. Its `clone()` creates a
separate owner, `persistTo(path)` atomically copies it to a persistent destination,
and `await close()` releases that owner. Closing the last temporary owner removes
the file; closing persistent owners leaves their files. `close()` is idempotent;
other operations on a closed owner raise `ArtifactClosedError`. I/O failures raise
`ArtifactError` without destroying the source. Keep a clone for independent use.

```js
import { Project, Note } from 'anki-forge-node';
const project = new Project('Deferred output');
project.addNote(Note.basic('one', '1'));
const report = await project.build();
try {
  report.ensureSuccess();
  const saved = await report.artifactHandle.persistTo('chosen-later.apkg');
  await saved.close(); // The persistent file remains.
} finally {
  await report.artifactHandle?.close();
}
const retained = await project.build({ artifactsDir: 'retained-artifacts' });
await retained.artifactHandle.close(); // Retained package.apkg remains.
```

Late `BuildError.report` values also retain an artifact when Rust has already
produced one. Inspect, clone or persist that handle before closing it. Retaining a
report keeps its owner alive; retaining only a path or JSON does not. A manual
`new BuildReport(raw, pretty)` has `artifactHandle === null`. Garbage collection
is a fallback; explicit close provides predictable cleanup. Paths passed to
persistTo resolve against the source Project/Deck's captured baseDir. In-flight
persistence owns its own snapshot, so closing the original does not interrupt it.

### Conversion, clones and readonly descriptions

```js
import { Deck, Project, Note, Field } from 'anki-forge-node';
const deck = new Deck('Variants', { stableId: 'variants' });
deck.basic('<b>HTML front</b>', 'answer');
const project = await Project.fromDeck(deck);
const variant = await project.clone();
variant.addNote(Note.cloze('{{c1::extra}}', { stableId: 'extra' }));
console.log((await deck.describe()).notes.length); // 1; the Deck remains usable.
console.log(new Field('Prompt').describe().key); // Canonical Rust key.
const report = await variant.build({ inspectLimits: { maxArchiveBytes: 9007199254740993n } });
await report.artifactHandle.close();
```

`Project.fromDeck(deck)` copies the current Rust state into an editable Project,
including identities, raw Deck HTML, tags, registered media and their original
fingerprints. It inherits baseDir and accepts no metadata overrides. `clone()` on
Project or Deck creates independent mutable state; immutable spooled media shares
resource ownership. Source and result may be edited and built independently.
These operations copy O(n) state in a worker and reserve the source immediately.

Field, Template, NoteType, IdentityRecipe, GenerationRule and Note expose
synchronous `describe()`; Deck exposes asynchronous `describe()`. These return
typed, deeply frozen snapshots of Rust values. Note observations include actual
stock field names and rendered content; Deck observations include resolved
identities. They neither add a note nor validate an incomplete NoteType against a
Project. Descriptions do not expose a mutable normalization IR. Deck.validate()
retains the existing Project-level checks and mapped `project.deck` diagnostics;
Deck.build() calls Rust Deck.build() directly.

Await each asynchronous operation before using the same Project or Deck again.
A conflicting operation throws/rejects `ProjectBusyError`. Domain failures restore
the object for reuse; an unrecoverable Rust panic produces `ProjectFailedError`.
There is no timeout/AbortSignal API that claims to cancel a running Rust build.
Independent objects can run concurrently.

## Legacy migration

The previous CLI wrapper is available at `anki-forge-node/legacy`. Move old imports
there if you still use raw normalize/build/inspect/diff or product documents. It
retains CLI/contracts runtime configuration; those options are unnecessary for the
new root API. `productBuild` accepts `baseDir` for inline document media.
`productValidate` is a compatibility preview that builds into temporary storage;
it rejects `apkgOut`, `reportJson` and `writeIdentityLockfile: true`. Use the new
Project validation API for direct, publication-free Rust validation.

## Develop and verify

From this directory with Node 22.13+ and Rust 1.92:

`setup` installs the private `toolchain` package using its own lockfile. This keeps
source development reproducible before the platform packages are published.
Installed consumers do not need these development tools.

```sh
npm run setup
npm run build
npm test
npm run test:parity
npm run test:legacy
npm run test:installed
npm run example:minimal
npm run check:package
```

`build -- --release` creates an optimized native runtime. `--target <Rust target>`
selects a cross target when its toolchain/linker is installed. Development tests
use an explicit absolute `ANKI_FORGE_NATIVE_PATH`; installed-consumer tests remove
it and install real tarballs through a disposable local registry.

Candidate platform packages: darwin-arm64, darwin-x64, win32-x64-msvc and
linux-x64-gnu. CI builds Linux on Ubuntu 22.04; lower glibc baselines are unverified.
Alpine/musl, Linux ARM64, Windows ARM64, Electron, Bun and browser use are outside
the current matrix. See [release procedure](RELEASING.md) for open release gates.
The [coverage index](COVERAGE.md) maps C01–C18 to tests and records remaining
verification. `npm run prepare:desktop` retains SDK-generated APKGs, hashes and
Rust/Node comparison evidence with a pending Anki Desktop checklist.

The main and native packages must have matching versions and binding protocol 2.
The loader rejects an older native protocol even if a development candidate kept
its package version. Rebuild all platform packages together before release.
Rust `Project.lower()` remains callable in source but is outside this SDK's
Supported Consumer Interface; the legacy CLI API does not lower a live Project.
See [the parity implementation record](../../docs/plans/2026-09-22-node-api-parity-progress.md).
