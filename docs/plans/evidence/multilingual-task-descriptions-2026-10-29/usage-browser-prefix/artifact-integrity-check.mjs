// Independent reviewer check: every file of a frozen artifact must hash-match its archived origin manifest.
// Usage: node artifact-integrity-check.mjs <artifactDirectory> <archivedManifest> <output.json>
import assert from 'node:assert/strict';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
const [directory, archived, output] = process.argv.slice(2);
assert.ok(directory && archived && output, 'usage: <artifactDirectory> <archivedManifest> <output.json>');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const archivedBytes = await readFile(archived);
const embeddedBytes = await readFile(path.join(directory, 'generated/site-manifest.json'));
const manifest = JSON.parse(archivedBytes);
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const fileFor = route => {
  assert.ok(route.startsWith(base + '/'), `route outside base: ${route}`);
  const relative = route.slice(base.length + 1);
  return relative === '' || relative.endsWith('/') ? relative + 'index.html' : relative;
};
async function walk(dir, prefix = '') {
  const out = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) out.push(...await walk(path.join(dir, entry.name), rel));
    else out.push(rel);
  }
  return out;
}
const expected = new Map();
const mismatches = [];
for (const entry of [...manifest.pages, ...manifest.resources]) {
  const file = fileFor(entry.route);
  assert.ok(!expected.has(file), `duplicate file ${file}`);
  expected.set(file, entry);
  let bytes;
  try { bytes = await readFile(path.join(directory, file)); } catch (error) { mismatches.push({ route: entry.route, file, problem: error.code }); continue; }
  if (hash(bytes) !== entry.sha256 || bytes.length !== entry.bytes) mismatches.push({ route: entry.route, file, problem: 'sha256/bytes differ', actual: hash(bytes), expected: entry.sha256 });
}
const present = await walk(directory);
const unlisted = present.filter(file => !expected.has(file));
// site-manifest.mjs deliberately omits generated/site-manifest.json (itself) and CNAME (custom-domain marker) from the manifest.
const knownExclusions = {};
for (const file of unlisted.filter(file => ['generated/site-manifest.json', 'CNAME'].includes(file))) {
  const bytes = await readFile(path.join(directory, file));
  knownExclusions[file] = { sha256: hash(bytes), bytes: bytes.length, ...(file === 'CNAME' ? { content: bytes.toString('utf8'), matchesCustomDomain: bytes.toString('utf8').trim() === manifest.configuration.customDomain } : {}) };
}
const cnameOk = !knownExclusions.CNAME || knownExclusions.CNAME.matchesCustomDomain;
const result = {
  schemaVersion: 1,
  checkedAt: new Date().toISOString(),
  artifactDirectory: directory,
  archivedManifest: archived,
  archivedManifestSha256: hash(archivedBytes),
  embeddedManifestSha256: hash(embeddedBytes),
  embeddedEqualsArchived: hash(archivedBytes) === hash(embeddedBytes),
  commit: manifest.commit,
  inputSha256: manifest.inputSha256,
  configuration: manifest.configuration,
  releaseLocales: manifest.releaseLocales,
  manifestEntries: expected.size,
  filesPresent: present.length,
  matched: expected.size - mismatches.length,
  mismatches,
  unlistedFiles: unlisted,
  knownExclusions,
  status: mismatches.length === 0 && hash(archivedBytes) === hash(embeddedBytes) && cnameOk && unlisted.every(file => file in knownExclusions) ? 'passed' : 'failed',
};
await writeFile(output, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ status: result.status, manifestEntries: result.manifestEntries, filesPresent: result.filesPresent, matched: result.matched, mismatches: mismatches.length, unlisted }, null, 0));
