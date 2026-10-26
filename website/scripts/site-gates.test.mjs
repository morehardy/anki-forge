import test from 'node:test';
import assert from 'node:assert/strict';
import { cp, mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import os from 'node:os';
import { withBase, siteConfig } from '../site.config.mjs';
const dist = fileURLToPath(new URL('../dist/', import.meta.url));
const checker = fileURLToPath(new URL('./check-site.mjs', import.meta.url));
for (const [name, mutate, expected] of [
  ['missing action', html => html.replace(/data-update(?=[\s>])/, 'data-removed'), /update\/reset action|missing translated capability/],
  ['broken anchor', html => html.replace('id="hero-title"', 'id="removed-heading"').replace('href="#main"', 'href="#hero-title"'), /missing anchor/],
  ['wrong-language link', html => html.replace(`href="${withBase('/zh-cn/docs/languages/', siteConfig.base)}"`, `href="${withBase('/docs/languages/', siteConfig.base)}"`), /cross-language link/],
]) test(`built-site gate rejects ${name}`, async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-artifact-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const file = path.join(directory, 'zh-cn/index.html');
  const original = await readFile(file, 'utf8');
  const changed = mutate(original);
  assert.ok(changed !== original, 'Mutation must affect the real artifact');
  await writeFile(file, changed);
  assert.throws(() => execFileSync(process.execPath, [checker, directory], { encoding: 'utf8', stdio: 'pipe' }), expected);
});
