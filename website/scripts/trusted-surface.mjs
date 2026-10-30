import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, mkdir, writeFile, appendFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// Initial migration only: independently downloaded CI publication 732a40f and
// the separately captured 2c07690 integration artifact. Later runs use the base
// revision's files, never a candidate branch's edited compatibility baseline.
export const bootstrapDigests = {
  'public-surface-baseline.json': '680221986bc657c8e694c13888a7e9e2f60110c3fac183134bb3fab9ef781513',
  'machine-surface-baseline.json': '2741648aed79700eb55d851949e9633e7c7c6eb44a1a34284300ab3513ac3ea5',
};
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

export async function selectTrustedSurfaces(readBase, readCandidate) {
  const names = Object.keys(bootstrapDigests);
  const old = await Promise.all(names.map(name => readBase(name)));
  if (old.every(Boolean)) return Object.fromEntries(names.map((name, index) => [name, old[index]]));
  assert.ok(old.every(value => !value), 'Base revision has an incomplete historic surface; do not replace it with candidate data');
  const result = {};
  for (const name of names) {
    const bytes = await readCandidate(name);
    assert.equal(digest(bytes), bootstrapDigests[name], `${name}: initial baseline differs from independently pinned artifact capture`);
    result[name] = bytes;
  }
  return result;
}

export async function prepareTrustedSurfaces(revision, destination) {
  assert.match(revision || '', /^[0-9a-f]{40}$/, 'Use the immutable PR base or previous production commit');
  const root = fileURLToPath(new URL('../../', import.meta.url));
  execFileSync('git', ['cat-file', '-e', `${revision}^{commit}`], { cwd: root });
  const files = await selectTrustedSurfaces(name => {
    try { return execFileSync('git', ['show', `${revision}:website/${name}`], { cwd: root, stdio: ['ignore', 'pipe', 'pipe'] }); }
    catch (error) {
      if (/does not exist|exists on disk, but not in/.test(String(error.stderr))) return null;
      throw error;
    }
  }, name => readFile(path.join(root, 'website', name)));
  await mkdir(destination, { recursive: true });
  for (const [name, bytes] of Object.entries(files)) await writeFile(path.join(destination, name), bytes);
  if (process.env.GITHUB_ENV) await appendFile(process.env.GITHUB_ENV, `PUBLIC_SURFACE_BASELINE=${path.join(destination, 'public-surface-baseline.json')}\nMACHINE_SURFACE_BASELINE=${path.join(destination, 'machine-surface-baseline.json')}\n`);
  console.log(`Selected immutable historic surfaces from base ${revision}; candidate baseline edits do not authorize removals.`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await prepareTrustedSurfaces(process.argv[2], path.resolve(process.argv[3] || '/tmp/anki-forge-trusted-surfaces'));
