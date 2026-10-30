import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import { homepageModules, stripBase, pagePath, resolvePage, localeIds, locales } from '../site-model.mjs';
import { socialImagePath, localePath } from '../locale-model.mjs';
import { assertIndexable } from './indexability.mjs';
import { assertCrawlable } from './robots-policy.mjs';
import { publishedPages } from './published-pages.mjs';
import { loadBenchmarkPresentation } from './benchmark-presentation.mjs';
import { siteConfig, withBase } from '../site.config.mjs';
import { loadAiQuickstart } from './ai-quickstart.mjs';
import { renderPrompt } from '../src/lib/ai-quickstart.mjs';
import { createTranslator } from '../src/i18n/messages.mjs';

const dist = process.argv[2] ? path.resolve(process.argv[2]) : fileURLToPath(new URL('../dist/', import.meta.url));
const corePages = await publishedPages();
const errors = new Set();
const pages = new Map();
const record = (condition, message) => { if (!condition) errors.add(message); };

async function walk(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : [path.join(directory, entry.name)]));
  return nested.flat();
}

const files = await walk(dist);
const release = JSON.parse(await readFile(path.join(dist, 'generated/site-manifest.json'), 'utf8'));
const resourceFiles = files.filter(file => {
  const relative = path.relative(dist, file).split(path.sep).join('/');
  return !relative.endsWith('.html') && relative !== 'generated/site-manifest.json' && relative !== 'CNAME' && !relative.startsWith('.');
});
assert.deepEqual(release.resources.map(resource => resource.route).sort(), resourceFiles.map(file => withBase('/' + path.relative(dist, file).split(path.sep).join('/'), siteConfig.base)).sort(), 'Release manifest must include every resource, including the complete search index');
for (const resource of release.resources) {
  const filename = stripBase(resource.route, siteConfig.base).slice(1);
  record(createHash('sha256').update(await readFile(path.join(dist, filename))).digest('hex') === resource.sha256, `${filename}: resource differs from verified manifest`);
}
for (const file of files.filter(file => file.endsWith('.html'))) {
  const relative = path.relative(dist, file).split(path.sep).join('/');
  const route = `/${relative.replace(/index\.html$/, '')}`;
  const url = new URL(withBase(route, siteConfig.base), siteConfig.site);
  const $ = load(await readFile(file, 'utf8'));
  pages.set(file, { $, url, relative });
}

async function checkURL(value, from, { anchor = false } = {}) {
  if (!value || /^(?:data:|mailto:|tel:|javascript:)/i.test(value)) return;
  let url;
  try { url = new URL(value, from.url); } catch { errors.add(`${from.relative}: invalid URL ${value}`); return; }
  if (url.origin !== siteConfig.site) return;
  const prefix = siteConfig.base === '/' ? '/' : `${siteConfig.base}/`;
  if (!url.pathname.startsWith(prefix)) {
    errors.add(`${from.relative}: URL lost its base path: ${value}`);
    return;
  }
  const relative = decodeURIComponent(url.pathname.slice(prefix.length));
  let target = path.join(dist, relative);
  try {
    const info = await stat(target);
    if (info.isDirectory()) target = path.join(target, 'index.html');
    await stat(target);
  } catch {
    errors.add(`${from.relative}: missing local target ${value}`);
    return;
  }
  if (anchor && url.hash && pages.has(target)) {
    const id = decodeURIComponent(url.hash.slice(1));
    const targetPage = pages.get(target);
    const ids = targetPage.$('[id]').toArray().map(element => targetPage.$(element).attr('id'));
    record(ids.includes(id), `${from.relative}: missing anchor ${value}`);
  }
}

