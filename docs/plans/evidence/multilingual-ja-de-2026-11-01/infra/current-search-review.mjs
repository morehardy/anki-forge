import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { rankSearchDocuments } from '../../../../../website/src/lib/search.mjs';

const source = await readFile('website/src/lib/search.mjs');
const brief = JSON.parse(await readFile('docs/plans/evidence/remaining-locales-2026-11-01/samples/ja/language-brief.json', 'utf8'));
const body = await readFile('docs/plans/evidence/remaining-locales-2026-11-01/samples/ja/updates.md', 'utf8');
const documents = [{ id: 'updates', type: 'docs', url: '/fixture/ja/updates/', title: body.match(/^# (.*)$/m)[1], description: body.split(/\n\s*\n/)[1], terms: [] }];
const concepts = { note: { terms: [brief.terms.note] } };
const observations = [];
for (const query of ['ノートルダム', 'ノートのフィールド', 'Ankiノートを更新', 'ノート生成']) {
  const result = rankSearchDocuments(query, documents, 'ja', { concepts });
  assert.equal(result.some(item => item.matchedTerms.includes('ノート')), query !== 'ノートルダム');
  observations.push({ query, result: result.map(({ id, matchedTerms }) => ({ id, matchedTerms })) });
}
await writeFile('docs/plans/evidence/multilingual-ja-de-2026-11-01/infra/current-search-review.json', JSON.stringify({ schemaVersion: 1, scope: 'Independent check of the integrator\'s narrow normalization/Katakana-boundary fix against existing Japanese source text. No frozen release questions were read.', sourceSha256: createHash('sha256').update(source).digest('hex'), observations }, null, 2) + '\n');
console.log('Current production fix rejects Notre-Dame and retains particle, Latin-adjacent, and Han-adjacent note aliases.');
