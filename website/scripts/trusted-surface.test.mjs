import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { bootstrapDigests, selectTrustedSurfaces } from './trusted-surface.mjs';

test('CI trusts the base revision even when a candidate edits its own baseline', async () => {
  const selected = await selectTrustedSurfaces(name => Buffer.from(`verified:${name}`), () => { throw new Error('Candidate baseline must not be read'); });
  for (const [name, bytes] of Object.entries(selected)) assert.equal(bytes.toString(), `verified:${name}`);
});
test('the initial migration accepts only the pinned independent captures', async () => {
  const selected = await selectTrustedSurfaces(() => null, name => readFile(new URL(`../${name}`, import.meta.url)));
  assert.deepEqual(Object.keys(selected), Object.keys(bootstrapDigests));
  await assert.rejects(selectTrustedSurfaces(() => null, () => Buffer.from('{"pages":[]}')), /initial baseline differs/);
});
test('a partial base cannot fall back to a permissive candidate baseline', async () => {
  await assert.rejects(selectTrustedSurfaces(name => name.startsWith('public') ? Buffer.from('base') : null, () => Buffer.from('candidate')), /incomplete historic surface/);
});
