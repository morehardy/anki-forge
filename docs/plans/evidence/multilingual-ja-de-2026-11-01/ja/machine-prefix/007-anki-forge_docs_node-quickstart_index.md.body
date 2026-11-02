# Generate Anki flashcards with Node.js

Canonical page: https://morehardy.github.io/anki-forge/docs/node-quickstart/
Language: en
Description: Install ankiforge from npm, generate Anki flashcards with Node.js or TypeScript, and export an APKG deck you can import into Anki.

Use Node.js 22.13 or later. In a new application directory:

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.1
```

This installs the [public npm package](https://www.npmjs.com/package/ankiforge) and the matching native optional dependency. See [verified environments](https://morehardy.github.io/anki-forge/docs/compatibility/) for platform coverage.

## Export a deck

Save this as `main.mjs`:

```js
import { Project, Note, BuildOptions } from 'ankiforge';

const project = new Project('spanish').defaultDeck('Spanish');
project.add('es:hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path);
await output.artifact.close();
```

```sh
node main.mjs
```

Open the persistent `spanish.apkg` in Anki. It contains **hola → hello** in **Spanish**. Keep namespace `spanish` and key `es:hola` stable for later edits. Closing the persistent artifact leaves the file on disk. Exporting does not require Anki to be installed.

## TypeScript and next tasks

The [JSON to Anki recipe](https://morehardy.github.io/anki-forge/docs/json-to-anki/#node) includes a complete TypeScript compile-and-run path.

Use `.mjs` for ESM, or `require('ankiforge')` for CommonJS. TypeScript declarations ship with the package; see [TypeScript configuration](https://morehardy.github.io/anki-forge/docs/node-api/#typescript).

-   [Images and audio](https://morehardy.github.io/anki-forge/docs/media/) and [Node media API](https://morehardy.github.io/anki-forge/docs/node-api/#media-and-image-occlusion)
-   [Custom note types](https://morehardy.github.io/anki-forge/docs/custom-notetypes/) and [template bundles](https://morehardy.github.io/anki-forge/docs/templates/)
-   [Image Occlusion](https://morehardy.github.io/anki-forge/docs/image-occlusion/)
-   [Compare and update](https://morehardy.github.io/anki-forge/docs/updates/)
-   [Node API](https://morehardy.github.io/anki-forge/docs/node-api/) and [troubleshooting](https://morehardy.github.io/anki-forge/docs/troubleshooting/)

Source builds and local packaging are in [source builds](https://morehardy.github.io/anki-forge/docs/development/).
