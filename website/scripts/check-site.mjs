import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import { pages as corePages, homepageModules, pagePair, stripBase } from '../site-model.mjs';
import { loadBenchmarkPresentation } from './benchmark-presentation.mjs';
import { siteConfig, withBase } from '../site.config.mjs';
import { loadAiQuickstart } from './ai-quickstart.mjs';
import { renderPrompt } from '../src/lib/ai-quickstart.mjs';
import { createTranslator } from '../src/i18n/messages.mjs';

const dist = process.argv[2] ? path.resolve(process.argv[2]) : fileURLToPath(new URL('../dist/', import.meta.url));
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
  if (relative !== '404.html') record($('link[rel="canonical"]').attr('href') === url.href, `${relative}: canonical does not match ${url.href}`);
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
const chinese = [...pages.values()].filter(page => page.relative.startsWith('zh-cn/'));
const englishRoutes = [...pages.values()].filter(page => !page.relative.startsWith('zh-cn/') && page.relative !== '404.html').map(page => '/zh-cn/' + page.relative.replace(/index\.html$/, ''));
assert.deepEqual(chinese.map(page => '/' + page.relative.replace(/index\.html$/, '')).sort(), englishRoutes.sort(), 'English and Chinese built content sets must match');
assert.deepEqual(chinese.map(page => '/' + page.relative.replace(/index\.html$/, '')).sort(), corePages.map(pair => pair.zh).sort(), 'Registered and actually built Chinese topic sets must match');
for (const pair of corePages) {
  for (const [locale, route] of [['en', pair.en], ['zh-CN', pair.zh]]) {
    const page = pages.get(path.join(dist, route.slice(1), 'index.html'));
    assert.ok(page, `Missing core page ${route}`);
    record(page.$('html').attr('lang') === locale, `${route}: wrong HTML language`);
    record(page.$('script[src="https://static.cloudflareinsights.com/beacon.min.js"]').length === 1, `${route}: expected one analytics beacon`);
    record(page.$('[data-language-switch]').attr('href') === withBase(locale === 'en' ? pair.zh : pair.en, siteConfig.base), `${route}: switch must retain the topic`);
    for (const [language, target] of [['en', pair.en], ['zh-CN', pair.zh]]) {
      const alternate = page.$(`link[rel="alternate"][hreflang="${language}"]`);
      record(alternate.length === 1 && alternate.attr('href') === new URL(withBase(target, siteConfig.base), siteConfig.site).href, `${route}: incorrect ${language} hreflang`);
    }
  }
}
for (const route of ['/docs/json-to-anki/', '/zh-cn/docs/json-to-anki/']) {
  const page = pages.get(path.join(dist, route.slice(1), 'index.html'));
  for (const id of ['rust', 'python', 'node']) {
    record(page.$(`#${id}`).length === 1, `${route}: missing SDK anchor ${id}`);
  }
  for (const expected of ['prepare_publication', 'preparePublication', 'Content.sequence']) record(page.$.html().includes(expected), `${route}: SDK program missing from static HTML`);
}
const sitemapOutput = await readFile(path.join(dist, 'sitemap-0.xml'), 'utf8');
for (const pair of corePages) for (const route of [pair.en, pair.zh]) record(sitemapOutput.includes(new URL(withBase(route, siteConfig.base), siteConfig.site).href), `Missing core sitemap URL ${route}`);

