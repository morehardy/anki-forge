import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, copyFile, readFile, writeFile, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { pages } from '../site-model.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const command = fileURLToPath(new URL('./check-content.mjs', import.meta.url));
async function fixture(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-i18n-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const files = new Set([...pages.flatMap(page => Object.values(page.sources)), 'website/content-reviews.json', ...['en', 'zh-CN'].flatMap(locale => [`website/src/i18n/${locale}.json`, `website/src/content/i18n/${locale}.json`])]);
  for (const file of files) { await mkdir(path.dirname(path.join(directory, file)), { recursive: true }); await copyFile(path.join(root, file), path.join(directory, file)); }
  return directory;
}
const run = directory => execFileSync(process.execPath, [command, directory], { encoding: 'utf8', stdio: 'pipe' });
test('the content gate accepts reviewed genuine translations', async t => {
  assert.match(run(await fixture(t)), /paired topics/);
});
test('the content gate rejects missing translations and unregistered sources', async t => {
  const directory = await fixture(t);
  await rm(path.join(directory, 'docs/zh-cn/media.md'));
  assert.throws(() => run(directory), /Unregistered or missing Chinese documentation/);
  await copyFile(path.join(root, 'docs/zh-cn/media.md'), path.join(directory, 'docs/zh-cn/media.md'));
  await writeFile(path.join(directory, 'docs/zh-cn/extra-topic.md'), '# 额外主题\n');
  assert.throws(() => run(directory), /Unregistered or missing Chinese documentation/);
});
test('unregistered drafts are not treated as public topics', async t => {
  const directory = await fixture(t);
  await writeFile(path.join(directory, 'website/src/content/blog/future.mdx'), '---\ndraft: true\n---\nA draft.\n');
  await writeFile(path.join(directory, 'website/src/content/docs/docs/future.mdx'), '---\ndraft: true\n---\nA draft.\n');
  await writeFile(path.join(directory, 'docs/zh-cn/future.md'), '---\ndraft: true\n---\n草稿。\n');
  assert.match(run(directory), /paired topics/);
});
test('the content gate rejects incomplete messages and mismatched interpolation', async t => {
  const directory = await fixture(t);
  const file = path.join(directory, 'website/src/i18n/zh-CN.json');
  const original = JSON.parse(await readFile(file, 'utf8'));
  const missing = { ...original }; delete missing['card.answer'];
  await writeFile(file, JSON.stringify(missing));
  assert.throws(() => run(directory), /message keys/);
  await writeFile(file, JSON.stringify({ ...original, 'card.answer': '答案：{wrong}' }));
  assert.throws(() => run(directory), /mismatched message parameters/);
});
test('technical edits require a paired translation review and fallback is rejected', async t => {
  const directory = await fixture(t);
  const source = path.join(directory, 'docs/concepts.md');
  await writeFile(source, (await readFile(source, 'utf8')) + '\nChanged technical guidance.\n');
  assert.throws(() => run(directory), /English source changed; review both languages/);
  await copyFile(path.join(root, 'docs/concepts.md'), source);
  await copyFile(source, path.join(directory, 'docs/zh-cn/concepts.md'));
  assert.throws(() => run(directory), /fallback/);
});
