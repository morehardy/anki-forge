import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

// Capture observations only. A separate independent semantic reading is required
// before a machine-read report can mark these tasks passed.
const environment = process.argv[2];
assert.ok(['prefix', 'root'].includes(environment));
const root = process.cwd();
const evidence = 'docs/plans/evidence/website-pr66-portable-search-2026-11-02';
const manifestPath = `${evidence}/${environment}-site-manifest.json`;
const directory = path.join(root, `target/website-pr66-portable-search/${environment}-dist`);
const manifestBytes = await readFile(manifestPath);
const manifest = JSON.parse(manifestBytes);
const sha = value => createHash('sha256').update(value).digest('hex');
assert.equal(sha(await readFile(path.join(directory, 'generated/site-manifest.json'))), sha(manifestBytes));
const prefix = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const filename = route => {
  assert.ok(route.startsWith(prefix + '/'));
  const result = path.resolve(directory, '.' + route.slice(prefix.length));
  assert.ok(result.startsWith(directory + path.sep));
  return result;
};
const oldReportPath = `docs/plans/evidence/website-pr66-rebuild-2026-11-02/machine-read-${environment}.json`;
const oldReportBytes = await readFile(oldReportPath);
const oldReport = JSON.parse(oldReportBytes);
assert.equal(oldReport.status, 'passed');
const oldManifestBytes = await readFile(oldReport.originManifest.path);
assert.equal(sha(oldManifestBytes), oldReport.originManifest.sha256);
const oldManifest = JSON.parse(oldManifestBytes);
assert.deepEqual(oldManifest.configuration, manifest.configuration);
const oldResources = new Map(oldManifest.resources.map(resource => [resource.route, resource]));
const machine = manifest.resources.filter(resource => /\/(?:index\.md|llms\.txt)$/.test(resource.route));
assert.equal(machine.length, 192);
const inventory = [];
for (const resource of machine) {
  const bytes = await readFile(filename(resource.route));
  assert.equal(sha(bytes), resource.sha256, resource.route);
  assert.equal(bytes.length, resource.bytes, resource.route);
  new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  inventory.push({ ...resource, previousSha256: oldResources.get(resource.route)?.sha256 ?? null, changed: oldResources.get(resource.route)?.sha256 !== resource.sha256 });
}
const owned = new Map(machine.map(resource => [resource.route, resource]));
const server = createServer(async (request, response) => {
  try {
    const route = new URL(request.url, 'http://localhost').pathname;
    if (!owned.has(route)) { response.writeHead(404); response.end('Not found'); return; }
    const bytes = await readFile(filename(route));
    response.writeHead(200, { 'Content-Type': route.endsWith('.txt') ? 'text/plain; charset=utf-8' : 'text/markdown; charset=utf-8' });
    response.end(bytes);
  } catch (error) { response.writeHead(500); response.end(String(error)); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const readOrigin = `http://127.0.0.1:${server.address().port}`;
const captureDirectory = `${evidence}/technical-machine-${environment}`;
await mkdir(`${captureDirectory}/bodies`, { recursive: true });
const responses = new Map();
async function capture(route) {
  if (responses.has(route)) return responses.get(route);
  assert.ok(owned.has(route));
  const url = readOrigin + route;
  const response = await fetch(url, { redirect: 'error' });
  assert.equal(response.status, 200, url);
  const bytes = Buffer.from(await response.arrayBuffer());
  assert.equal(sha(bytes), owned.get(route).sha256, url);
  const bodyPath = `${captureDirectory}/bodies/${route.slice(1).replaceAll('/', '__')}`;
  await writeFile(bodyPath, bytes);
  const record = { url, route, status: response.status, contentType: response.headers.get('content-type'), sha256: sha(bytes), bytes: bytes.length, bodyPath };
  responses.set(route, record);
  return record;
}
const tasks = {
  'python-install': ['python-quickstart', 'python-api', 'compatibility'],
  'node-api': ['node-api', 'node-quickstart'],
  'json-update': ['json-to-anki', 'updates', 'media'],
  'compatibility-constraints': ['compatibility', 'build-guarantees'],
};
const readings = {};
try {
  for (const locale of manifest.releaseLocales) {
    const home = manifest.pages.find(page => page.id === 'home' && page.locale === locale);
    assert.ok(home);
    const llms = await capture(`${home.route}llms.txt`);
    const index = await readFile(llms.bodyPath, 'utf8');
    const links = [...index.matchAll(/\]\((https?:\/\/[^\s)]+)\)/g)].map(match => new URL(match[1]));
    readings[locale] = {};
    for (const [task, topics] of Object.entries(tasks)) {
      const actual = [llms];
      for (const topic of topics) {
        const route = `${home.route}docs/${topic}/index.md`;
        assert.ok(links.some(link => link.origin === manifest.configuration.site && link.pathname === route), `${locale}/${task}: missing llms discovery link ${route}`);
        actual.push(await capture(route));
      }
      readings[locale][task] = actual.map(record => ({ url: record.url, route: record.route, bodyPath: record.bodyPath }));
    }
  }
  for (const resource of inventory.filter(resource => resource.changed)) await capture(resource.route);
} finally { await new Promise(resolve => server.close(resolve)); }
const observation = {
  schemaVersion: 1, status: 'captured-awaiting-independent-semantic-review',
  captureRun: `/root/standards_review/pr66-portable-search-actual-http-${environment}-2026-11-02`,
  environment, readOrigin,
  manifest: { path: manifestPath, sha256: sha(manifestBytes), commit: manifest.commit, inputSha256: manifest.inputSha256 },
  previousReport: { path: oldReportPath, sha256: sha(oldReportBytes) },
  previousManifest: oldReport.originManifest,
  configuration: manifest.configuration, releaseLocales: manifest.releaseLocales,
  inventory, responses: [...responses.values()], readings,
  method: 'This independent reviewer actually fetched the listed new archive resources over localhost HTTP and independently verified all 192 archived machine-resource UTF-8 bytes and manifest digests. Actually fetched each listed same-locale llms index and followed its explicit Markdown links through a loopback HTTP server serving the frozen archive. Saved actual response bodies. The local server MIME values are preview observations, not evidence of hosting MIME or deployment. No semantic pass is generated by this capture script.',
};
await writeFile(`${captureDirectory}/observations.json`, JSON.stringify(observation, null, 2) + '\n');
console.log(JSON.stringify({ environment, actualHttpResponses: responses.size, taskCount: Object.keys(readings).length * 4, machineResources: inventory.length, changedResources: inventory.filter(resource => resource.changed).map(resource => resource.route), readOrigin }));
