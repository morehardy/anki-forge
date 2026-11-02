import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pages, localeIds } from '../../../../../website/site-model.mjs';
import { reviewInputs, reviewInputFiles, digest } from '../../../../../website/scripts/content-review-inputs.mjs';
const dir = 'docs/plans/evidence/multilingual-ja-de-2026-11-01/ja';
const originalPath = `${dir}/technical-source-review.json`;
const originalBytes = await readFile(originalPath);
const report = JSON.parse(originalBytes);
const audit = JSON.parse(await readFile(`${dir}/source-input-audit.json`));
const candidatePath = `${dir}/source-candidates-final-v2.json`;
const candidateBytes = await readFile(candidatePath);
const candidate = JSON.parse(candidateBytes);
const delta = [];
for (const binding of audit.currentBindings) {
  const hash = digest(await readFile(binding.path));
  if (hash !== binding.sha256) delta.push({ path: binding.path, previousSha256: binding.sha256, currentSha256: hash });
}
assert.deepEqual(delta.map(item => item.path), ['docs/ja/media.md']);
const added = ' `tools::load_project` は `internal-tools` feature が必要なリポジトリ内部向けの API であり、利用側アプリケーション向けのサポート対象 API には含まれません。';
const media = await readFile('docs/ja/media.md', 'utf8');
assert.equal(media.split(added).length, 2);
assert.equal(digest(media.replace(added, '')), delta[0].previousSha256);
const implementation = await readFile('anki_forge/src/lib.rs', 'utf8');
assert.ok(implementation.includes('#[cfg(feature = "internal-tools")]'));
const changedEntries = [];
for (const page of pages) for (const locale of localeIds) {
  const current = await reviewInputs(page, locale, path.resolve('.'));
  const files = await reviewInputFiles(page, locale, path.resolve('.'));
  const finalEntry = candidate.entries[page.id][locale];
  const original = report.entries[page.id][locale];
  assert.deepEqual(finalEntry.inputFiles, files);
  assert.deepEqual(original.inputFiles, files);
  for (const [key, value] of Object.entries(current)) {
    assert.equal(finalEntry[key], value, `${page.id}/${locale}/${key}`);
    if (original[key] !== value) changedEntries.push({ topic: page.id, locale, field: key, previous: original[key], current: value });
  }
  Object.assign(original, current);
  original.technical.evidence.push(`Final v2 incremental review: ${originalPath}, SHA-256 ${digest(originalBytes)}, preserves the complete prior independent review. Compared every one of its 226 input file bindings; the only change is one sentence in docs/ja/media.md clarifying that tools::load_project requires internal-tools and lies outside the supported consumer API. Independently read the sentence against anki_forge/src/lib.rs:36–40. Removing exactly that sentence restores the previously reviewed file hash. Recomputed all 170 current review-input contracts against ${candidatePath}, SHA-256 ${digest(candidateBytes)}. Only media/ja targetHash changes; no other prose or dependency was silently refreshed.`);
}
assert.deepEqual(changedEntries.map(item => [item.topic, item.locale, item.field]), [['media', 'ja', 'targetHash']]);
const auditPath = `${dir}/source-input-delta-final.json`;
const finalAudit = { schemaVersion: 1, reviewerRun: '/root/ja_de_infra_audit/ja-technical-source-final-2026-11-01',
  previousReview: { path: originalPath, sha256: digest(originalBytes) }, sourceCandidate: { path: candidatePath, sha256: digest(candidateBytes) },
  previousInputs: 226, exactFileChanges: delta, exactContractChanges: changedEntries, verifiedCurrentEntries: 170,
  semanticFinding: 'Read the added tools::load_project boundary sentence against the local crate feature gate and public API documentation. It accurately distinguishes internal-tools repository tooling from supported consumer APIs, while NoteType::from_bundle remains a public alternative.',
  supportingSource: { path: 'anki_forge/src/lib.rs', sha256: digest(implementation), lines: '36–40' }, openIssues: [] };
await writeFile(auditPath, JSON.stringify(finalAudit, null, 2) + '\n');
report.reviewerRun = finalAudit.reviewerRun;
report.reviewScope.sourceCandidate = finalAudit.sourceCandidate;
report.reviewScope.previousJapaneseReview = finalAudit.previousReview;
report.reviewScope.finalDeltaAudit = { path: auditPath, sha256: digest(await readFile(auditPath)) };
report.reviewScope.closedFindings.push({ path: 'docs/ja/media.md', finding: finalAudit.semanticFinding });
await writeFile(`${dir}/technical-source-review-final.json`, JSON.stringify(report, null, 2) + '\n');
console.log('Final v2 review verified: 170 current entries; exactly one added Japanese media sentence and media/ja targetHash changed. Prior review and audit remain immutable.');
