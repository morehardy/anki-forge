import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { configuredLocales, releaseLocales } from '../locale-model.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const extraLocale = 'qaa';
const fixtureLocale = { prefix: '/fixture', lang: 'qaa', og: 'en_US', label: 'Fixture language', format: 'en-US', starlightKey: 'fixture', dir: 'ltr' };
// Add a new locale key to model data without changing module code or existing translations.
const addLocale = (source, label, locale = extraLocale) => source.replace(/(["'])zh-CN\1: (["'])(?:(?!\2)[^\\]|\\.)*\2/g, match => `${match}, ${JSON.stringify(locale)}: ${JSON.stringify(label)}`);

// Synthetic translations belong only to this disposable fixture.
async function additionalRelease(t, selected) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-third-locale-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  for (const file of ['website/site-model.mjs', 'website/locale-model.mjs', 'website/src/i18n', 'website/src/lib/ai-quickstart.mjs', 'docs/ai-quickstart.json']) {
    await cp(path.join(root, file), path.join(directory, file), { recursive: true });
  }
  const website = path.join(directory, 'website');
  const registry = path.join(website, 'locale-model.mjs');
  await writeFile(registry, (await readFile(registry, 'utf8')).replace('export const configuredLocales = {', `export const configuredLocales = {\n  ${extraLocale}: ${JSON.stringify(fixtureLocale)},`));
  const added = [...selected.filter(locale => !releaseLocales.includes(locale)), extraLocale];
  await writeFile(path.join(website, 'release-locales.json'), JSON.stringify({ schemaVersion: 1, locales: [...selected, extraLocale] }));
  const model = path.join(website, 'site-model.mjs');
  let source = await readFile(model, 'utf8');
  for (const locale of added) {
    await cp(path.join(website, 'src/i18n/en.json'), path.join(website, `src/i18n/${locale}.json`));
    source = addLocale(source, 'Fixture label', locale);
  }
  await writeFile(model, source);
  const data = JSON.parse(await readFile(path.join(directory, 'docs/ai-quickstart.json'), 'utf8'));
  for (const locale of selected.filter(locale => !releaseLocales.includes(locale))) {
    data.prompts[locale] = data.prompts.en;
    data.requestPlaceholders[locale] = '<fixture task>';
    data.promptContracts[locale] = data.promptContracts.en;
  }
  data.prompts[extraLocale] = data.prompts.en;
  data.requestPlaceholders[extraLocale] = '<fixture task>';
  data.promptContracts[extraLocale] = data.promptContracts.en;
  await writeFile(path.join(directory, 'docs/ai-quickstart.json'), JSON.stringify(data));
  return directory;
}

function probe(directory) {
  const url = file => JSON.stringify(pathToFileURL(path.join(directory, file)).href);
  const script = `
    const { pages, docGroups, navigation } = await import(${url('website/site-model.mjs')});
    const { catalogs } = await import(${url('website/src/i18n/messages.mjs')});
    const { renderPrompt } = await import(${url('website/src/lib/ai-quickstart.mjs')});
    const { readFile } = await import('node:fs/promises');
    const data = JSON.parse(await readFile(new URL(${url('docs/ai-quickstart.json')}), 'utf8'));
    const locale = ${JSON.stringify(extraLocale)};
    console.log(JSON.stringify({
      catalogs: Object.keys(catalogs),
      pages: pages.filter(page => page.type !== 'article').map(page => ({ id: page.id, type: page.type, title: page.title[locale], description: page.description[locale], metadata: page.metadata[locale] })),
      compatibility: pages.filter(page => Object.hasOwn(page, 'en') || Object.hasOwn(page, 'zh')).map(page => page.id),
      groups: docGroups.map(group => group.labels[locale]),
      navigation: navigation.map(item => item.label[locale]),
      prompt: renderPrompt(data, 'rust', '', locale),
    }));`;
  return JSON.parse(execFileSync(process.execPath, ['--input-type=module', '-e', script], { cwd: directory, encoding: 'utf8' }));
}

for (const [label, selected] of [['current release', releaseLocales], ['all configured languages released', Object.keys(configuredLocales)]]) test(`${label}: a further locale needs only catalogs, model data and prompt data`, async t => {
  const result = probe(await additionalRelease(t, selected));
  assert.deepEqual(result.catalogs, [...selected, extraLocale]);
  for (const page of result.pages) {
    if (page.type === 'docs') assert.deepEqual([page.title, page.description], ['Fixture label', 'Fixture label'], page.id);
    else assert.ok(page.title?.trim() && page.description?.trim(), `${page.id}: additional catalog metadata`);
    assert.deepEqual(page.metadata, { title: page.title, description: page.description }, page.id);
  }
  assert.deepEqual(result.compatibility, [], 'pages expose languages only through locale-keyed maps');
  assert.ok(result.groups.length && result.groups.every(label => label === 'Fixture label'));
  assert.ok(result.navigation.length && result.navigation.every(label => label === 'Fixture label'));
  assert.ok(result.prompt.includes('<fixture task>'), 'the empty request uses the additional locale placeholder');
  assert.ok(result.prompt.includes('https://ankiforge.dev/fixture/docs/quickstart/'));
});
