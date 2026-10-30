// Independent usage-review browser tour (reviewer-authored; not part of the product).
// Serves one frozen artifact with the repository's static server, drives real Google Chrome with
// clipboard permission, and records what was actually seen, copied and fetched. It records
// observations only; pass/fail judgement is made by the reviewer from this record.
//
// Usage (environment must match the artifact's configuration; serve-built-site reads it at import):
//   SITE_URL=... SITE_BASE=... SITE_CUSTOM_DOMAIN=... node usage-tour-helper.mjs <artifactDir> <archivedManifest> <outputDir>
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { chromium } from '/Users/hp/Desktop/2026/anki-forge/website/node_modules/playwright/index.mjs';
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

const tour = { schemaVersion: 1, helper: 'usage-tour-helper.mjs', startedAt: new Date().toISOString(), browser: { executable: 'Google Chrome (headless)', version: browser.version() }, readOrigin: server.origin, artifactDirectory: directory, archivedManifest: rel(archivedManifest), archivedManifestSha256: hash(archivedBytes), embeddedManifestSha256: hash(embeddedBytes), commit: manifest.commit, inputSha256: manifest.inputSha256, configuration: manifest.configuration, releaseLocales: manifest.releaseLocales, entries: {} };
await mkdir(output, { recursive: true });

const routeOf = href => { const url = new URL(href, server.origin); return url.pathname; };
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
  return { url, route, pageId: owner?.id, manifestLocale: owner?.locale, htmlLang: info.lang, title: info.title, h1: info.h1, canonical: info.canonical, alternates: info.alternates, languageSwitch: info.languageSwitch, sidebarCurrent: info.sidebarCurrent, servedStatus: servedPage.status, servedSha256: servedPage.sha256, manifestSha256: servedPage.manifestSha256, servedMatchesManifest: servedPage.matchesManifest, linkCount: info.links.length, foreignLocaleLinks: foreign, mainText: await page.locator('main').innerText() };
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

