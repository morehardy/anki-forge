import test from 'node:test';
import assert from 'node:assert/strict';
import { load } from 'cheerio';
import { buildStructuredData, serializeJsonLd } from '../src/lib/structured-data.mjs';

const context = {
  siteConfig: { site: 'https://example.org', base: '/project', customDomain: '' },
  repository: 'https://github.com/example/project',
  publicInstall: { publicVersion: '1.2.3' },
  homeDescription: 'A visible introduction.',
};

test('article graph relates the localized page, article and shared software', () => {
  const page = { id: 'blog:hello', type: 'article', locale: 'zh-CN', canonical: 'https://example.org/project/zh-cn/blog/hello/', title: '标题', description: '摘要', publishedAt: '2026-09-22T00:00:00.000Z', category: '教程' };
  const graph = buildStructuredData(page, context)['@graph'];
  const byType = Object.fromEntries(graph.map(node => [node['@type'], node]));
  assert.equal(byType.WebPage.mainEntity['@id'], `${page.canonical}#article`);
  assert.equal(byType.BlogPosting.mainEntityOfPage['@id'], `${page.canonical}#webpage`);
  assert.equal(byType.BlogPosting.about['@id'], 'https://example.org/project/#software');
  assert.equal(byType.BlogPosting.datePublished, page.publishedAt);
  assert.equal(byType.BlogPosting.articleSection, '教程');
  assert.equal(byType.BlogPosting.encoding.contentUrl, `${page.canonical}index.md`);
  assert.deepEqual(Object.keys(byType.SoftwareApplication).sort(), ['@id', '@type', 'name']);
});

test('third-language graphs use the exact build set and retain shared entity identity', () => {
  const buildLocales = ['en', 'zh-CN', 'es'];
  const page = { id: 'quickstart', type: 'docs', locale: 'es', canonical: 'https://example.org/project/es/docs/quickstart/', title: 'Primera baraja', description: 'Crea una baraja.' };
  const nodes = buildStructuredData(page, { ...context, buildLocales })['@graph'];
  assert.deepEqual(nodes.find(node => node['@type'] === 'WebSite').inLanguage, buildLocales);
  assert.equal(nodes.find(node => node['@type'] === 'SoftwareApplication')['@id'], 'https://example.org/project/#software');
  assert.equal(nodes.find(node => node['@type'] === 'TechArticle')['@id'], `${page.canonical}#article`);
  assert.throws(() => buildStructuredData(page, context), /outside the build/);
});

test('JSON-LD serialization cannot close its script element and round trips Unicode', () => {
  const original = { '@context': 'https://schema.org', '@graph': [{ headline: '中文 "x" </script><script>bad()</script>\u2028\u2029' }] };
  const encoded = serializeJsonLd(original);
  const $ = load(`<script type="application/ld+json">${encoded}</script>`);
  assert.equal($('script').length, 1);
  assert.deepEqual(JSON.parse($('script').text()), original);
  assert.ok(!encoded.includes('</script>'));
});
