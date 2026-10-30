import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
// Every locale-keyed literal in the model gains a Spanish value; module code stays untouched.
const addSpanish = (source, label) => source.replace(/(["'])zh-CN\1: (["'])(?:(?!\2)[^\\]|\\.)*\2/g, match => `${match}, "es": ${JSON.stringify(label)}`);

// A disposable release adds Spanish through data files only.
async function spanishRelease(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-third-locale-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  for (const file of ['website/site-model.mjs', 'website/locale-model.mjs', 'website/src/i18n', 'website/src/lib/ai-quickstart.mjs', 'docs/ai-quickstart.json']) {
    await cp(path.join(root, file), path.join(directory, file), { recursive: true });
  }
  const website = path.join(directory, 'website');
  await writeFile(path.join(website, 'release-locales.json'), JSON.stringify({ schemaVersion: 1, locales: ['en', 'zh-CN', 'es'] }));
  await cp(path.join(website, 'src/i18n/en.json'), path.join(website, 'src/i18n/es.json'));
  const model = path.join(website, 'site-model.mjs');
  await writeFile(model, addSpanish(await readFile(model, 'utf8'), 'Texto revisado'));
  const data = JSON.parse(await readFile(path.join(directory, 'docs/ai-quickstart.json'), 'utf8'));
  data.prompts = { ...data.prompts, es: data.prompts.en };
  data.requestPlaceholders = { ...data.requestPlaceholders, es: '<describe la tarea>' };
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
    console.log(JSON.stringify({
      catalogs: Object.keys(catalogs),
      pages: pages.filter(page => page.type !== 'article').map(page => ({ id: page.id, type: page.type, title: page.title.es, description: page.description.es, metadata: page.metadata.es })),
      compatibility: pages.filter(page => Object.hasOwn(page, 'en') || Object.hasOwn(page, 'zh')).map(page => page.id),
      groups: docGroups.map(group => group.labels.es),
      navigation: navigation.map(item => item.label.es),
      prompt: renderPrompt(data, 'rust', '', 'es'),
    }));`;
  return JSON.parse(execFileSync(process.execPath, ['--input-type=module', '-e', script], { cwd: directory, encoding: 'utf8' }));
}

test('a third release locale is added through catalogs, model data and prompt data only', async t => {
  const result = probe(await spanishRelease(t));
  assert.deepEqual(result.catalogs, ['en', 'zh-CN', 'es']);
  for (const page of result.pages) {
    if (page.type === 'docs') assert.deepEqual([page.title, page.description], ['Texto revisado', 'Texto revisado'], page.id);
    else assert.ok(page.title?.trim() && page.description?.trim(), `${page.id}: Spanish catalog metadata`);
    assert.deepEqual(page.metadata, { title: page.title, description: page.description }, page.id);
  }
  assert.deepEqual(result.compatibility, [], 'pages expose languages only through locale-keyed maps');
  assert.ok(result.groups.length && result.groups.every(label => label === 'Texto revisado'));
  assert.ok(result.navigation.length && result.navigation.every(label => label === 'Texto revisado'));
  assert.ok(result.prompt.includes('<describe la tarea>'), 'the empty request uses the Spanish placeholder');
  assert.ok(result.prompt.includes('https://ankiforge.dev/es/docs/quickstart/'));
});
