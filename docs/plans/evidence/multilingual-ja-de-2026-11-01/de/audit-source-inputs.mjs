// Independent reviewer utility. It records byte/lineage evidence only; semantic
// findings and final technical status must be supplied by the actual reviewer.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { pages, localeIds, docGroups, navigation } from '../../../../../website/site-model.mjs';
import { reviewInputs, reviewInputFiles, digest } from '../../../../../website/scripts/content-review-inputs.mjs';

const root = fileURLToPath(new URL('../../../../../', import.meta.url));
const evidence = 'docs/plans/evidence/multilingual-ja-de-2026-11-01';
const baseline = '270f26aa5653a0f56297f46fd35e586da444622b';
const scratch = path.join(root, 'target/remaining-locales/de-technical');
const baselineRoot = path.join(scratch, 'baseline-270');
const candidatePath = process.argv[2] || `${evidence}/de/source-candidates-preview.json`;
const outputPath = process.argv[3] || `${evidence}/de/source-input-audit-preview.json`;
const bytes = async filename => readFile(path.join(root, filename));
const json = async filename => JSON.parse(await bytes(filename));
const priorPath = `${evidence}/ja/technical-source-review-final.json`;
const prior = await json(priorPath);
const candidate = await json(candidatePath);
const readHashes = await json(`${evidence}/de/technical-input-hashes.json`);
const patchPath = `${evidence}/de/authoring/de-data-patch.json`;
const patch = await json(patchPath);
assert.deepEqual(localeIds, ['en', 'zh-CN', 'es', 'pt-BR', 'ja', 'de']);
assert.equal(pages.length, 34);

