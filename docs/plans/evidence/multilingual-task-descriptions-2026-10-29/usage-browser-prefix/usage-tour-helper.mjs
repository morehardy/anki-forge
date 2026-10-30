// Independent usage-review browser tour (reviewer-authored; not part of the product).
// Serves one frozen artifact with the repository's static server, drives real Google Chrome with
// clipboard permission, and records what was actually seen, copied, fetched and followed. It records
// observations only; pass/fail judgement is made by the reviewer from this record.
//
// Round: task-descriptions (commit 89e4ae7). Compared with the previous round's helper this one also
// follows the feature pages' links into the Python and Node.js API sections (cards, Image Occlusion,
// updates, Node TypeScript), reads the media page's batch-import section, records each page's meta
// description and opening paragraph, runs a second localized search query, and sweeps every in-main
// fragment link of each localized page for a resolvable target id in the same locale.
//
// Usage (environment must match the artifact's configuration; serve-built-site reads it at import):
//   SITE_URL=... SITE_BASE=... SITE_CUSTOM_DOMAIN=... node usage-tour-helper.mjs <artifactDir> <archivedManifest> <outputDir>
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { chromium } from '/Users/hp/Desktop/2026/anki-forge/website/node_modules/playwright/index.mjs';
import { load } from '/Users/hp/Desktop/2026/anki-forge/website/node_modules/cheerio/dist/esm/index.js';
import { serveBuiltSite } from '/Users/hp/Desktop/2026/anki-forge/website/scripts/serve-built-site.mjs';
import { siteConfig } from '/Users/hp/Desktop/2026/anki-forge/website/site.config.mjs';

const root = '/Users/hp/Desktop/2026/anki-forge';
const [directory, archivedManifest, output] = process.argv.slice(2);
assert.ok(directory && archivedManifest && output, 'usage: <artifactDir> <archivedManifest> <outputDir>');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const rel = file => path.relative(root, file);
const archivedBytes = await readFile(archivedManifest);
const embeddedBytes = await readFile(path.join(directory, 'generated/site-manifest.json'));
assert.equal(hash(embeddedBytes), hash(archivedBytes), 'artifact manifest differs from the archived manifest');
const manifest = JSON.parse(archivedBytes);
assert.deepEqual(siteConfig, manifest.configuration, 'server environment does not match the artifact configuration');
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const records = new Map([...manifest.pages, ...manifest.resources].map(entry => [entry.route, entry]));
const pageOwners = new Map(manifest.pages.filter(page => page.id !== '404').map(page => [page.route, page]));
const quickstart = JSON.parse(await readFile(path.join(root, 'docs/ai-quickstart.json'), 'utf8'));

const server = await serveBuiltSite({ directory });
const browser = await chromium.launch({ executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true });
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, permissions: ['clipboard-read', 'clipboard-write'], reducedMotion: 'reduce' });
const blockedExternal = [];
await context.route('https://static.cloudflareinsights.com/**', route => { blockedExternal.push(route.request().url()); return route.abort(); });
const page = await context.newPage();
page.setDefaultTimeout(20000);
let requestLog = [];
const consoleErrors = [];
page.on('response', response => requestLog.push({ url: response.url(), status: response.status() }));
page.on('requestfailed', request => requestLog.push({ url: request.url(), failed: request.failure()?.errorText }));
page.on('console', message => { if (message.type() === 'error') consoleErrors.push({ url: page.url(), text: message.text() }); });
page.on('pageerror', error => consoleErrors.push({ url: page.url(), text: String(error) }));

const tour = { schemaVersion: 1, helper: 'usage-tour-helper.mjs', startedAt: new Date().toISOString(), browser: { executable: 'Google Chrome (headless)', version: browser.version() }, readOrigin: server.origin, artifactDirectory: directory, archivedManifest: rel(archivedManifest), archivedManifestSha256: hash(archivedBytes), embeddedManifestSha256: hash(embeddedBytes), commit: manifest.commit, inputSha256: manifest.inputSha256, configuration: manifest.configuration, releaseLocales: manifest.releaseLocales, entries: {}, fragmentSweep: {} };
await mkdir(output, { recursive: true });

