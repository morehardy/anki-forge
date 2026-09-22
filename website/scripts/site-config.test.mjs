import test from 'node:test';
import assert from 'node:assert/strict';
import { resolveSiteConfig, withBase } from '../site.config.mjs';

test('GitHub project pages keep their repository prefix', () => {
  const config = resolveSiteConfig({});
  assert.deepEqual(config, { site: 'https://morehardy.github.io', base: '/anki-forge', customDomain: '' });
  assert.equal(withBase('/docs/', config.base), '/anki-forge/docs/');
});

test('custom domains require an origin and a root base together', () => {
  const config = resolveSiteConfig({ SITE_URL: 'https://example.org/', SITE_BASE: '/', SITE_CUSTOM_DOMAIN: 'example.org' });
  assert.equal(config.site, 'https://example.org');
  assert.equal(withBase('/docs/', config.base), '/docs/');
  assert.throws(() => resolveSiteConfig({ SITE_URL: 'https://example.org', SITE_CUSTOM_DOMAIN: 'example.org' }));
  assert.throws(() => resolveSiteConfig({ SITE_URL: 'https://example.org', SITE_BASE: '/', SITE_CUSTOM_DOMAIN: 'other.example' }));
});

test('configuration rejects URL paths, credentials, and unsafe base paths', () => {
  for (const SITE_URL of ['file:///tmp/site', 'https://example.org/path', 'https://user:pass@example.org', 'https://example.org/?q=1', 'https://example.org/#part']) {
    assert.throws(() => resolveSiteConfig({ SITE_URL }));
  }
  for (const SITE_BASE of ['//example.org', '../outside', '/one/../two', '/?query', '/a#hash']) {
    assert.throws(() => resolveSiteConfig({ SITE_BASE }));
  }
  assert.throws(() => withBase('//example.org', '/anki-forge'));
});