for (const page of pages.values()) {
  const { $, url, relative } = page;
  record($('h1').length === 1, `${relative}: expected one h1`);
  record($('title').text().trim().length > 0, `${relative}: missing title`);
  record(($('meta[name="description"]').attr('content') || '').length > 0, `${relative}: missing description`);
  if (relative !== '404.html') {
    record($('link[rel="canonical"]').length === 1 && $('link[rel="canonical"]').attr('href') === url.href, `${relative}: canonical does not match ${url.href}`);
    try { assertIndexable($, { route: relative }); } catch (error) { errors.add(error.message); }
  }
  record($('meta[property="og:image"]').length === 1, `${relative}: expected one social image`);
  record($('meta[name="twitter:card"]').length === 1, `${relative}: expected one Twitter card setting`);
  if (relative === '404.html') record($('meta[name="robots"]').attr('content') === 'noindex', '404 page must be excluded from indexing');
  const ids = $('[id]').toArray().map(element => $(element).attr('id'));
  record(new Set(ids).size === ids.length, `${relative}: duplicate element IDs`);
  for (const element of $('a[href], link[href], script[src], img[src], audio[src], source[src]').toArray()) {
    const node = $(element);
    await checkURL(node.attr('href') || node.attr('src'), page, { anchor: element.tagName === 'a' });
  }
  await checkURL($('meta[property="og:image"]').attr('content'), page);
}

// Validate the actual output set, not just routes derived from the build config.
const normalRoutes = [...pages.values()].filter(page => page.relative !== '404.html').map(page => '/' + page.relative.replace(/index\.html$/, '')).sort();
assertCrawlable(await readFile(path.join(dist, 'robots.txt'), 'utf8'), normalRoutes.map(route => withBase(route, siteConfig.base)));
assert.deepEqual(normalRoutes, corePages.flatMap(page => localeIds.map(locale => pagePath(page.id, locale))).sort(), 'Registered and actually built topic and release-language sets must match');
for (const pair of corePages) {
  for (const locale of localeIds) {
    const route = pagePath(pair.id, locale);
    const page = pages.get(path.join(dist, route.slice(1), 'index.html'));
    assert.ok(page, `Missing core page ${route}`);
    record(page.$('html').attr('lang') === locale, `${route}: wrong HTML language`);
    record(page.$('script[src="https://static.cloudflareinsights.com/beacon.min.js"]').length === 1, `${route}: expected one analytics beacon`);
    // Starlight renders controls in both the header and mobile preferences.
    // Every target language must be available; each control is validated below.
    const languageTargets = new Set(page.$('[data-language-switch]').toArray().map(node => page.$(node).attr('href')));
    assert.deepEqual([...languageTargets].sort(), localeIds.filter(id => id !== locale).map(id => withBase(pagePath(pair.id, id), siteConfig.base)).sort(), `${route}: switches must retain topic and release languages`);
    for (const node of page.$('[data-language-switch]').toArray()) {
      const link = page.$(node);
      const targetLocale = link.attr('hreflang');
      assert.equal(link.attr('href'), withBase(pagePath(pair.id, targetLocale), siteConfig.base), `${route}: language switch must match its hreflang and topic`);
      const target = pages.get(path.join(dist, pagePath(pair.id, targetLocale).slice(1), 'index.html'));
      const ids = new Set(target.$('[id]').toArray().map(element => target.$(element).attr('id')));
      for (const id of JSON.parse(link.attr('data-section-ids') || '[]')) {
        if (id !== '_top') record(ids.has(id), `${route}: language switch inventory points to missing ${targetLocale} anchor ${id}`);
      }
      for (const id of Object.values(JSON.parse(link.attr('data-section-map') || '{}'))) {
        if (id !== '_top') record(ids.has(id), `${route}: language switch mapping points to missing ${targetLocale} task ${id}`);
      }
    }
    for (const language of localeIds) {
      const target = pagePath(pair.id, language);
      const alternate = page.$(`link[rel="alternate"][hreflang="${language}"]`);
      record(alternate.length === 1 && alternate.attr('href') === new URL(withBase(target, siteConfig.base), siteConfig.site).href, `${route}: incorrect ${language} hreflang`);
    }
    record(page.$('link[rel="alternate"][hreflang="x-default"]').attr('href') === new URL(withBase(pagePath(pair.id, 'en'), siteConfig.base), siteConfig.site).href, `${route}: incorrect x-default`);
  }
}
for (const route of corePages.some(page => page.id === 'json-to-anki') ? localeIds.map(locale => pagePath('json-to-anki', locale)) : []) {
  const page = pages.get(path.join(dist, route.slice(1), 'index.html'));
  for (const id of ['rust', 'python', 'node']) {
    record(page.$(`#${id}`).length === 1, `${route}: missing SDK anchor ${id}`);
  }
  for (const expected of ['prepare_publication', 'preparePublication', 'Content.sequence']) record(page.$.html().includes(expected), `${route}: SDK program missing from static HTML`);
}
const sitemapOutput = await readFile(path.join(dist, 'sitemap-0.xml'), 'utf8');
const sitemap = load(sitemapOutput, { xml: true });
assert.deepEqual(sitemap('url > loc').toArray().map(node => sitemap(node).text()).sort(), normalRoutes.map(route => new URL(withBase(route, siteConfig.base), siteConfig.site).href).sort(), 'Sitemap must include exactly normal HTML release pages');

