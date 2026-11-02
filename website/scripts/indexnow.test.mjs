import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import config from '../indexnow.config.json' with { type: 'json' };
import { capturePrevious, changedUrls, submitChanges } from './indexnow.mjs';

const page = (route, hash = 'a') => ({ id: route, route, sha256: hash.repeat(64) });
const manifest = pages => ({ schemaVersion: 1, configuration: { site: config.origin, base: '/' }, pages, resources: [{ route: `/${config.key}.txt` }] });

test('notifies additions, edits and deletions while excluding unchanged pages and 404', () => {
  const before = manifest([page('/'), page('/old/'), page('/edited/')]);
  const after = manifest([page('/'), page('/new/'), page('/edited/', 'b'), { id: '404', route: '/404.html' }]);
  assert.deepEqual(changedUrls(after, before), ['/edited/', '/new/', '/old/'].map(route => config.origin + route));
  assert.deepEqual(changedUrls(after, after), []);
});

test('first key deployment initializes existing URLs; foreign origins and unsafe routes fail', () => {
  const current = manifest([page('/')]);
  assert.deepEqual(changedUrls(current, { ...current, resources: [] }), [config.origin + '/']);
  assert.deepEqual(changedUrls(current, null), [config.origin + '/']);
  assert.throws(() => changedUrls({ ...current, configuration: { site: 'https://example.com', base: '/' } }, null));
  assert.throws(() => changedUrls(manifest([page('//example.com/'), page('/docs/')]), null));
});

test('only a missing baseline is initialization; HTTP failures must fail closed', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'indexnow-'));
  const destination = path.join(directory, 'previous.json');
  try {
    await capturePrevious(destination, async () => new Response('', { status: 404 }));
    assert.equal(JSON.parse(await readFile(destination, 'utf8')), null);
    await assert.rejects(capturePrevious(destination, async () => new Response('', { status: 503 })), /503/);
    for (const invalid of [null, false]) {
      await assert.rejects(capturePrevious(destination, async () => Response.json(invalid)));
    }
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('checks live artifact and key before POST, accepts 202 and reports throttling', async () => {
  const current = manifest([page('/')]);
  const calls = [];
  let status = 202;
  const fetcher = async (url, options) => {
    calls.push({ url, options });
    if (url.endsWith('/site-manifest.json')) return Response.json(current);
    if (url.endsWith('.txt')) return new Response(config.key + '\n');
    return new Response('', { status });
  };
  const result = await submitChanges(current, null, fetcher);
  assert.deepEqual(result.batches, [{ count: 1, status: 202 }]);
  const post = calls.find(call => call.options.method === 'POST');
  assert.equal(post.url, 'https://api.indexnow.org/indexnow');
  assert.deepEqual(JSON.parse(post.options.body).urlList, [config.origin + '/']);
  status = 429;
  await assert.rejects(submitChanges(current, null, fetcher), /429/);
  await assert.rejects(submitChanges(current, null, async () => Response.json({})), /expected artifact/);
  await assert.rejects(submitChanges(current, null, async url => url.endsWith('.txt') ? new Response('wrong key') : Response.json(current)), /key does not match/);
});

test('static proof file matches configured key', async () => {
  assert.equal((await readFile(new URL(`../public/${config.key}.txt`, import.meta.url), 'utf8')).trim(), config.key);
});
