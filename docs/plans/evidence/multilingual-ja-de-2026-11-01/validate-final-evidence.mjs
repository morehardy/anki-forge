// Revalidate sealed reviews after evidence-only commits; never rewrite review verdicts.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { buildInputIdentity, sha256 } from '../../../../website/scripts/site-manifest.mjs';
import { verifyPostBuildReviews } from '../../../../website/scripts/post-build-reviews.mjs';

const base = new URL('./de/', import.meta.url);
const current = await buildInputIdentity();
const reports = {};
for (const environment of ['prefix', 'root']) {
  const bytes = await readFile(new URL(`final-${environment}-origin-manifest.json`, base));
  const manifest = JSON.parse(bytes);
  assert.equal(current.inputSha256, manifest.inputSha256, 'Evidence-only commits must preserve every build input');
  assert.deepEqual(current.releaseLocales, manifest.releaseLocales);
  reports[environment] = await verifyPostBuildReviews(manifest, sha256(bytes));
}
const validation = {
  scope: 'Current build inputs equal the sealed source inputs; independent reports still validate against the original two artifact manifests. CI at a new HEAD must rebuild engineering receipts.',
  current,
  reports,
};
if (process.argv[2]) await writeFile(process.argv[2], JSON.stringify(validation, null, 2) + '\n');
console.log(JSON.stringify({ commit: current.commit, inputSha256: current.inputSha256, reports: Object.fromEntries(Object.entries(reports).map(([environment, report]) => [environment, report.status])) }));
