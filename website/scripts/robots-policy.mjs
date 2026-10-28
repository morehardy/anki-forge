import assert from 'node:assert/strict';

// RFC 9309 compares UTF-8 octets, decoding percent escapes only for unreserved
// characters. Keep encoded separators distinct from actual path separators.
const normalizePath = value => encodeURI(value).replaceAll('%25', '%').replace(/%[0-9a-f]{2}/gi, escaped => {
  const character = String.fromCharCode(Number.parseInt(escaped.slice(1), 16));
  return /[A-Za-z0-9._~-]/.test(character) ? character : escaped.toUpperCase();
});

// Match robots product tokens, not full user-agent headers. A specific group
// replaces the wildcard group; equally specific groups combine their rules.
function parseGroups(source) {
  const groups = [];
  let group;
  let hasRules = false;
  for (const raw of source.replace(/^\uFEFF/, '').split(/\r?\n/)) {
    const line = raw.split('#')[0].trim();
    const separator = line.indexOf(':');
    if (separator < 0) continue;
    const field = line.slice(0, separator).trim().toLowerCase();
    const value = line.slice(separator + 1).trim();
    if (field === 'user-agent') {
      assert.ok(value, 'robots.txt: empty user-agent group');
      if (!group || hasRules) {
        group = { agents: [], rules: [] };
        groups.push(group);
        hasRules = false;
      }
      group.agents.push(value.toLowerCase());
    } else if (group && (field === 'allow' || field === 'disallow')) {
      hasRules = true;
      if (!value) continue; // Empty Disallow permits crawling.
      assert.ok(value.startsWith('/'), `robots.txt: invalid ${field} path: ${value}`);
      const anchored = value.endsWith('$');
      const pattern = normalizePath(anchored ? value.slice(0, -1) : value);
      const expression = pattern.split('*').map(part => part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('.*');
      group.rules.push({ allow: field === 'allow', pattern: new RegExp(`^${expression}${anchored ? '$' : ''}`), specificity: Buffer.byteLength(pattern.replaceAll('*', '')) });
    }
  }
  return groups;
}

/** Verify actual normal routes against the delivered robots policy. */
export function assertCrawlable(source, routes, { agents = ['*', 'Googlebot'] } = {}) {
  const groups = parseGroups(source);
  for (const product of agents) {
    const agent = product.toLowerCase();
    const matches = groups.map(group => ({ group, specificity: Math.max(-1, ...group.agents.map(token => token === '*' ? 0 : agent !== '*' && agent.includes(token) ? token.length : -1)) }));
    const specificity = Math.max(-1, ...matches.map(match => match.specificity));
    const rules = matches.filter(match => match.specificity >= 0 && match.specificity === specificity).flatMap(match => match.group.rules);
    for (const route of routes) {
      assert.ok(typeof route === 'string' && route.startsWith('/'), `robots.txt: expected site-relative route: ${route}`);
      const normalized = normalizePath(route);
      const applicable = rules.filter(rule => rule.pattern.test(normalized)).sort((a, b) => b.specificity - a.specificity || Number(b.allow) - Number(a.allow));
      assert.ok(!applicable.length || applicable[0].allow, `${route}: robots.txt blocks ${product}`);
    }
  }
}
