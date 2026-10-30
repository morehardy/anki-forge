import test from 'node:test';
import assert from 'node:assert/strict';
import { pages } from '../site-model.mjs';
import { createPageMetadata, metadataLinks, mergeMetadataHead, socialImageUrl } from '../src/lib/page-metadata.mjs';

const config = { site: 'https://example.org', base: '/project', customDomain: '' };
const page = pages.find(page => page.id === 'quickstart');

test('a third-language descriptor shares the canonical and keeps all alternate meanings', () => {
  const buildLocales = ['en', 'zh-CN', 'es'];
  const metadata = createPageMetadata(page, 'es', config, { title: 'Primera baraja', description: 'Crea una baraja con Rust.' }, buildLocales);
  const links = metadataLinks(metadata, config, buildLocales);
  assert.equal(metadata.canonical, 'https://example.org/project/es/docs/quickstart/');
  assert.deepEqual(links.filter(link => link.hreflang).map(link => link.hreflang), ['en', 'zh-CN', 'es', 'x-default']);
  assert.equal(links.find(link => link.hreflang === 'x-default').href, 'https://example.org/project/docs/quickstart/');
  assert.equal(links.find(link => link.type === 'text/markdown').href, `${metadata.canonical}index.md`);
  assert.equal(links.find(link => link.rel === 'describedby').href, 'https://example.org/project/es/llms.txt');
  const feed = { tag: 'link', attrs: { rel: 'alternate', type: 'application/rss+xml', href: 'feed.xml' } };
  const old = [feed, { tag: 'link', attrs: { rel: 'canonical', href: 'old' } }, { tag: 'link', attrs: { rel: 'alternate', hreflang: 'es', href: 'old' } }, { tag: 'link', attrs: { rel: 'alternate', hreflang: 'fr', href: 'stale-candidate' } }];
  const head = mergeMetadataHead(old, links.map(attrs => ({ tag: 'link', attrs })));
  assert.equal(head.filter(entry => entry.attrs.rel === 'canonical').length, 1);
  assert.equal(head.filter(entry => entry.attrs.hreflang === 'es').length, 1);
  assert.equal(head.filter(entry => entry.attrs.hreflang === 'fr').length, 0);
  assert.ok(head.includes(feed), 'language metadata must preserve the feed alternate');
  assert.equal(head.find(entry => entry.attrs.type === 'text/markdown').attrs.href, `${metadata.canonical}index.md`);
  assert.equal(socialImageUrl('zh-CN', config), 'https://example.org/project/generated/og-zh-cn.png');
  assert.equal(socialImageUrl('es', config), 'https://example.org/project/generated/og-es.png');
});

test('incomplete or inactive page descriptors fail before rendering', () => {
  assert.throws(() => createPageMetadata(page, 'es', config, { title: 'Título', description: 'Descripción' }), /Unregistered/);
  assert.throws(() => createPageMetadata(page, 'en', config, { title: '', description: 'Description' }), /Missing page metadata/);
});
