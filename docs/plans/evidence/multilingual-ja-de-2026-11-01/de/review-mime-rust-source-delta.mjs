import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { pages, localeIds } from '../../../../../website/site-model.mjs';
import { reviewInputs, digest } from '../../../../../website/scripts/content-review-inputs.mjs';

const root = fileURLToPath(new URL('../../../../../', import.meta.url));
const area = 'docs/plans/evidence/multilingual-ja-de-2026-11-01/de';
const baseline = '6f5f860117f17859d8cbe1fbe0073590e33564b4';
const bytes = filename => readFile(path.join(root, filename));
const load = async filename => JSON.parse(await bytes(filename));
const proof = async filename => ({ path: filename, sha256: digest(await bytes(filename)) });
const fullPath = `${area}/technical-source-review-final.json`;
const pairPath = `${area}/technical-source-review-mime-fixed.json`;
const priorCandidatePath = `${area}/source-candidates-mime-fixed.json`;
const candidatePath = `${area}/source-candidates-mime-complete.json`;
const full = await load(fullPath), priorCandidate = await load(priorCandidatePath), candidate = await load(candidatePath);
const originalAudit = await load(`${area}/source-input-audit-final.json`);
const bindings = [];
let paragraphs;
for (const item of originalAudit.currentInputBindings) {
  const before = execFileSync('git', ['show', `${baseline}:${item.path}`], { cwd: root });
  const current = await bytes(item.path);
  const unchanged = before.equals(current);
  if (!unchanged) {
    assert.equal(item.path, 'docs/de/rust-api.md');
    let expected = before.toString();
    for (const mime of ['image/*', 'audio/*', 'video/*']) {
      assert.equal(expected.split(mime).length - 1, 1);
      expected = expected.replace(mime, `\`${mime}\``);
    }
    assert.equal(current.toString(), expected);
    paragraphs = { before: before.toString().split('\n').find(line => line.includes('image/*')), after: current.toString().split('\n').find(line => line.includes('image/*')) };
  }
  bindings.push({ path: item.path, beforeSha256: digest(before), afterSha256: digest(current), unchanged });
}
assert.deepEqual(bindings.filter(item => !item.unchanged).map(item => item.path), ['docs/de/rust-api.md']);
const checks = [];
for (const page of pages) for (const locale of localeIds) {
  const now = await reviewInputs(page, locale, root);
  const old = priorCandidate.entries[page.id][locale];
  for (const [key, value] of Object.entries(now)) assert.equal(candidate.entries[page.id][locale][key], value);
  const changed = Object.keys(now).filter(key => now[key] !== old[key]);
  assert.deepEqual(changed, page.id === 'rust-api' && locale === 'de' ? ['targetHash'] : []);
  checks.push({ entry: `${page.id}/${locale}`, changedHashes: changed });
}
assert.equal(checks.length, 204);
const inspectedPages = [];
for (const filename of ['node-api', 'python-api', 'rust-api', 'media', 'rust-guide']) {
  const filepath = `docs/de/${filename}.md`;
  const text = (await bytes(filepath)).toString();
  for (const mime of ['image/*', 'audio/*', 'video/*']) {
    for (const match of text.matchAll(new RegExp(mime.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g'))) {
      assert.equal(text[match.index - 1], '`');
      assert.equal(text[match.index + mime.length], '`');
    }
  }
  inspectedPages.push(await proof(filepath));
}
const auditPath = `${area}/source-mime-rust-delta-audit.json`;
const audit = { schemaVersion: 1, reviewerRun: '/root/de_technical_review/de-mime-rust-source-delta-2026-11-01', baseline, priorCompleteReview: await proof(fullPath), priorPairCorrection: await proof(pairPath), priorCandidate: await proof(priorCandidatePath), sourceCandidate: await proof(candidatePath), paragraphsIndependentlyRead: paragraphs, sourceInputBindings: bindings, candidateChecks: checks, allFiveGermanMimePagesInspected: inspectedPages, scope: 'One Rust API formatting delta after the two-API correction. All five German MIME-bearing pages independently inspected; actual final artifacts still require confirmation.' };
await writeFile(path.join(root, auditPath), JSON.stringify(audit, null, 2) + '\n');
const previous = full.entries['rust-api'].de.technical;
const entry = { ...candidate.entries['rust-api'].de, technical: { status: 'reviewed', evidence: [
  ...previous.evidence,
  `Full 204-entry independent review remains at ${fullPath}, sha256 ${(await proof(fullPath)).sha256}; the previous two-API correction remains at ${pairPath}, sha256 ${(await proof(pairPath)).sha256}. This report adds only the Rust API entry.`,
  'Independently read the complete Rust addition-context paragraph and the exact three inline-code changes. image/* is the typed-image MIME category; audio/* or video/* is accepted for sound references. Category mismatch remains NOTE.MEDIA_USAGE_INVALID; constructors, explicit media/raw HTML, retained import MIME and separate build extension validation keep their original semantics. Code delimiters prevent German punctuation from swallowing wildcard characters.',
  'Independently scanned all German MIME occurrences: exactly media, rust-guide and the three API pages contain these MIME categories. All five now mark every image/*, audio/* and video/* literal as inline code. The media and Rust authoring pages were already protected. No SDK code, command, schema or publication behavior changed.',
  'Compared all 259 source-input files against immutable two-API-fix commit 6f5f860117f17859d8cbe1fbe0073590e33564b4 and recomputed all 204 candidate contracts. Only docs/de/rust-api.md changes, and only rust-api/de targetHash differs; the other 203 entry contracts and all source/technicalDependencyHash values remain identical. Source fix is reviewed; the broader DE-ARTIFACT-001 is closed only after fresh final artifact checks cover all three APIs.',
], openIssues: [] }, linguistic: { status: 'pending', score: {}, evidence: [], openIssues: [] } };
const target = `${area}/technical-source-review-mime-rust.json`;
const report = { schemaVersion: 1, rubricVersion: candidate.rubricVersion, glossaryVersion: candidate.glossaryVersion, reviewerType: 'ai', reviewerRun: audit.reviewerRun, entries: { 'rust-api': { de: entry } }, reviewScope: { axis: 'technical', incrementalEntries: 1, priorCompleteEntries: 204, priorCompleteReview: await proof(fullPath), priorPairCorrection: await proof(pairPath), sourceCandidate: await proof(candidatePath), deltaAudit: await proof(auditPath), independentOfContentAuthor: true, openIssues: [], limits: 'Source-only Rust MIME correction; all three API artifact literals require fresh actual output confirmation.' } };
await writeFile(path.join(root, target), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...(await proof(target)), entries: 1, unchangedEntryContracts: 203 }));
