import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, copyFile, readFile, writeFile, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { pages, localeIds } from '../site-model.mjs';
import { reviewInputFiles, reviewInputs, rubricVersion, glossaryVersion, digest } from './content-review-inputs.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const command = fileURLToPath(new URL('./check-content.mjs', import.meta.url));
async function fixture(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-i18n-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const inputs = (await Promise.all(pages.flatMap(page => localeIds.map(locale => reviewInputFiles(page, locale, root))))).flat();
  const files = new Set([...inputs, 'website/content-reviews.json', 'website/legacy-sections.json', ...localeIds.flatMap(locale => [`website/src/i18n/${locale}.json`, `website/src/content/i18n/${locale}.json`])]);
  for (const file of files) { await mkdir(path.dirname(path.join(directory, file)), { recursive: true }); await copyFile(path.join(root, file), path.join(directory, file)); }
  // Synthetic evidence is confined to this disposable fixture, never production.
  const records = { schemaVersion: 2, entries: {} };
  const report = { schemaVersion: 1, reviewerType: 'ai', reviewerRun: 'synthetic-unit-test-only', entries: {} };
  for (const page of pages) {
    records.entries[page.id] = {}; report.entries[page.id] = {};
    for (const locale of localeIds) {
      const inputs = await reviewInputs(page, locale, directory);
      const entry = { ...inputs, technical: { status: 'reviewed', evidence: ['Synthetic gate fixture'], openIssues: [] }, linguistic: { status: 'reviewed', evidence: ['Synthetic gate fixture'], openIssues: [], score: { terminology: 4, syntax: 4, register: 4, usability: 4 } } };
      report.entries[page.id][locale] = entry;
      records.entries[page.id][locale] = { ...inputs, rubricVersion, glossaryVersion, technical: { ...entry.technical, reviewerType: 'ai', reviewerRun: report.reviewerRun }, linguistic: { ...entry.linguistic, reviewerType: 'ai', reviewerRun: report.reviewerRun } };
    }
  }
  const evidencePath = 'docs/plans/evidence/unit-fixture/reviews.json';
  const bytes = JSON.stringify(report);
  await mkdir(path.dirname(path.join(directory, evidencePath)), { recursive: true });
  await writeFile(path.join(directory, evidencePath), bytes);
  for (const entries of Object.values(records.entries)) for (const entry of Object.values(entries)) for (const axis of ['technical','linguistic']) entry[axis].evidence = { path: evidencePath, sha256: digest(bytes) };
  await writeFile(path.join(directory, 'website/content-reviews.json'), JSON.stringify(records));
  return directory;
}
const run = directory => execFileSync(process.execPath, [command, directory], { encoding: 'utf8', stdio: 'pipe' });
test('the content gate accepts reviewed genuine translations', async t => {
  assert.match(run(await fixture(t)), /paired topics/);
});
test('the content gate rejects missing translations and unregistered sources', async t => {
  const directory = await fixture(t);
  await rm(path.join(directory, 'docs/zh-cn/media.md'));
  assert.throws(() => run(directory), /Unregistered or missing zh-CN documentation/);
  await copyFile(path.join(root, 'docs/zh-cn/media.md'), path.join(directory, 'docs/zh-cn/media.md'));
  await writeFile(path.join(directory, 'docs/zh-cn/extra-topic.md'), '# 额外主题\n');
  assert.throws(() => run(directory), /Unregistered or missing zh-CN documentation/);
});
test('unregistered drafts are not treated as public topics', async t => {
  const directory = await fixture(t);
  await writeFile(path.join(directory, 'website/src/content/blog/future.mdx'), '---\ndraft: true\n---\nA draft.\n');
  await writeFile(path.join(directory, 'website/src/content/docs/docs/future.mdx'), '---\ndraft: true\n---\nA draft.\n');
  await writeFile(path.join(directory, 'docs/zh-cn/future.md'), '---\ndraft: true\n---\n草稿。\n');
  assert.match(run(directory), /paired topics/);
});
test('the content gate rejects incomplete messages and mismatched interpolation', async t => {
  const directory = await fixture(t);
  const file = path.join(directory, 'website/src/i18n/zh-CN.json');
  const original = JSON.parse(await readFile(file, 'utf8'));
  const missing = { ...original }; delete missing['card.answer'];
  await writeFile(file, JSON.stringify(missing));
  assert.throws(() => run(directory), /message keys/);
  await writeFile(file, JSON.stringify({ ...original, 'card.answer': '答案：{wrong}' }));
  assert.throws(() => run(directory), /mismatched message parameters/);
});
test('technical edits require a paired translation review and fallback is rejected', async t => {
  const directory = await fixture(t);
  const source = path.join(directory, 'docs/concepts.md');
  await writeFile(source, (await readFile(source, 'utf8')) + '\nChanged technical guidance.\n');
  assert.throws(() => run(directory), /sourceHash changed; independent source review required/);
  await copyFile(path.join(root, 'docs/concepts.md'), source);
  await copyFile(source, path.join(directory, 'docs/zh-cn/concepts.md'));
  assert.throws(() => run(directory), /fallback/);
});