for (const name of ['sitemap-index.xml', 'sitemap-0.xml', ...localeIds.map(locale => localePath('/rss.xml', locale).slice(1))]) {
  const $ = load(await readFile(path.join(dist, name), 'utf8'), { xml: true });
  const from = { relative: name, url: new URL(withBase(`/${name}`, siteConfig.base), siteConfig.site) };
  const selector = name.endsWith('rss.xml') ? 'channel > link, item > link' : 'loc';
  for (const node of $(selector).toArray()) await checkURL($(node).text(), from);
}
await stat(path.join(dist, 'pagefind/pagefind.js'));
const searchIndex = JSON.parse(await readFile(path.join(dist, 'pagefind/pagefind-entry.json'), 'utf8'));
for (const locale of localeIds) {
  const contentCount = [...pages.values()].filter(page => page.relative !== '404.html' && page.$('html').attr('lang') === locale).length;
  record(searchIndex.languages[locale.toLowerCase()]?.page_count === contentCount, `${locale}: search index must include every content page and exclude the error page`);
}
record(pages.get(path.join(dist, '404.html')).$('[data-pagefind-ignore]').length > 0, '404 content must be excluded from Pagefind');
const manifest = JSON.parse(await readFile(path.join(dist, 'generated/showcase.json'), 'utf8'));
assert.equal(manifest.schemaVersion, 2);
assert.deepEqual(manifest.examples.map(example => example.id), ['basic', 'cloze', 'media', 'occlusion']);
assert.deepEqual(manifest.update.artifacts.map(artifact => artifact.file), [manifest.update.previous, manifest.update.next]);
for (const example of [...manifest.examples, manifest.combined, ...manifest.update.artifacts]) {
  const bytes = await readFile(path.join(dist, 'generated', example.file));
  record(createHash('sha256').update(bytes).digest('hex') === example.sha256, `${example.file}: download hash does not match preview manifest`);
  record(bytes.length === example.bytes, `${example.file}: download size does not match preview manifest`);
}
for (const example of manifest.examples) record(example.counts.notes === 1 && example.counts.cards === 1, `${example.file}: unexpected verified counts`);
const combined = manifest.combined;
record(combined.counts.notes === 4 && combined.counts.cards === 4 && combined.counts.media === 3, `${combined.file}: unexpected verified counts`);
const comparison = JSON.parse(await readFile(path.join(dist, 'generated', manifest.update.report), 'utf8'));
assert.deepEqual(comparison, manifest.update.comparison, 'Update report does not match displayed evidence');
record(comparison.policy.allows_publication && comparison.findings.some(finding => finding.code === 'RISK.NOTE_CHANGED'), 'Update report must describe the demonstrated content change');
record(!comparison.findings.some(finding => ['RISK.NOTE_ADDED', 'RISK.NOTE_REMOVED'].includes(finding.code)), 'Update unexpectedly changed note identity');

for (const file of [manifest.update.previous, manifest.update.next]) await stat(path.join(dist, 'generated', file));
if (siteConfig.customDomain) assert.equal((await readFile(path.join(dist, 'CNAME'), 'utf8')).trim(), siteConfig.customDomain);

