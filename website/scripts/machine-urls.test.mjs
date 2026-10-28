import test from 'node:test';
import assert from 'node:assert/strict';
import { canonicalUrl, llmsPath, llmsUrl, markdownUrl, entityIds, machineResourceKind } from '../src/lib/machine-urls.mjs';

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

test('configured third-language paths and ownership do not overwrite the Chinese namespace', () => {
  const config = { site: 'https://example.org', base: '/project', customDomain: '' };
  assert.equal(llmsPath('es'), '/es/llms.txt');
  assert.equal(llmsUrl('pt-BR', config), 'https://example.org/project/pt-br/llms.txt');
  assert.equal(canonicalUrl('quickstart', 'es', config, ['en', 'zh-CN', 'es']), 'https://example.org/project/es/docs/quickstart/');
  assert.throws(() => canonicalUrl('quickstart', 'es', config), /inactive/);
  assert.throws(() => llmsPath('unknown'), /locale/i);
  for (const file of ['llms.txt', 'zh-cn/llms.txt', 'es/llms.txt', 'pt-br/llms.txt']) assert.equal(machineResourceKind(file), 'index');
  for (const file of ['docs/index.md', 'zh-cn/docs/quickstart/index.md', 'es/docs/quickstart/index.md', 'pt-br/blog/example/index.md']) assert.equal(machineResourceKind(file), 'markdown');
  for (const file of ['es/custom.md', 'es/docs/guide.md', 'generated/guide/index.md', 'not-a-locale/docs/index.md', 'src/content/docs/index.mdx']) assert.equal(machineResourceKind(file), null);
});
