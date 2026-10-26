// Deliberately independent of build configuration: a self-consistent wrong origin
// must not pass the production smoke check.
import assert from 'node:assert/strict';
import { corePages } from './core-pages.mjs';

const origin = 'https://ankiforge.dev';
async function get(route) {
  const expected = new URL(route, origin).href;
  let response;
  for (let attempt = 0; attempt < 4; attempt++) {
    response = await fetch(expected, { redirect: 'follow', signal: AbortSignal.timeout(20_000), headers: { 'Cache-Control': 'no-cache' } });
    if (response.ok) break;
    if (attempt < 3) await new Promise(resolve => setTimeout(resolve, 3000));
  }
  assert.ok(response.ok, `${expected}: HTTP ${response.status}`);
  assert.equal(response.url, expected, `${route}: unexpected final URL`);
  return response.text();
}
const attribute = (tag, name) => tag.match(new RegExp(`\\b${name}=["']([^"']*)["']`, 'i'))?.[1];
for (const pair of corePages) {
  for (const [locale, route] of [['en', pair.en], ['zh-CN', pair.zh]]) {
    const html = await get(route);
    assert.equal(attribute(html.match(/<html\b[^>]*>/i)?.[0] ?? '', 'lang'), locale, `${route}: wrong HTML language`);
    const links = html.match(/<link\b[^>]*>/gi) ?? [];
    const canonical = links.filter(link => attribute(link, 'rel') === 'canonical');
    assert.equal(canonical.length, 1, `${route}: canonical count`);
    assert.equal(attribute(canonical[0], 'href'), new URL(route, origin).href, `${route}: wrong canonical`);
    for (const [lang, translated] of [['en', pair.en], ['zh-CN', pair.zh]]) {
      const alternate = links.filter(link => attribute(link, 'hreflang') === lang);
      assert.equal(alternate.length, 1, `${route}: missing or duplicated ${lang} alternate`);
      assert.equal(attribute(alternate[0], 'href'), new URL(translated, origin).href);
    }
    assert.equal((html.match(/<h1\b/gi) ?? []).length, 1, `${route}: heading count`);
    assert.equal((html.match(/src=["']https:\/\/static.cloudflareinsights.com\/beacon.min.js["']/g) ?? []).length, 1, `${route}: beacon count`);
    assert.match(html, /Anki Forge/);
    console.log(`Verified ${locale} ${route}`);
  }
}
assert.match(await get('/robots.txt'), /Sitemap: https:\/\/ankiforge\.dev\/sitemap-index\.xml/);
const index = await get('/sitemap-index.xml');
const maps = [...index.matchAll(/<loc>([^<]+)<\/loc>/g)].map(match => new URL(match[1]));
assert.ok(maps.length);
let sitemap = '';
for (const url of maps) {
  assert.equal(url.origin, origin, 'Sitemap index points outside production');
  sitemap += await get(url.pathname);
}
for (const pair of corePages) for (const route of [pair.en, pair.zh]) assert.ok(sitemap.includes(new URL(route, origin).href), `Missing sitemap URL ${route}`);
const missing = await fetch(`${origin}/zh-cn/docs/rust-api/`, { signal: AbortSignal.timeout(20_000) });
assert.equal(missing.status, 404, 'Untranslated Chinese reference must return 404');
console.log('Production URL, content, language, beacon, robots and sitemap checks passed. This does not verify the analytics dashboard.');
