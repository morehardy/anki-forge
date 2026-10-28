import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import { localeIds, pagePath } from '../site-model.mjs';
import { repository, siteConfig, withBase } from '../site.config.mjs';
import { createTranslator } from '../src/i18n/messages.mjs';
import { publishedPages, frontmatter } from './published-pages.mjs';
import { loadAiQuickstart } from './ai-quickstart.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const root = path.resolve(website, '..');
const dist = process.argv[2] ? path.resolve(process.argv[2]) : path.join(website, 'dist');
const topics = await publishedPages();
const install = await loadAiQuickstart();
const siteRoot = new URL(withBase('/', siteConfig.base), siteConfig.site).href;
const absolute = route => new URL(withBase(route, siteConfig.base), siteConfig.site).href;
const ref = id => ({ '@id': id });

function checkConflicts(blocks, label) {
  const entities = new Map();
  function visit(value) {
    if (!value || typeof value !== 'object') return;
    if (Array.isArray(value)) { value.forEach(visit); return; }
    if (value['@id']) {
      const existing = entities.get(value['@id']) || {};
      for (const [key, field] of Object.entries(value)) if (key !== '@id') {
        if (key in existing) assert.deepEqual(field, existing[key], `${label}: conflicting ${key} for ${value['@id']}`);
        existing[key] = field;
      }
      entities.set(value['@id'], existing);
    }
    Object.values(value).forEach(visit);
  }
  blocks.forEach(visit);
}

