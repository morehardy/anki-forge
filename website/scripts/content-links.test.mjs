import test from 'node:test';
import assert from 'node:assert/strict';
import { rewriteMarkdown, rewriteTarget, stripDocumentTitle } from './content-links.mjs';

test('imported guide links point to the deployed documentation', () => {
  assert.equal(rewriteTarget('template-bundles.md#fields', 'docs/rust-guide.md', '/anki-forge'), '/anki-forge/docs/templates/#fields');
  assert.equal(rewriteTarget('api-overview.md#stable-note-identity', 'docs/rust-guide.md', '/'), '/docs/api/#stable-note-identity');
  assert.equal(rewriteTarget('../anki_forge/examples/target_api_basic.rs', 'docs/rust-guide.md', '/'), 'https://github.com/morehardy/anki-forge/blob/main/anki_forge/examples/target_api_basic.rs');
});

test('imported Markdown and branded HTML titles defer to the page title without changing code', () => {
  assert.equal(stripDocumentTitle('# Guide\n\nBody\n'), 'Body\n');
  assert.equal(stripDocumentTitle('<p>Logo</p>\n<h1 align="center">anki-forge</h1>\n\n## API\n'), '<p>Logo</p>\n## API\n');
  const code = '# Guide\n\n```html\n<h1>Card template</h1>\n```\n';
  assert.equal(stripDocumentTitle(code), '```html\n<h1>Card template</h1>\n```\n');
});

test('anchors and external links are preserved', () => {
  for (const target of ['#media', 'https://example.org/path#part', 'mailto:hello@example.org', '//example.org/path']) {
    assert.equal(rewriteTarget(target, 'docs/rust-guide.md', '/anki-forge'), target);
  }
});

test('documentation import rewrites prose and references without changing fenced examples', () => {
  const source = '[Guide](rust-guide.md#media-troubleshooting)\n\n```md\n[Guide](rust-guide.md)\n```\n\n[api]: api-overview.md\n';
  const result = rewriteMarkdown(source, 'docs/template-bundles.md', '/anki-forge');
  assert.match(result, /\[Guide\]\(\/anki-forge\/docs\/rust-guide\/#media-troubleshooting\)/);
  assert.match(result, /```md\n\[Guide\]\(rust-guide.md\)\n```/);
  assert.match(result, /\[api\]: \/anki-forge\/docs\/api\//);
});


test('translated core guides keep their topic and deployment base', () => {
  assert.equal(rewriteTarget('json-to-anki.md#python', 'docs/zh-cn/languages.md', '/anki-forge'), '/anki-forge/zh-cn/docs/json-to-anki/#python');
  assert.equal(rewriteTarget('../python/api.md', 'docs/zh-cn/python-quickstart.md', '/'), '/zh-cn/docs/python-api/');
});