const benchmark = await loadBenchmarkPresentation();
const ai = await loadAiQuickstart();
for (const locale of localeIds) {
const home = pages.get(path.join(dist, pagePath('home', locale).slice(1), 'index.html'));
const t = createTranslator(locale);
record(home.$('#ai-quickstart[data-ai-quickstart]').length === 1, 'Missing AI quickstart anchor');
for (const [key, language] of Object.entries(ai.languages)) {
  const panel = home.$(`[data-ai-panel="${key}"]`);
  record(panel.length === 1 && panel.attr('hidden') === undefined, `${key}: missing no-script AI template`);
  record(panel.find('[data-ai-prompt]').text() === renderPrompt(ai, key, '', locale), `${key}: rendered prompt drift`);
  record(panel.find('[data-copy-content]').last().text() === language.install, `${key}: rendered installation drift`);
  record(panel.find('[data-copy-status][aria-live="polite"]').length === 2, `${key}: missing copy announcements`);
  record(home.$('.home-language-grid [data-copy-content]').toArray().some(node => home.$(node).text() === language.install), `${locale}/${key}: missing shared installation command`);
  record(home.$(`#ai-tab-${key}`).attr('aria-controls') === `ai-panel-${key}`, `${key}: incorrect tab target`);
}
for (const result of benchmark.results) {
  const row = home.$(`[data-profile="${result.profile}"]`);
  record(row.length === 1, `Missing benchmark workload ${result.profile}`);
  record(row.find('p').text() === `${result.rust.toFixed(1)} ms / ${result.genanki.toFixed(1)} ms`, `Incorrect benchmark times ${result.profile}`);
  record(row.find('strong').text() === t(result.saved >= 0 ? 'performance.less' : 'performance.more', { value: Math.abs(result.saved).toFixed(1) }), `Incorrect benchmark advantage ${result.profile}`);
  for (const implementation of ['rust', 'genanki']) {
    record(row.attr('style').includes(`--${implementation}-width: ${result[implementation] / benchmark.maxTime * 100}%`), `Incorrect chart scale ${result.profile}/${implementation}`);
  }
}
const metrics = home.$('.home-performance-metric').text();
record(metrics.includes(benchmark.results[0].rust.toFixed(1)), 'Incorrect text export headline');
if (benchmark.minimumSaved >= 0) record(metrics.includes(`${benchmark.minimumSaved.toFixed(1)}–${benchmark.maximumSaved.toFixed(1)}`), 'Incorrect time advantage headline');


}

