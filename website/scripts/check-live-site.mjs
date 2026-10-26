import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { load } from 'cheerio';
import { pages, localeIds, pagePath } from '../site-model.mjs';
import { sha256 } from './site-manifest.mjs';

// Deliberately independent of build configuration: a self-consistent wrong
// origin must not pass production acceptance. Require the verified CI artifact.
const origin = 'https://ankiforge.dev';
const expected = JSON.parse(await readFile(process.argv[2] || new URL('../dist/generated/site-manifest.json', import.meta.url), 'utf8'));
assert.deepEqual(expected.configuration, { site: origin, base: '/', customDomain: 'ankiforge.dev' });
if (process.env.GITHUB_SHA) assert.equal(expected.commit, process.env.GITHUB_SHA, 'Artifact is from another commit');

async function get(route, status = 200) {
  const url = new URL(route, origin).href;
  const response = await fetch(url, { redirect: 'follow', signal: AbortSignal.timeout(20_000), headers: { 'Cache-Control': 'no-cache' } });
  assert.equal(response.status, status, `${route}: HTTP status`);
  assert.equal(response.url, url, `${route}: unexpected redirect`);
  return Buffer.from(await response.arrayBuffer());
}
// Pages/CDN publication may take a few seconds. Only the exact verified manifest
// counts as publication; a successful response from the previous site does not.
for (let attempt = 0; ; attempt++) {
  try {
    assert.deepEqual(JSON.parse((await get('/generated/site-manifest.json')).toString()), expected, 'Production is serving another artifact');
    break;
  } catch (error) {
    if (attempt === 9) throw error;
    await new Promise(resolve => setTimeout(resolve, 3000));
  }
}
const normal = expected.pages.filter(page => page.id !== '404');
assert.deepEqual(normal.map(page => page.route).sort(), pages.flatMap(page => localeIds.map(locale => pagePath(page.id, locale))).sort(), 'Artifact topic set differs from the reviewed registry');

for (const pair of pages) for (const locale of localeIds) {
  const route = pagePath(pair.id, locale);
  const artifact = normal.find(page => page.route === route);
  const bytes = await get(route);
  assert.equal(sha256(bytes), artifact.sha256, `${route}: deployed HTML differs from verified artifact`);
  const $ = load(bytes.toString());
  assert.equal($('html').attr('lang'), locale, `${route}: wrong language`);
  assert.equal($('link[rel="canonical"]').length, 1, `${route}: canonical count`);
  assert.equal($('link[rel="canonical"]').attr('href'), origin + route);
  assert.equal($('meta[name="anki-forge:page-id"]').attr('content'), pair.id);
  for (const alternate of localeIds) {
    const link = $(`link[hreflang="${alternate}"]`);
    assert.equal(link.length, 1, `${route}: alternate count`);
    assert.equal(link.attr('href'), origin + pagePath(pair.id, alternate));
  }
  assert.equal($('h1').length, 1);
  assert.equal($('script[src="https://static.cloudflareinsights.com/beacon.min.js"]').length, 1);
  const selectors = { downloads: '[download]', search: 'site-search', codeCopy: '[data-copy], .expressive-code .copy button', flips: '[data-flip]', audio: 'audio', updates: '[data-update]' };
  for (const [capability, selector] of Object.entries(selectors)) assert.equal($(selector).length, artifact.capabilities[capability], `${route}: missing ${capability}`);
  console.log(`Verified ${locale} ${route}`);
}
for (let offset = 0; offset < expected.resources.length; offset += 8) {
  await Promise.all(expected.resources.slice(offset, offset + 8).map(async resource => {
    const bytes = await get(resource.route);
    assert.equal(sha256(bytes), resource.sha256, `${resource.route}: deployed resource differs from verified artifact`);
  }));
}
const xml = load((await get('/sitemap-0.xml')).toString(), { xml: true });
assert.deepEqual(xml('loc').toArray().map(node => xml(node).text()).sort(), normal.map(page => origin + page.route).sort(), 'Production sitemap set differs from verified content');
assert.match((await get('/robots.txt')).toString(), /Sitemap: https:\/\/ankiforge\.dev\/sitemap-index\.xml/);
await get('/pagefind/pagefind.js');
for (const route of ['/nonexistent-topic-62/', '/zh-cn/nonexistent-topic-62/']) {
  const $ = load((await get(route, 404)).toString());
  assert.equal($('meta[name="robots"]').attr('content'), 'noindex');
  // GitHub Pages has one static error document, with bilingual no-script recovery.
  for (const href of ['/', '/docs/', '/zh-cn/', '/zh-cn/docs/']) assert.ok($(`a[href="${href}"]`).length, `${route}: missing recovery ${href}`);
}
console.log(`Production matches verified commit ${expected.commit}: ${normal.length} content URLs, capabilities, resources, feeds, sitemap and real 404s. Analytics dashboard delivery is outside this check.`);
