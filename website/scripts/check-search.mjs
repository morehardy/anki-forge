import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { chromium } from 'playwright';
import { localeIds, pagePath, resolvePage, pages } from '../site-model.mjs';
import { withBase, siteConfig } from '../site.config.mjs';
import { sha256 } from './site-manifest.mjs';
import { serveBuiltSite } from './serve-built-site.mjs';
import { searchOptions } from '../search.config.mjs';
import { searchIndexPath, searchEnginePath } from '../src/lib/search-terms.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const preview = process.argv.includes('--preview');
const queryBytes = await readFile(path.join(website, 'search-queries.json'));
const fixtures = JSON.parse(queryBytes);
assert.equal(fixtures.schemaVersion, 1);
const manifestBytes = await readFile(path.join(website, 'dist/generated/site-manifest.json')).catch(error => {
  if (preview && error.code === 'ENOENT') return null;
  throw error;
});
const configuration = siteConfig.base === '/' ? 'root' : 'prefix';
const output = process.env.RELEASE_EVIDENCE_DIR || path.resolve(website, `../target/website-localization/${configuration}`);
await mkdir(output, { recursive: true });
const server = await serveBuiltSite();
let browser;
const results = {};
try {
  browser = await chromium.launch({ executablePath: process.env.CHROME_EXECUTABLE || undefined, headless: true });
  for (const locale of localeIds) {
    const queries = fixtures.locales[locale];
    assert.ok(queries, `${locale}: missing independently authored search fixtures`);
    for (const [kind, count] of [['core', 12], ['terminology', 8], ['variant', 5], ['negative', 5]]) assert.equal(queries.filter(query => query.kind === kind).length, count, `${locale}: ${kind} query count`);
    for (const query of queries) {
      assert.equal(query.locale, locale);
      for (const id of query.expectedPageIds) assert.ok(pages.some(page => page.id === id), `${query.id}: unknown expected page ${id}`);
    }
    const page = await browser.newPage();
    await page.route('https://static.cloudflareinsights.com/**', request => request.abort());
    await page.goto(server.url(pagePath('quickstart', locale)));
    const observed = await page.evaluate(async ({ url, engineUrl, directoryUrl, queries, options, basePath, baseUrl, locale }) => {
      const [module, engine, directoryResponse] = await Promise.all([import(url), import(engineUrl), fetch(directoryUrl)]);
      if (!directoryResponse.ok) throw new Error('Current-language search directory is unavailable');
      const directory = await directoryResponse.json();
      if (directory.schemaVersion !== 1 || directory.locale !== locale) throw new Error('Current-language search directory has a mismatched locale');
      const index = module.createInstance({ ...options, basePath, baseUrl });
      await index.init();
      const observations = [];
      try {
        for (const query of queries) {
          const result = await index.search(query.query);
          const pagefindResults = await Promise.all(result.results.slice(0, 20).map(entry => entry.data()));
          const ranked = engine.rankSearchDocuments(query.query, directory.documents, locale, { pagefindResults, concepts: directory.concepts });
          const top3 = ranked.slice(0, 3).map(data => ({ url: data.url, title: data.title, excerpt: data.excerpt, score: data.score, matchedTerms: data.matchedTerms }));
          observations.push({ id: query.id, query: query.query, top3 });
        }
      } finally {
        await index.destroy();
      }
      return observations;
    }, { url: server.origin + withBase('/pagefind/pagefind.js', siteConfig.base), engineUrl: server.origin + withBase(searchEnginePath, siteConfig.base), directoryUrl: server.origin + withBase(searchIndexPath(locale), siteConfig.base), queries, options: searchOptions, basePath: withBase('/pagefind/', siteConfig.base), baseUrl: withBase('/', siteConfig.base), locale });
    results[locale] = observed.map((entry, index) => {
      const query = queries[index];
      const top3 = entry.top3.map(result => {
        const resolved = resolvePage(new URL(result.url, server.origin).pathname, siteConfig.base);
        assert.equal(resolved?.locale, locale, `${query.id}: search returned another language or an unregistered page`);
        return { ...result, pageId: resolved.page.id };
      });
      const hit = top3.some(result => query.expectedPageIds.includes(result.pageId));
      return { ...entry, kind: query.kind, intentId: query.intentId, expectedPageIds: query.expectedPageIds, top3, passed: query.kind === 'negative' ? top3.length === 0 || hit : hit, ...(query.kind === 'negative' ? { expectedBoundary: query.expectedBoundary, boundaryReviewRequired: top3.length > 0 } : {}) };
    });
    await page.close();
  }
} finally {
  await browser?.close();
  await server.close();
}
const summaries = Object.fromEntries(Object.entries(results).map(([locale, entries]) => [locale, {
  core: entries.filter(entry => entry.kind === 'core' && entry.passed).length,
  positive: entries.filter(entry => entry.kind !== 'negative' && entry.passed).length,
  negative: entries.filter(entry => entry.kind === 'negative' && entry.passed).length,
}]));
const passed = Object.values(summaries).every(summary => summary.core === 12 && summary.positive >= 23 && summary.negative === 5);
await writeFile(path.join(output, 'search-results.json'), JSON.stringify({ schemaVersion: 1, mode: preview ? 'editorial-preview' : 'release-candidate', status: passed ? 'passed' : 'failed', manifestSha256: manifestBytes ? sha256(manifestBytes) : null, querySetSha256: sha256(queryBytes), configuration: siteConfig, summaries, results }, null, 2) + '\n');
for (const [locale, summary] of Object.entries(summaries)) {
  console.log(`${locale}: core ${summary.core}/12, positives ${summary.positive}/25, negatives ${summary.negative}/5`);
  for (const entry of results[locale].filter(entry => !entry.passed)) console.log(`  FAIL ${entry.id}: ${entry.query} → ${entry.top3.map(result => result.pageId).join(', ') || 'no results'}`);
}
assert.ok(passed, 'Independent local-language search fixtures did not meet the release threshold; inspect search-results.json');
