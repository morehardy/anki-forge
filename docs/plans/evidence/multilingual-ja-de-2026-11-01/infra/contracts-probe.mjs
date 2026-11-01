import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { validateMessages } from '../../../../../website/src/i18n/messages.mjs';
import { formatMessage, messageParameters } from '../../../../../website/src/i18n/format-message.mjs';
import { renderLlms } from '../../../../../website/scripts/machine-content.mjs';

const root = process.cwd();
const output = path.join(root, 'docs/plans/evidence/multilingual-ja-de-2026-11-01/infra');
const scratch = path.join(root, 'target/remaining-locales/ja-de-infra');
const fixture = path.join(scratch, 'six-locale-model');
const selected = ['en', 'zh-CN', 'es', 'pt-BR', 'ja', 'de'];
await mkdir(fixture, { recursive: true });
await writeFile(path.join(fixture, 'locale-model.mjs'), await readFile(path.join(root, 'website/locale-model.mjs')));
await writeFile(path.join(fixture, 'release-locales.json'), JSON.stringify({ schemaVersion: 1, locales: selected }));
const model = await import(pathToFileURL(path.join(fixture, 'locale-model.mjs')));
const routes = selected.map(locale => {
  const route = model.localePath('/docs/quickstart/', locale);
  for (const base of ['/', '/anki-forge']) assert.deepEqual(model.resolveLocalePath(`${base === '/' ? '' : base}${route}`, base), { locale, route: '/docs/quickstart/' });
  return { locale, route, image: model.socialImagePath(locale) };
});
assert.equal(new Set(routes.map(item => item.image)).size, 6);
const counts = {
  en: { count: { type: 'plural', variable: 'count', forms: { one: '{count} note', other: '{count} notes' } } },
  ja: { count: { type: 'plural', variable: 'count', forms: { other: 'ノート{count}件' } } },
  de: { count: { type: 'plural', variable: 'count', forms: { one: '{count} Notiz', other: '{count} Notizen' } } },
};
validateMessages(counts);
assert.throws(() => validateMessages({ en: counts.en, de: { count: { ...counts.de.count, forms: { other: '{count} Notizen' } } } }), /missing plural form/);
assert.throws(() => validateMessages({ en: counts.en, ja: { count: { ...counts.ja.count, forms: { other: 'ノート{wrong}件' } } } }), /plural count missing/);
const plural = Object.entries(counts).map(([locale, catalog]) => ({ locale, categories: new Intl.PluralRules(locale).resolvedOptions().pluralCategories, formatted: [0, 1, 2, 21, 1000, 1.5].map(count => ({ count, text: formatMessage(catalog.count, 'count', { count }, locale) })) }));
const parameterControls = ['Section titled “{{title}}”', 'No results for [SEARCH_TERM]. Showing [DIFFERENT_TERM]', '{language} {version} [COUNT]'].map(text => ({ text, parameters: messageParameters(text) }));
const context = {
  siteConfig: { site: 'https://example.invalid', base: '/anki-forge', customDomain: '' },
  repository: 'https://github.com/morehardy/anki-forge',
  publicInstall: { publicVersion: '1.2.3', languages: { rust: { label: 'Rust', install: 'cargo add fixture@1.2.3', runtime: '1.92+' } } },
  buildLocales: selected,
  translator: (key, values = {}) => key === 'machine.optional' ? 'Optional' : `${key} ${values.language || values.version || ''}`,
};
const indexControls = [];
for (const locale of ['ja', 'de']) {
  const index = renderLlms(locale, [], context);
  const own = `${model.configuredLocales[locale].prefix}/llms.txt`;
  assert.ok(!index.includes(`(${context.siteConfig.site}/anki-forge${own})`));
  for (const other of selected.filter(value => value !== locale)) assert.ok(index.includes(`(${context.siteConfig.site}/anki-forge${model.configuredLocales[other].prefix}/llms.txt)`));
  const emptyIntro = renderLlms(locale, [], { ...context, translator: (key, values) => key === 'machine.intro' ? '' : context.translator(key, values) });
  const budget = 24 * 1024 - Buffer.byteLength(emptyIntro);
  const intro = '日'.repeat(Math.floor(budget / 3)) + 'a'.repeat(budget % 3);
  const boundaryContext = { ...context, translator: (key, values) => key === 'machine.intro' ? intro : context.translator(key, values) };
  const boundary = renderLlms(locale, [], boundaryContext);
  assert.equal(Buffer.byteLength(boundary), 24 * 1024);
  assert.throws(() => renderLlms(locale, [], { ...context, translator: (key, values) => key === 'machine.intro' ? `${intro}a` : context.translator(key, values) }), /24 KiB/);
  indexControls.push({ locale, sixLocaleFixtureBytes: Buffer.byteLength(index), exactBoundaryBytes: Buffer.byteLength(boundary), rejectedOneByteOver: true });
}
const { rankSearchDocuments } = await import(pathToFileURL(path.join(scratch, 'search-original.mjs')));
const de = await readFile(path.join(root, 'docs/plans/evidence/remaining-locales-2026-11-01/samples/de/python-api.md'), 'utf8');
const line = de.split('\n').find(text => text.includes('Große Medien'));
const document = { id: 'python-api', url: '/fixture/de/python-api/', type: 'docs', title: line, description: '', terms: [] };
const germanExactLine = ['Große', 'große', 'GROẞE', 'Grosse', 'grosse'].map(query => ({ query, matches: rankSearchDocuments(query, [document], 'de').map(({ id, matchedTerms }) => ({ id, matchedTerms })) }));
assert.ok(germanExactLine.slice(0, 3).every(item => item.matches.length === 1));
assert.ok(germanExactLine.slice(3).every(item => item.matches.length === 0));
await writeFile(path.join(output, 'contract-observations.json'), JSON.stringify({ schemaVersion: 1, scope: 'Synthetic six-locale routing/count/index boundary fixtures, not authored locale acceptance; actual German source sentence is used only to isolate sharp-s lexical behavior.', node: process.version, routes, plural, parameterControls, indexControls, germanExactLine: { sourceSentence: line, observations: germanExactLine } }, null, 2) + '\n');
console.log('Passed six-locale route/image uniqueness, Japanese/German plurals, interpolation, 24 KiB UTF-8 exact/overflow boundaries, and actual German sharp-s lexical controls.');