const routeOf = href => new URL(href, server.origin).pathname;
async function served(urlString) {
  const response = await context.request.get(urlString);
  const bytes = await response.body();
  const route = routeOf(urlString);
  return { url: urlString, route, status: response.status(), contentType: response.headers()['content-type'], bytes: bytes.length, sha256: hash(bytes), manifestSha256: records.get(route)?.sha256 ?? null, matchesManifest: records.get(route)?.sha256 === hash(bytes), body: bytes };
}

async function observePage(locale) {
  const url = page.url();
  const route = routeOf(url);
  const servedPage = await served(url);
  const owner = pageOwners.get(route);
  const info = await page.evaluate(() => ({
    title: document.title,
    lang: document.documentElement.lang,
    h1: document.querySelector('main h1, h1')?.textContent.trim(),
    description: document.querySelector('meta[name="description"]')?.content,
    // First visible paragraph (skips empty section-alias paragraphs).
    opening: [...document.querySelectorAll('main .sl-markdown-content > p, main p')].map(node => node.innerText.trim()).find(Boolean),
    canonical: document.querySelector('link[rel="canonical"]')?.href,
    alternates: [...document.querySelectorAll('link[rel="alternate"][hreflang]')].map(link => ({ hreflang: link.hreflang, href: link.href })),
    languageSwitch: [...document.querySelectorAll('[data-language-switch]')].map(link => link.getAttribute('href')),
    sidebarCurrent: document.querySelector('#starlight__sidebar [aria-current="page"]')?.textContent.trim(),
    links: [...document.querySelectorAll('a[href]')].map(link => ({ href: link.getAttribute('href'), text: link.textContent.trim().slice(0, 80), languageSwitch: link.hasAttribute('data-language-switch'), region: link.closest('main') ? 'main' : link.closest('#starlight__sidebar') ? 'sidebar' : link.closest('header') ? 'header' : link.closest('footer') ? 'footer' : 'other' })),
  }));
  // A link is a locale fallback when it points at another locale's page, other than the explicit language switch.
  const foreign = [];
  for (const link of info.links) {
    let target;
    try { target = new URL(link.href, url); } catch { continue; }
    if (![server.origin, manifest.configuration.site].includes(target.origin)) continue;
    const targetPage = pageOwners.get(target.pathname) ?? pageOwners.get(target.pathname.replace(/index\.md$/, ''));
    if (targetPage && targetPage.locale !== locale && !link.languageSwitch) foreign.push({ ...link, targetLocale: targetPage.locale });
  }
  return { url, route, pageId: owner?.id, manifestLocale: owner?.locale, htmlLang: info.lang, title: info.title, h1: info.h1, description: info.description, opening: info.opening, canonical: info.canonical, alternates: info.alternates, languageSwitch: info.languageSwitch, sidebarCurrent: info.sidebarCurrent, servedStatus: servedPage.status, servedSha256: servedPage.sha256, manifestSha256: servedPage.manifestSha256, servedMatchesManifest: servedPage.matchesManifest, linkCount: info.links.length, foreignLocaleLinks: foreign, mainText: await page.locator('main').innerText() };
}

async function blockText(block) {
  return (await block.evaluate(element => {
    const lines = [...element.querySelectorAll('.ec-line .code')];
    if (lines.length) return lines.map(line => line.textContent.replace(/\r?\n$/, '')).join('\n');
    return (element.querySelector('pre code') ?? element.querySelector('[data-copy-content]') ?? element).textContent;
  })).trim();
}

// Click the real copy control of the block whose rendered text satisfies `matches`, then read the system clipboard.
async function copyBlock(matches, label) {
  const blocks = page.locator('main [data-copy-block], main .expressive-code');
  for (let index = 0; index < await blocks.count(); index++) {
    const block = blocks.nth(index);
    const rendered = await blockText(block);
    if (!matches(rendered)) continue;
    await block.scrollIntoViewIfNeeded();
    await page.evaluate(() => navigator.clipboard.writeText('__reviewer-clipboard-sentinel__'));
    const control = block.locator('[data-copy], .copy button').first();
    await control.waitFor({ state: 'visible' });
    await page.waitForFunction(element => !element.disabled, await control.elementHandle());
    await control.click();
    await page.waitForFunction(() => navigator.clipboard.readText().then(text => text !== '__reviewer-clipboard-sentinel__'));
    const clipboard = await page.evaluate(() => navigator.clipboard.readText());
    const status = await block.evaluate(element => element.querySelector('[data-copy-status]')?.textContent.trim() || element.querySelector('.copy button')?.dataset.copied || null);
    return { label, component: await block.evaluate(element => element.classList.contains('expressive-code') ? 'Expressive Code block (site script adds the shared data-copy-block contract)' : 'site CodeBlock component (data-copy-block in served HTML)'), controlLabel: await control.evaluate(element => element.getAttribute('title') || element.textContent.trim()), statusAfterCopy: status, rendered, clipboard, clipboardSha256: hash(clipboard) };
  }
  return { label, notFound: true };
}

