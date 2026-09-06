import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { root } from './platforms.mjs';

// A single CJS implementation keeps instanceof and native module state identical
// when an application mixes require() and import().
await fs.writeFile(path.join(root, 'dist/cjs/package.json'), '{"type":"commonjs"}\n');
const sdk = createRequire(import.meta.url)(path.join(root, 'dist/cjs/index.js'));
const exports = Object.keys(sdk).sort();
await fs.writeFile(
  path.join(root, 'dist/index.mjs'),
  `import sdk from './cjs/index.js';\nexport const { ${exports.join(', ')} } = sdk;\n`,
);
await fs.writeFile(path.join(root, 'dist/index.d.mts'), "export * from './cjs/index.js';\n");
