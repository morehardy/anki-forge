import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import config from '../indexnow.config.json' with { type: 'json' };

const { origin, key } = config;
const keyRoute = `/${key}.txt`;
const manifestUrl = `${origin}/generated/site-manifest.json`;
const requestOptions = () => ({ redirect: 'error', signal: AbortSignal.timeout(20_000), headers: { 'Cache-Control': 'no-cache' } });

function pageHashes(manifest) {
  assert.ok(manifest && typeof manifest === 'object', 'Missing manifest object');
  assert.equal(manifest.schemaVersion, 1, 'Unsupported manifest');
  assert.equal(manifest.configuration.site, origin, 'IndexNow only accepts the production origin');
  assert.equal(manifest.configuration.base, '/', 'IndexNow only accepts the production root');
  assert.ok(Array.isArray(manifest.pages) && manifest.pages.length, 'Missing manifest pages');
  const result = new Map();
  for (const page of manifest.pages) {
    if (page.id === '404') continue;
    assert.match(page.route, /^\/(?:[A-Za-z0-9_-]+\/)*$/, 'Invalid public page route');
    assert.match(page.sha256, /^[a-f0-9]{64}$/, 'Missing page digest');
    assert.ok(!result.has(page.route), 'Duplicate page route');
    result.set(page.route, page.sha256);
  }
  return result;
}

export function changedUrls(current, previous) {
  const after = pageHashes(current);
  const before = previous ? pageHashes(previous) : new Map();
  // The first deployment with this key initializes notifications for existing pages.
  const initialize = !previous?.resources?.some(resource => resource.route === keyRoute);
  return [...new Set([...after.keys(), ...before.keys()])]
    .filter(route => (initialize && after.has(route)) || before.get(route) !== after.get(route))
    .sort().map(route => origin + route);
}

export async function capturePrevious(destination, fetcher = fetch) {
  const response = await fetcher(manifestUrl, requestOptions());
  // A pre-manifest deployment has no baseline. Network and other HTTP errors
  // must not silently turn an incremental notification into a full submission.
  assert.ok(response.status === 200 || response.status === 404, `Previous manifest: HTTP ${response.status}`);
  const manifest = response.status === 404 ? null : await response.json();
  if (response.status === 200) pageHashes(manifest);
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(destination, JSON.stringify(manifest) + '\n');
}

export async function submitChanges(current, previous, fetcher = fetch) {
  const urls = changedUrls(current, previous);
  assert.match(key, /^[a-zA-Z0-9-]{8,128}$/);
  const live = await fetcher(manifestUrl, requestOptions());
  assert.equal(live.status, 200, 'Published manifest must be reachable');
  assert.deepEqual(await live.json(), current, 'Production has not published the expected artifact');
  const proof = await fetcher(origin + keyRoute, requestOptions());
  assert.equal(proof.status, 200, 'Publish the IndexNow key file before submitting');
  assert.equal((await proof.text()).trim(), key, 'Published IndexNow key does not match');
  const batches = [];
  for (let offset = 0; offset < urls.length; offset += 10_000) {
    const urlList = urls.slice(offset, offset + 10_000);
    const response = await fetcher('https://api.indexnow.org/indexnow', {
      ...requestOptions(), method: 'POST',
      headers: { 'Content-Type': 'application/json; charset=utf-8' },
      body: JSON.stringify({ host: new URL(origin).host, key, keyLocation: origin + keyRoute, urlList }),
    });
    assert.ok([200, 202].includes(response.status), `IndexNow: HTTP ${response.status}; retry this saved manifest pair after resolving the error`);
    batches.push({ count: urlList.length, status: response.status });
  }
  return { urls, batches };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, currentPath, previousPath] = process.argv.slice(2);
  if (command === 'capture' && currentPath && !previousPath) {
    await capturePrevious(currentPath);
    console.log('Saved the pre-deployment IndexNow baseline.');
  } else if (['plan', 'submit'].includes(command) && currentPath && previousPath) {
    const [current, previous] = await Promise.all([currentPath, previousPath].map(async file => JSON.parse(await readFile(file, 'utf8'))));
    if (command === 'plan') console.log(JSON.stringify({ urls: changedUrls(current, previous) }, null, 2));
    else {
      const result = await submitChanges(current, previous);
      console.log(JSON.stringify(result, null, 2));
      console.log('200 = received; 202 = received, key validation pending. Neither guarantees indexing.');
    }
  } else throw new Error('Usage: indexnow.mjs capture PREVIOUS | plan CURRENT PREVIOUS | submit CURRENT PREVIOUS');
}
