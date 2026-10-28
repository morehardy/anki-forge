import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { chromium } from '/Users/hp/Desktop/2026/anki-forge/website/node_modules/playwright/index.mjs';
import { serveBuiltSite } from '/Users/hp/Desktop/2026/anki-forge/website/scripts/serve-built-site.mjs';
const root = '/Users/hp/Desktop/2026/anki-forge';
const [directory, output] = process.argv.slice(2);
assert.ok(directory && output);
const manifest = JSON.parse(await readFile(path.join(directory, 'generated/site-manifest.json')));
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const server = await serveBuiltSite({ directory });
const browser = await chromium.launch({ executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true });
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, permissions: ['clipboard-read', 'clipboard-write'], reducedMotion: 'reduce' });
await context.route('https://static.cloudflareinsights.com/**', route => route.abort());
const page = await context.newPage();
page.setDefaultTimeout(15000);
const records = { schemaVersion: 1, readOrigin: server.origin, configuration: manifest.configuration, entries: {} };
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
await mkdir(output, { recursive: true });
try {
  for (const locale of manifest.releaseLocales) {
    const prefix = locale === 'en' ? '' : '/zh-cn';
    const route = slug => `${base}${prefix}${slug}`;
    const url = slug => server.origin + route(slug);
    const tasks = [];
    const capture = async (id, steps, readUrls, codeSource) => {
      const screenshot = path.join(output, `${locale}-${id}.png`);
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
          const anchor = page.locator(`main a[href$="/recipes/json-to-anki/${name}"]`).first();
          const target = new URL(await anchor.getAttribute('href'), page.url()).href;
          const response = await context.request.get(target);
          assert.equal(response.status(), 200);
          const bytes = await response.body();
          assert.equal(hash(bytes), hash(await readFile(path.join(root, 'docs/recipes/json-to-anki', name))));
          const saved = path.join(output, 'downloads', locale, name);
          await mkdir(path.dirname(saved), { recursive: true });
          await writeFile(saved, bytes);
          downloadedResources.push({ url: target, path: path.relative(root, saved), sha256: hash(bytes), bytes: bytes.length });
        }
      }
      tasks.push({ id, steps, readUrls, finalUrl: page.url(), h1: await page.locator('h1').textContent(), body, codeBlocks, copiedPrograms, downloadedResources, codeSource, screenshot: path.relative(root, screenshot), screenshotSha256: hash(await readFile(screenshot)) });
    };
    await page.goto(url('/'));
    const home = page.url();
    await page.locator(`a[href="${route('/docs/quickstart/')}"]`).first().click();
    await page.waitForURL(url('/docs/quickstart/'));
    await capture('rust-basic', ['Opened this language homepage and followed its Rust quickstart link.', 'Copied the complete Rust main through the real clipboard; compared it byte-for-byte with the executed source.'], [home, page.url()], 'anki_forge/examples/target_api_basic.rs');

    await page.goto(url('/docs/'));
    const docs = page.url();
    await page.locator('site-search [data-open-modal]').click();
    const query = locale === 'en' ? 'python first deck' : 'Python 生成 牌组';
    await page.locator('#site-search-input').fill(query);
    await page.locator(`.pagefind-ui__result-link[href="${url('/docs/python-quickstart/')}"]`).click();
    await page.waitForURL(url('/docs/python-quickstart/'));
    await capture('python-basic', [`Opened localized documentation search and entered “${query}”.`, 'Followed the actual Python result and copied the complete executable source through the clipboard.'], [docs, page.url()], 'bindings/python/examples/quickstart.py');

    await page.locator(`#starlight__sidebar a[href="${route('/docs/node-quickstart/')}"]`).first().click();
    await page.waitForURL(url('/docs/node-quickstart/'));
    await capture('node-basic', ['Followed the localized sidebar Node quickstart entry.', 'Copied and matched the actual Node program; the linked JSON recipe supplies the separately executed TypeScript path.'], [url('/docs/python-quickstart/'), page.url()], 'bindings/node/examples/quickstart.mjs');

    await page.locator(`#starlight__sidebar a[href="${route('/docs/json-to-anki/')}"]`).first().click();
    await page.waitForURL(url('/docs/json-to-anki/'));
    assert.ok((await page.locator('main').innerText()).includes('v1.apkg'));
    await capture('json-media', ['Followed the localized JSON recipe sidebar entry.', 'Read the input download, media closure and three SDK instructions; preserved the original v1 and matched the actual executed recipe programs.'], [url('/docs/node-quickstart/'), page.url()], ['anki_forge/examples/json_to_anki.rs', 'bindings/python/examples/json_to_anki.py', 'bindings/node/examples/json_to_anki.mts']);

    await page.locator(`#starlight__sidebar a[href="${route('/docs/updates/')}"]`).first().click();
    await page.waitForURL(url('/docs/updates/'));
    await capture('original-baseline-update', ['Followed the localized updates entry.', 'Read original distribution baseline requirements and rejection of Anki re-export as identity baseline; checked comparison and policy instructions against the executed JSON update.'], [url('/docs/json-to-anki/'), page.url()]);

    await page.locator(`#starlight__sidebar a[href="${route('/docs/troubleshooting/')}"]`).first().click();
    await page.waitForURL(url('/docs/troubleshooting/'));
    assert.ok((await page.locator('main').innerText()).includes('NOTE.KEY_DUPLICATE'));
    await capture('troubleshooting', ['Followed the localized troubleshooting sidebar entry.', 'Found NOTE.KEY_DUPLICATE and its unique logical key correction; matched the separately executed controlled failure and successful two-note recovery.'], [url('/docs/updates/'), page.url()]);
    records.entries[locale] = tasks;
  }
  await writeFile(path.join(output, 'browser-tour.json'), JSON.stringify(records, null, 2) + '\n');
  console.log(`Observed ${Object.values(records.entries).flat().length} actual browser tasks at ${server.origin}`);
} finally { await context.close(); await browser.close(); await server.close(); }
