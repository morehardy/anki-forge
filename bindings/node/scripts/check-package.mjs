import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { root, repo, targets } from './platforms.mjs';

const manifest = JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8'));
const cargo = await fs.readFile(path.join(root, 'native/Cargo.toml'), 'utf8');
assert.match(cargo, new RegExp(`version\\s*=\\s*"${manifest.version.replaceAll('.', '\\.')}"`));
const coreVersion = (await fs.readFile(path.join(repo, 'anki_forge/Cargo.toml'), 'utf8')).match(
  /^version\s*=\s*"([^"]+)"/m,
)[1];
const contractVersion = (
  await fs.readFile(path.join(repo, 'contracts/manifest.yaml'), 'utf8')
).match(/^bundle_version:\s*['"]?([^'"\s]+)/m)[1];
const require = createRequire(import.meta.url);
const all = process.argv.includes('--all');
for (const target of targets) {
  const directory = path.join(root, 'npm', target.suffix);
  const platform = JSON.parse(await fs.readFile(path.join(directory, 'package.json'), 'utf8'));
  assert.equal(platform.version, manifest.version);
  assert.equal(manifest.optionalDependencies[platform.name], manifest.version);
  const binary = path.join(directory, 'anki-forge.node');
  const host = target.os === process.platform && target.cpu === process.arch;
  if (all || host) assert.ok((await fs.stat(binary)).size > 0, `Missing ${target.target} binary`);
  if (host) {
    const metadata = JSON.parse(require(binary).bindingMetadata());
    assert.equal(metadata.bindingVersion, manifest.version);
    assert.equal(metadata.coreVersion, coreVersion);
    assert.equal(metadata.contractVersion, contractVersion);
    assert.equal(metadata.target, target.target);
    assert.equal(metadata.nodeApiVersion, 8);
  }
  assert.equal(platform.scripts?.install, undefined);
  assert.ok(platform.files.includes('THIRD_PARTY_NOTICES.md'));
  assert.equal(
    await fs.readFile(path.join(directory, 'THIRD_PARTY_NOTICES.md'), 'utf8'),
    await fs.readFile(path.join(root, 'THIRD_PARTY_NOTICES.md'), 'utf8'),
  );
}
assert.equal(manifest.scripts?.install, undefined);
for (const file of [
  'dist/index.mjs',
  'dist/index.d.mts',
  'dist/cjs/index.js',
  'dist/cjs/index.d.ts',
  'README.md',
  'LICENSE',
  'THIRD_PARTY_NOTICES.md',
])
  await fs.access(path.join(root, file));
console.log(`Package metadata, versions and ${all ? 'all' : 'host'} native artifacts: passed`);
