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

test('the third build language participates in draft checks while planned sources stay unread', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-three-locales-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const registry = [{ id: 'example', type: 'docs', sources: { en: 'en.md', 'zh-CN': 'zh.md', es: 'es.md', fr: 'not-authored.md' } }];
  for (const file of ['en.md', 'zh.md', 'es.md']) await writeFile(path.join(directory, file), '---\ndraft: false\n---\nPublished\n');
  assert.equal((await publishedPages(directory, registry, ['en', 'zh-CN', 'es'])).length, 1);
  await writeFile(path.join(directory, 'es.md'), '---\ndraft: true\n---\nDraft\n');
  await assert.rejects(publishedPages(directory, registry, ['en', 'zh-CN', 'es']), /draft status must match/);
  await assert.rejects(publishedPages(directory, registry, ['en', 'de']), /example\/de: missing published source/);
  await assert.rejects(publishedPages(directory, registry, []), /nonempty/);
  await assert.rejects(publishedPages(directory, registry, ['en', 'en']), /unique/);
});