// Compare observed capabilities and destinations, not HTML strings or a fixed page count.
const normalized = value => {
  const url = new URL(value, siteConfig.site);
  if (url.origin !== siteConfig.site) return url.href;
  const pathname = stripBase(url.pathname, siteConfig.base);
  assert.ok(pathname !== null, `${value}: local destination lost the configured base`);
  const prefix = Object.values(locales).find(locale => locale.prefix && (pathname === locale.prefix || pathname.startsWith(locale.prefix + '/')))?.prefix;
  return prefix ? pathname.slice(prefix.length) || '/' : pathname;
};
const hrefs = (page, selector) => page.$(selector).toArray().map(element => page.$(element).attr('href')).filter(Boolean).map(normalized);
for (const pair of corePages) {
  const reference = pages.get(path.join(dist, pagePath(pair.id, 'en').slice(1), 'index.html'));
  for (const locale of localeIds) {
    const page = pages.get(path.join(dist, pagePath(pair.id, locale).slice(1), 'index.html'));
    const $ = page.$;
    record($('meta[name="anki-forge:page-id"]').attr('content') === pair.id, `${page.relative}: missing topic identity`);
    record($('meta[property="og:locale"]').attr('content') === locales[locale].og, `${page.relative}: incorrect OG locale`);
    record($('meta[property="og:image"]').attr('content').endsWith(socialImagePath(locale)), `${page.relative}: wrong social image language`);
    for (const nav of ['.desktop-nav', '.mobile-menu nav']) {
      assert.deepEqual(hrefs(page, `${nav} [data-section]`), ['/docs/', '/examples/', '/blog/'], `${page.relative}: complete shared navigation required`);
      record($(`${nav} [aria-current="page"]`).attr('data-section') === (pair.section === 'home' ? undefined : pair.section), `${page.relative}: wrong current section`);
    }
    const feed = $('link[rel="alternate"][type="application/rss+xml"]');
    record(feed.length === 1 && feed.attr('href') === withBase(localePath('/rss.xml', locale), siteConfig.base), `${page.relative}: feed changed language`);
    for (const node of $('a[href]').toArray()) {
      const link = $(node);
      if (link.is('[data-language-switch]') || link.attr('href').startsWith('#')) continue;
      const url = new URL(link.attr('href'), page.url);
      if (url.origin !== siteConfig.site) continue;
      const linked = resolvePage(url.pathname, siteConfig.base);
      if (linked) record(linked.locale === locale, `${page.relative}: unexpected cross-language link ${link.attr('href')}`);
    }
    if (pair.type === 'docs') {
      record($('site-search [data-open-modal]').length === 1, `${page.relative}: missing search action`);
      record($('starlight-toc').length >= 1, `${page.relative}: missing table of contents`);
      record($('a[href="https://github.com/morehardy/anki-forge/edit/main/' + pair.sources[locale] + '"]').length === 1, `${page.relative}: edit link must reach the authoritative translation`);
      record($('meta[name="anki-forge:content-source"]').attr('content') === pair.sources[locale], `${page.relative}: fallback or wrong content source`);
      record($('pre').length === $('.expressive-code .copy button, [data-copy-block] [data-copy]').length, `${page.relative}: every code block must be copyable`);
      assert.deepEqual(hrefs(page, '#starlight__sidebar a'), hrefs(reference, '#starlight__sidebar a'), `${pair.id}/${locale}: sidebar topics or order differ`);
      assert.deepEqual(hrefs(page, '.pagination-links a'), hrefs(reference, '.pagination-links a'), `${pair.id}/${locale}: reading order differs`);
      // Cross-language data integrity belongs to semantic content contracts;
      // natural translations need not keep the English table shape.
    }
    assert.deepEqual(hrefs(page, '.site-footer nav a'), hrefs(reference, '.site-footer nav a'), `${pair.id}/${locale}: footer destinations differ`);
    if (['home', 'examples'].includes(pair.type)) {
      for (const selector of ['[download]', '[data-workbench]', '[data-card-preview]', '[data-flip]', 'audio', '[data-update]', '[data-profile]', '[data-ai-panel]', '[data-copy]']) {
        record(reference.$(selector).length === $(selector).length, `${pair.id}/${locale}: missing translated capability ${selector}`);
      }
      assert.deepEqual(hrefs(page, '[download]'), hrefs(reference, '[download]'), `${pair.id}/${locale}: download resources differ`);
    }
  }
}
for (const locale of localeIds) {
  const route = pagePath('home', locale);
  const page = pages.get(path.join(dist, route.slice(1), 'index.html'));
  assert.deepEqual(page.$('[data-module]').toArray().map(element => page.$(element).attr('data-module')), homepageModules, `${route}: homepage module order differs`);
  record(page.$('[data-workbench] [role="tab"]').length === 4, `${route}: all four examples required`);
  record(page.$('[download]').length === 6, `${route}: all six downloads required`);
  record(page.$('[data-profile]').length === 5, `${route}: five benchmark workloads required`);
  for (const id of ['basic', 'cloze', 'media', 'occlusion']) record(page.$(`[data-example="${id}"] [data-flip]`).length === 1, `${route}: ${id} flip action required`);
  record(page.$('[data-update-demo] [data-update]').length === 1, `${route}: update/reset action required`);
  record(page.$('[data-example="media"] audio[controls]').length === 1, `${route}: audio playback required`);
  const filename = localePath('/rss.xml', locale).slice(1);
  const $ = load(await readFile(path.join(dist, filename), 'utf8'), { xml: true });
  record($('channel > language').text() === locale, `${filename}: wrong feed language`);
  assert.deepEqual($('item > link').toArray().map(node => new URL($(node).text()).pathname).sort(), corePages.filter(page => page.type === 'article').map(page => withBase(pagePath(page.id, locale), siteConfig.base)).sort(), `${filename}: feed topic set differs`);
}

if (errors.size) {
  console.error([...errors].join('\n'));
  process.exitCode = 1;
} else {
  console.log(`Checked ${pages.size} pages: metadata, local links and anchors, search, feeds, sitemap, and verified deck downloads. Base: ${siteConfig.base}`);
}
