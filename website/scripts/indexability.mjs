import assert from 'node:assert/strict';

const blocked = /\b(?:none|noindex|nosnippet)\b|\bmax-snippet\s*:\s*0\b/i;

/** Check both rendered directives and the actual response, when available. */
export function assertIndexable($, { route = 'page', headers } = {}) {
  for (const node of $('meta[name]').toArray()) {
    const name = String($(node).attr('name')).toLowerCase();
    if (name === 'robots' || name === 'googlebot' || name === 'googlebot-news') {
      assert.ok(!blocked.test($(node).attr('content') || ''), `${route}: indexing or snippet restriction in ${name}`);
    }
  }
  const directive = headers?.get?.('x-robots-tag') || '';
  assert.ok(!blocked.test(directive), `${route}: indexing or snippet restriction in X-Robots-Tag`);
  for (const node of $('main[data-nosnippet], main [data-nosnippet]').toArray()) {
    assert.ok(!$(node).text().trim(), `${route}: meaningful body is excluded by data-nosnippet`);
  }
  for (const main of $('main').toArray()) {
    assert.ok(!$(main).text().trim() || !$(main).parents('[data-nosnippet]').length, `${route}: meaningful body is excluded by ancestor data-nosnippet`);
  }
}