const oldFiles = [...new Set(Object.values(prior.entries).flatMap(translations => Object.values(translations).flatMap(entry => entry.inputFiles)))].sort();
const baselineBindings = [];
for (const filename of oldFiles) {
  assert(!filename.includes('queries'), 'Withheld queries must not be read');
  const old = execFileSync('git', ['show', `${baseline}:${filename}`], { cwd: root, maxBuffer: 8 * 1024 * 1024 });
  const current = await bytes(filename);
  await mkdir(path.dirname(path.join(baselineRoot, filename)), { recursive: true });
  await writeFile(path.join(baselineRoot, filename), old);
  baselineBindings.push({ path: filename, baselineSha256: digest(old), currentSha256: digest(current), unchanged: old.equals(current) });
}
const expectedChanged = ['docs/ai-quickstart.json', 'website/language-briefs.json', 'website/release-locales.json', 'website/search-concepts.json', 'website/search-terms.json', 'website/site-model.mjs'];
assert.deepEqual(baselineBindings.filter(entry => !entry.unchanged).map(entry => entry.path), expectedChanged);
// Review code is not in the page closure. Verify its behavior is the same as at
// the accepted prior baseline before using it to reproduce the prior hashes.
const reviewTools = [];
for (const filename of ['website/scripts/content-review-inputs.mjs', 'website/scripts/content-sections.mjs']) {
  const old = execFileSync('git', ['show', `${baseline}:${filename}`], { cwd: root });
  const current = await bytes(filename);
  assert(old.equals(current), `${filename}: review contract changed`);
  reviewTools.push({ path: filename, sha256: digest(current), unchangedFromBaseline: true });
}
const priorEntries = [];
const currentEntries = [];
const currentFiles = new Set();
for (const page of pages) for (const locale of localeIds) {
  const inputs = await reviewInputs(page, locale, root);
  const files = await reviewInputFiles(page, locale, root);
  assert.deepEqual(files, candidate.entries[page.id][locale].inputFiles);
  const candidateMatches = Object.keys(inputs).every(key => inputs[key] === candidate.entries[page.id][locale][key]);
  currentEntries.push({ entry: `${page.id}/${locale}`, ...inputs, candidateMatches });
  for (const filename of files) { assert(!filename.includes('queries')); currentFiles.add(filename); }
  if (locale === 'de') continue;
  const previous = prior.entries[page.id][locale];
  assert.equal(previous.technical.status, 'reviewed');
  assert(previous.technical.evidence.length > 0);
  assert(!previous.technical.openIssues.some(issue => ['major', 'critical'].includes(issue.severity)));
  const oldInputs = await reviewInputs(page, locale, baselineRoot);
  for (const [key, value] of Object.entries(oldInputs)) assert.equal(value, previous[key], `${page.id}/${locale}: prior ${key}`);
  assert.deepEqual(await reviewInputFiles(page, locale, baselineRoot), previous.inputFiles);
  priorEntries.push({ entry: `${page.id}/${locale}`, priorHashesReproducedAtBaseline: true, evidenceCount: previous.technical.evidence.length, changedHashes: Object.keys(inputs).filter(key => inputs[key] !== oldInputs[key]) });
}
assert.equal(priorEntries.length, 170);
assert.equal(currentEntries.length, 204);
const formalGerman = [];
const reviewedContractInputs = [];
for (const entry of readHashes.files) {
  const stagedPrefix = `${evidence}/de/staging/`;
  const filename = entry.path.startsWith(stagedPrefix) ? entry.path.slice(stagedPrefix.length) : entry.path;
  const current = await bytes(filename);
  const changed = digest(current) !== entry.sha256;
  if (changed) {
    assert(expectedChanged.includes(filename), `${filename}: changed since independent semantic reading; re-read delta required`);
    assert.equal(baselineBindings.find(item => item.path === filename)?.baselineSha256, entry.sha256, `${filename}: earlier reading not bound to baseline`);
  }
  const binding = { path: filename, reviewedPath: entry.path, reviewedSha256: entry.sha256, currentSha256: digest(current), readMode: entry.readMode, changedSharedDeltaReviewedSeparately: changed };
  (entry.path.startsWith(stagedPrefix) ? formalGerman : reviewedContractInputs).push(binding);
}
assert.equal(formalGerman.length, 33);
const before = async filename => JSON.parse(await readFile(path.join(baselineRoot, filename)));
const additions = {};
for (const [filename, keys] of [
  ['docs/ai-quickstart.json', ['prompts', 'requestPlaceholders', 'promptContracts']],
  ['website/search-terms.json', ['locales']], ['website/search-concepts.json', ['locales']],
]) {
  const current = await json(filename);
  const original = await before(filename);
  additions[filename] = {};
  for (const key of keys) { additions[filename][key] = current[key].de; delete current[key].de; }
  assert.deepEqual(current, original, `${filename}: non-German data changed`);
}
const briefs = await json('website/language-briefs.json');
additions['website/language-briefs.json'] = briefs.de;
delete briefs.de;
assert.deepEqual(briefs, await before('website/language-briefs.json'));
const release = await json('website/release-locales.json');
release.locales = release.locales.filter(locale => locale !== 'de');
assert.deepEqual(release, await before('website/release-locales.json'));
const model = (await bytes('website/site-model.mjs')).toString();
const added = [...model.matchAll(/,\s*"de":\s*"(?:\\.|[^"\\])*"/g)];
assert.equal(added.length, 66);
assert.equal(model.replace(/,\s*"de":\s*"(?:\\.|[^"\\])*"/g, ''), await readFile(path.join(baselineRoot, 'website/site-model.mjs'), 'utf8'));
for (const page of pages.filter(page => page.type === 'docs')) assert.deepEqual({ title: page.title.de, description: page.description.de }, patch.documents[page.id]);
assert.deepEqual(docGroups.map(group => group.labels.de), patch.docGroups);
assert.deepEqual(Object.fromEntries(navigation.map(item => [item.id, item.label.de])), patch.navigation);
assert.deepEqual(additions['website/language-briefs.json'], patch.languageBrief);
for (const key of ['prompts', 'requestPlaceholders', 'promptContracts']) assert.deepEqual(additions['docs/ai-quickstart.json'][key], patch[key].de);
const vocabularyDelta = {};
for (const [filename, authored] of [['website/search-terms.json', patch.searchTerms], ['website/search-concepts.json', patch.searchConcepts]]) {
  const current = additions[filename].locales;
  for (const key of new Set([...Object.keys(authored), ...Object.keys(current)])) {
    if (JSON.stringify(current[key]) !== JSON.stringify(authored[key])) (vocabularyDelta[filename] ??= {})[key] = { authored: authored[key], current: current[key] };
  }
}
const currentBindings = [];
for (const filename of [...currentFiles].sort()) {
  const data = await bytes(filename);
  currentBindings.push({ path: filename, sha256: digest(data) });
  await mkdir(path.dirname(path.join(scratch, 'current-review-snapshot', filename)), { recursive: true });
  await writeFile(path.join(scratch, 'current-review-snapshot', filename), data);
}
const report = {
  schemaVersion: 1, reviewerRun: '/root/de_technical_review/de-source-incremental-2026-11-01', immutableBaseline: baseline,
  scope: 'Actual baseline reconstruction and prior 170-entry hash reproduction, 33 formal German files checked against independently fully read staging, six-file shared delta inspection, and recorded candidate bindings. Mechanical evidence supports semantic review; it is not itself a source or artifact approval. Withheld query files were not read.',
  priorReview: { path: priorPath, sha256: digest(await bytes(priorPath)), originalReviewScope: prior.reviewScope },
  sourceCandidate: { path: candidatePath, sha256: digest(await bytes(candidatePath)), all204EntriesCurrent: currentEntries.every(entry => entry.candidateMatches) },
  reviewTools, baselineInputBindings: baselineBindings, priorEntryChecks: priorEntries,
  formalGermanBindings: formalGerman, reviewedContractBindings: reviewedContractInputs,
  sharedDeltaChecks: { changedFiles: expectedChanged, nonGermanJsonValuesIdentical: true, only66GermanModelValuesAdded: true, exactPriorModelBytesRecovered: true, releaseMenuAddsOnlyGerman: true, metadataBriefPromptMatchReadAuthoring: true, vocabularyDeltaFromReadAuthoring: vocabularyDelta },
  currentInputBindings: currentBindings, currentEntryBindings: currentEntries,
  openIssues: [], status: 'input-audit-complete-semantic-final-approval-pending',
};
await writeFile(path.join(root, outputPath), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ outputPath, priorEntries: priorEntries.length, germanFiles: formalGerman.length, baselineInputs: baselineBindings.length, currentInputs: currentBindings.length, changed: expectedChanged, candidateCurrent: report.sourceCandidate.all204EntriesCurrent, vocabularyDeltaKeys: Object.fromEntries(Object.entries(vocabularyDelta).map(([file, values]) => [file, Object.keys(values)])) }, null, 2));