test('source-ready review records cannot substitute for their actual evidence', async t => {
  const directory = await fixture(t);
  const file = path.join(directory, 'docs/plans/evidence/unit-fixture/reviews.json');
  await writeFile(file, (await readFile(file, 'utf8')) + '\n');
  assert.throws(() => run(directory), /evidence digest changed/);
});
test('a source review cannot be refreshed by changing only the top-level hash', async t => {
  const directory = await fixture(t);
  const source = path.join(directory, 'docs/concepts.md');
  await writeFile(source, (await readFile(source, 'utf8')) + '\nChanged technical guidance.\n');
  const file = path.join(directory, 'website/content-reviews.json');
  const records = JSON.parse(await readFile(file, 'utf8'));
  const page = pages.find(page => page.id === 'concepts');
  for (const locale of localeIds) Object.assign(records.entries.concepts[locale], await reviewInputs(page, locale, directory));
  await writeFile(file, JSON.stringify(records));
  assert.throws(() => run(directory), /evidence used stale sourceHash/);
});

test('framework translations and dynamic quickstart sources invalidate only their affected inputs', async t => {
  const directory = await fixture(t);
  const quickstart = pages.find(page => page.id === 'quickstart');
  const article = pages.find(page => page.id === 'blog:your-first-anki-deck');
  const home = pages.find(page => page.type === 'home');
  const api = pages.find(page => page.id === 'api');
  const before = Object.fromEntries(await Promise.all([quickstart, article, home, api].map(async page => [page.id, await reviewInputs(page, 'zh-CN', directory)])));
  const framework = path.join(directory, 'website/src/content/i18n/zh-CN.json');
  const catalog = JSON.parse(await readFile(framework, 'utf8'));
  catalog['pagefind.searching'] = '正在查找“[SEARCH_TERM]”…';
  await writeFile(framework, JSON.stringify(catalog));
  assert.notEqual((await reviewInputs(quickstart, 'zh-CN', directory)).targetHash, before.quickstart.targetHash);
  assert.equal((await reviewInputs(home, 'zh-CN', directory)).targetHash, before.home.targetHash);
  const basic = path.join(directory, 'anki_forge/examples/target_api_basic.rs');
  await writeFile(basic, (await readFile(basic, 'utf8')) + '\n// Changed raw example.\n');
  assert.notEqual((await reviewInputs(quickstart, 'zh-CN', directory)).technicalDependencyHash, before.quickstart.technicalDependencyHash);
  assert.notEqual((await reviewInputs(article, 'zh-CN', directory)).technicalDependencyHash, before[article.id].technicalDependencyHash);
  assert.equal((await reviewInputs(api, 'zh-CN', directory)).technicalDependencyHash, before.api.technicalDependencyHash);
});

test('homepage review follows benchmark config, current CSV and converter without invalidating API pages', async t => {
  const directory = await fixture(t);
  const home = pages.find(page => page.type === 'home');
  const api = pages.find(page => page.id === 'api');
  const before = await reviewInputs(home, 'en', directory);
  const apiBefore = await reviewInputs(api, 'en', directory);
  const configPath = path.join(directory, 'docs/benchmark-presentation.json');
  const config = JSON.parse(await readFile(configPath, 'utf8'));
  const csv = await readFile(path.join(directory, config.dataFile), 'utf8');
  config.dataFile = 'docs/performance/20261009/comparison.csv';
  await mkdir(path.dirname(path.join(directory, config.dataFile)), { recursive: true });
  await writeFile(path.join(directory, config.dataFile), csv);
  await writeFile(configPath, JSON.stringify(config));
  assert.ok((await reviewInputFiles(home, 'en', directory)).includes(config.dataFile));
  const changedConfig = await reviewInputs(home, 'en', directory);
  assert.notEqual(changedConfig.technicalDependencyHash, before.technicalDependencyHash);
  await writeFile(path.join(directory, config.dataFile), csv + '\n');
  const changedCsv = await reviewInputs(home, 'en', directory);
  assert.notEqual(changedCsv.technicalDependencyHash, changedConfig.technicalDependencyHash);
  const converter = path.join(directory, 'website/scripts/benchmark-presentation.mjs');
  await writeFile(converter, (await readFile(converter, 'utf8')) + '\n// Changed presentation.\n');
  assert.notEqual((await reviewInputs(home, 'en', directory)).technicalDependencyHash, changedCsv.technicalDependencyHash);
  assert.deepEqual(await reviewInputs(api, 'en', directory), apiBefore);
});

