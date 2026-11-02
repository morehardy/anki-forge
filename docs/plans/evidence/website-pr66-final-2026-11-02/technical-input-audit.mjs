import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { pages, localeIds } from '../../../../website/site-model.mjs';
import { reviewInputs, reviewInputFiles, digest } from '../../../../website/scripts/content-review-inputs.mjs';
import { headings, sectionMapping, legacyAliases } from '../../../../website/scripts/content-sections.mjs';

const root = process.cwd();
const beforeRoot = '/private/tmp/anki-forge-multilingual-fixes-20261101';
const directory = 'docs/plans/evidence/website-pr66-final-2026-11-02';
const expectedHead = '09d431830568345a9e6ae2bd1cda2154af5a08e1';
const git = args => execFileSync('git', args, { cwd: root, encoding: 'utf8', maxBuffer: 64 << 20 });
assert.equal(git(['rev-parse', 'HEAD']).trim(), expectedHead);
const candidatePath = `${directory}/source-candidate.json`;
const candidateBytes = await readFile(candidatePath);
const candidate = JSON.parse(candidateBytes);
const priorIndex = JSON.parse(git(['show', `${expectedHead}:website/content-reviews.json`]));
const currentLegacy = JSON.parse(await readFile('website/legacy-sections.json'));
const oldLegacy = JSON.parse(await readFile(path.join(beforeRoot, 'website/legacy-sections.json')));
const entries = {}, snapshots = {}, evidenceReports = {};
const pairs = [];
for (const page of pages) for (const locale of localeIds) if (candidate.entries[page.id]?.[locale]) pairs.push([page, locale]);
assert.equal(pairs.length, 204);
assert.equal(Object.values(candidate.entries).reduce((total, locales) => total + Object.keys(locales).length, 0), 204);
for (const [page, locale] of pairs) {
  const label = `${page.id}/${locale}`;
  const current = await reviewInputs(page, locale, root);
  const pending = candidate.entries[page.id][locale];
  for (const key of Object.keys(current)) assert.equal(current[key], pending[key], `${label}: candidate ${key}`);
  const currentFiles = await reviewInputFiles(page, locale, root);
  assert.deepEqual(currentFiles, pending.inputFiles);
  const prior = priorIndex.entries[page.id][locale];
  const reference = prior.technical.evidence;
  if (!evidenceReports[reference.path]) {
    const bytes = await readFile(reference.path);
    assert.equal(digest(bytes), reference.sha256);
    evidenceReports[reference.path] = { sha256: digest(bytes), report: JSON.parse(bytes) };
  }
  const reviewed = evidenceReports[reference.path].report.entries[page.id][locale];
  assert.equal(reviewed.technical.status, 'reviewed');
  const previous = await reviewInputs(page, locale, beforeRoot);
  for (const key of Object.keys(previous)) {
    assert.equal(reviewed[key], prior[key], `${label}: old report/index ${key}`);
    assert.equal(previous[key], reviewed[key], `${label}: old snapshot ${key}`);
  }
  assert.deepEqual(await reviewInputFiles(page, locale, beforeRoot), currentFiles);
  for (const filename of currentFiles) if (!snapshots[filename]) {
    const before = await readFile(path.join(beforeRoot, filename), 'utf8');
    const after = await readFile(path.join(root, filename), 'utf8');
    assert.equal(after, git(['show', `${expectedHead}:${filename}`]), `${filename}: source changed after HEAD`);
    snapshots[filename] = { previousSha256: digest(before), currentSha256: digest(after), previousUtf8: before, currentUtf8: after, changed: before !== after };
  }
  const changedFiles = currentFiles.filter(filename => snapshots[filename].changed);
  const sourceFiles = [...new Set([page.sources.en, page.sources[locale]])];
  const sourceChanges = sourceFiles.filter(filename => snapshots[filename]?.changed);
  let sections = null;
  if (page.type === 'docs' || page.type === 'article') {
    const source = page.sources[locale];
    const beforeText = snapshots[source].previousUtf8;
    const afterText = snapshots[source].currentUtf8;
    const oldMap = sectionMapping(page.id, locale, beforeText, legacyAliases(oldLegacy, page.id, locale));
    const newMap = sectionMapping(page.id, locale, afterText, legacyAliases(currentLegacy, page.id, locale));
    const oldHeadings = headings(beforeText), newHeadings = headings(afterText);
    const renamed = oldHeadings.filter(heading => !newHeadings.some(next => next.id === heading.id));
    for (const heading of renamed) assert.equal(newMap[heading.id], oldMap[heading.id], `${label}: old heading ${heading.id} lost semantic target`);
    sections = { oldMapping: oldMap, currentMapping: newMap, renamedHeadings: renamed.map(heading => ({ title: heading.title, slug: heading.id, semanticTarget: newMap[heading.id] })) };
  }
  entries[page.id] ??= {};
  entries[page.id][locale] = {
    current, previous, inheritedTechnicalEvidence: reference,
    inheritedOpenIssues: reviewed.technical.openIssues,
    inputFiles: currentFiles, changedFiles, sourceFiles, sourceChanges, sections
  };
}
const changedFiles = Object.keys(snapshots).filter(filename => snapshots[filename].changed).sort();
const sourceFiles = new Set(pairs.flatMap(([page, locale]) => [page.sources.en, page.sources[locale]]));
const fences = text => [...text.matchAll(/^```[^\n]*\n([\s\S]*?)^```\s*$/gm)].map(match => match[0]);
const destinations = text => [...text.matchAll(/\]\(([^\s)]+)(?:\s+[^)]*)?\)/g)].map(match => match[1]);
const sourceInvariants = [];
for (const filename of changedFiles.filter(filename => sourceFiles.has(filename))) {
  const source = snapshots[filename];
  assert.deepEqual(fences(source.currentUtf8), fences(source.previousUtf8), `${filename}: fenced code changed`);
  assert.deepEqual(destinations(source.currentUtf8), destinations(source.previousUtf8), `${filename}: Markdown destination changed`);
  sourceInvariants.push({ path: filename, fencedCodeUnchanged: true, markdownDestinationsUnchanged: true });
}
const snapshotPath = `${directory}/technical-input-snapshots.json`;
await writeFile(snapshotPath, JSON.stringify({ schemaVersion: 1, reviewerRun: '/root/standards_review/pr66-source-2026-11-02', currentCommit: expectedHead, previousSnapshot: 'The actual isolated candidate previously independently reviewed; all 204 old hash triples recomputed and matched their immutable reports.', files: snapshots }, null, 2) + '\n');
const audit = {
  schemaVersion: 1, reviewerRun: '/root/standards_review/pr66-source-2026-11-02',
  currentCommit: expectedHead, reviewBaseline: '6d4625639fc1477222b41880d29724c162baa3be',
  sourceCandidate: { path: candidatePath, sha256: digest(candidateBytes) },
  snapshots: { path: snapshotPath, sha256: digest(await readFile(snapshotPath)) },
  method: 'Recomputed all 204 current candidate triples from disk. Recomputed all 204 previous triples from the actual previously reviewed isolated source snapshot and matched each immutable report and committed index. Saved exact before/current UTF-8 bytes for every closure input. Current closure files match HEAD. This audit establishes lineage and input identity; independent prose/API review is documented separately.',
  inputFileCount: Object.keys(snapshots).length, changedInputFiles: changedFiles.map(filename => ({ path: filename, previousSha256: snapshots[filename].previousSha256, currentSha256: snapshots[filename].currentSha256 })),
  sourceInvariants,
  priorReports: Object.entries(evidenceReports).map(([filename, value]) => ({ path: filename, sha256: value.sha256 })),
  entries
};
await writeFile(`${directory}/technical-source-input-audit.json`, JSON.stringify(audit, null, 2) + '\n');
console.log(JSON.stringify({ currentTriples: pairs.length, verifiedPreviousTriples: pairs.length, inputFileCount: audit.inputFileCount, changedFiles, sourceInvariants: sourceInvariants.length, renamedHeadings: Object.values(entries).flatMap(locales => Object.values(locales)).flatMap(entry => entry.sections?.renamedHeadings ?? []) }, null, 2));
