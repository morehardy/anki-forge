// Bind the independently read three-MIME-literal formatting correction only.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { pages, localeIds } from '../../../../../website/site-model.mjs';
import { reviewInputs, reviewInputFiles, digest } from '../../../../../website/scripts/content-review-inputs.mjs';

const root = fileURLToPath(new URL('../../../../../', import.meta.url));
const area = 'docs/plans/evidence/multilingual-ja-de-2026-11-01/de';
const originalCommit = '8f2c77e331147961dce5c0d929a9f5097270a5f8';
const bytes = filename => readFile(path.join(root, filename));
const load = async filename => JSON.parse(await bytes(filename));
const proof = async filename => ({ path: filename, sha256: digest(await bytes(filename)) });
const previousPath = `${area}/technical-source-review-final.json`;
const candidatePath = `${area}/source-candidates-mime-fixed.json`;
const previous = await load(previousPath);
const candidate = await load(candidatePath);
const oldAudit = await load(`${area}/source-input-audit-final.json`);
const correctionFiles = ['docs/de/node-api.md', 'docs/de/python-api.md'];
const fileChanges = [];
for (const file of oldAudit.currentInputBindings) {
  const current = await bytes(file.path);
  if (digest(current) === file.sha256) continue;
  assert(correctionFiles.includes(file.path), `${file.path}: unexpected source delta`);
  const before = execFileSync('git', ['show', `${originalCommit}:${file.path}`], { cwd: root }).toString();
  assert.equal(digest(before), file.sha256);
  let expected = before;
  for (const mime of ['image/*', 'audio/*', 'video/*']) {
    assert.equal(before.split(mime).length - 1, 1, `${file.path}: MIME occurrence`);
    expected = expected.replace(mime, `\`${mime}\``);
  }
  assert.equal(current.toString(), expected, `${file.path}: more than literal code formatting changed`);
  fileChanges.push({ path: file.path, beforeSha256: file.sha256, afterSha256: digest(current), beforeParagraph: before.split('\n').find(line => line.includes('image/*')), afterParagraph: current.toString().split('\n').find(line => line.includes('image/*')) });
}
assert.deepEqual(fileChanges.map(item => item.path), correctionFiles);
const checked = [];
const entries = {};
for (const page of pages) for (const locale of localeIds) {
  const current = await reviewInputs(page, locale, root);
  const frozen = candidate.entries[page.id][locale];
  const old = previous.entries[page.id][locale];
  for (const [key, value] of Object.entries(current)) assert.equal(frozen[key], value, `${page.id}/${locale}/${key}: stale candidate`);
  assert.deepEqual(frozen.inputFiles, await reviewInputFiles(page, locale, root));
  const changed = Object.keys(current).filter(key => current[key] !== old[key]);
  const targeted = locale === 'de' && ['node-api', 'python-api'].includes(page.id);
  assert.deepEqual(changed, targeted ? ['targetHash'] : [], `${page.id}/${locale}: unexpected contract change`);
  checked.push({ entry: `${page.id}/${locale}`, changedHashes: changed });
  if (!targeted) continue;
  entries[page.id] = { de: { ...frozen, technical: { status: 'reviewed', evidence: [
    ...old.technical.evidence,
    `Complete independent DE/old-five-language source review is preserved verbatim above and at ${previousPath}, sha256 ${(await proof(previousPath)).sha256}. This is a two-entry incremental review, not a new wholesale approval.`,
    'Independently read the exact paragraph before and after correction. The only source changes put image/*, audio/* and video/* in inline code. Their MIME-category meaning and NOTE.MEDIA_USAGE_INVALID behavior are unchanged; literal formatting prevents German sentence punctuation from turning wildcard stars into emphasis. Correct typed category semantics remains identical to the previously inspected SDK implementation.',
    'All 259 prior source-input files were checked: exactly these two DE files changed, solely by the three inline-code delimiters in each paragraph. All 204 current candidate contracts were recomputed; only node-api/de and python-api/de targetHash change. Their sourceHash and technicalDependencyHash, other 202 entry contracts, code fences, public SDK programs and prior execution audit remain unchanged.',
    'DE-ARTIFACT-001 is corrected in source. Actual new HTTP Markdown/HTML must still be observed and independently checked before closing the artifact finding; old source/capture/answer evidence is preserved at its original path.',
  ], openIssues: [] }, linguistic: { status: 'pending', score: {}, evidence: [], openIssues: [] } } };
}
assert.equal(checked.length, 204);
const auditPath = `${area}/source-mime-delta-audit.json`;
const audit = { schemaVersion: 1, reviewerRun: '/root/de_technical_review/de-mime-source-delta-2026-11-01', originalCommit, previousReview: await proof(previousPath), sourceCandidate: await proof(candidatePath), fileChanges, unchangedInputFiles: oldAudit.currentInputBindings.length - fileChanges.length, checkedEntryContracts: checked, scope: 'Exact two-paragraph MIME formatting delta and 204 contract comparison. Actual new artifact validation remains pending.' };
await writeFile(path.join(root, auditPath), JSON.stringify(audit, null, 2) + '\n');
const report = { schemaVersion: 1, rubricVersion: candidate.rubricVersion, glossaryVersion: candidate.glossaryVersion, reviewerType: 'ai', reviewerRun: audit.reviewerRun, entries, reviewScope: { axis: 'technical', incrementalEntries: 2, priorCompleteEntries: 204, previousReview: await proof(previousPath), sourceCandidate: await proof(candidatePath), deltaAudit: await proof(auditPath), independentOfContentAuthor: true, openIssues: [], limits: 'Source-only two-entry correction; final artifact finding requires new HTTP observation.' } };
const target = `${area}/technical-source-review-mime-fixed.json`;
await writeFile(path.join(root, target), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...(await proof(target)), entries: 2, unchangedEntryContracts: 202, changedFiles: fileChanges.map(item => item.path) }));
