import assert from 'node:assert/strict';
import { mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { createHash } from 'node:crypto';
import * as pagefind from 'pagefind';
import { load } from 'cheerio';
import { pagePath, localeIds } from '../site-model.mjs';
import { configuredLocales } from '../locale-model.mjs';
import { siteConfig, withBase } from '../site.config.mjs';
import { publishedPages } from './published-pages.mjs';
import vocabulary from '../search-terms.json' with { type: 'json' };
import { searchIndexPath, searchEnginePath, searchTermsFor } from '../src/lib/search-terms.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const defaultDist = path.join(website, 'dist');
const runtimeDirectory = path.join(website, 'vendor/pagefind/1.5.2');

async function htmlFiles(directory, relative = '') {
  const files = [];
  for (const entry of await readdir(path.join(directory, relative), { withFileTypes: true })) {
    const filename = path.posix.join(relative, entry.name);
    if (filename === 'pagefind') continue;
    if (entry.isDirectory()) files.push(...await htmlFiles(directory, filename));
    else if (entry.isFile() && filename.endsWith('.html')) files.push(filename);
  }
  return files.sort();
}

/** Stable page numbers and one verified upstream runtime make the index portable. */
export async function generatePagefind(dist = defaultDist) {
  const runtime = JSON.parse(await readFile(path.join(runtimeDirectory, 'runtime.json'), 'utf8'));
  assert.equal(runtime.schemaVersion, 1);
  try {
    const created = await pagefind.createIndex();
    assert.deepEqual(created.errors, []);
    // addDirectory follows filesystem order; each addition assigns persistent page numbers.
    for (const filename of await htmlFiles(dist)) {
      const added = await created.index.addHTMLFile({
        sourcePath: filename,
        url: '/' + filename.replace(/index\.html$/, ''),
        content: await readFile(path.join(dist, filename), 'utf8'),
      });
      assert.deepEqual(added.errors, [], filename);
    }
    await rm(path.join(dist, 'pagefind'), { recursive: true, force: true });
    const written = await created.index.writeFiles({ outputPath: path.join(dist, 'pagefind') });
    assert.deepEqual(written.errors, []);
    await canonicalizePagefindEntry(dist);
    const entry = JSON.parse(await readFile(path.join(dist, 'pagefind/pagefind-entry.json'), 'utf8'));
    assert.equal(entry.version, runtime.version, 'Review and update the pinned browser runtime when upgrading Pagefind');
    for (const language of new Set(Object.values(entry.languages).map(value => value.wasm || 'unknown'))) {
      assert.match(language, /^[a-z-]+$/);
      const filename = `wasm.${language}.pagefind`;
      const expected = runtime.files[filename];
      assert.ok(expected, `${filename}: add the official runtime for this language to the reviewed inventory`);
      const bytes = await readFile(path.join(runtimeDirectory, filename));
      assert.equal(bytes.length, expected.bytes, filename);
      assert.equal(createHash('sha256').update(bytes).digest('hex'), expected.sha256, filename);
      await writeFile(path.join(dist, 'pagefind', filename), bytes);
    }
  } finally {
    await pagefind.close();
  }
}

/** Pagefind serializes its language map in random order; preserve its data with stable bytes. */
export async function canonicalizePagefindEntry(dist = defaultDist) {
  const filename = path.join(dist, 'pagefind/pagefind-entry.json');
  const entry = JSON.parse(await readFile(filename, 'utf8'));
  assert.ok(entry.languages && typeof entry.languages === 'object' && !Array.isArray(entry.languages), 'Pagefind language map required');
  entry.languages = Object.fromEntries(Object.keys(entry.languages).sort().map(locale => [locale, entry.languages[locale]]));
  await writeFile(filename, JSON.stringify(entry) + '\n');
}

/** A small, current-language metadata directory; Pagefind still owns body indexing. */
export async function generateSearchDocuments(dist = defaultDist) {
  const concepts = JSON.parse(await readFile(path.join(website, 'search-concepts.json'), 'utf8'));
  assert.equal(concepts.schemaVersion, 1);
  const topics = await publishedPages();
  const outputs = [];
  for (const locale of localeIds) {
    assert.ok(concepts.locales[locale], `${locale}: missing search concepts`);
    const documents = [];
    for (const page of topics) {
      const route = pagePath(page.id, locale);
      const $ = load(await readFile(path.join(dist, route.slice(1), 'index.html'), 'utf8'));
      const canonical = new URL(withBase(route, siteConfig.base), siteConfig.site).href;
      assert.equal($('html').attr('lang'), locale, `${route}: search language mismatch`);
      assert.equal($('link[rel="canonical"]').attr('href'), canonical, `${route}: search canonical mismatch`);
      const expectedTerms = searchTermsFor(page, locale);
      assert.equal($('meta[name="anki-forge:search-terms"]').attr('content') || '', expectedTerms, `${route}: search vocabulary differs from rendered HTML`);
      assert.equal($('h1').length, 1, `${route}: search title is not unique`);
      const title = $('h1').text().trim();
      const description = $('meta[name="description"]').attr('content');
      assert.ok(title && description, `${route}: missing search metadata`);
      documents.push({ id: page.id, type: page.type, url: new URL(canonical).pathname, title, pageTitle: $('title').text().trim(), description, terms: vocabulary.locales[locale][page.id] || [] });
    }
    outputs.push({ filename: path.join(dist, searchIndexPath(locale).slice(1)), bytes: JSON.stringify({ schemaVersion: 1, locale, documents, concepts: concepts.locales[locale] }) + '\n' });
  }
  // The UI and acceptance runner load these exact same engine bytes.
  outputs.push({ filename: path.join(dist, searchEnginePath.slice(1)), bytes: await readFile(path.join(website, 'src/lib/search.mjs')) });
  await mkdir(path.join(dist, 'generated'), { recursive: true });
  const owned = new Set(Object.keys(configuredLocales).map(locale => path.basename(searchIndexPath(locale))));
  for (const name of await readdir(path.join(dist, 'generated'))) if (owned.has(name) && !outputs.some(output => path.basename(output.filename) === name)) await rm(path.join(dist, 'generated', name));
  for (const output of outputs) await writeFile(output.filename, output.bytes);
  await generatePagefind(dist);
  console.log(`Generated ${localeIds.length} current-language search directories and the shared search engine from actual HTML.`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await generateSearchDocuments(process.argv[2] ? path.resolve(process.argv[2]) : defaultDist);
