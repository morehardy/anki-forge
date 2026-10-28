import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { publishedPages } from './published-pages.mjs';

test('draft translations are excluded together and mismatched status fails', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-draft-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const registry = [
    { id: 'home', type: 'home' },
    { id: 'blog:example', type: 'article', sources: { en: 'en.mdx', 'zh-CN': 'zh.mdx' } },
  ];
  await writeFile(path.join(directory, 'en.mdx'), '---\ndraft: true\n---\nDraft\n');
  await writeFile(path.join(directory, 'zh.mdx'), '---\ndraft: true\n---\n草稿\n');
  assert.deepEqual((await publishedPages(directory, registry)).map(page => page.id), ['home']);
  await writeFile(path.join(directory, 'zh.mdx'), '---\ndraft: false\n---\nPublished\n');
  await assert.rejects(publishedPages(directory, registry), /draft status must match/);
});
