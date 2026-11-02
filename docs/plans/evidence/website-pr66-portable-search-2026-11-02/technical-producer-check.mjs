import assert from 'node:assert/strict';
import { readFile, writeFile, readdir, mkdir, mkdtemp, rm, cp, symlink } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import os from 'node:os';
import { gunzipSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import { generatePagefind } from '../../../../website/scripts/search-documents.mjs';

const root = process.cwd();
const batch = 'docs/plans/evidence/website-pr66-portable-search-2026-11-02';
const archive = path.join(root, 'target/website-pr66-rebuild/prefix-dist');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const temp = await mkdtemp(path.join(os.tmpdir(), 'pr66-technical-producer-'));
const result = { schemaVersion: 1, reviewerRun: '/root/standards_review/pr66-portable-search-2026-11-02', archive, productionSourceSha256: digest(await readFile('website/scripts/search-documents.mjs')) };
async function files(directory) {
  return (await readdir(directory, { recursive: true, withFileTypes: true })).filter(file => file.isFile()).map(file => path.relative(directory, path.join(file.parentPath, file.name))).sort();
}
async function fragments(directory) {
  const rows = [];
  for (const filename of await files(directory)) if (filename.endsWith('.pf_fragment')) {
    const raw = gunzipSync(await readFile(path.join(directory, filename)));
    assert.equal(raw.subarray(0, 12).toString(), 'pagefind_dcd');
    rows.push(JSON.parse(raw.subarray(12)));
  }
  return rows.sort((a, b) => a.url < b.url ? -1 : a.url > b.url ? 1 : 0);
}
try {
  const site = path.join(temp, 'actual-site');
  const htmlFiles = (await files(archive)).filter(filename => filename.endsWith('.html'));
  for (const filename of [...htmlFiles].reverse()) {
    await mkdir(path.dirname(path.join(site, filename)), { recursive: true });
    await cp(path.join(archive, filename), path.join(site, filename));
  }
  await mkdir(path.join(site, 'pagefind'), { recursive: true });
  await writeFile(path.join(site, 'pagefind', 'stale.pf_index'), 'stale');
  await generatePagefind(site);
  const oldRows = await fragments(path.join(archive, 'pagefind'));
  const newRows = await fragments(path.join(site, 'pagefind'));
  assert.deepEqual(newRows, oldRows);
  assert.equal(newRows.length, 204);
  assert.ok(newRows.every(row => !row.url.includes('404')));
  assert.ok(!(await files(path.join(site, 'pagefind'))).includes('stale.pf_index'));
  const entry = JSON.parse(await readFile(path.join(site, 'pagefind/pagefind-entry.json')));
  assert.deepEqual(Object.keys(entry.languages), ['de', 'en', 'es', 'ja', 'pt-br', 'zh-cn']);
  for (const value of Object.values(entry.languages)) assert.equal(value.page_count, 34);
  const fileHashes = {};
  for (const filename of await files(path.join(site, 'pagefind'))) fileHashes[filename] = digest(await readFile(path.join(site, 'pagefind', filename)));
  const crossPlatform = JSON.parse(await readFile(`${batch}/portable-cross-platform-results.json`));
  assert.deepEqual(fileHashes, crossPlatform.files);
  result.actualFullSite = { htmlFiles: htmlFiles.length, fragments: newRows.length, fullFragmentValuesEqualToPriorReviewedArchive: true, notFoundExcluded: true, oldShardRemoved: true, languageCounts: Object.fromEntries(Object.entries(entry.languages).map(([key, value]) => [key, value.page_count])), fileCount: Object.keys(fileHashes).length, completeHashesEqualCoordinatorMacAndNativeLinux: true, fileHashes };

  // Test failure handling without touching the frozen vendor directory. This
  // sandbox copy changes relative import resolution only; function bodies are
  // identical and its import.meta-based runtime directory is isolated.
  const isolatedWebsite = path.join(temp, 'isolated/website');
  await mkdir(path.join(isolatedWebsite, 'scripts'), { recursive: true });
  await symlink(path.join(root, 'website/node_modules'), path.join(isolatedWebsite, 'node_modules'));
  let isolatedSource = await readFile('website/scripts/search-documents.mjs', 'utf8');
  isolatedSource = isolatedSource.replace(/from (['"])(\.[^'"]+)\1/g, (_whole, quote, relative) => `from ${quote}${pathToFileURL(path.resolve(root, 'website/scripts', relative)).href}${quote}`);
  await writeFile(path.join(isolatedWebsite, 'scripts/search-documents.mjs'), isolatedSource);
  const { generatePagefind: isolatedGenerate } = await import(pathToFileURL(path.join(isolatedWebsite, 'scripts/search-documents.mjs')));
  const isolatedVendor = path.join(isolatedWebsite, 'vendor/pagefind/1.5.2');
  const validVendor = path.join(root, 'website/vendor/pagefind/1.5.2');
  const fixture = path.join(temp, 'failure-site');
  await mkdir(fixture, { recursive: true });
  await writeFile(path.join(fixture, 'index.html'), '<html lang="en"><title>Test</title><main data-pagefind-body>Known good searchable content.</main></html>');
  result.rejectedCases = [];
  for (const kind of ['version', 'missing-inventory', 'wrong-length', 'same-length-byte-tamper', 'missing-file']) {
    await rm(isolatedVendor, { recursive: true, force: true });
    await cp(validVendor, isolatedVendor, { recursive: true });
    const manifestPath = path.join(isolatedVendor, 'runtime.json');
    const manifest = JSON.parse(await readFile(manifestPath));
    const runtimePath = path.join(isolatedVendor, 'wasm.en.pagefind');
    if (kind === 'version') manifest.version = '0.0.0';
    if (kind === 'missing-inventory') delete manifest.files['wasm.en.pagefind'];
    if (kind === 'wrong-length') manifest.files['wasm.en.pagefind'].bytes++;
    if (kind === 'same-length-byte-tamper') { const bytes = await readFile(runtimePath); bytes[30] ^= 1; await writeFile(runtimePath, bytes); }
    if (kind === 'missing-file') await rm(runtimePath);
    await writeFile(manifestPath, JSON.stringify(manifest));
    await assert.rejects(isolatedGenerate(fixture));
    result.rejectedCases.push(kind);
  }
  result.failureTestIsolation = { importRewriteOnly: true, isolatedSourceSha256: digest(isolatedSource), productionAndVendorFilesNeverModified: true };
  await writeFile(`${batch}/technical-producer-observations.json`, JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify({ htmlFiles: htmlFiles.length, fragments: newRows.length, completeFiles: Object.keys(fileHashes).length, rejectedCases: result.rejectedCases }, null, 2));
} finally { await rm(temp, { recursive: true, force: true }); }