async function copyProgram(source, label = source) {
  const raw = await readFile(path.join(root, source), 'utf8');
  const expected = raw.trim();
  const copy = await copyBlock(rendered => rendered === expected, label);
  if (copy.notFound) return { source, sourceSha256: hash(raw), ...copy };
  return { source, sourceSha256: hash(raw), sourceBytes: Buffer.byteLength(raw), ...copy, clipboardEqualsSourceExactly: copy.clipboard === raw, clipboardEqualsSourceTrimmed: copy.clipboard.trim() === expected, difference: copy.clipboard === raw ? 'none' : copy.clipboard.trim() === expected ? 'surrounding whitespace only' : 'content differs' };
}

async function copyFragment(source, marker, label) {
  const raw = await readFile(path.join(root, source), 'utf8');
  const start = raw.indexOf(`// docs:${marker}:start`);
  const end = raw.indexOf(`// docs:${marker}:end`);
  assert.ok(start >= 0 && end > start, `missing ${marker} markers in ${source}`);
  const expected = raw.slice(raw.indexOf('\n', start) + 1, end).trim();
  const copy = await copyBlock(rendered => rendered === expected, label);
  return { source, fragment: marker, sourceSha256: hash(raw), ...copy, clipboardEqualsFragmentTrimmed: copy.clipboard?.trim() === expected };
}

async function copyCommand(required, label) {
  const copy = await copyBlock(rendered => rendered.split('\n').some(line => line.trim() === required), label);
  return { required, ...copy, clipboardContainsRequiredLine: !!copy.clipboard?.split('\n').some(line => line.trim() === required) };
}

async function screenshot(locale, id, locator) {
  if (locator) await locator.first().scrollIntoViewIfNeeded();
  const file = path.join(output, `${locale}-${id}.png`);
  await page.screenshot({ path: file });
  return { path: rel(file), sha256: hash(await readFile(file)) };
}

async function clickAndWait(locator, target) {
  await locator.first().click();
  await page.waitForURL(target);
  await page.waitForLoadState('load');
}

// Read the section that the current URL fragment lands on, as the browser rendered it after navigation.
async function landedSection() {
  return page.evaluate(() => {
    const id = decodeURIComponent(location.hash.slice(1));
    const target = id ? document.getElementById(id) : null;
    if (!target) return { fragment: id, found: false };
    const headingOf = node => node?.matches?.('.sl-heading-wrapper') ? node.querySelector('h1,h2,h3,h4') : node?.matches?.('h1,h2,h3,h4') ? node : null;
    let heading = headingOf(target);
    let anchorBlock = target.closest('.sl-heading-wrapper') ?? target;
    if (!heading) {
      anchorBlock = target.closest('p') ?? target;
      let next = anchorBlock.nextElementSibling;
      while (next && !headingOf(next)) next = next.nextElementSibling;
      heading = headingOf(next);
      anchorBlock = next ?? anchorBlock;
    }
    const level = heading ? Number(heading.tagName[1]) : 2;
    const parts = [];
    for (let sibling = (heading?.closest('.sl-heading-wrapper') ?? heading)?.nextElementSibling; sibling; sibling = sibling.nextElementSibling) {
      const nested = headingOf(sibling);
      if (nested && Number(nested.tagName[1]) <= level) break;
      if (sibling.querySelector?.('[data-section-alias]') && !sibling.innerText.trim()) continue;
      parts.push(sibling.innerText.trim());
    }
    const headerBottom = document.querySelector('header')?.getBoundingClientRect().bottom ?? 0;
    const rect = heading?.getBoundingClientRect();
    return { fragment: id, found: true, targetTag: target.tagName.toLowerCase(), isSectionAlias: target.hasAttribute('data-section-alias'), headingId: heading?.id ?? null, headingLevel: heading ? level : null, headingText: heading?.textContent.trim() ?? null, scrollY: Math.round(scrollY), headingTop: rect ? Math.round(rect.top) : null, headerBottom: Math.round(headerBottom), headingVisibleBelowHeader: !!rect && rect.top >= headerBottom - 1 && rect.bottom <= innerHeight, sectionText: parts.filter(Boolean).join('\n').slice(0, 6000) };
  });
}

