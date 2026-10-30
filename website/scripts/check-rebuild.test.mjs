import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { captureRebuildSnapshot, assertRebuildMatches } from './check-rebuild.mjs';
import { sha256 } from './site-manifest.mjs';

const identity = { commit: 'a'.repeat(40), inputSha256: 'b'.repeat(64), configuration: { site: 'https://example.test', base: '/', customDomain: '' }, releaseLocales: ['en', 'zh-CN'] };
async function artifact(t, { base = '/', themeName = 'theme-en', changeFile, changeIdentity = {}, omitFile, unrecordedFile, misclassify } = {}) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-rebuild-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const current = { ...structuredClone(identity), ...changeIdentity, configuration: { ...identity.configuration, base } };
  const files = { 'index.html': `<main><input name="${themeName}"></main>`, 'zh-cn/index.html': '<main>中文</main>', '404.html': '<main>Not found</main>', 'docs/updates/index.md': '# Updates\n', 'llms.txt': '# Anki Forge\n', 'zh-cn/llms.txt': '# 中文\n', 'style.css': 'main { color: navy; }' };
  if (changeFile) files[changeFile] += 'changed';
  if (omitFile) delete files[omitFile];
  const manifest = { schemaVersion: 1, ...current, pages: [], resources: [] };
  for (const [file, text] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(directory, file)), { recursive: true });
    await writeFile(path.join(directory, file), text);
    if (file === unrecordedFile) continue;
    const entry = { route: (base === '/' ? '' : base) + '/' + file.replace(/index\.html$/, ''), sha256: sha256(text), bytes: Buffer.byteLength(text) };
    manifest[file.endsWith('.html') && file !== misclassify ? 'pages' : 'resources'].push(entry);
  }
  await mkdir(path.join(directory, 'generated'), { recursive: true });
  await writeFile(path.join(directory, 'generated/site-manifest.json'), JSON.stringify(manifest));
  return { directory, current, snapshot: () => captureRebuildSnapshot(directory, current) };
}

test('two independently verified artifacts preserve exact root and prefix HTML/machine bytes', async t => {
  for (const base of ['/', '/anki-forge']) {
    const first = await artifact(t, { base });
    const second = await artifact(t, { base });
    const before = await first.snapshot();
    assert.equal(before.files.length, 6);
    assert.ok(before.files.some(entry => entry.file === '404.html'));
    const after = await second.snapshot();
    assert.doesNotThrow(() => assertRebuildMatches(before, after));
  }
});
test('actual changed HTML including 404, Markdown and both llms files fail despite valid fresh manifests', async t => {
  const before = await (await artifact(t)).snapshot();
  for (const changeFile of ['index.html', 'zh-cn/index.html', '404.html', 'docs/updates/index.md', 'llms.txt', 'zh-cn/llms.txt']) {
    const changed = await (await artifact(t, { changeFile })).snapshot();
    assert.throws(() => assertRebuildMatches(before, changed), /same-input rebuild changed delivered bytes/);
  }
});
test('fresh self-consistent manifests cannot hide random theme radio group names', async t => {
  const before = await (await artifact(t, { themeName: 'theme-2ae04163-699d-434c-9480-e422037b6aca' })).snapshot();
  const after = await (await artifact(t, { themeName: 'theme-7a936188-aad9-432b-8316-a34fcc31da8d' })).snapshot();
  assert.throws(() => assertRebuildMatches(before, after), /index.html: same-input rebuild changed delivered bytes/);
});
test('source inputs, locale selection and commit cannot change across rebuilding', async t => {
  const before = await (await artifact(t)).snapshot();
  for (const changeIdentity of [{ inputSha256: 'c'.repeat(64) }, { releaseLocales: ['en'] }, { commit: 'd'.repeat(40) }]) {
    const changed = await (await artifact(t, { changeIdentity })).snapshot();
    assert.throws(() => assertRebuildMatches(before, changed), /changed source input identity/);
  }
  const changed = await (await artifact(t, { base: '/project' })).snapshot();
  assert.throws(() => assertRebuildMatches(before, changed), /changed source input identity/);
});
test('scoped omissions, unrecorded files, stale hashes and HTML classified as a resource fail closed', async t => {
  const before = await (await artifact(t)).snapshot();
  const missing = await (await artifact(t, { omitFile: 'zh-cn/llms.txt' })).snapshot();
  assert.throws(() => assertRebuildMatches(before, missing), /file set/);
  await assert.rejects((await artifact(t, { omitFile: '404.html' })).snapshot(), /real 404/);
  await assert.rejects((await artifact(t, { unrecordedFile: 'docs/updates/index.md' })).snapshot(), /cover every delivered/);
  await assert.rejects((await artifact(t, { misclassify: 'zh-cn/index.html' })).snapshot(), /manifest page row/);
  const changed = await artifact(t);
  const file = path.join(changed.directory, 'index.html');
  await writeFile(file, (await readFile(file, 'utf8')).replace('theme-en', 'theme-random-UUID'));
  await assert.rejects(changed.snapshot(), /bytes differ from verified manifest/);
});
