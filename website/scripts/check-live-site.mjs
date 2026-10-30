import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { load } from 'cheerio';
import { localeIds, pagePath } from '../site-model.mjs';
import { sha256 } from './site-manifest.mjs';
import { publishedPages } from './published-pages.mjs';
import { checkMachineHttp } from './machine-http.mjs';
import { llmsPath } from '../src/lib/machine-urls.mjs';
import { localePath } from '../locale-model.mjs';
import { assertIndexable } from './indexability.mjs';
import { assertCrawlable } from './robots-policy.mjs';

// Deliberately independent of build configuration: a self-consistent wrong
// origin must not pass production acceptance. Require the verified CI artifact.
const origin = 'https://ankiforge.dev';
const expected = JSON.parse(await readFile(process.argv[2] || new URL('../dist/generated/site-manifest.json', import.meta.url), 'utf8'));
const pages = await publishedPages();
assert.deepEqual(expected.configuration, { site: origin, base: '/', customDomain: 'ankiforge.dev' });
if (process.env.GITHUB_SHA) assert.equal(expected.commit, process.env.GITHUB_SHA, 'Artifact is from another commit');

async function get(route, status = 200) {
  const url = new URL(route, origin).href;
  const response = await fetch(url, { redirect: 'manual', signal: AbortSignal.timeout(20_000), headers: { 'Cache-Control': 'no-cache' } });
  assert.equal(response.status, status, `${route}: HTTP status`);
  assert.equal(response.url, url, `${route}: unexpected redirect`);
  const bytes = Buffer.from(await response.arrayBuffer());
  if (status === 200 && (route.endsWith('/index.md') || route.endsWith('/llms.txt'))) checkMachineHttp(route, response, bytes);
  if (status === 200 && route.endsWith('/')) assertIndexable(load(bytes.toString()), { route, headers: response.headers });
  return bytes;
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
assert.deepEqual(expected.releaseLocales, localeIds, 'Artifact language snapshot differs from reviewed release target');
const machineRoutes = pages.filter(page => ['docs', 'article'].includes(page.type)).flatMap(page => localeIds.map(locale => `${pagePath(page.id, locale)}index.md`)).concat(localeIds.map(locale => llmsPath(locale))).sort();
const recordedMachineRoutes = expected.resources.map(resource => resource.route).filter(route => route.endsWith('/index.md') || route.endsWith('/llms.txt')).sort();
assert.deepEqual(recordedMachineRoutes, machineRoutes, 'Verified manifest must contain exactly the published machine resources');

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
  assert.equal($('link[rel="alternate"][hreflang="x-default"]').attr('href'), origin + pagePath(pair.id, 'en'), `${route}: x-default`);
  assert.equal($('h1').length, 1);
  assert.equal($('link[rel="describedby"]').length, 1);
  assert.equal($('link[rel="describedby"]').attr('href'), origin + llmsPath(locale));
  const markdown = $('link[rel="alternate"][type="text/markdown"]');
  assert.equal(markdown.length, ['docs', 'article'].includes(pair.type) ? 1 : 0, `${route}: Markdown discovery`);
  if (markdown.length) assert.equal(markdown.attr('href'), `${origin}${route}index.md`);
  const schema = $('script[type="application/ld+json"][data-anki-forge-schema]');
  assert.equal(schema.length, 1, `${route}: missing structured data`);
  const graph = JSON.parse(schema.text())['@graph'];
  assert.ok(graph.some(node => node['@id'] === `${origin}${route}#webpage` && node.inLanguage === locale), `${route}: wrong structured page identity`);
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
const robots = (await get('/robots.txt')).toString();
assert.match(robots, /Sitemap: https:\/\/ankiforge\.dev\/sitemap-index\.xml/);
assertCrawlable(robots, normal.map(page => page.route));
await get('/pagefind/pagefind.js');
for (const route of [...localeIds.flatMap(locale => [localePath('/nonexistent-topic-62/', locale), localePath('/docs/nonexistent-machine-62/index.md', locale), localePath('/nonexistent-llms-62.txt', locale)])]) {
  const $ = load((await get(route, 404)).toString());
  assert.equal($('meta[name="robots"]').attr('content'), 'noindex');
  // GitHub Pages has one static error document, with bilingual no-script recovery.
  for (const href of localeIds.flatMap(locale => [pagePath('home', locale), pagePath('docs', locale)])) assert.ok($(`a[href="${href}"]`).length, `${route}: missing recovery ${href}`);
}
console.log(`Production matches verified commit ${expected.commit}: ${normal.length} content URLs, capabilities, resources, feeds, sitemap and real 404s. Analytics dashboard delivery is outside this check.`);