for (const name of ['sitemap-index.xml', 'sitemap-0.xml', 'rss.xml', 'zh-cn/rss.xml']) {
  const $ = load(await readFile(path.join(dist, name), 'utf8'), { xml: true });
  const from = { relative: name, url: new URL(withBase(`/${name}`, siteConfig.base), siteConfig.site) };
  const selector = name.endsWith('rss.xml') ? 'channel > link, item > link' : 'loc';
  for (const node of $(selector).toArray()) await checkURL($(node).text(), from);
}
await stat(path.join(dist, 'pagefind/pagefind.js'));
const searchIndex = JSON.parse(await readFile(path.join(dist, 'pagefind/pagefind-entry.json'), 'utf8'));
for (const locale of ['en', 'zh-CN']) {
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
for (const locale of ['en', 'zh-CN']) {
const home = pages.get(path.join(dist, locale === 'en' ? 'index.html' : 'zh-cn/index.html'));
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
const normalized = value => stripBase(new URL(value, siteConfig.site).pathname, siteConfig.base).replace(/^\/zh-cn(?=\/)/, '');
const hrefs = (page, selector) => page.$(selector).toArray().map(element => page.$(element).attr('href')).filter(Boolean).map(normalized);
for (const pair of corePages) {
  const en = pages.get(path.join(dist, pair.en.slice(1), 'index.html'));
  const zh = pages.get(path.join(dist, pair.zh.slice(1), 'index.html'));
  for (const [locale, page] of [['en', en], ['zh-CN', zh]]) {
    const $ = page.$;
    record($('meta[name="anki-forge:page-id"]').attr('content') === pair.id, `${page.relative}: missing topic identity`);
    record($('meta[property="og:locale"]').attr('content') === (locale === 'en' ? 'en_US' : 'zh_CN'), `${page.relative}: incorrect OG locale`);
    record($('meta[property="og:image"]').attr('content').endsWith(locale === 'en' ? '/generated/og.png' : '/generated/og-zh-cn.png'), `${page.relative}: wrong social image language`);
    for (const nav of ['.desktop-nav', '.mobile-menu nav']) {
      assert.deepEqual(hrefs(page, `${nav} [data-section]`), ['/docs/', '/examples/', '/blog/'], `${page.relative}: complete shared navigation required`);
      record($(`${nav} [aria-current="page"]`).attr('data-section') === (pair.section === 'home' ? undefined : pair.section), `${page.relative}: wrong current section`);
    }
    const feed = $('link[rel="alternate"][type="application/rss+xml"]');
    record(feed.attr('href') === withBase(locale === 'en' ? '/rss.xml' : '/zh-cn/rss.xml', siteConfig.base), `${page.relative}: feed changed language`);
    for (const node of $('a[href]').toArray()) {
      const link = $(node);
      if (link.is('[data-language-switch]') || link.attr('href').startsWith('#')) continue;
      const url = new URL(link.attr('href'), page.url);
      if (url.origin !== siteConfig.site) continue;
      const linked = pagePair(url.pathname, siteConfig.base);
      if (linked) record(stripBase(url.pathname, siteConfig.base) === (locale === 'en' ? linked.en : linked.zh), `${page.relative}: unexpected cross-language link ${link.attr('href')}`);
    }
    if (locale === 'zh-CN') record(/[\u3400-\u9fff]/.test($('main').text()), `${page.relative}: untranslated body`);
    if (pair.type === 'docs') {
      record($('site-search [data-open-modal]').length === 1, `${page.relative}: missing search action`);
      record($('starlight-toc').length >= 1, `${page.relative}: missing table of contents`);
      record($('a[href="https://github.com/morehardy/anki-forge/edit/main/' + pair.sources[locale] + '"]').length === 1, `${page.relative}: edit link must reach the authoritative translation`);
      record($('meta[name="anki-forge:content-source"]').attr('content') === pair.sources[locale], `${page.relative}: fallback or wrong content source`);
      record($('pre').length === $('.expressive-code .copy button, [data-copy-block] [data-copy]').length, `${page.relative}: every code block must be copyable`);
    }
  }
  assert.deepEqual(hrefs(zh, '.site-footer nav a'), hrefs(en, '.site-footer nav a'), `${pair.id}: footer destinations differ`);
  if (pair.type === 'docs') {
    assert.deepEqual(hrefs(zh, '#starlight__sidebar a'), hrefs(en, '#starlight__sidebar a'), `${pair.id}: sidebar topics or order differ`);
    assert.deepEqual(hrefs(zh, '.pagination-links a'), hrefs(en, '.pagination-links a'), `${pair.id}: reading order differs`);
    const tables = page => page.$('main table').toArray().map(table => page.$(table).find('tr').toArray().map(row => page.$(row).find('th,td').length));
    assert.deepEqual(tables(zh), tables(en), `${pair.id}: table structure differs`);
  }
  if (['home', 'examples'].includes(pair.type)) {
    for (const selector of ['[download]', '[data-workbench]', '[data-card-preview]', '[data-flip]', 'audio', '[data-update]', '[data-profile]', '[data-ai-panel]', '[data-copy]']) {
      record(en.$(selector).length === zh.$(selector).length, `${pair.id}: missing translated capability ${selector}`);
    }
    assert.deepEqual(hrefs(zh, '[download]'), hrefs(en, '[download]'), `${pair.id}: download resources differ`);
  }
}
for (const route of ['/', '/zh-cn/']) {
  const page = pages.get(path.join(dist, route.slice(1), 'index.html'));
  assert.deepEqual(page.$('[data-module]').toArray().map(element => page.$(element).attr('data-module')), homepageModules, `${route}: homepage module order differs`);
  record(page.$('[data-workbench] [role="tab"]').length === 4, `${route}: all four examples required`);
  record(page.$('[download]').length === 6, `${route}: all six downloads required`);
  record(page.$('[data-profile]').length === 5, `${route}: five benchmark workloads required`);
  for (const id of ['basic', 'cloze', 'media', 'occlusion']) record(page.$(`[data-example="${id}"] [data-flip]`).length === 1, `${route}: ${id} flip action required`);
  record(page.$('[data-update-demo] [data-update]').length === 1, `${route}: update/reset action required`);
  record(page.$('[data-example="media"] audio[controls]').length === 1, `${route}: audio playback required`);
}
for (const locale of ['en', 'zh-CN']) {
  const filename = locale === 'en' ? 'rss.xml' : 'zh-cn/rss.xml';
  const $ = load(await readFile(path.join(dist, filename), 'utf8'), { xml: true });
  record($('channel > language').text() === locale, `${filename}: wrong feed language`);
  assert.deepEqual($('item > link').toArray().map(node => new URL($(node).text()).pathname).sort(), corePages.filter(page => page.type === 'article').map(page => withBase(locale === 'en' ? page.en : page.zh, siteConfig.base)).sort(), `${filename}: feed topic set differs`);
}

if (errors.size) {
  console.error([...errors].join('\n'));
  process.exitCode = 1;
} else {
  console.log(`Checked ${pages.size} pages: metadata, local links and anchors, search, feeds, sitemap, and verified deck downloads. Base: ${siteConfig.base}`);
}
