# Your first publication with Node

The SDK uses Rust's default public API through a native addon. A Project has an
explicit namespace; every note has an explicit source key. A successful build
returns an artifact and observations. See [release status](../compatibility.md)
before choosing a registry package.

## Build and run from source

Use Node 22.13+ and Rust 1.92+. From the repository root:

```sh
cd bindings/node
npm run setup
npm run build
npm run example:minimal
```

The helper selects your rebuilt host binary. This source example creates a
publication, compares an edited version and builds the update. Its temporary
artifacts are closed after use:

<!-- source: bindings/node/examples/basic.mjs -->
```js
import {
  Project,
  Note,
  Content,
  BuildOptions,
  CompareOptions,
} from "../dist/index.mjs";
const project = new Project("example").name("Example").defaultDeck("Learning");
project.add("hello", Note.basic("Hello <world>", Content.html("<b>你好</b>")));
const first = await project.build(BuildOptions.temporary());
try {
  const next = new Project("example").defaultDeck("Learning");
  next.add("hello", Note.basic("Hello <world>", "你好，世界"));
  console.log(
    (
      await next.compare(CompareOptions.against(first.artifact.path))
    ).snapshot(),
  );
  const updated = await next.build(
    BuildOptions.temporary().updateFrom(first.artifact.path),
  );
  console.log(updated.snapshot());
  await updated.artifact.close();
} finally {
  await first.artifact.close();
}
```
<!-- /source -->

To retain an APKG for manual import, choose `BuildOptions.to('example.apkg')`
instead of temporary output, or call `output.artifact.persistTo(path)` while the
original owner remains alive. Closing a persistent artifact does not delete it.

## Install a local build

From `bindings/node`, run:

```sh
npm run pack:local
```

Install both actual tarballs printed by that command: the facade and matching
host-native package. For an unpublished local build, pass their absolute paths
to `npm install --offline --ignore-scripts --omit=optional`. Installing the
host-native tarball explicitly avoids fetching a candidate platform version from
a registry.

Application code imports from `anki-forge-node`; the repository example above
uses its sibling built `dist` directory. A minimal installed application is:

```js
import { Project, Note, BuildOptions } from 'anki-forge-node';
const project = new Project('spanish').defaultDeck('Spanish');
project.add('hola', Note.basic('hola', 'hello'));
const output = await project.build(BuildOptions.to('spanish.apkg'));
console.log(output.artifact.path, output.report.counts);
await output.artifact.close();
```

Use `.mjs` or configure `type: module`. CommonJS can
`require('anki-forge-node')`; both formats share the same implementation and
class identities. TypeScript declarations ship with the package.

## Values and asynchronous operations

`add` and completed-model construction are synchronous. Media import, bundle
loading, comparison, build, artifact persistence and artifact close are
asynchronous. Await their results. Build and compare capture project state when
invoked, so later additions cannot alter an in-flight request. Clone a project
when you want independent future edits.

Strings mean Text for every note kind. Use `Content.html` for intentional HTML.
Models and media are reusable immutable values; no separate registration step
is required. Continue with the [Node API](api.md), [core concepts](../concepts.md),
or [update workflow](../updates.md).