try {
  for (const locale of manifest.releaseLocales) {
    const prefix = locale === 'en' ? '' : '/zh-cn';
    const route = slug => `${base}${prefix}${slug}`;
    const url = slug => server.origin + route(slug);
    const tasks = [];
    const begin = () => { requestLog = []; return []; };
    const finish = (id, actions, pages, extra) => {
      const sameOrigin = requestLog.filter(entry => entry.url.startsWith(server.origin));
      tasks.push({ id, actions, pages, ...extra, requests: { sameOrigin: sameOrigin.map(entry => ({ path: entry.url.slice(server.origin.length), status: entry.status ?? entry.failed })), external: requestLog.filter(entry => !entry.url.startsWith(server.origin)).map(entry => entry.url) } });
    };

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

    // python-basic: localized docs index -> real site search -> Python result.
    actions = begin();
    await page.goto(url('/docs/'));
    const docsIndex = await observePage(locale);
    const query = locale === 'en' ? 'python deck' : 'Python 牌组';
    const opener = page.locator('site-search [data-open-modal]').first();
    await page.waitForFunction(element => !element.disabled, await opener.elementHandle());
    await opener.click();
    await page.locator('#site-search-input').fill(query);
    await page.locator('#site-search-results .pagefind-ui__result-link').first().waitFor();
    await page.waitForTimeout(500);
    const results = await page.locator('#site-search-results .pagefind-ui__result-link').evaluateAll(links => links.map(link => ({ title: link.textContent.trim(), href: link.getAttribute('href') })));
    const searchShot = await screenshot(locale, 'python-basic-search');
    const pythonRank = results.findIndex(result => routeOf(result.href) === route('/docs/python-quickstart/'));
    actions.push(`Opened ${docsIndex.url}, opened the site search, typed "${query}"; ${results.length} results; Python quickstart rank ${pythonRank + 1}.`);
    assert.ok(pythonRank >= 0, `${locale}: search did not offer the Python quickstart`);
    await clickAndWait(page.locator(`#site-search-results .pagefind-ui__result-link[href="${results[pythonRank].href}"]`), new RegExp(`${route('/docs/python-quickstart/').replaceAll('/', '\\/')}`));
    const pythonPage = await observePage(locale);
    const pythonInstall = await copyCommand(quickstart.languages.python.install, 'Python install commands');
    const pythonRun = await copyCommand('python main.py', 'Python run command');
    const pythonProgram = await copyProgram('bindings/python/examples/quickstart.py');
    finish('python-basic', actions, [docsIndex, pythonPage], { search: { query, results: results.slice(0, 10), pythonQuickstartRank: pythonRank + 1, screenshot: searchShot }, copies: [pythonInstall, pythonProgram, pythonRun], screenshot: await screenshot(locale, 'python-basic', page.locator('main .expressive-code').nth(1)) });

    // node-basic: sidebar Node entry from the Python page.
    actions = begin();
    const nodeLink = page.locator(`#starlight__sidebar a[href="${route('/docs/node-quickstart/')}"]`);
    actions.push(`From ${page.url()} clicked sidebar entry "${(await nodeLink.first().textContent()).trim()}".`);
    await clickAndWait(nodeLink, url('/docs/node-quickstart/'));
    const nodePage = await observePage(locale);
    const nodeInstall = await copyCommand(quickstart.languages.node.install, 'Node install commands');
    const nodeRun = await copyCommand('node main.mjs', 'Node run command');
    const nodeProgram = await copyProgram('bindings/node/examples/quickstart.mjs');
    finish('node-basic', actions, [pythonPage, nodePage], { copies: [nodeInstall, nodeProgram, nodeRun], screenshot: await screenshot(locale, 'node-basic', page.locator('main .expressive-code').nth(1)) });

    // json-media: the Node page's in-text link to the JSON recipe; fetch the four linked inputs; copy all programs.
    actions = begin();
    const jsonLink = page.locator(`main a[href^="${route('/docs/json-to-anki/')}"]`);
    actions.push(`From ${page.url()} clicked in-page link "${(await jsonLink.first().textContent()).trim()}" (${await jsonLink.first().getAttribute('href')}).`);
    await clickAndWait(jsonLink, new RegExp(`${route('/docs/json-to-anki/').replaceAll('/', '\\/')}`));
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
      const target = fetched.url;
      const bytes = Buffer.from(fetched.base64, 'base64');
      const response = { status: () => fetched.status, headers: () => ({ 'content-type': fetched.contentType }) };
      const saved = path.join(output, 'downloads', locale, name);
      await mkdir(path.dirname(saved), { recursive: true });
      await writeFile(saved, bytes);
      const fixture = await readFile(path.join(root, 'docs/recipes/json-to-anki', name));
      downloads.push({ name, anchorText: (await anchor.textContent()).trim(), href, url: target, status: response.status(), contentType: response.headers()['content-type'], bytes: bytes.length, sha256: hash(bytes), savedPath: rel(saved), fixture: `docs/recipes/json-to-anki/${name}`, fixtureSha256: hash(fixture), equalsFixture: hash(bytes) === hash(fixture), manifestSha256: records.get(routeOf(target))?.sha256, equalsManifest: records.get(routeOf(target))?.sha256 === hash(bytes) });
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
    finish('json-media', actions, [nodePage, jsonPage], { downloads, copies: jsonCopies, screenshot: await screenshot(locale, 'json-media', page.locator(`main a[href$="/recipes/json-to-anki/v1.json"]`)) });

    // original-baseline-update: the JSON page's in-text link to update conditions.
    actions = begin();
    const updatesLink = page.locator(`main a[href^="${route('/docs/updates/')}"]`);
    actions.push(`From ${page.url()} clicked in-page link "${(await updatesLink.first().textContent()).trim()}".`);
    await clickAndWait(updatesLink, new RegExp(`${route('/docs/updates/').replaceAll('/', '\\/')}`));
    const updatesPage = await observePage(locale);
    const updateCopies = [await copyFragment('anki_forge/examples/docs_workflow.rs', 'updates', 'docs_workflow updates fragment'), await copyProgram('anki_forge/examples/docs_publication.rs')];
    finish('original-baseline-update', actions, [jsonPage, updatesPage], { copies: updateCopies, screenshot: await screenshot(locale, 'original-baseline-update', page.locator('main p', { hasText: 'ankiforge-identity.json' })) });

    // troubleshooting: sidebar troubleshooting entry.
    actions = begin();
    const troubleLink = page.locator(`#starlight__sidebar a[href="${route('/docs/troubleshooting/')}"]`);
    actions.push(`From ${page.url()} clicked sidebar entry "${(await troubleLink.first().textContent()).trim()}".`);
    await clickAndWait(troubleLink, url('/docs/troubleshooting/'));
    const troublePage = await observePage(locale);
    const duplicateRow = await page.locator('main tr', { hasText: 'NOTE.KEY_DUPLICATE' }).first().innerText();
    finish('troubleshooting', actions, [updatesPage, troublePage], { duplicateKeyRow: duplicateRow, screenshot: await screenshot(locale, 'troubleshooting', page.locator('main tr', { hasText: 'NOTE.KEY_DUPLICATE' })) });

    tour.entries[locale] = tasks;
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
