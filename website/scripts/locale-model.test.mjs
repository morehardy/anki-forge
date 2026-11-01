import test from 'node:test';
import assert from 'node:assert/strict';
import release from '../release-locales.json' with { type: 'json' };
import { configuredLocales, localeIds, locales, localePath, resolveLocalePath, socialImagePath, validateReleaseLocales } from '../locale-model.mjs';
import { pagePath, resolvePage, documentationEntries } from '../site-model.mjs';
import { translatedPagePaths } from '../src/lib/localized-routes.mjs';

test('configured languages do not activate unfinished translations', () => {
  assert.deepEqual(localeIds, release.locales);
  assert.deepEqual(Object.keys(locales), release.locales);
  for (const locale of release.locales) {
    const route = `${configuredLocales[locale].prefix}/docs/`;
    assert.equal(pagePath('docs', locale), route);
    assert.equal(resolvePage(route)?.locale, locale);
  }
  for (const locale of Object.keys(configuredLocales).filter(locale => !release.locales.includes(locale))) {
    assert.equal(resolvePage(`${configuredLocales[locale].prefix}/docs/`), null, `${locale}: planned route must stay inactive`);
    assert.throws(() => pagePath('docs', locale), /release locale/);
  }
  assert.ok(documentationEntries().every(entry => release.locales.includes(entry.locale) && entry.source));
});

test('release selection rejects duplicates, unknown languages and lost published languages', () => {
  const snapshot = locales => ({ schemaVersion: 1, locales });
  assert.throws(() => validateReleaseLocales(snapshot(['en', 'en'])), /Duplicate/);
  assert.throws(() => validateReleaseLocales(snapshot(['en', 'xx'])), /Unknown/);
  assert.throws(() => validateReleaseLocales(snapshot(['en']), configuredLocales, ['en', 'zh-CN']), /remove published/);
  assert.throws(() => validateReleaseLocales(snapshot(['es', 'en'])), /start with en/);
  assert.deepEqual(validateReleaseLocales(snapshot(['en', 'zh-CN', 'pt-BR'])), ['en', 'zh-CN', 'pt-BR']);
});

test('third-language pure routing respects regional directory and base without reading sources', () => {
  const active = ['en', 'zh-CN', 'pt-BR'];
  assert.equal(localePath('/docs/quickstart/', 'pt-BR'), '/pt-br/docs/quickstart/');
  assert.deepEqual(resolveLocalePath('/project/pt-br/docs/', '/project', active), { locale: 'pt-BR', route: '/docs/' });
  assert.equal(resolveLocalePath('/project-copy/pt-br/docs/', '/project', active), null);
  assert.equal(resolveLocalePath('/project/es/docs/', '/project', active), null);
  assert.equal(resolvePage('/project/docs/', '/project')?.page.id, 'docs');
  assert.equal(resolvePage('/project/zh-cn/docs/', '/project')?.locale, 'zh-CN');
  assert.equal(resolvePage('/unknown/docs/'), null);
});

test('translated page wrappers and social images derive from the release selection', () => {
  const wrappers = translatedPagePaths();
  assert.deepEqual(wrappers.map(entry => entry.props.locale), release.locales.filter(locale => locale !== 'en'));
  assert.equal(new Set(wrappers.map(entry => entry.params.locale)).size, wrappers.length, 'each released translation has a distinct route namespace');
  for (const entry of wrappers) {
    assert.equal(resolvePage(`/${entry.params.locale}/docs/`)?.locale, entry.props.locale, 'wrapper params must reach the matching released document');
    assert.equal(resolvePage(`/${entry.params.locale}/`)?.page.id, 'home', 'each wrapper reaches the shared home page');
  }
  assert.equal(socialImagePath('en'), '/generated/og.png');
  assert.equal(socialImagePath('zh-CN'), '/generated/og-zh-cn.png');
  assert.equal(socialImagePath('es'), '/generated/og-es.png');
  assert.equal(new Set(release.locales.map(socialImagePath)).size, release.locales.length, 'released language images must not overwrite one another');
});