test('explicit source directives and nested Astro templates belong to the review closure', async t => {
  const directory = await fixture(t);
  const page = pages.find(page => page.id === 'templates');
  const before = await reviewInputs(page, 'en', directory);
  const files = await reviewInputFiles(page, 'en', directory);
  const source = 'contracts/fixtures/template-bundle/custom-cloze/anki-template.yaml';
  assert.ok(files.includes(source));
  assert.ok(files.includes('website/site-model.mjs'));
  assert.ok(files.includes('website/locale-model.mjs'));
  assert.ok(files.includes('website/release-locales.json'));
  assert.ok(files.includes('website/src/components/ThemeControl.astro'));
  await writeFile(path.join(directory, source), (await readFile(path.join(directory, source), 'utf8')) + '\n# Changed source fixture.\n');
  const changedSource = await reviewInputs(page, 'en', directory);
  assert.notEqual(changedSource.technicalDependencyHash, before.technicalDependencyHash);
  const model = path.join(directory, 'website/site-model.mjs');
  await writeFile(model, (await readFile(model, 'utf8')) + '\n// Context-only comment.\n');
  assert.deepEqual(await reviewInputs(page, 'en', directory), changedSource, 'raw model text must not invalidate unrelated per-page metadata');
});

async function modelSamples(directory) {
  const ids = ['home', 'examples', 'blog', 'concepts', 'blog:your-first-anki-deck'];
  return Object.fromEntries(await Promise.all(ids.flatMap(id => localeIds.map(async locale =>
    [`${id}/${locale}`, await reviewInputs(pages.find(page => page.id === id), locale, directory)]))));
}

test('header and footer model data invalidate reviews across page types and destinations', async t => {
  const directory = await fixture(t);
  const model = path.join(directory, 'website/site-model.mjs');
  const original = await readFile(model, 'utf8');
  const before = await modelSamples(directory);
  await writeFile(model, original.replace("label: { en: 'Guide'", "label: { en: 'Browse the guide'"));
  const header = await modelSamples(directory);
  for (const key of Object.keys(before)) {
    if (key.endsWith('/en')) assert.notEqual(header[key].targetHash, before[key].targetHash, key);
    else assert.equal(header[key].targetHash, before[key].targetHash, 'English header wording is not Chinese target wording');
  }
  for (const [from, to] of [
    ["{ id: 'docs', label:", "{ id: 'languages', label:"],
    ["{ id: 'articles', pageId: 'blog'", "{ id: 'articles', pageId: 'examples'"],
    ["id: 'examples', route: '/examples/'", "id: 'examples', route: '/sample-decks/'"],
    ["id: 'home', route: '/'", "id: 'home', route: '/start/'"],
  ]) {
    assert.ok(original.includes(from));
    await writeFile(model, original.replace(from, to));
    const after = await modelSamples(directory);
    for (const key of Object.keys(before)) assert.notEqual(after[key].targetHash, before[key].targetHash, `${from}: ${key}`);
  }
});

test('current-page metadata uses the supplied root without invalidating unrelated page descriptions', async t => {
  const directory = await fixture(t);
  const model = path.join(directory, 'website/site-model.mjs');
  const original = await readFile(model, 'utf8');
  const before = await modelSamples(directory);
  const page = pages.find(page => page.id === 'concepts');
  for (const [field, value, locale] of [
    ['description', page.description.en, 'en'],
    ['zhDescription', page.description['zh-CN'], 'zh-CN'],
  ]) {
    const from = `"${field}": ${JSON.stringify(value)}`;
    assert.ok(original.includes(from));
    await writeFile(model, original.replace(from, `"${field}": "Changed current-page metadata in this supplied root."`));
    const after = await modelSamples(directory);
    for (const key of Object.keys(before)) {
      if (key === `concepts/${locale}`) assert.notEqual(after[key].targetHash, before[key].targetHash, key);
      else assert.equal(after[key].targetHash, before[key].targetHash, `unrelated target: ${key}`);
      if (locale === 'en' && key.startsWith('concepts/')) assert.notEqual(after[key].sourceHash, before[key].sourceHash, key);
      else assert.equal(after[key].sourceHash, before[key].sourceHash, `unrelated source: ${key}`);
    }
  }
});

