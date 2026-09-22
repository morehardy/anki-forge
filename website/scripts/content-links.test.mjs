import test from 'node:test';
import assert from 'node:assert/strict';
import { rewriteMarkdown, rewriteTarget } from './content-links.mjs';

test('imported guide links point to the deployed documentation', () => {
  assert.equal(rewriteTarget('template-bundles.md#fields', 'docs/rust-guide.md', '/anki-forge'), '/anki-forge/docs/templates/#fields');
  assert.equal(rewriteTarget('../anki_forge/README.md#stable-note-identity', 'docs/rust-guide.md', '/'), '/docs/api/#stable-note-identity');
  assert.equal(rewriteTarget('../anki_forge/examples/target_api_basic.rs', 'docs/rust-guide.md', '/'), 'https://github.com/morehardy/anki-forge/blob/main/anki_forge/examples/target_api_basic.rs');
});

test('anchors and external links are preserved', () => {
  for (const target of ['#media', 'https://example.org/path#part', 'mailto:hello@example.org', '//example.org/path']) {
    assert.equal(rewriteTarget(target, 'docs/rust-guide.md', '/anki-forge'), target);
  }
});

test('documentation import rewrites prose and references without changing fenced examples', () => {
  const source = '[Guide](rust-guide.md#media-troubleshooting)\n\n```md\n[Guide](rust-guide.md)\n```\n\n[api]: ../anki_forge/README.md\n';
  const result = rewriteMarkdown(source, 'docs/template-bundles.md', '/anki-forge');
  assert.match(result, /\[Guide\]\(\/anki-forge\/docs\/rust-guide\/#media-troubleshooting\)/);
  assert.match(result, /```md\n\[Guide\]\(rust-guide.md\)\n```/);
  assert.match(result, /\[api\]: \/anki-forge\/docs\/api\//);
});
