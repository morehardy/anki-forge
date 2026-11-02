import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { pages, localeIds } from '../../../../../website/site-model.mjs';
import { reviewInputs, reviewInputFiles, digest } from '../../../../../website/scripts/content-review-inputs.mjs';

const root = path.resolve('.');
const base = '231bcd56b71765f71b6700141e7237d8561f20bc';
const output = 'docs/plans/evidence/multilingual-ja-de-2026-11-01/ja';
const priorPath = 'docs/plans/evidence/multilingual-pt-br-2026-11-01/technical-source-review-final.json';
const candidatePath = `${output}/source-candidates-final.json`;
const priorBytes = await readFile(priorPath);
const candidateBytes = await readFile(candidatePath);
const prior = JSON.parse(priorBytes);
const candidate = JSON.parse(candidateBytes);
assert.deepEqual(localeIds, ['en', 'zh-CN', 'es', 'pt-BR', 'ja']);
assert.equal(pages.length, 34);
const scratch = path.join(root, 'target/remaining-locales/ja-de-infra/source-baseline');
const oldLocales = localeIds.filter(locale => locale !== 'ja');
const oldFiles = [...new Set(Object.values(prior.entries).flatMap(translations => oldLocales.flatMap(locale => translations[locale].inputFiles)))].sort();
const bindings = [];
for (const filename of oldFiles) {
  const old = execFileSync('git', ['show', `${base}:${filename}`]);
  const current = await readFile(filename);
  await mkdir(path.dirname(path.join(scratch, filename)), { recursive: true });
  await writeFile(path.join(scratch, filename), old);
  bindings.push({ path: filename, baselineSha256: digest(old), currentSha256: digest(current), unchanged: old.equals(current) });
}
const changed = bindings.filter(item => !item.unchanged).map(item => item.path);
assert.deepEqual(changed, ['docs/ai-quickstart.json', 'website/language-briefs.json', 'website/locale-model.mjs', 'website/release-locales.json', 'website/search-concepts.json', 'website/search-terms.json', 'website/site-model.mjs', 'website/src/lib/search.mjs']);
const oldText = name => execFileSync('git', ['show', `${base}:${name}`], { encoding: 'utf8' });
const jsonAdditions = [];
for (const filename of ['website/language-briefs.json', 'website/search-concepts.json', 'website/search-terms.json']) {
  const current = JSON.parse(await readFile(filename, 'utf8'));
  delete (current.locales ?? current).ja;
  assert.deepEqual(current, JSON.parse(oldText(filename)));
  jsonAdditions.push({ path: filename, finding: 'Removing only the new ja key yields identical baseline data, including all four prior locale values.' });
}
const ai = JSON.parse(await readFile('docs/ai-quickstart.json', 'utf8'));
for (const name of ['prompts', 'requestPlaceholders', 'promptContracts']) delete ai[name].ja;
assert.deepEqual(ai, JSON.parse(oldText('docs/ai-quickstart.json')));
const model = await readFile('website/site-model.mjs', 'utf8');
const jaMapEntries = model.match(/, "ja": "(?:\\.|[^"\\])*"/g);
assert.equal(jaMapEntries.length, 66);
assert.equal(model.replace(/, "ja": "(?:\\.|[^"\\])*"/g, ''), oldText('website/site-model.mjs'));
const entries = [];
for (const page of pages) for (const locale of localeIds) {
  const current = await reviewInputs(page, locale, root);
  const files = await reviewInputFiles(page, locale, root);
  for (const [key, value] of Object.entries(current)) assert.equal(candidate.entries[page.id][locale][key], value, `${page.id}/${locale}/${key}`);
  assert.deepEqual(candidate.entries[page.id][locale].inputFiles, files);
  let priorVerified = false;
  if (locale !== 'ja') {
    const original = await reviewInputs(page, locale, scratch);
    for (const [key, value] of Object.entries(original)) assert.equal(prior.entries[page.id][locale][key], value, `${page.id}/${locale}/baseline/${key}`);
    assert.deepEqual(await reviewInputFiles(page, locale, scratch), prior.entries[page.id][locale].inputFiles);
    assert.equal(prior.entries[page.id][locale].technical.status, 'reviewed');
    assert.deepEqual(prior.entries[page.id][locale].technical.openIssues, []);
    priorVerified = true;
  }
  entries.push({ topic: page.id, locale, ...current, inputFiles: files, immutablePriorInputsVerified: priorVerified });
}
const currentFiles = [...new Set(entries.flatMap(entry => entry.inputFiles))].sort();
const currentBindings = await Promise.all(currentFiles.map(async filename => ({ path: filename, sha256: digest(await readFile(filename)) })));
const report = { schemaVersion: 1, reviewerRun: '/root/ja_de_infra_audit/ja-technical-source-2026-11-01', immutableBaseline: base,
  sourceCandidate: { path: candidatePath, sha256: digest(candidateBytes) }, priorReview: { path: priorPath, sha256: digest(priorBytes) },
  scope: 'Independent technical reading plus actual eight-file shared delta review. No independent search queries read. This input audit supplies bindings, not semantic or artifact approval by itself.',
  baselineInputBindings: bindings, changedReviewInputs: changed, currentBindings, jsonAdditions,
  siteModel: { newJapaneseMapEntries: 66, allOtherBytesIdentical: true },
  currentCandidateEntriesVerified: entries.length, immutablePriorEntriesVerified: entries.filter(entry => entry.immutablePriorInputsVerified).length, entries };
await writeFile(`${output}/source-input-audit.json`, JSON.stringify(report, null, 2) + '\n');
console.log(`Verified ${entries.length} current candidate entries, ${report.immutablePriorEntriesVerified} immutable prior reviews, ${bindings.length} baseline files and ${currentBindings.length} current files. Exactly eight reviewed shared source files changed.`);
