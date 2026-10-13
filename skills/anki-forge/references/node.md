# Node / TypeScript: public Anki Forge 0.3.0

Requires Node.js 22.13+. Inspect `package.json` and its lockfile first.
For a new application:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.0
```

Keep optional dependencies enabled: they contain the matching native package.
Save as `main.mjs`:

<!-- source: bindings/node/examples/quickstart.mjs -->
```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```
<!-- /source -->

```sh
node --check main.mjs
node main.mjs
npm ls ankiforge
```

Expect `spanish.apkg` in the application's working directory, with one note,
one Basic card (**hola → hello**) and no media. Check that the file remains after
closing the artifact; `output.report.counts` provides programmatic counts.
Import into Anki when the user wants client verification; export needs no Anki.

Namespace `spanish` and note key `es:hola` identify the source record.
Build and compare are asynchronous. Close artifacts in a `finally` block when
adding more operations. `BuildOptions.to(path)` is persistent; closing an owner
leaves that file on disk. The last owner of temporary output removes its file.
Reports and snapshots do not own files.

TypeScript declarations ship in the package. Follow the existing project config
and type-check with its own command. For a new Node ESM TypeScript application,
consult the [TypeScript setup](https://ankiforge.dev/docs/node-api/#typescript)
before choosing compiler settings. CommonJS uses `require('ankiforge')`.

For advanced tasks use [workflows.md](workflows.md) and the Node API. Node methods
use camelCase, including `defaultDeck`, `withExportName`, `addAsset` and `updateFrom`;
do not copy Rust/Python spellings.

- [Node quickstart](https://ankiforge.dev/docs/node-quickstart/)
- [Node / TypeScript API](https://ankiforge.dev/docs/node-api/)
- [Compatibility](https://ankiforge.dev/docs/compatibility/)
- [Source quickstart](https://github.com/morehardy/anki-forge/blob/main/docs/node/quick-start.md)
