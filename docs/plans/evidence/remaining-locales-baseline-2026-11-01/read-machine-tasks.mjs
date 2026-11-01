// Actual artifact HTTP observations. This script does not issue AI review verdicts.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
const [artifact, output, repository] = process.argv.slice(2).map(value => path.resolve(value));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const { serveBuiltSite } = await import(pathToFileURL(path.join(repository, 'website/scripts/serve-built-site.mjs')));
const { inspectReleaseArtifact } = await import(pathToFileURL(path.join(repository, 'website/scripts/release-evidence.mjs')));
const { buildInputIdentity } = await import(pathToFileURL(path.join(repository, 'website/scripts/site-manifest.mjs')));
const manifestBytes = await readFile(path.join(artifact, 'generated/site-manifest.json'));
const manifest = JSON.parse(manifestBytes);
const records = new Map([...manifest.pages, ...manifest.resources].map(row => [row.route, row]));
const integrity = await inspectReleaseArtifact(artifact, await buildInputIdentity(repository));
await mkdir(output, { recursive: true });
await writeFile(path.join(output, 'artifact-integrity.json'), JSON.stringify(integrity, null, 2) + '\n');
const server = await serveBuiltSite({ directory: artifact });
const observations = { schemaVersion: 1, scope: 'actual-http-observations-only', reviewerRun: '/root/baseline_acceptance', manifestSha256: sha(manifestBytes), configuration: manifest.configuration, releaseLocales: manifest.releaseLocales, inputSha256: manifest.inputSha256, readOrigin: server.origin, entries: {} };
const definitions = { 'python-install': ['python-quickstart', 'python-api', 'compatibility'], 'node-api': ['node-api', 'node-quickstart'], 'json-update': ['json-to-anki', 'updates', 'media'], 'compatibility-constraints': ['compatibility', 'build-guarantees'] };
let counter = 0;
async function get(route) {
  const response = await fetch(server.origin + route, { redirect: 'manual', headers: { accept: 'text/markdown, text/plain' } });
  const bytes = Buffer.from(await response.arrayBuffer());
  assert.equal(response.status, 200);
  assert.equal(response.url, server.origin + route);
  assert.match(response.headers.get('content-type'), /^text\/(?:plain|markdown); charset=utf-8$/);
  assert.equal(response.headers.get('content-disposition'), null);
  assert.equal(sha(bytes), records.get(route)?.sha256);
  assert.equal(bytes.length, records.get(route)?.bytes);
  const body = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  const filename = `${String(++counter).padStart(3, '0')}-${route.slice(1).replaceAll('/', '_')}.body`;
  await writeFile(path.join(output, filename), bytes);
  return { route, url: response.url, status: response.status, headers: Object.fromEntries(response.headers), sha256: sha(bytes), bytes: bytes.length, path: path.relative(repository, path.join(output, filename)), body };
}
try {
  for (const locale of manifest.releaseLocales) {
    const home = manifest.pages.find(page => page.id === 'home' && page.locale === locale);
    observations.entries[locale] = [];
    for (const [id, topics] of Object.entries(definitions)) {
      const entry = await get(home.route + 'llms.txt');
      const links = [...entry.body.matchAll(/\[([^\]]+)\]\((https?:\/\/[^)]+)\)/g)].map(match => ({ title: match[1], route: new URL(match[2]).pathname }));
      const followed = [];
      for (const topic of topics) {
        const page = manifest.pages.find(page => page.id === topic && page.locale === locale);
        const route = page.route + 'index.md';
        const link = links.find(link => link.route === route);
        assert.ok(link, `${locale}/${id}: missing real llms link for ${topic}`);
        followed.push({ topic, title: link.title, ...await get(route) });
      }
      observations.entries[locale].push({ id, index: entry, followed, readUrls: [entry.url, ...followed.map(row => row.url)] });
      console.log(`${locale}/${id}: read llms plus ${followed.length} real linked Markdown documents`);
    }
  }
  await writeFile(path.join(output, 'http-observations.json'), JSON.stringify(observations, null, 2) + '\n');
} finally { await server.close(); }
