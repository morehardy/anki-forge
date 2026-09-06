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
Deck media supports `addFile(path)` and `addBytes(name, bytes)` using Rust Deck
semantics; file exports use the source basename. Project media additionally
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
the native project until it is released. No data is written into node_modules.

Registered files retain their original fingerprints. Changing a source before
build produces `MEDIA.SOURCE_CHANGED`. Duplicate names and invalid filenames use
Rust errors. A MediaRef denotes its export filename, so cross-project use is
allowed, but the destination must register the media that its notes reference.
`media.image()`/`.sound()` yield Content values; text escaping happens in Rust.

`baseDir` is captured when a Project or Deck is constructed. Relative media,
template, output, comparison, report and lockfile paths always use that directory,
even after `process.chdir()`. Temporary staging is kept in the build workspace.

## Build, compare and output

`build({ output, ...options })` and `writeApkg(output, options)` return a full,
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

`defaultInspectLimits()` reads all defaults from Rust. `firstUpdateSafeBuild(path)`
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
`writeTo(writable)` writes a complete Buffer, awaits the callback and backpressure,
propagates stream errors, and leaves the stream open. APKG generation is not
incrementally streamed.

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
