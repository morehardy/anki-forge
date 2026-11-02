// Independent current-artifact observations. Produces no passing review conclusions.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
const [directory, output, repositoryRoot, mode = 'editorial-preview'] = process.argv.slice(2);
assert.ok(directory && output && repositoryRoot, 'Usage: helper artifact-directory output-directory repository-root editorial-preview|release-candidate');
assert.ok(['editorial-preview', 'release-candidate'].includes(mode));
const root = path.resolve(repositoryRoot);
const { chromium } = await import(pathToFileURL(path.join(root, 'website/node_modules/playwright/index.mjs')));

const manifestBytes = await readFile(path.join(directory, 'generated/site-manifest.json'));
const manifest = JSON.parse(manifestBytes);
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const hashBytes = bytes => createHash('sha256').update(bytes).digest('hex');
await mkdir(output, { recursive: true });
await writeFile(path.join(output, 'origin-manifest.json'), manifestBytes);
const artifactRecords = [...manifest.pages, ...manifest.resources];
for (const record of artifactRecords) {
  let relative = record.route.slice(base.length).replace(/^\//, '');
  if (!relative || relative.endsWith('/')) relative += 'index.html';
  assert.equal(hashBytes(await readFile(path.join(directory, relative))), record.sha256, `Artifact bytes changed: ${record.route}`);
}
await writeFile(path.join(output, 'artifact-integrity.json'), JSON.stringify({ schemaVersion: 1, observation: 'Compared every archived artifact page/resource byte digest with its actual manifest before browser observation.', manifestSha256: hashBytes(manifestBytes), pages: manifest.pages.length, resources: manifest.resources.length, matched: artifactRecords.length }, null, 2) + '\n');
const types = { '.html': 'text/html', '.md': 'text/markdown; charset=utf-8', '.txt': 'text/plain; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.png': 'image/png', '.wav': 'audio/wav', '.woff2': 'font/woff2' };
const httpServer = createServer(async (request, response) => {
  try {
    const requestUrl = new URL(request.url, 'http://localhost');
    assert.ok(requestUrl.pathname.startsWith(base + '/'));
    let relative = decodeURIComponent(requestUrl.pathname.slice(base.length));
    if (relative.endsWith('/')) relative += 'index.html';
    const file = path.resolve(directory, '.' + relative);
    assert.ok(file.startsWith(path.resolve(directory) + path.sep));
    const bytes = await readFile(file);
    response.writeHead(200, { 'Content-Type': types[path.extname(file)] ?? 'application/octet-stream' }); response.end(bytes);
  } catch {
    response.writeHead(404, { 'Content-Type': 'text/html' }); response.end(await readFile(path.join(directory, '404.html')));
  }
});
await new Promise((resolve, reject) => { httpServer.once('error', reject); httpServer.listen(0, '127.0.0.1', resolve); });
const server = { origin: `http://127.0.0.1:${httpServer.address().port}`, close: () => new Promise(resolve => httpServer.close(resolve)) };
const browser = await chromium.launch({ executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true });
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, permissions: ['clipboard-read', 'clipboard-write'], reducedMotion: 'reduce' });
await context.route('https://static.cloudflareinsights.com/**', route => route.abort());
const page = await context.newPage();
page.setDefaultTimeout(15000);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const records = { schemaVersion: 1, scope: 'actual-browser-observations-only', mode, manifestSha256: hash(manifestBytes), commit: manifest.commit, inputSha256: manifest.inputSha256, readOrigin: server.origin, configuration: manifest.configuration, releaseLocales: manifest.releaseLocales, helper: {path: path.relative(root,process.argv[1]),sha256:hashBytes(await readFile(process.argv[1]))}, entries: {}, pageErrors: [], loadedResources: [], responseReadErrors: [] };
const queries = { en: 'python first deck', 'zh-CN': 'Python 生成 牌组', es: 'crear tarjetas Python', 'pt-BR': 'primeiro baralho Python', ja: 'Python 最初のデッキ', de: 'erster Stapel Python' };
const pendingResponses=[];page.on('response',response=>{if(!response.url().startsWith(server.origin+'/'))return;pendingResponses.push((async()=>{const route=new URL(response.url()).pathname,record=artifactRecords.find(r=>r.route===route);if(!record)return;const bytes=await response.body();assert.equal(hashBytes(bytes),record.sha256,`Browser response differs from archived manifest: ${route}`);records.loadedResources.push({route,sha256:hashBytes(bytes),status:response.status(),contentType:response.headers()['content-type']});})().catch(error=>{records.responseReadErrors.push({url:response.url(),message:error.message});}));});
page.on('pageerror', error => records.pageErrors.push({url: page.url(), message: error.message}));
await mkdir(output, { recursive: true });
try {
  for (const locale of manifest.releaseLocales) {
    const homePage = manifest.pages.find(entry => entry.id === 'home' && entry.locale === locale);
    assert.ok(homePage?.route?.startsWith(base + '/') && homePage.route.endsWith('/'), `Missing localized home route: ${locale}`);
    assert.ok(queries[locale], `Add a real localized Python task query for ${locale}`);
    const route = slug => {
      assert.ok(slug.startsWith('/'));
      const actual = homePage.route + slug.slice(1);
      assert.ok(manifest.pages.some(entry => entry.route === actual && entry.locale === locale && entry.id !== '404'), `Unregistered task route: ${locale}/${actual}`);
      return actual;
    };
    const url = slug => server.origin + route(slug);
    const tasks = [];
    await page.evaluate(() => {}).catch(() => {});
    const capture = async (id, steps, readUrls, codeSource) => {
      assert.equal(await page.locator('html').getAttribute('lang'), locale, `Task page used a different locale: ${locale}/${id}`);
      const screenshot = path.join(output, `${locale}-${id}.png`);
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({ path: screenshot });
      const body = await page.locator('main').innerText();
      const codeBlocks = await page.locator('main pre code').evaluateAll(nodes => nodes.map(node => node.innerText));
      let copiedCode;
      const copiedPrograms = [];
      for (const program of codeSource ? (Array.isArray(codeSource) ? codeSource : [codeSource]) : []) {
        const expected = (await readFile(path.join(root, program), 'utf8')).trim();
        const blocks = page.locator('main [data-copy-block], main .expressive-code');
        for (let index = 0; index < await blocks.count(); index++) {
          const block = blocks.nth(index);
          const text = (await block.evaluate(element => {
            const lines = [...element.querySelectorAll('.ec-line .code')];
            return lines.length ? lines.map(line => line.textContent.replace(/\r?\n$/, '')).join('\n') : element.querySelector('pre code').textContent;
          })).trim();
          if (text !== expected) continue;
          await block.locator('[data-copy], .copy button').click();
          copiedCode = await page.evaluate(() => navigator.clipboard.readText());
          assert.equal(copiedCode.trim(), expected, `Real clipboard code differs: ${locale}/${id}`);
          break;
        }
        assert.ok(copiedCode, `Did not copy complete executable program: ${locale}/${id}/${program}`);
        copiedPrograms.push({ source: program, sourceSha256: hash(await readFile(path.join(root, program))), copiedCode, clipboardSha256: hash(copiedCode.trim()) });
        copiedCode = undefined;
      }
      const downloadedResources = [];
      if (id === 'json-media') {
        for (const name of ['v1.json', 'v2.json', 'cell.png', 'tone.wav']) {
          const links = await page.locator('main a[href]').evaluateAll(nodes => nodes.map(node => ({ url: node.href, text: node.textContent })));
          const anchor = links.find(link => decodeURIComponent(new URL(link.url).pathname).split('/').at(-1) === name);
          assert.ok(anchor, `Missing actual input link: ${locale}/${name}`);
          const target = anchor.url;
          assert.equal(new URL(target).origin, server.origin, 'Input link must stay on the actual artifact origin');
          assert.ok(manifest.resources.some(entry => entry.route === new URL(target).pathname), `Input link is absent from the frozen manifest: ${target}`);
          const response = await context.request.get(target);
          assert.equal(response.status(), 200);
          const bytes = await response.body();
          assert.equal(hash(bytes), hash(await readFile(path.join(root, 'docs/recipes/json-to-anki', name))));
          const saved = path.join(output, 'downloads', locale, name);
          await mkdir(path.dirname(saved), { recursive: true });
          await writeFile(saved, bytes);
          downloadedResources.push({ url: target, anchorText: anchor.text, httpStatus: response.status(), contentType: response.headers()['content-type'], path: path.relative(root, saved), sha256: hash(bytes), bytes: bytes.length });
        }
      }
      tasks.push({ id, steps, readUrls, finalUrl: page.url(), h1: await page.locator('h1').textContent(), body, codeBlocks, copiedPrograms, downloadedResources, codeSource, screenshot: path.relative(root, screenshot), screenshotSha256: hash(await readFile(screenshot)) });
    };
    await page.goto(url('/'));
    const home = page.url();
    await page.locator(`a[href="${route('/docs/quickstart/')}"]`).first().click();
    await page.waitForURL(url('/docs/quickstart/'));
    await capture('rust-basic', ['Opened this language homepage and followed its Rust quickstart link.', 'Copied the complete Rust main through the real clipboard; matched the current repository program after trimming terminal whitespace.'], [home, page.url()], 'anki_forge/examples/target_api_basic.rs');

    await page.goto(url('/docs/'));
    const docs = page.url();
    await page.locator('site-search [data-open-modal]').click();
    const query = queries[locale];
    await page.locator('#site-search-input').fill(query);
    await page.locator(`.pagefind-ui__result-link[href="${url('/docs/python-quickstart/')}"]`).click();
    await page.waitForURL(url('/docs/python-quickstart/'));
    await capture('python-basic', [`Opened localized documentation search and entered “${query}”.`, 'Followed the actual Python result and copied the complete executable source through the clipboard.'], [docs, page.url()], 'bindings/python/examples/quickstart.py');

    await page.locator(`#starlight__sidebar a[href="${route('/docs/node-quickstart/')}"]`).first().click();
    await page.waitForURL(url('/docs/node-quickstart/'));
    await capture('node-basic', ['Followed the localized sidebar Node quickstart entry.', 'Copied and matched the actual Node program; TypeScript instructions are captured separately in the JSON recipe task.'], [url('/docs/python-quickstart/'), page.url()], 'bindings/node/examples/quickstart.mjs');

    await page.locator(`#starlight__sidebar a[href="${route('/docs/json-to-anki/')}"]`).first().click();
    await page.waitForURL(url('/docs/json-to-anki/'));
    assert.ok((await page.locator('main').innerText()).includes('v1.apkg'));
    await capture('json-media', ['Followed the localized JSON recipe sidebar entry.', 'Read the four actual input anchors and media/three-SDK instructions; downloaded the exact files and copied all three complete repository programs through the real clipboard.'], [url('/docs/node-quickstart/'), page.url()], ['anki_forge/examples/json_to_anki.rs', 'bindings/python/examples/json_to_anki.py', 'bindings/node/examples/json_to_anki.mts']);

    await page.locator(`#starlight__sidebar a[href="${route('/docs/updates/')}"]`).first().click();
    await page.waitForURL(url('/docs/updates/'));
    await capture('original-baseline-update', ['Followed the localized updates entry.', 'Read original distribution baseline requirements, rejection of Anki re-export as identity baseline, comparison policy and separate-candidate publication instructions.'], [url('/docs/json-to-anki/'), page.url()]);

    await page.locator(`#starlight__sidebar a[href="${route('/docs/troubleshooting/')}"]`).first().click();
    await page.waitForURL(url('/docs/troubleshooting/'));
    assert.ok((await page.locator('main').innerText()).includes('NOTE.KEY_DUPLICATE'));
    await capture('troubleshooting', ['Followed the localized troubleshooting sidebar entry.', 'Found NOTE.KEY_DUPLICATE and the unique logical key correction in actual localized rendered instructions.'], [url('/docs/updates/'), page.url()]);
    records.entries[locale] = tasks;
    await writeFile(path.join(output, 'browser-tour.json'), JSON.stringify(records, null, 2) + '\n');
    console.log(`Captured ${locale}: ${tasks.length} tasks, live clipboard and input downloads.`);
  }
  await writeFile(path.join(output, 'browser-tour.json'), JSON.stringify(records, null, 2) + '\n');
  await Promise.all(pendingResponses);records.loadedResources=[...new Map(records.loadedResources.map(r=>[r.route,r])).values()];await writeFile(path.join(output,'browser-tour.json'),JSON.stringify(records,null,2)+'\n');
  console.log(`Observed ${Object.values(records.entries).flat().length} actual browser tasks at ${server.origin}; ${records.loadedResources.length} actual loaded resources bound.`);
} catch(error) { records.failure = { message: error.message, url: page.url() }; await writeFile(path.join(output, 'browser-tour.json'), JSON.stringify(records, null, 2) + '\n'); throw error; } finally { await context.close(); await browser.close(); await server.close(); }