for (const topic of topics) for (const locale of localeIds) {
  const route = pagePath(topic.id, locale);
  const canonical = absolute(route);
  const label = `${topic.id}/${locale}`;
  const $ = load(await readFile(path.join(dist, route.slice(1), 'index.html'), 'utf8'));
  assert.equal($('html').attr('lang'), locale, `${label}: HTML language`);
  assert.equal($('link[rel="canonical"]').attr('href'), canonical, `${label}: canonical URL`);
  const scripts = $('script[type="application/ld+json"]');
  const managed = $('script[type="application/ld+json"][data-anki-forge-schema]');
  if (/\bnoindex\b/i.test($('meta[name="robots"]').attr('content') || '')) {
    assert.equal(managed.length, 0, `${label}: noindex page has a managed graph`);
    continue;
  }
  assert.equal(managed.length, 1, `${label}: managed graph count`);
  const blocks = scripts.toArray().map(node => JSON.parse($(node).text()));
  checkConflicts(blocks, label);
  const schema = JSON.parse(managed.text());
  assert.equal(schema['@context'], 'https://schema.org', `${label}: schema context`);
  assert.ok(Array.isArray(schema['@graph']), `${label}: schema graph`);
  const graph = schema['@graph'];
  const type = topic.type === 'home' ? ['WebSite', 'WebPage', 'SoftwareApplication', 'SoftwareSourceCode']
    : topic.type === 'article' ? ['WebSite', 'WebPage', 'BlogPosting', 'SoftwareApplication']
    : topic.type === 'docs' && topic.id !== 'docs' ? ['WebSite', 'WebPage', 'TechArticle', 'SoftwareApplication']
    : ['WebSite', 'CollectionPage', 'SoftwareApplication'];
  assert.deepEqual(graph.map(node => node['@type']), type, `${label}: graph types`);
  const nodes = Object.fromEntries(graph.map(node => [node['@type'], node]));
  assert.deepEqual(nodes.WebSite, { '@id': `${siteRoot}#website`, '@type': 'WebSite', name: 'Anki Forge', url: siteRoot, inLanguage: ['en', 'zh-CN'] }, `${label}: website entity`);
  const web = nodes.WebPage || nodes.CollectionPage;
  assert.equal(web['@id'], `${canonical}#webpage`, `${label}: webpage identity`);
  assert.equal(web.name, ['docs', 'article'].includes(topic.type) ? $('h1').text().trim() : topic.title[locale], `${label}: webpage name`);
  assert.equal(web.description, $('meta[name="description"]').attr('content'), `${label}: webpage description`);
  assert.equal(web.url, canonical, `${label}: webpage URL`);
  assert.equal(web.inLanguage, locale, `${label}: webpage language`);
  assert.deepEqual(web.isPartOf, ref(`${siteRoot}#website`));
  assert.deepEqual(web.about, ref(`${siteRoot}#software`));
  const software = nodes.SoftwareApplication;
  assert.equal(software['@id'], `${siteRoot}#software`);
  assert.equal(software.name, 'Anki Forge');
  if (topic.type === 'home') {
    assert.deepEqual(web.mainEntity, ref(`${siteRoot}#software`));
    assert.equal(software.url, siteRoot);
    assert.equal(software.applicationCategory, 'DeveloperApplication');
    assert.equal(software.softwareVersion, install.publicVersion, `${label}: wrong public version`);
    assert.ok($('[data-module="install"]').text().includes(install.publicVersion), `${label}: version not visible`);
    assert.equal(software.description, $('.home-hero-copy > p').text().trim(), `${label}: software description is not visible`);
    assert.equal(software.license, `${repository}/blob/main/LICENSE`);
    assert.ok($('footer a[href]').toArray().some(node => $(node).attr('href') === software.license), `${label}: license not visible`);
    assert.equal(software.softwareHelp, absolute(pagePath('docs', locale)));
    assert.deepEqual(nodes.SoftwareSourceCode, { '@id': `${siteRoot}#source`, '@type': 'SoftwareSourceCode', name: 'Anki Forge', codeRepository: repository, license: `${repository}/blob/main/LICENSE`, targetProduct: ref(`${siteRoot}#software`) });
  } else assert.deepEqual(Object.keys(software).sort(), ['@id', '@type', 'name'], `${label}: software reference has extra fields`);
  const article = nodes.TechArticle || nodes.BlogPosting;
  if (article) {
    assert.deepEqual(web.mainEntity, ref(`${canonical}#article`));
    assert.equal(article['@id'], `${canonical}#article`);
    assert.equal(article.headline, $('h1').text().trim(), `${label}: headline differs from H1`);
    assert.equal(article.description, $('meta[name="description"]').attr('content'));
    assert.equal(article.url, canonical);
    assert.equal(article.inLanguage, locale);
    assert.deepEqual(article.mainEntityOfPage, ref(`${canonical}#webpage`));
    assert.deepEqual(article.about, ref(`${siteRoot}#software`));
    assert.deepEqual(article.encoding, { '@type': 'MediaObject', contentUrl: `${canonical}index.md`, encodingFormat: 'text/markdown', inLanguage: locale });
    for (const field of ['author', 'publisher', 'dateModified', 'image']) assert.ok(!(field in article), `${label}: unsupported ${field}`);
    if (topic.type === 'article') {
      const data = frontmatter(await readFile(path.join(root, topic.sources[locale]), 'utf8'));
      assert.equal(article.headline, data.title, `${label}: title differs from source`);
      assert.equal(article.description, data.description, `${label}: description differs from source`);
      assert.equal(article.datePublished, new Date(data.date).toISOString(), `${label}: publication date differs from source`);
      assert.equal(article.datePublished, $('article time').attr('datetime'), `${label}: publication date not visible`);
      assert.equal(article.articleSection, createTranslator(locale)(`blog.${data.category}`));
      assert.equal(article.articleSection, $('.post-meta span').first().text().trim());
    } else {
      assert.equal(article.headline, topic.title[locale], `${label}: title differs from registry`);
      assert.equal(article.description, topic.description[locale], `${label}: description differs from registry`);
      assert.ok(!('datePublished' in article) && !('dateModified' in article), `${label}: unmaintained documentation date`);
    }
  }
}
const error = load(await readFile(path.join(dist, '404.html'), 'utf8'));
assert.equal(error('script[data-anki-forge-schema]').length, 0, '404 must not have a managed graph');
console.log(`Verified structured data for ${topics.length * localeIds.length} published HTML pages.`);
