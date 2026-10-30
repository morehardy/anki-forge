import test from 'node:test';
import assert from 'node:assert/strict';
import { assertCrawlable } from './robots-policy.mjs';

const routes = ['/', '/docs/updates/', '/zh-cn/docs/updates/'];
const sitemap = 'Sitemap: https://ankiforge.dev/sitemap-index.xml\n';
test('the existing open policy permits normal root and project-prefix routes', () => {
  const policy = `User-agent: *\nAllow: /\n\n${sitemap}`;
  assert.doesNotThrow(() => assertCrawlable(policy, routes));
  assert.doesNotThrow(() => assertCrawlable(policy, routes.map(route => '/anki-forge' + route)));
});
test('a sitemap does not excuse a global or language-specific crawl block', () => {
  assert.throws(() => assertCrawlable(`User-agent: *\nDisallow: /\n${sitemap}`, routes), /robots.txt blocks/);
  assert.throws(() => assertCrawlable(`User-agent: *\nDisallow: /zh-cn/\n${sitemap}`, routes), /zh-cn.*blocks/);
  assert.throws(() => assertCrawlable('User-agent: *\nDisallow: /anki-forge/docs/\n', routes.map(route => '/anki-forge' + route)), /anki-forge\/docs.*blocks/);
});
test('Googlebot-specific restrictions override wildcard permissions', () => {
  assert.throws(() => assertCrawlable('User-agent: *\nAllow: /\nUser-agent: Googlebot\nDisallow: /docs/\n', routes), /docs\/updates.*Googlebot/);
});
test('unrelated user-agent groups do not restrict normal crawling', () => {
  assert.doesNotThrow(() => assertCrawlable('User-agent: UnrelatedBot\nDisallow: /\n', routes));
  assert.doesNotThrow(() => assertCrawlable('User-agent: *\nDisallow: /\nUser-agent: Googlebot\nAllow: /\n', routes, { agents: ['Googlebot'] }));
});
test('rules combine equally specific groups, use longest paths, and prefer Allow on a tie', () => {
  const policy = 'User-agent: *\nDisallow: /private/\nUser-agent: Googlebot\nDisallow: /docs/\nUser-agent: googlebot\nAllow: /docs/updates/\n';
  assert.doesNotThrow(() => assertCrawlable(policy, routes));
  assert.throws(() => assertCrawlable(policy, ['/docs/other/']), /Googlebot/);
  assert.doesNotThrow(() => assertCrawlable('User-agent: *\nDisallow: /\nAllow: /\n', routes));
});
test('wildcards, end anchors, empty rules, comments and literal punctuation have their robots meaning', () => {
  const policy = '\uFEFFUser-Agent: * # all clients\r\nDisallow:\r\nDisallow: /*/updates/$\r\nDisallow: /docs/file.xml$\r\n';
  assert.throws(() => assertCrawlable(policy, ['/zh-cn/docs/updates/']), /blocks/);
  assert.throws(() => assertCrawlable(policy, ['/docs/file.xml']), /blocks/);
  assert.doesNotThrow(() => assertCrawlable(policy, ['/docs/updates/child/', '/docs/file-xml', '/docs/file.xml/child']));
});
test('encoded unreserved characters and UTF-8 paths cannot disguise a block', () => {
  assert.throws(() => assertCrawlable('User-agent: *\nDisallow: /docs/%75pdates/\n', ['/docs/updates/']), /blocks/);
  assert.throws(() => assertCrawlable('User-agent: *\nDisallow: /docs/更新/\n', ['/docs/%E6%9B%B4%E6%96%B0/']), /blocks/);
  assert.doesNotThrow(() => assertCrawlable('User-agent: *\nDisallow: /docs%2Fupdates/\n', ['/docs/updates/']));
});
