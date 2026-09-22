# Your first deck with Node

Build one Basic note and one Cloze note using the native Node SDK. The result
contains two notes and three cards. This is a source-checkout workflow for the
0.2 candidate; see [release status](../compatibility.md).

## Build and run from source

Use Node 22.13 or later and Rust 1.92. From the repository root:

```sh
cd bindings/node
npm run setup
npm run build
npm run example:minimal
```

The example prints the APKG's absolute path in a temporary directory. Open that
file with Anki to see **hola → hello** and the two Cloze questions **uno / dos**.
The example helper selects the native binary built for your host.

This is the complete program run by the helper:

<!-- source: bindings/node/examples/basic.mjs -->
```js
import { Project, Note } from 'anki-forge-node';
import os from 'node:os';
import path from 'node:path';
import fs from 'node:fs/promises';

const baseDir = await fs.mkdtemp(path.join(os.tmpdir(), 'anki-forge-example-'));
const project = new Project('Spanish', {
  stableId: 'spanish-a1',
  defaultDeck: 'Spanish::A1',
  baseDir,
});
project.addNote(Note.basic('hola', 'hello', { stableId: 'es:hola' }));
project.addNote(Note.cloze('{{c1::uno}}, {{c2::dos}}', { stableId: 'es:numbers' }));
(await project.validate()).ensureSuccess();
const report = await project.writeApkg('spanish.apkg');
report.ensureSuccess();
console.log(report.prettyReport());
console.log(`APKG: ${report.artifact.path}`);
```
<!-- /source -->

## Install a local build in your application

From `bindings/node`, package the built SDK and its host-platform binary:

```sh
npm run pack:local
```

The command prints an `artifacts/<version>/<platform>/` directory. It contains
the main `anki-forge-node` tarball and a separate platform tarball, plus an
integrity manifest. In your application's directory, install **both actual files**
with `npm install --offline --ignore-scripts --omit=optional`, followed by their
absolute paths. Explicitly installing the platform tarball avoids requesting
unpublished optional platform packages from a registry.

Save the program above as `make-deck.mjs` and run `node make-deck.mjs`. To write
into your application directory, set `baseDir` to `process.cwd()` instead of the
example's generated temporary directory. Use `.mjs` or configure `type: module`.
CommonJS uses `require('anki-forge-node')`; TypeScript consumers should install
TypeScript and `@types/node` as development dependencies.

A future public installation uses the released package and matching platform
binary. Confirm registry availability and platform support before relying on it.

## Await each operation

`addNote` and `addNoteType` are synchronous. Media registration, validation,
importing bundles and exporting are asynchronous. Await each asynchronous
operation before using the same Project/Deck again. Independent objects may work
concurrently. Errors carry structured codes and reports.

Continue with the [Node API](api.md), [core concepts](../concepts.md), or
[update workflow](../updates.md). [Installation troubleshooting](../troubleshooting.md#installation)
covers native load and candidate-package problems.
