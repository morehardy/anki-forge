import test from 'node:test';
import assert from 'node:assert/strict';
import { canonicalUrl, llmsUrl, markdownUrl, entityIds } from '../src/lib/machine-urls.mjs';

test('machine URLs preserve the configured project base and distinguish language pages', () => {
  const config = { site: 'https://example.org', base: '/project', customDomain: '' };
  const page = { id: 'quickstart', type: 'docs', locale: 'zh-CN', canonical: canonicalUrl('quickstart', 'zh-CN', config) };
  assert.equal(page.canonical, 'https://example.org/project/zh-cn/docs/quickstart/');
  assert.equal(markdownUrl(page), 'https://example.org/project/zh-cn/docs/quickstart/index.md');
  assert.equal(llmsUrl('zh-CN', config), 'https://example.org/project/zh-cn/llms.txt');
  assert.equal(llmsUrl('en', config), 'https://example.org/project/llms.txt');
  assert.deepEqual(entityIds(page, config), {
    website: 'https://example.org/project/#website', software: 'https://example.org/project/#software',
    source: 'https://example.org/project/#source', webpage: `${page.canonical}#webpage`, article: `${page.canonical}#article`,
  });
  assert.equal(markdownUrl({ ...page, type: 'home' }), null);
});