// Follow an in-page link into another page's section, record the landing, then optionally return with the browser's Back.
async function followSection(locale, locator, label, { back = false, shot } = {}) {
  const link = locator.first();
  await link.waitFor({ state: 'attached' });
  const from = page.url();
  const href = await link.getAttribute('href');
  const linkText = (await link.textContent()).trim();
  const context = await link.evaluate(element => (element.closest('p, li, td') ?? element).innerText.trim().slice(0, 600));
  const target = new URL(href, from);
  await link.scrollIntoViewIfNeeded();
  await link.click();
  await page.waitForURL(url => url.pathname === target.pathname && url.hash === target.hash);
  await page.waitForLoadState('load');
  await page.waitForTimeout(400);
  const landed = await landedSection();
  const observed = await observePage(locale);
  const { mainText, ...pageRecord } = observed;
  const record = { label, from, href, linkText, linkContext: context, landedUrl: page.url(), landed, page: pageRecord, screenshot: shot ? await screenshot(locale, shot) : undefined };
  if (back) { await page.goBack(); await page.waitForURL(from); await page.waitForLoadState('load'); record.returnedTo = page.url(); }
  return record;
}

// Read a section of the current page by its heading text.
async function sectionByHeading(pattern) {
  return page.evaluate(source => {
    const expression = new RegExp(source);
    const heading = [...document.querySelectorAll('main h2, main h3')].find(element => expression.test(element.textContent));
    if (!heading) return { found: false, pattern: source };
    const level = Number(heading.tagName[1]);
    const parts = [];
    for (let sibling = (heading.closest('.sl-heading-wrapper') ?? heading).nextElementSibling; sibling; sibling = sibling.nextElementSibling) {
      const nested = sibling.matches('.sl-heading-wrapper') ? sibling.querySelector('h2,h3,h4') : sibling.matches('h2,h3,h4') ? sibling : null;
      if (nested && Number(nested.tagName[1]) <= level) break;
      parts.push(sibling.innerText.trim());
    }
    return { found: true, pattern: source, headingId: heading.id, headingText: heading.textContent.trim(), sectionText: parts.filter(Boolean).join('\n').slice(0, 6000) };
  }, pattern.source);
}

async function runSearch(query) {
  const opener = page.locator('site-search [data-open-modal]').first();
  await page.waitForFunction(element => !element.disabled, await opener.elementHandle());
  await opener.click();
  await page.locator('#site-search-input').fill(query);
  await page.locator('#site-search-results .pagefind-ui__result-link').first().waitFor();
  await page.waitForTimeout(700);
  const results = await page.locator('#site-search-results .pagefind-ui__result-link').evaluateAll(links => links.map(link => ({ title: link.textContent.trim(), href: link.getAttribute('href') })));
  const excerpts = await page.locator('#site-search-results .pagefind-ui__result-excerpt').evaluateAll(nodes => nodes.slice(0, 3).map(node => node.textContent.trim()));
  return { query, results, excerpts };
}

