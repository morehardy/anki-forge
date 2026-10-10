<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/morehardy/anki-forge/main/docs/assets/brand/ankiforge-dark.svg">
    <img src="https://raw.githubusercontent.com/morehardy/anki-forge/main/docs/assets/brand/ankiforge.svg" alt="Anki Forge logo: stacked cards with a folded corner and card loop" width="96" height="96">
  </picture>
</p>

<h1 align="center">anki-forge</h1>

<p align="center">
  <a href="https://github.com/morehardy/anki-forge/actions/workflows/contract-ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/morehardy/anki-forge/contract-ci.yml?branch=main&label=tests%20passing" alt="Tests passing"></a>
  <a href="https://www.npmjs.com/package/ankiforge"><img src="https://img.shields.io/npm/v/ankiforge?logo=npm" alt="npm version"></a>
  <a href="https://ankiforge.dev/docs/"><img src="https://img.shields.io/badge/docs-ankiforge.dev-blue" alt="Documentation"></a>
  <a href="https://github.com/morehardy/anki-forge/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">Node.js / TypeScript SDK</p>

Build Basic, Cloze, Image Occlusion and custom cards, package media, and check updates before distributing a new deck.

## Install

Node.js 22.13+; a matching native package is installed through optional dependencies. See [verified environments](https://ankiforge.dev/docs/compatibility/).
The public package is [`ankiforge` 0.3.0](https://www.npmjs.com/package/ankiforge).

```sh
mkdir anki-deck
cd anki-deck
npm install --include=optional ankiforge@0.3.0
```

## Your first deck

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

Open the persistent `spanish.apkg` in Anki. It contains one Basic card in **Spanish**, with **hola → hello**. Keep `spanish` and `es:hola` stable when editing this publication.

## Continue

- [Complete quickstart](https://ankiforge.dev/docs/node-quickstart/)
- [Images and audio](https://ankiforge.dev/docs/media/)
- [Custom note types](https://ankiforge.dev/docs/custom-notetypes/) and [template bundles](https://ankiforge.dev/docs/templates/)
- [Image Occlusion](https://ankiforge.dev/docs/image-occlusion/)
- [Compare and update](https://ankiforge.dev/docs/updates/)
- [Node API](https://ankiforge.dev/docs/node-api/)
- [Build SDKs from source](https://ankiforge.dev/docs/development/)

Use `.mjs` for ESM, or `require('ankiforge')` for CommonJS. TypeScript declarations ship with the package; see the [TypeScript configuration](https://ankiforge.dev/docs/node-api/#typescript).
