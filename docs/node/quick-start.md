# Your first deck with Node

Use Node.js 22.13 or later. In a new application directory:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.2.0
```

This installs the [public npm package](https://www.npmjs.com/package/ankiforge) and the matching native optional dependency.
See [verified environments](../compatibility.md) for platform coverage.

## Export a deck

Save this as `main.mjs`:

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
node main.mjs
```

Open the persistent `spanish.apkg` in Anki. It contains **hola → hello** in **Spanish**.
Keep namespace `spanish` and key `es:hola` stable for later edits. Closing the persistent artifact leaves the file on disk.
Exporting does not require Anki to be installed.

## TypeScript and next tasks

Use `.mjs` for ESM, or `require('ankiforge')` for CommonJS. TypeScript declarations ship with the package;
see [TypeScript configuration](api.md#typescript).

- [Images and audio](../media.md) and [Node media API](api.md#media-and-image-occlusion)
- [Custom note types](../custom-notetypes.md) and [template bundles](../template-bundles.md)
- [Image Occlusion](../image-occlusion.md)
- [Compare and update](../updates.md)
- [Node API](api.md) and [troubleshooting](../troubleshooting.md)

Source builds and local packaging are in [source builds](../source-builds.md).
