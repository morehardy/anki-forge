import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { readFile } from 'node:fs/promises';
const root = process.cwd();
const { rankSearchDocuments } = await import(pathToFileURL(path.resolve(root, process.env.ENGINE_PATH || 'target/remaining-locales/ja-de-infra/search-original.mjs')));
const sample = await readFile(path.join(root, 'docs/plans/evidence/remaining-locales-2026-11-01/samples/ja/quickstart.md'), 'utf8');
const documents = [{ id: 'quickstart', title: sample.match(/^# (.*)$/m)[1], description: '', terms: [], type: 'docs', url: '/fixture/ja/quickstart/' }];

test('actual Japanese title supports composed, decomposed and halfwidth card spellings', () => {
  for (const query of ['カード', 'カード'.normalize('NFD'), 'ｶｰﾄﾞ']) assert.equal(rankSearchDocuments(query, documents, 'ja')[0]?.id, 'quickstart');
});

test('cart must not retrieve the actual card title by deleting its dakuten', () => {
  assert.deepEqual(rankSearchDocuments('カート', documents, 'ja'), []);
});

test('Latin accent folding still works after preserving kana marks', () => {
  const document = [{ ...documents[0], title: 'ursprünglichen Größe' }];
  assert.equal(rankSearchDocuments('ursprunglichen', document, 'de')[0]?.id, 'quickstart');
});
