import test from 'node:test';
import assert from 'node:assert/strict';
import { configuredLocales, localeIds, locales, localePath, resolveLocalePath, socialImagePath, validateReleaseLocales } from '../locale-model.mjs';
import { pagePath, resolvePage, documentationEntries } from '../site-model.mjs';
import { translatedPagePaths } from '../src/lib/localized-routes.mjs';

test('configured languages do not activate unfinished translations', () => {
  assert.equal(Object.keys(configuredLocales).length, 8);
  assert.deepEqual(localeIds, ['en', 'zh-CN']);
  assert.deepEqual(Object.keys(locales), ['en', 'zh-CN']);
  assert.equal(resolvePage('/es/docs/'), null);
  assert.throws(() => pagePath('docs', 'es'), /release locale/);
  assert.ok(documentationEntries().every(entry => ['en', 'zh-CN'].includes(entry.locale) && entry.source));
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
  assert.deepEqual(translatedPagePaths(), [{ params: { locale: 'zh-cn' }, props: { locale: 'zh-CN' } }]);
  assert.equal(socialImagePath('en'), '/generated/og.png');
  assert.equal(socialImagePath('zh-CN'), '/generated/og-zh-cn.png');
});