test('sidebar and next-task model changes invalidate only the page types that render them', async t => {
  const directory = await fixture(t);
  const model = path.join(directory, 'website/site-model.mjs');
  const original = await readFile(model, 'utf8');
  const before = await modelSamples(directory);
  for (const [from, to, affected] of [
    ["'docs', 'languages', 'quickstart'", "'docs', 'quickstart', 'languages'", key => key.startsWith('concepts/')],
    ["zh: '从这里开始'", "zh: '开始使用'", key => key === 'concepts/zh-CN'],
    ['"title": "Choose your language"', '"title": "Select your SDK"', key => key === 'concepts/en'],
    ["{ pageId: 'json-to-anki', label: 'home.json'", "{ pageId: 'media', label: 'home.json'", key => key.startsWith('home/')],
  ]) {
    assert.ok(original.includes(from));
    await writeFile(model, original.replace(from, to));
    const after = await modelSamples(directory);
    for (const key of Object.keys(before)) {
      if (affected(key)) assert.notEqual(after[key].targetHash, before[key].targetHash, `${from}: ${key}`);
      else assert.equal(after[key].targetHash, before[key].targetHash, `${from}: unrelated ${key}`);
    }
  }
  await writeFile(model, original.replace('"description": "Install the public Rust, Node or Python package', '"description": "An unrelated description: install the public Rust, Node or Python package') + '\n// Context-only model comment.\n');
  assert.deepEqual(await modelSamples(directory), before, 'unrendered page descriptions and comments are not shared UI dependencies');
});

test('release language menus fingerprint active locale data without reading planned translations', async t => {
  const directory = await fixture(t);
  const model = path.join(directory, 'website/locale-model.mjs');
  const original = await readFile(model, 'utf8');
  const before = await modelSamples(directory);
  for (const [from, to] of [["label: '简体中文'", "label: '中文（简体）'"], ["prefix: '/zh-cn'", "prefix: '/chinese'"], ["lang: 'zh-CN'", "lang: 'zh-Hans'"]]) {
    assert.ok(original.includes(from));
    await writeFile(model, original.replace(from, to));
    const after = await modelSamples(directory);
    for (const key of Object.keys(before)) assert.notEqual(after[key].targetHash, before[key].targetHash, `${from}: ${key}`);
  }
  await writeFile(model, original.replace("label: 'Español'", "label: 'Español revisado'").replace("  es: {", "  it: { prefix: '/it', lang: 'it', label: 'Italiano', dir: 'ltr' },\n  es: {") + '\n// No planned-language source files exist in this fixture.\n');
  assert.deepEqual(await modelSamples(directory), before, 'inactive locale records do not change selected-language review inputs');
});

test('review model rejects untracked transformations and top-level data mutations', async t => {
  const directory = await fixture(t);
  const model = path.join(directory, 'website/site-model.mjs');
  const original = await readFile(model, 'utf8');
  const home = pages.find(page => page.id === 'home');
  for (const name of ['documents', 'common', 'navigation', 'footerNavigation', 'nextTasks']) {
    const start = original.indexOf(`const ${name} = [`);
    const end = original.indexOf('];', start);
    assert.ok(start >= 0 && end > start);
    await writeFile(model, original.slice(0, end) + '].map(item => ({ ...item, label: "Changed at render time" }));' + original.slice(end + 2));
    await assert.rejects(reviewInputs(home, 'en', directory), /must remain literal/);
  }
  await writeFile(model, original + '\nnavigation.push({ id: "docs", label: { en: "Unreviewed" } });\n');
  await assert.rejects(reviewInputs(home, 'en', directory), /must not mutate/);
  await writeFile(model, original + '\nif (navigation.length) navigation[0].label.en = "Unreviewed";\n');
  await assert.rejects(reviewInputs(home, 'en', directory), /must not mutate/);
  await writeFile(model, original);
  const before = await reviewInputs(home, 'zh-CN', directory);
  await writeFile(model, original + '\nconst unintendedNavigationMutation = (navigation[0].label["zh-CN"] = "新指南标签");\n');
  assert.notEqual((await reviewInputs(home, 'zh-CN', directory)).targetHash, before.targetHash, 'unprojected initializer logic belongs to the review fingerprint');
  await writeFile(model, original);
  const registry = path.join(directory, 'website/locale-model.mjs');
  await writeFile(registry, (await readFile(registry, 'utf8')) + '\nconfiguredLocales.en.label = "Unreviewed";\n');
  await assert.rejects(reviewInputs(home, 'en', directory), /must not mutate/);
});