// Every in-main link with a fragment, on every page of this locale, must land on an element id of a same-locale page.
async function fragmentSweep(locale) {
  const cache = new Map();
  const htmlOf = async route => { if (!cache.has(route)) cache.set(route, load((await served(server.origin + route)).body.toString('utf8'))); return cache.get(route); };
  const out = [];
  for (const owner of manifest.pages.filter(entry => entry.locale === locale && entry.id !== '404')) {
    const $ = await htmlOf(owner.route);
    $('main a[href*="#"]').each((_, element) => out.push({ fromRoute: owner.route, href: $(element).attr('href'), text: $(element).text().trim().slice(0, 80) }));
  }
  const results = [];
  for (const link of out) {
    const target = new URL(link.href, server.origin + link.fromRoute);
    if (target.origin !== server.origin && target.origin !== manifest.configuration.site) continue;
    if (!target.hash || target.hash === '#' || target.hash === '#_top') continue;
    const targetPage = pageOwners.get(target.pathname);
    if (!targetPage) { results.push({ ...link, targetRoute: target.pathname, resolved: false, problem: 'target is not a page' }); continue; }
    const $ = await htmlOf(target.pathname);
    const id = decodeURIComponent(target.hash.slice(1));
    const element = $('[id]').filter((_, node) => $(node).attr('id') === id).first();
    let heading = null;
    if (element.length) {
      if (element.is('h1,h2,h3,h4')) heading = element.text().trim();
      else if (element.is('[data-section-alias]')) heading = element.closest('p').nextAll('.sl-heading-wrapper').first().find('h2,h3,h4').first().text().trim() || null;
    }
    results.push({ ...link, targetRoute: target.pathname, fragment: id, targetLocale: targetPage.locale, sameLocale: targetPage.locale === locale, samePage: target.pathname === link.fromRoute, resolved: element.length > 0, targetKind: element.length ? (element.is('[data-section-alias]') ? 'section-alias' : element.get(0).tagName) : null, heading });
  }
  const api = results.filter(result => /\/docs\/(?:python|node)-api\/$/.test(result.targetRoute) && !result.samePage);
  return { checked: results.length, unresolved: results.filter(result => !result.resolved), crossLocale: results.filter(result => !result.sameLocale), apiSectionLinks: api };
}

