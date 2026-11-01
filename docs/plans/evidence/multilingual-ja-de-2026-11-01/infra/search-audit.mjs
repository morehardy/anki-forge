import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const root = process.cwd();
const output = path.join(root, 'docs/plans/evidence/multilingual-ja-de-2026-11-01/infra');
const scratch = path.join(root, 'target/remaining-locales/ja-de-infra');
const samples = 'docs/plans/evidence/remaining-locales-2026-11-01/samples';
const inputs = [];
async function read(file) {
  const bytes = await readFile(path.join(root, file));
  inputs.push({ path: file, sha256: createHash('sha256').update(bytes).digest('hex') });
  return bytes.toString('utf8');
}
await mkdir(scratch, { recursive: true });
// Keep the red-case snapshot immutable after the product fix lands.
const snapshot = path.join(scratch, 'search-original.mjs');
const source = await readFile(snapshot, 'utf8').catch(async error => {
  if (error.code !== 'ENOENT') throw error;
  return read('website/src/lib/search.mjs');
});
const before = ".normalize('NFD').replace(/\\p{M}/gu, '').replace(/[’']/g, '')";
const after = ".normalize('NFD').replace(/(\\p{Script=Latin})\\p{M}+/gu, '$1').normalize('NFC').replace(/[’']/g, '')";
if (!source.includes(before)) throw new Error('The frozen source normalization has changed; inspect before rerunning.');
await writeFile(snapshot, source, { flag: 'wx' }).catch(error => { if (error.code !== 'EEXIST') throw error; });
await writeFile(path.join(scratch, 'search-latin-marks-only.mjs'), source.replace(before, after));
const original = await import(pathToFileURL(path.join(scratch, 'search-original.mjs')));
const proposed = await import(pathToFileURL(path.join(scratch, 'search-latin-marks-only.mjs')));
const frozen = JSON.parse(await read(`${samples}/probes/queries.json`));
const corpora = {};
for (const locale of ['ja', 'de']) {
  const brief = JSON.parse(await read(`${samples}/${locale}/language-brief.json`));
  const ui = JSON.parse(await read(`${samples}/${locale}/home-ui.json`));
  const documents = [{ id: 'home', type: 'home', url: `/fixture/${locale}/`, title: ui['home.title'], description: ui['home.description'], terms: [], body: Object.values(ui).join('\n') }];
  for (const id of ['quickstart', 'json-to-anki', 'updates', 'python-api']) {
    const body = await read(`${samples}/${locale}/${id}.md`);
    const title = body.match(/^# (.*)$/m)[1];
    const description = body.split(/\n\s*\n/).find(text => text.trim() && !text.startsWith('#') && !text.startsWith('<!--'));
    documents.push({ id, type: 'docs', url: `/fixture/${locale}/${id}/`, title, description, terms: [], body });
  }
  corpora[locale] = { documents, concepts: Object.fromEntries(['note', 'card', 'deck', 'namespace', 'baseline'].map(id => [id, { terms: [brief.terms[id]] }])) };
}
const summaries = result => result.slice(0, 3).map(({ id, score, matchedTerms }) => ({ id, score, matchedTerms }));
const observations = [];
for (const [locale, corpus] of Object.entries(corpora)) {
  const probes = [...frozen[locale].map(test => ({ ...test, origin: 'previously frozen sample diagnostic' })),
    ...(locale === 'ja' ? [
      { query: 'カード', origin: 'actual sample term: cards', expected: 'card evidence' },
      { query: 'カート', origin: 'dakuten counterexample: cart differs from card', expected: null },
      { query: 'パス', origin: 'actual sample term: file path', expected: 'path evidence' },
      { query: 'バス', origin: 'handakuten counterexample: bus differs from path', expected: null },
      { query: 'ノートルダム', origin: 'alias boundary counterexample: proper noun differs from note', expected: null },
    ] : [
      { query: 'Große', origin: 'actual sample spelling: Große Medien', expected: 'python-api' },
      { query: 'Grosse', origin: 'ss spelling comparison; no production synonym assertion', expected: 'diagnostic only' },
      { query: 'ursprünglichen', origin: 'actual sample spelling', expected: 'diagnostic only' },
      { query: 'ursprunglichen', origin: 'Latin accent folding control', expected: 'diagnostic only' },
      { query: 'Stapelaktualisierung', origin: 'isolate compound from the previously frozen sample query', expected: 'diagnostic only' },
    ])];
  for (const [engine, { rankSearchDocuments }] of [['original', original], ['latin-marks-only-proposal', proposed]]) {
    for (const mode of ['metadata-only', 'sample-body-lexical', 'sample-body-brief-aliases']) for (const probe of probes) {
      const result = rankSearchDocuments(probe.query, corpus.documents, locale, {
        concepts: mode === 'sample-body-brief-aliases' ? corpus.concepts : {},
        pagefindResults: mode === 'metadata-only' ? [] : corpus.documents.map(document => ({ url: document.url, content: document.body, excerpt: document.description })),
      });
      observations.push({ locale, engine, mode, ...probe, top3: summaries(result), resultCount: result.length });
    }
  }
}
const outputData = {
  schemaVersion: 1,
  scope: 'Read-only infrastructure diagnostics on unchanged historical ja/de samples. Body inputs are synthetic Pagefind-shaped fixture inputs, not an actual index. No production vocabulary or independent release query file was changed. The proposed engine exists only in excluded scratch storage; these results do not authorize locale release.',
  head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
  node: process.version, icu: process.versions.icu, inputs,
  observations,
};
await writeFile(path.join(output, 'search-observations.json'), JSON.stringify(outputData, null, 2) + '\n');
for (const result of observations.filter(item => item.origin.includes('counterexample'))) console.log(result.locale, result.engine, result.mode, result.query, JSON.stringify(result.top3));
