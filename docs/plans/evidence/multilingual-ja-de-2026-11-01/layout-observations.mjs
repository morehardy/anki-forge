// Capture real candidate pages and selected fonts; no review verdict is generated.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, copyFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
const [directory, output, locale] = process.argv.slice(2), root = process.cwd();
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const mb = await readFile(path.join(directory, 'generated/site-manifest.json')), manifest = JSON.parse(mb);
assert.ok(manifest.releaseLocales.includes(locale));
process.env.SITE_URL = manifest.configuration.site;
process.env.SITE_BASE = manifest.configuration.base;
process.env.SITE_CUSTOM_DOMAIN = manifest.configuration.customDomain || '';
const { serveBuiltSite } = await import(pathToFileURL(path.join(root, 'website/scripts/serve-built-site.mjs')));
const { chromium } = await import(pathToFileURL(path.join(root, 'website/node_modules/playwright/index.mjs')));
await mkdir(output, { recursive: true });
await writeFile(path.join(output, 'origin-manifest.json'), mb);
await copyFile(path.join(directory, `generated/og-${locale}.png`), path.join(output, `og-${locale}.png`));
const server = await serveBuiltSite({ directory });
const browser = await chromium.launch({ executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true });
const page = await browser.newPage({ reducedMotion: 'reduce' });
await page.route('https://static.cloudflareinsights.com/**', request => request.abort());
const records = { scope: 'actual-layout-font-and-image-observations-only', locale, manifestSha256: sha(mb), configuration: manifest.configuration, observations: [], pageErrors: [] };
page.on('pageerror', error => records.pageErrors.push(error.message));
try {
  for (const id of ['home', 'python-quickstart']) for (const width of [320, 390, 768, 1440]) for (const scale of [1, 2]) {
    const entry = manifest.pages.find(p => p.id === id && p.locale === locale);
    await page.setViewportSize({ width, height: 900 });
    await page.goto(server.origin + entry.route);
    await page.evaluate(() => document.fonts.ready);
    if (scale === 2) await page.evaluate(() => { document.documentElement.style.fontSize = `${parseFloat(getComputedStyle(document.documentElement).fontSize) * 2}px`; });
    await page.locator('.site-header details.language-menu > summary').first().click();
    const geometry = await page.evaluate(() => ({ lang: document.documentElement.lang, viewport: innerWidth, scrollWidth: document.documentElement.scrollWidth, h1: document.querySelector('h1')?.textContent, family: getComputedStyle(document.querySelector('main')).fontFamily }));
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('DOM.enable');
    await cdp.send('CSS.enable');
    const { root: dom } = await cdp.send('DOM.getDocument');
    const { nodeId } = await cdp.send('DOM.querySelector', { nodeId: dom.nodeId, selector: 'main h1' });
    const fonts = await cdp.send('CSS.getPlatformFontsForNode', { nodeId });
    await cdp.detach();
    const filename = `${locale}-${id}-${width}-${scale * 100}.png`, file = path.join(output, filename);
    await page.screenshot({ path: file, animations: 'disabled' });
    records.observations.push({ id, width, scale, url: page.url(), route: entry.route, sha256: entry.sha256, geometry, fonts, screenshot: path.relative(root, file), screenshotSha256: sha(await readFile(file)) });
  }
  await writeFile(path.join(output, 'observations.json'), JSON.stringify(records, null, 2) + '\n');
  console.log(`Captured ${locale}: ${records.observations.length} actual layout/font observations.`);
} finally { await browser.close(); await server.close(); }