try {
  for (const locale of manifest.releaseLocales) {
    const prefix = locale === 'en' ? '' : '/zh-cn';
    const route = slug => `${base}${prefix}${slug}`;
    const url = slug => server.origin + route(slug);
    const escaped = slug => new RegExp(route(slug).replaceAll('/', '\\/'));
    const tasks = [];
    const begin = () => { requestLog = []; return []; };
    const finish = (id, actions, pages, extra) => {
      const sameOrigin = requestLog.filter(entry => entry.url.startsWith(server.origin));
      tasks.push({ id, actions, pages, ...extra, requests: { sameOrigin: sameOrigin.map(entry => ({ path: entry.url.slice(server.origin.length), status: entry.status ?? entry.failed })), external: requestLog.filter(entry => !entry.url.startsWith(server.origin)).map(entry => entry.url) } });
    };
    const sidebar = slug => page.locator(`#starlight__sidebar a[href="${route(slug)}"]`);

    // rust-basic: localized home -> its Rust quickstart link.
    let actions = begin();
    await page.goto(url('/'));
    const home = await observePage(locale);
    actions.push(`Opened ${home.url} (lang=${home.htmlLang}, h1="${home.h1}").`);
    const rustLink = page.locator(`main a[href="${route('/docs/quickstart/')}"]`);
    actions.push(`Clicked the home-page link "${(await rustLink.first().textContent()).trim()}" -> ${route('/docs/quickstart/')}.`);
    await clickAndWait(rustLink, url('/docs/quickstart/'));
    const rustPage = await observePage(locale);
    const rustInstall = await copyCommand(quickstart.languages.rust.install, 'Rust install commands');
    const rustRun = await copyCommand('cargo run', 'Rust run command');
    const rustProgram = await copyProgram('anki_forge/examples/target_api_basic.rs');
    finish('rust-basic', actions, [home, rustPage], { copies: [rustInstall, rustProgram, rustRun], screenshot: await screenshot(locale, 'rust-basic', page.locator('main [data-copy-block], main .expressive-code').nth(1)) });

    // python-basic: localized docs index -> real site search -> Python result -> cards page -> its Python API section link.
    actions = begin();
    const queries = locale === 'en' ? ['python deck', 'python flashcards'] : ['Python 牌组', 'Python 问答 生成牌组'];
    await page.goto(url('/docs/'));
    const secondary = await runSearch(queries[1]);
    secondary.pythonQuickstartRank = secondary.results.findIndex(result => routeOf(result.href) === route('/docs/python-quickstart/')) + 1;
    actions.push(`Observation-only search "${queries[1]}": ${secondary.results.length} results; Python quickstart rank ${secondary.pythonQuickstartRank}.`);
    await page.goto(url('/docs/'));
    const docsIndex = await observePage(locale);
    const primary = await runSearch(queries[0]);
    const searchShot = await screenshot(locale, 'python-basic-search');
    const pythonRank = primary.results.findIndex(result => routeOf(result.href) === route('/docs/python-quickstart/'));
    primary.pythonQuickstartRank = pythonRank + 1;
    actions.push(`Opened ${docsIndex.url}, opened the site search, typed "${queries[0]}"; ${primary.results.length} results; Python quickstart rank ${pythonRank + 1}.`);
    assert.ok(pythonRank >= 0, `${locale}: search did not offer the Python quickstart`);
    await clickAndWait(page.locator(`#site-search-results .pagefind-ui__result-link[href="${primary.results[pythonRank].href}"]`), escaped('/docs/python-quickstart/'));
    const pythonPage = await observePage(locale);
    const pythonInstall = await copyCommand(quickstart.languages.python.install, 'Python install commands');
    const pythonRun = await copyCommand('python main.py', 'Python run command');
    const pythonProgram = await copyProgram('bindings/python/examples/quickstart.py');
    const pythonShot = await screenshot(locale, 'python-basic', page.locator('main .expressive-code').nth(1));
    actions.push(`From ${page.url()} clicked the sidebar entry "${(await sidebar('/docs/cards/').first().textContent()).trim()}".`);
    await clickAndWait(sidebar('/docs/cards/'), url('/docs/cards/'));
    const cardsPage = await observePage(locale);
    const pythonCardsApi = await followSection(locale, page.locator(`main a[href^="${route('/docs/python-api/')}#"]`), 'cards page -> Python API section', { shot: 'python-basic-api' });
    actions.push(`On the cards page clicked in-text "${pythonCardsApi.linkText}" (${pythonCardsApi.href}); landed on heading "${pythonCardsApi.landed.headingText}".`);
    finish('python-basic', actions, [docsIndex, pythonPage, cardsPage, pythonCardsApi.page], { search: { primary: { ...primary, results: primary.results.slice(0, 10), screenshot: searchShot }, secondary: { ...secondary, results: secondary.results.slice(0, 10) } }, copies: [pythonInstall, pythonProgram, pythonRun], sectionLinks: [pythonCardsApi], screenshot: pythonShot });

    // node-basic: sidebar Node entry, copies, TypeScript section link, and the cards page's Node API section link.
    actions = begin();
    const nodeLink = sidebar('/docs/node-quickstart/');
    actions.push(`From ${page.url()} clicked sidebar entry "${(await nodeLink.first().textContent()).trim()}".`);
    await clickAndWait(nodeLink, url('/docs/node-quickstart/'));
    const nodePage = await observePage(locale);
    const nodeInstall = await copyCommand(quickstart.languages.node.install, 'Node install commands');
    const nodeRun = await copyCommand('node main.mjs', 'Node run command');
    const nodeProgram = await copyProgram('bindings/node/examples/quickstart.mjs');
    const nodeShot = await screenshot(locale, 'node-basic', page.locator('main .expressive-code').nth(1));
    const typescript = await followSection(locale, page.locator(`main a[href="${route('/docs/node-api/')}#typescript"]`), 'Node quickstart -> TypeScript section', { back: true });
    actions.push(`Clicked in-text "${typescript.linkText}" (${typescript.href}); landed on heading "${typescript.landed.headingText}"; returned with Back.`);
    await clickAndWait(sidebar('/docs/cards/'), url('/docs/cards/'));
    const nodeCardsApi = await followSection(locale, page.locator(`main a[href^="${route('/docs/node-api/')}#"]`), 'cards page -> Node API section', { shot: 'node-basic-api' });
    actions.push(`Opened the cards page from the sidebar and clicked in-text "${nodeCardsApi.linkText}" (${nodeCardsApi.href}); landed on heading "${nodeCardsApi.landed.headingText}".`);
    await clickAndWait(sidebar('/docs/node-quickstart/'), url('/docs/node-quickstart/'));
    actions.push('Returned to the Node quickstart from the sidebar.');
    finish('node-basic', actions, [nodePage, typescript.page, cardsPage, nodeCardsApi.page], { copies: [nodeInstall, nodeProgram, nodeRun], sectionLinks: [typescript, nodeCardsApi], screenshot: nodeShot });

    // json-media: the Node page's in-text link to the JSON recipe; fetch the four linked inputs; copy all programs;
    // then the recipe's media link, the media page's batch-import section and the Image Occlusion page's API section links.
    actions = begin();
    const jsonLink = page.locator(`main a[href^="${route('/docs/json-to-anki/')}"]`);
    actions.push(`From ${page.url()} clicked in-page link "${(await jsonLink.first().textContent()).trim()}" (${await jsonLink.first().getAttribute('href')}).`);
    await clickAndWait(jsonLink, escaped('/docs/json-to-anki/'));
    const jsonPage = await observePage(locale);
    const downloads = [];
    for (const name of ['v1.json', 'v2.json', 'cell.png', 'tone.wav']) {
      const anchor = page.locator(`main a[href$="/recipes/json-to-anki/${name}"]`).first();
      const href = await anchor.getAttribute('href');
      // The browser resolves the anchor and fetches it from the page (a media tab never fires `load` for WAV).
      const fetched = await anchor.evaluate(async element => {
        const response = await fetch(element.href, { cache: 'no-store' });
        const buffer = new Uint8Array(await response.arrayBuffer());
        let binary = '';
        for (let index = 0; index < buffer.length; index += 0x8000) binary += String.fromCharCode(...buffer.subarray(index, index + 0x8000));
        return { url: element.href, status: response.status, contentType: response.headers.get('content-type'), base64: btoa(binary) };
      });
      const bytes = Buffer.from(fetched.base64, 'base64');
      const saved = path.join(output, 'downloads', locale, name);
      await mkdir(path.dirname(saved), { recursive: true });
      await writeFile(saved, bytes);
      const fixture = await readFile(path.join(root, 'docs/recipes/json-to-anki', name));
      downloads.push({ name, anchorText: (await anchor.textContent()).trim(), href, url: fetched.url, status: fetched.status, contentType: fetched.contentType, bytes: bytes.length, sha256: hash(bytes), savedPath: rel(saved), fixture: `docs/recipes/json-to-anki/${name}`, fixtureSha256: hash(fixture), equalsFixture: hash(bytes) === hash(fixture), manifestSha256: records.get(routeOf(fetched.url))?.sha256, equalsManifest: records.get(routeOf(fetched.url))?.sha256 === hash(bytes) });
    }
    actions.push('Fetched each of the four linked recipe inputs from the page (browser-resolved anchor href) and saved the received bytes.');
    const jsonCopies = [
      await copyCommand('cargo add serde_json', 'Rust recipe install commands'),
      await copyProgram('anki_forge/examples/json_to_anki.rs'),
      await copyCommand('cargo run -- recipe output/rust', 'Rust recipe run command'),
      await copyCommand('python -m pip install ankiforge==0.3.1', 'Python recipe install commands'),
      await copyProgram('bindings/python/examples/json_to_anki.py'),
      await copyCommand('python main.py recipe output/python', 'Python recipe run command'),
      await copyCommand('npm install --save-dev typescript@6.0.3 @types/node@24.13.6', 'Node recipe install commands'),
      await copyProgram('bindings/node/examples/json_to_anki.mts'),
      await copyCommand('npx tsc main.mts --target ES2022 --module NodeNext --moduleResolution NodeNext --strict --outDir dist --types node', 'TypeScript compile and run commands'),
    ];
    const jsonShot = await screenshot(locale, 'json-media', page.locator(`main a[href$="/recipes/json-to-anki/v1.json"]`));
    const mediaLink = page.locator(`main a[href="${route('/docs/media/')}"]`);
    actions.push(`On the JSON recipe clicked in-text "${(await mediaLink.first().textContent()).trim()}" -> ${route('/docs/media/')}.`);
    await clickAndWait(mediaLink, url('/docs/media/'));
    const mediaPage = await observePage(locale);
    const batchSection = await sectionByHeading(locale === 'en' ? /^Batch media import$/ : /^批量媒体导入$/);
    const mediaShot = await screenshot(locale, 'json-media-batch', page.locator('main h2', { hasText: locale === 'en' ? 'Batch media import' : '批量媒体导入' }));
    actions.push(`Read the media page opening and its "${batchSection.headingText}" section.`);
    await clickAndWait(sidebar('/docs/image-occlusion/'), url('/docs/image-occlusion/'));
    const occlusionPage = await observePage(locale);
    const pythonMediaApi = await followSection(locale, page.locator(`main a[href^="${route('/docs/python-api/')}#"]`), 'Image Occlusion page -> Python API media section', { back: true, shot: 'json-media-python-api' });
    const nodeMediaApi = await followSection(locale, page.locator(`main a[href^="${route('/docs/node-api/')}#"]`), 'Image Occlusion page -> Node API media section', { shot: 'json-media-node-api' });
    actions.push(`Opened Image Occlusion from the sidebar; clicked "${pythonMediaApi.linkText}" (${pythonMediaApi.href}) -> "${pythonMediaApi.landed.headingText}", Back, then "${nodeMediaApi.linkText}" (${nodeMediaApi.href}) -> "${nodeMediaApi.landed.headingText}".`);
    await clickAndWait(sidebar('/docs/json-to-anki/'), url('/docs/json-to-anki/'));
    actions.push('Returned to the JSON recipe from the sidebar.');
    finish('json-media', actions, [nodePage, jsonPage, mediaPage, occlusionPage, pythonMediaApi.page, nodeMediaApi.page], { downloads, copies: jsonCopies, mediaBatchSection: batchSection, sectionLinks: [pythonMediaApi, nodeMediaApi], screenshot: jsonShot, extraScreenshots: [mediaShot] });

    // original-baseline-update: the JSON page's in-text link to update conditions; then the Python and Node API section links.
    actions = begin();
    const updatesLink = page.locator(`main a[href^="${route('/docs/updates/')}"]`);
    actions.push(`From ${page.url()} clicked in-page link "${(await updatesLink.first().textContent()).trim()}".`);
    await clickAndWait(updatesLink, escaped('/docs/updates/'));
    const updatesPage = await observePage(locale);
    const updateCopies = [await copyFragment('anki_forge/examples/docs_workflow.rs', 'updates', 'docs_workflow updates fragment'), await copyProgram('anki_forge/examples/docs_publication.rs')];
    const updatesShot = await screenshot(locale, 'original-baseline-update', page.locator('main p', { hasText: 'ankiforge-identity.json' }));
    const pythonPublish = await followSection(locale, page.locator(`main a[href="${route('/docs/python-api/')}#prepare-review-and-publish"]`), 'updates page -> Python prepare/review/publish section', { back: true, shot: 'original-baseline-update-python-api' });
    const nodePublish = await followSection(locale, page.locator(`main a[href="${route('/docs/node-api/')}#prepare-review-and-publish"]`), 'updates page -> Node prepare/review/publish section', { back: true, shot: 'original-baseline-update-node-api' });
    actions.push(`Clicked "${pythonPublish.linkText}" (${pythonPublish.href}) -> "${pythonPublish.landed.headingText}", Back; clicked "${nodePublish.linkText}" (${nodePublish.href}) -> "${nodePublish.landed.headingText}", Back.`);
    finish('original-baseline-update', actions, [jsonPage, updatesPage, pythonPublish.page, nodePublish.page], { copies: updateCopies, sectionLinks: [pythonPublish, nodePublish], screenshot: updatesShot });

    // troubleshooting: sidebar troubleshooting entry.
    actions = begin();
    const troubleLink = sidebar('/docs/troubleshooting/');
    actions.push(`From ${page.url()} clicked sidebar entry "${(await troubleLink.first().textContent()).trim()}".`);
    await clickAndWait(troubleLink, url('/docs/troubleshooting/'));
    const troublePage = await observePage(locale);
    const duplicateRow = await page.locator('main tr', { hasText: 'NOTE.KEY_DUPLICATE' }).first().innerText();
    finish('troubleshooting', actions, [updatesPage, troublePage], { duplicateKeyRow: duplicateRow, screenshot: await screenshot(locale, 'troubleshooting', page.locator('main tr', { hasText: 'NOTE.KEY_DUPLICATE' })) });

    tour.entries[locale] = tasks;
    tour.fragmentSweep[locale] = await fragmentSweep(locale);
  }

  // Integrity of every Markdown/llms resource bound by the report, as served by this exact server.
  tour.machineResources = [];
  for (const resource of manifest.resources.filter(resource => /\/(?:index\.md|llms\.txt)$/.test(resource.route))) {
    const result = await served(server.origin + resource.route);
    tour.machineResources.push({ route: resource.route, status: result.status, sha256: result.sha256, matchesManifest: result.matchesManifest });
  }
  tour.blockedExternal = [...new Set(blockedExternal)];
  tour.consoleErrors = consoleErrors;
  tour.finishedAt = new Date().toISOString();
  await writeFile(path.join(output, 'browser-tour.json'), JSON.stringify(tour, null, 2) + '\n');
  console.log(`Recorded ${Object.values(tour.entries).flat().length} task observations at ${server.origin}`);
} finally { await context.close(); await browser.close(); await server.close(); }
