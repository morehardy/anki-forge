import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import { siteConfig, withBase } from '../site.config.mjs';

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

for (const name of ['sitemap-index.xml', 'sitemap-0.xml', 'rss.xml']) {
  const $ = load(await readFile(path.join(dist, name), 'utf8'), { xml: true });
  const from = { relative: name, url: new URL(withBase(`/${name}`, siteConfig.base), siteConfig.site) };
  const selector = name === 'rss.xml' ? 'channel > link, item > link' : 'loc';
  for (const node of $(selector).toArray()) await checkURL($(node).text(), from);
}
await stat(path.join(dist, 'pagefind/pagefind.js'));
const manifest = JSON.parse(await readFile(path.join(dist, 'generated/showcase.json'), 'utf8'));
assert.equal(manifest.examples.length, 3);
for (const example of manifest.examples) {
  const bytes = await readFile(path.join(dist, 'generated', example.file));
  record(createHash('sha256').update(bytes).digest('hex') === example.sha256, `${example.file}: download hash does not match preview manifest`);
  record(bytes.length === example.bytes, `${example.file}: download size does not match preview manifest`);
  record(example.counts.notes === 1 && example.counts.cards === 1, `${example.file}: unexpected verified counts`);
}
record(manifest.update.notesPreserved === 1, 'Update example did not preserve note identity');
for (const file of [manifest.update.previous, manifest.update.next]) await stat(path.join(dist, 'generated', file));
if (siteConfig.customDomain) assert.equal((await readFile(path.join(dist, 'CNAME'), 'utf8')).trim(), siteConfig.customDomain);

if (errors.size) {
  console.error([...errors].join('\n'));
  process.exitCode = 1;
} else {
  console.log(`Checked ${pages.size} pages: metadata, local links and anchors, search, feeds, sitemap, and verified deck downloads. Base: ${siteConfig.base}`);
}
