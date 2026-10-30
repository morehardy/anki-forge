// Independent technical delta probe: HEAD (extracted with git archive) versus the current working tree.
// Usage: node technical-locale-data-probe.mjs <extracted HEAD root> [working-tree root] [--dist <built dist> --baseline <HEAD manifest>]
// The HEAD root needs website/node_modules linked to the working tree's installed packages.
// The probe writes only to a disposable os.tmpdir() directory, which it removes before exiting.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { cp, mkdtemp, mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { registerHooks } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const argv = process.argv.slice(2);
const flag = name => { const index = argv.indexOf(name); if (index < 0) return undefined; const [, value] = argv.splice(index, 2); return path.resolve(value); };
const distRoot = flag('--dist'), baselineManifest = flag('--baseline');
const [headRoot, workRoot = fileURLToPath(new URL('../../../../', import.meta.url))] = argv.map(value => path.resolve(value));
if (!headRoot) throw new Error('Pass the extracted HEAD root');
const evidence = path.join(workRoot, 'docs/plans/evidence/multilingual-common-2026-10-27');
const sha = value => createHash('sha256').update(value).digest('hex');
const json = value => JSON.stringify(value);
const read = (root, file) => readFile(path.join(root, file), 'utf8');
const load = (root, file) => import(pathToFileURL(path.join(root, file)).href);
let checks = 0;
const equal = (actual, expected, message) => { assert.deepStrictEqual(actual, expected, message); checks++; };
const ok = (value, message) => { assert.ok(value, message); checks++; };

// astro.config.mjs is imported for its actual Starlight options; only the two framework factories are replaced.
registerHooks({ resolve(specifier, context, next) {
  if (specifier === '@astrojs/starlight') return { url: 'data:text/javascript,export default options => ({ name: "starlight-options-probe", options })', shortCircuit: true };
  if (specifier === 'astro/config') return { url: 'data:text/javascript,export const defineConfig = config => config', shortCircuit: true };
  return next(specifier, context);
} });

const side = async root => ({
  root,
  model: await load(root, 'website/site-model.mjs'),
  locales: await load(root, 'website/locale-model.mjs'),
  messages: await load(root, 'website/src/i18n/messages.mjs'),
  ai: await load(root, 'website/src/lib/ai-quickstart.mjs'),
  aiScript: await load(root, 'website/scripts/ai-quickstart.mjs'),
  sections: await load(root, 'website/scripts/content-sections.mjs'),
  reviews: await load(root, 'website/scripts/content-review-inputs.mjs'),
  astro: (await load(root, 'website/astro.config.mjs')).default,
  aiData: JSON.parse(await read(root, 'docs/ai-quickstart.json')),
  legacy: JSON.parse(await read(root, 'website/legacy-sections.json')),
});
const H = await side(headRoot);
const N = await side(workRoot);
const out = { headRoot: 'git archive HEAD (temporary extraction)', workRoot: 'working tree', results: {} };

// 0. Input bytes: the prior immutable snapshot is exactly HEAD, and only the expected candidate inputs differ from it.
const priorSnapshot = JSON.parse(await read(workRoot, 'docs/plans/evidence/multilingual-common-2026-10-27/technical-source-input-snapshot-stable-theme-scopes-34.json'));
const candidateFiles = new Set(Object.values(JSON.parse(await read(workRoot, 'docs/plans/evidence/multilingual-common-2026-10-27/source-review-locale-data-34.json')).entries).flatMap(entry => Object.values(entry).flatMap(record => record.inputFiles)));
const inputChanges = [];
for (const file of priorSnapshot.files) {
  equal(sha(await readFile(path.join(headRoot, file.path))), file.sha256, `${file.path}: prior snapshot equals HEAD`);
  equal(sha(await readFile(path.join(evidence, file.snapshotPath))), file.sha256, `${file.path}: immutable copy`);
  if (sha(await readFile(path.join(workRoot, file.path))) !== file.sha256) inputChanges.push(`${candidateFiles.has(file.path) ? 'input' : 'implementation'}:${file.path}`);
}
for (const file of candidateFiles) ok(priorSnapshot.files.some(item => item.path === file), `${file}: candidate input was in the prior snapshot`);
equal(inputChanges.sort(), ['implementation:website/astro.config.mjs', 'implementation:website/scripts/content-contract.test.mjs', 'implementation:website/scripts/content-review-inputs.mjs', 'input:docs/ai-quickstart.json', 'input:website/legacy-sections.json', 'input:website/site-model.mjs', 'input:website/src/i18n/messages.mjs', 'input:website/src/lib/ai-quickstart.mjs'], 'changed snapshot files');
out.results.inputs = { priorSnapshotFiles: priorSnapshot.files.length, candidateInputs: candidateFiles.size, changed: inputChanges };

// 1. Release locales and catalogs.
equal([...N.model.localeIds], [...H.model.localeIds], 'release locales');
const localeIds = [...N.model.localeIds];
equal(localeIds, ['en', 'zh-CN'], 'reviewed release locales');
equal(Object.keys(N.messages.catalogs), Object.keys(H.messages.catalogs), 'catalog key order');
for (const locale of localeIds) {
  equal(N.messages.catalogs[locale], H.messages.catalogs[locale], `${locale} catalog`);
  equal(Object.keys(N.messages.catalogs[locale]), Object.keys(H.messages.catalogs[locale]), `${locale} catalog key order`);
  const tH = H.messages.createTranslator(locale), tN = N.messages.createTranslator(locale);
  for (const [key, message] of Object.entries(N.messages.catalogs[locale])) {
    const params = { count: 3, language: 'Rust', version: '0.3.1', query: 'apkg', total: 7, current: 2, name: 'x', size: '1 KB', notes: 4, cards: 4, media: 3 };
    let a, b; try { a = tH(key, params); } catch (error) { a = `throws:${error.message}`; } try { b = tN(key, params); } catch (error) { b = `throws:${error.message}`; }
    equal(b, a, `${locale}/${key} translator output`);
    void message;
  }
}
out.results.catalogs = { locales: Object.keys(N.messages.catalogs), keys: Object.fromEntries(localeIds.map(locale => [locale, Object.keys(N.messages.catalogs[locale]).length])), sha256: sha(json(N.messages.catalogs)) };

// 2. Pages: every field except the three removed compatibility views is identical.
equal(N.model.pages.map(page => page.id), H.model.pages.map(page => page.id), 'page order');
const removed = {};
for (const [index, page] of N.model.pages.entries()) {
  const before = H.model.pages[index];
  const extra = Object.keys(before).filter(key => !Object.hasOwn(page, key));
  const added = Object.keys(page).filter(key => !Object.hasOwn(before, key));
  equal(added, [], `${page.id}: no new page fields`);
  const expectedRemoved = ['docs', 'article'].includes(page.type) ? ['source', 'en', 'zh'] : ['en', 'zh'];
  equal(extra.sort(), expectedRemoved.sort(), `${page.id}: removed compatibility fields`);
  equal(before.en, before.route, `${page.id}: HEAD en view was the route`);
  equal(before.zh, H.locales.localePath(before.route, 'zh-CN'), `${page.id}: HEAD zh view was the zh-CN path`);
  equal(before.source, before.sources.en, `${page.id}: HEAD source view was sources.en`);
  if (!['docs', 'article'].includes(page.type)) equal(page.source, before.source, `${page.id}: common page keeps its own source field`);
  const stripped = Object.fromEntries(Object.entries(before).filter(([key]) => Object.hasOwn(page, key)));
  equal(page, stripped, `${page.id}: all retained page fields`);
  for (const locale of localeIds) {
    equal(page.title?.[locale], before.title?.[locale], `${page.id}/${locale} title`);
    equal(page.description?.[locale], before.description?.[locale], `${page.id}/${locale} description`);
    equal(page.metadata[locale], before.metadata[locale], `${page.id}/${locale} metadata`);
    equal(N.model.pagePath(page.id, locale), H.model.pagePath(page.id, locale), `${page.id}/${locale} path`);
    equal(N.model.resolvePage(N.model.pagePath(page.id, locale), '/')?.page.id, page.id, `${page.id}/${locale} resolves`);
    equal(N.model.resolvePage(`/anki-forge${N.model.pagePath(page.id, locale)}`, '/anki-forge')?.locale, H.model.resolvePage(`/anki-forge${H.model.pagePath(page.id, locale)}`, '/anki-forge')?.locale, `${page.id}/${locale} prefixed resolution`);
  }
  for (const key of extra) removed[key] = (removed[key] ?? 0) + 1;
}
const consumers = ['docPages', 'importedDocs', 'translatedDocs', 'navigation', 'footerNavigation', 'nextTasks', 'homepageModules'];
for (const name of consumers) equal(N.model[name], ['docPages'].includes(name) ? H.model[name].map(page => Object.fromEntries(Object.entries(page).filter(([key]) => !['source', 'en', 'zh'].includes(key)))) : H.model[name], name);
equal(N.model.documentationEntries(), H.model.documentationEntries(), 'documentation entries (generated frontmatter title/description/source)');
out.results.pages = { pages: N.model.pages.length, pageLocales: N.model.pages.length * localeIds.length, removedCompatibilityFields: removed, metadataSha256: sha(json(N.model.pages.map(page => [page.id, page.metadata]))) };

// 3. Sidebar data and the actual Starlight options object built by astro.config.mjs.
equal(N.model.docGroups.map(group => ({ labels: group.labels, collapsed: group.collapsed, ids: group.ids })), H.model.docGroups.map(group => ({ labels: group.labels, collapsed: group.collapsed, ids: group.ids })), 'doc groups');
for (const [index, group] of H.model.docGroups.entries()) { equal(group.labels.en, group.label, 'HEAD label'); equal(group.labels['zh-CN'], group.zh, 'HEAD zh'); equal(Object.keys(N.model.docGroups[index]).sort(), Object.keys(group).filter(key => !['label', 'zh'].includes(key)).sort(), 'group fields'); }
const starlight = config => config.integrations.find(item => item.name === 'starlight-options-probe').options;
equal(starlight(N.astro).sidebar, starlight(H.astro).sidebar, 'Starlight sidebar');
equal(json(starlight(N.astro)), json(starlight(H.astro)), 'all Starlight options');
equal(json({ ...N.astro, integrations: [] }), json({ ...H.astro, integrations: [] }), 'Astro configuration');
out.results.sidebar = { groups: starlight(N.astro).sidebar.map(group => ({ label: group.label, translations: group.translations, collapsed: group.collapsed, items: group.items.length })), sha256: sha(json(starlight(N.astro).sidebar)) };

// 4. AI prompt rendering for every SDK, locale and request form.
const { requestPlaceholders, promptContracts, ...rest } = N.aiData;
equal(rest, H.aiData, 'AI quickstart data apart from the two new maps');
equal(requestPlaceholders, { en: '<describe your task>', 'zh-CN': '<描述你的需求>' }, 'placeholders equal the former hard-coded values');
equal(promptContracts, {
  en: { required: ['published', 'public API', 'persistent', 'stable namespace and note keys', 'run command', 'verification', 'original distribution APKG'], maxWords: 500 },
  'zh-CN': { required: ['公开 API', '持久化', 'namespace 和 note key', '运行命令', '验证步骤', '原始分发 APKG'], maxCharacters: 700 },
}, 'contracts equal the former hard-coded regex literals and limits');
const requests = ['', ' \n\t ', '__ANKI_FORGE_TASK_INPUT__', 'Build a Spanish vocabulary deck from cards.csv', '用 CSV 生成西班牙语牌组', 'Keep {request} and $& $1 literal'];
let prompts = 0;
const promptHashes = {};
for (const language of Object.keys(N.aiData.languages)) for (const locale of localeIds) for (const request of requests) {
  const after = N.ai.renderPrompt(N.aiData, language, request, locale);
  equal(after, H.ai.renderPrompt(H.aiData, language, request, locale), `${language}/${locale}/${json(request)} prompt`);
  if (!request.trim()) ok(after.includes(requestPlaceholders[locale]), 'placeholder rendered');
  promptHashes[`${language}/${locale}/${requests.indexOf(request)}`] = sha(after);
  prompts++;
}
for (const locale of localeIds) equal(N.aiScript.renderReadme(N.aiData, locale), H.aiScript.renderReadme(H.aiData, locale), `${locale} README AI block`);
assert.throws(() => N.ai.renderPrompt(N.aiData, 'rust', '', 'es')); checks++;
equal(json(await N.aiScript.loadAiQuickstart(workRoot)), json(N.aiData), 'new loader accepts current data');
equal(json(await H.aiScript.loadAiQuickstart(headRoot)), json(H.aiData), 'HEAD loader accepts HEAD data');
await N.aiScript.checkAiQuickstart({ directory: workRoot }); checks++;
await H.aiScript.checkAiQuickstart({ directory: headRoot }); checks++;
out.results.prompts = { rendered: prompts, sdks: Object.keys(N.aiData.languages), requests: requests.length, sha256: sha(json(promptHashes)) };

// 5. Section anchors: the exact HEAD call versus the new locale-keyed lookup for every docs/article page.
equal(Object.keys(N.legacy), ['zh-CN'], 'legacy aliases are keyed by the publishing locale');
equal(N.legacy['zh-CN'], H.legacy, 'legacy alias content unchanged');
let mappings = 0, aliases = 0;
const languageSwitch = {};
const generated = {};
for (const page of N.model.pages.filter(page => ['docs', 'article'].includes(page.type))) {
  generated[page.id] = { H: {}, N: {} };
  for (const locale of localeIds) {
    const source = await read(workRoot, page.sources[locale]);
    equal(source, await read(headRoot, page.sources[locale]), `${page.id}/${locale} source bytes`);
    const before = H.sections.sectionMapping(page.id, locale, source, locale === 'zh-CN' ? H.legacy[page.id] : {});
    const lookup = N.sections.legacyAliases(N.legacy, page.id, locale);
    equal(lookup, locale === 'zh-CN' ? H.legacy[page.id] ?? {} : {}, `${page.id}/${locale} alias lookup`);
    const after = N.sections.sectionMapping(page.id, locale, source, lookup);
    equal(after, before, `${page.id}/${locale} section mapping`);
    equal(N.sections.addSectionAliases(source, after), H.sections.addSectionAliases(source, before), `${page.id}/${locale} generated alias anchors`);
    generated[page.id].H[locale] = before; generated[page.id].N[locale] = after;
    aliases += Object.keys(lookup).length; mappings++;
  }
  for (const locale of localeIds) for (const target of localeIds.filter(other => other !== locale)) {
    const view = side => ({ mapping: generated[page.id][side][locale], targetIds: Object.entries(generated[page.id][side][target]).flat() });
    equal(view('N'), view('H'), `${page.id}/${locale} language switch data`);
    languageSwitch[`${page.id}/${locale}`] = sha(json(view('N')));
  }
}
assert.throws(() => N.sections.legacyAliases({ quickstart: {} }, 'quickstart', 'zh-CN'), /Unknown legacy section locale/); checks++;
out.results.sections = { pageLocales: mappings, aliasesApplied: aliases, sha256: sha(json(languageSwitch)) };

// 6. Review fingerprints: HEAD code on HEAD bytes reproduces the prior report; new code on current bytes reproduces the candidate.
const before = JSON.parse(await read(workRoot, 'docs/plans/evidence/multilingual-common-2026-10-27/technical-source-review-before-locale-data.json'));
const candidate = JSON.parse(await read(workRoot, 'docs/plans/evidence/multilingual-common-2026-10-27/source-review-locale-data-34.json'));
const headRecords = JSON.parse(await read(headRoot, 'website/content-reviews.json'));
const pick = value => ({ sourceHash: value.sourceHash, targetHash: value.targetHash, technicalDependencyHash: value.technicalDependencyHash });
let records = 0;
const changed = { sourceHash: 0, targetHash: 0, technicalDependencyHash: 0 };
for (const page of N.model.pages) for (const locale of localeIds) {
  const headPage = H.model.pages.find(item => item.id === page.id);
  const oldHashes = await H.reviews.reviewInputs(headPage, locale, headRoot);
  equal(oldHashes, pick(before.entries[page.id][locale]), `${page.id}/${locale} HEAD inputs equal prior report`);
  equal(oldHashes, pick(headRecords.entries[page.id][locale]), `${page.id}/${locale} HEAD inputs equal HEAD content-reviews.json`);
  const newHashes = await N.reviews.reviewInputs(page, locale, workRoot);
  equal(newHashes, pick(candidate.entries[page.id][locale]), `${page.id}/${locale} current inputs equal candidate`);
  equal(await N.reviews.reviewInputFiles(page, locale, workRoot), candidate.entries[page.id][locale].inputFiles, `${page.id}/${locale} candidate input files`);
  equal(await H.reviews.reviewInputFiles(headPage, locale, headRoot), before.entries[page.id][locale].inputFiles, `${page.id}/${locale} prior input files`);
  for (const key of Object.keys(changed)) if (oldHashes[key] !== newHashes[key]) changed[key]++;
  records++;
}
out.results.fingerprints = { records, changed };

// 7. Projected review data: identical apart from the implementation text and the removed sidebar transform string.
const temporary = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-locale-data-probe-'));
try {
  const projection = async (root, label) => {
    const source = await read(root, 'website/scripts/content-review-inputs.mjs');
    const patched = source
      .replace("from 'typescript'", `from ${json(pathToFileURL(path.join(workRoot, 'website/node_modules/typescript/lib/typescript.js')).href)}`)
      .replace("from './content-sections.mjs'", `from ${json(pathToFileURL(path.join(root, 'website/scripts/content-sections.mjs')).href)}`)
      + '\nexport { renderedModelInputs };\n';
    const file = path.join(temporary, `${label}.mjs`);
    await writeFile(file, patched);
    return (await import(pathToFileURL(file).href)).renderedModelInputs;
  };
  const projectH = await projection(headRoot, 'head'), projectN = await projection(workRoot, 'work');
  const implementationDiff = new Set();
  for (const page of N.model.pages) for (const locale of localeIds) {
    const headPage = H.model.pages.find(item => item.id === page.id);
    const filesFor = async (root, item) => Object.fromEntries(await Promise.all((await (root === headRoot ? H : N).reviews.reviewInputFiles(item, locale, root)).map(async name => [name, await read(root, name)])));
    const a = projectH(await filesFor(headRoot, headPage), headPage, locale), b = projectN(await filesFor(workRoot, page), page, locale);
    const strip = value => { const { implementation, ...data } = value; if (data.sidebar) { const { transform, ...sidebar } = data.sidebar; data.sidebar = sidebar; } return data; };
    equal(strip(b), strip(a), `${page.id}/${locale} projected metadata/navigation/sidebar/footer/locale data`);
    equal(b.implementation.locales, a.implementation.locales, `${page.id}/${locale} locale-model implementation`);
    for (const statement of b.implementation.site) if (!a.implementation.site.includes(statement)) implementationDiff.add(statement.slice(0, 60));
    for (const statement of a.implementation.site) if (!b.implementation.site.includes(statement)) implementationDiff.add(`HEAD: ${statement.slice(0, 54)}`);
  }
  out.results.projection = { records: N.model.pages.length * localeIds.length, implementationStatementsChanged: [...implementationDiff] };

  // 8. Mutation sensitivity of the new fingerprint, on a disposable copy of every candidate input file.
  const fixture = path.join(temporary, 'fixture');
  for (const file of new Set(Object.values(candidate.entries).flatMap(entry => Object.values(entry).flatMap(record => record.inputFiles)))) {
    await mkdir(path.dirname(path.join(fixture, file)), { recursive: true });
    await cp(path.join(workRoot, file), path.join(fixture, file));
  }
  const sample = async () => Object.fromEntries(await Promise.all(N.model.pages.flatMap(page => localeIds.map(async locale => [`${page.id}/${locale}`, await N.reviews.reviewInputs(page, locale, fixture)]))));
  const baseline = await sample();
  equal(Object.fromEntries(Object.entries(baseline).map(([key, value]) => [key, value])), Object.fromEntries(Object.entries(candidate.entries).flatMap(([id, entry]) => Object.entries(entry).map(([locale, record]) => [`${id}/${locale}`, pick(record)]))), 'fixture reproduces candidate');
  const mutations = [];
  const mutate = async (name, file, change, expected) => {
    const original = await readFile(path.join(fixture, file), 'utf8');
    const next = change(original);
    assert.notEqual(next, original, `${name}: mutation applied`);
    await writeFile(path.join(fixture, file), next);
    const after = await sample();
    const affected = Object.entries(after).flatMap(([key, value]) => Object.keys(value).filter(hash => value[hash] !== baseline[key][hash]).map(hash => `${key}:${hash}`)).sort();
    equal(affected, [...expected].sort(), `${name}: affected records`);
    mutations.push({ name, affected: affected.length });
    await writeFile(path.join(fixture, file), original);
  };
  const docsIds = N.model.pages.filter(page => page.type === 'docs').map(page => page.id);
  const all = N.model.pages.map(page => page.id);
  const records = (ids, locales, hashes) => ids.flatMap(id => locales.flatMap(locale => hashes.map(hash => `${id}/${locale}:${hash}`)));
  // A document title is the page title and a sidebar item for every docs page in that locale; zh-CN records also hash the en source model.
  await mutate('zh-CN document title', 'website/site-model.mjs', text => text.replace('"zh-CN": "核心概念"', '"zh-CN": "核心概念（改）"'), records(docsIds, ['zh-CN'], ['targetHash']));
  await mutate('en document title', 'website/site-model.mjs', text => text.replace('"en": "Core concepts"', '"en": "Core concepts (changed)"'), [...records(docsIds, ['en'], ['sourceHash', 'targetHash']), ...records(docsIds, ['zh-CN'], ['sourceHash'])]);
  await mutate('zh-CN document description', 'website/site-model.mjs', text => text.replace('"zh-CN": "理解 Project、拥有所有权的笔记与媒体、稳定键、文本和更新身份。"', '"zh-CN": "改动的描述。"'), records(['concepts'], ['zh-CN'], ['targetHash']));
  await mutate('en document description', 'website/site-model.mjs', text => text.replace('"en": "Projects, owned notes and media, stable keys, text and update identity."', '"en": "Changed description."'), [...records(['concepts'], ['en'], ['sourceHash', 'targetHash']), ...records(['concepts'], ['zh-CN'], ['sourceHash'])]);
  await mutate('zh-CN sidebar group label', 'website/site-model.mjs', text => text.replace("'zh-CN': '参考'", "'zh-CN': '参考资料'"), records(docsIds, ['zh-CN'], ['targetHash']));
  await mutate('sidebar collapsed flag', 'website/site-model.mjs', text => text.replace("'zh-CN': '参考' }, collapsed: true", "'zh-CN': '参考' }, collapsed: false"), [...records(docsIds, ['en'], ['sourceHash', 'targetHash']), ...records(docsIds, ['zh-CN'], ['sourceHash', 'targetHash'])]);
  await mutate('zh-CN navigation label', 'website/site-model.mjs', text => text.replace("'zh-CN': '示例'", "'zh-CN': '示例集'"), records(all, ['zh-CN'], ['targetHash']));
  await mutate('en navigation label', 'website/site-model.mjs', text => text.replace("en: 'Examples'", "en: 'Samples'"), [...records(all, ['en'], ['sourceHash', 'targetHash']), ...records(all, ['zh-CN'], ['sourceHash'])]);
  await mutate('zh-CN legacy alias', 'website/legacy-sections.json', text => text.replace('"生成-apkg": "export-your-first-deck"', '"生成-apkg": "install"'), records(['quickstart'], ['zh-CN'], ['technicalDependencyHash']));
  await mutate('unreleased-locale legacy aliases', 'website/legacy-sections.json', text => text.replace('{\n  "zh-CN"', '{\n  "es": {"quickstart": {"instalar": "install"}},\n  "zh-CN"'), []);
  await mutate('comment-only model edit', 'website/site-model.mjs', text => `${text}\n// Comment-only edit.\n`, []);
  await mutate('request placeholder data', 'docs/ai-quickstart.json', text => text.replace('"zh-CN": "<描述你的需求>"', '"zh-CN": "<请描述需求>"'), records(all, ['en', 'zh-CN'], ['technicalDependencyHash']));
  for (const [name, file, change, error] of [
    ['old flat legacy format', 'website/legacy-sections.json', text => JSON.stringify(JSON.parse(text)['zh-CN']), /Unknown legacy section locale/],
    ['docGroups transform', 'website/site-model.mjs', text => text.replace(/\n\];\n\/\*\* @type \{\{ id: string, label/, "\n].map(group => ({ ...group }));\n/** @type {{ id: string, label"), /must remain literal/],
    ['missing zh-CN group label', 'website/site-model.mjs', text => text.replace("{ labels: { en: 'Help', 'zh-CN': '帮助' }", "{ labels: { en: 'Help' }"), /Missing review sidebar label: zh-CN/],
  ]) {
    const original = await readFile(path.join(fixture, file), 'utf8');
    const next = change(original);
    assert.notEqual(next, original, `${name}: mutation applied`);
    await writeFile(path.join(fixture, file), next);
    const page = name.includes('legacy') ? N.model.pages.find(item => item.id === 'quickstart') : N.model.pages.find(item => item.id === 'concepts');
    await assert.rejects(N.reviews.reviewInputs(page, 'zh-CN', fixture), error); checks++;
    mutations.push({ name, rejected: true });
    await writeFile(path.join(fixture, file), original);
  }
  out.results.mutations = mutations;
} finally {
  await rm(temporary, { recursive: true, force: true });
}

// 9. Optional artifact comparison: a preview build of the working tree against the HEAD-input release manifest.
if (distRoot && baselineManifest) {
  const manifest = JSON.parse(await readFile(baselineManifest, 'utf8'));
  const expected = new Map([...manifest.pages, ...manifest.resources].map(entry => [entry.route, entry]));
  const walk = async directory => (await Promise.all((await readdir(directory, { withFileTypes: true })).map(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : path.join(directory, entry.name)))).flat();
  const actual = new Map();
  for (const file of (await walk(distRoot)).sort()) {
    const relative = path.relative(distRoot, file).split(path.sep).join('/');
    if (relative === 'generated/site-manifest.json' || relative === 'CNAME' || relative.startsWith('.')) continue;
    actual.set(`${manifest.configuration.base}/${relative.replace(/index\.html$/, '')}`, { relative, bytes: await readFile(file) });
  }
  equal([...actual.keys()].sort(), [...expected.keys()].sort(), 'artifact routes');
  const differ = [...actual].filter(([route, value]) => sha(value.bytes) !== expected.get(route).sha256).map(([, value]) => value.relative).sort();
  const volatile = relative => /^generated\/[^/]+\.apkg$/.test(relative) || ['generated/showcase.json', 'pagefind/pagefind-entry.json'].includes(relative);
  equal(differ.filter(relative => !volatile(relative)), [], 'only known volatile artifacts differ');
  const html = [...actual.values()].filter(value => value.relative.endsWith('.html'));
  equal(html.filter(value => differ.includes(value.relative)).length, 0, 'every HTML page is byte-identical');
  const resource = relative => expected.get(`${manifest.configuration.base}/${relative}`);
  // Pagefind writes the same language records in either order; reversing ours reproduces the baseline when it differs.
  const entry = JSON.parse(actual.get(`${manifest.configuration.base}/pagefind/pagefind-entry.json`).bytes.toString());
  const reordered = JSON.stringify({ ...entry, languages: Object.fromEntries(Object.entries(entry.languages).reverse()) });
  ok([sha(JSON.stringify(entry)), sha(reordered)].includes(resource('pagefind/pagefind-entry.json').sha256), 'pagefind entry equals baseline up to language order');
  // showcase.json embeds generated APKG identities and the commit; substituting the baseline's reproduces it exactly.
  const showcase = JSON.parse(actual.get(`${manifest.configuration.base}/generated/showcase.json`).bytes.toString());
  for (const item of [...showcase.examples, showcase.combined, ...showcase.update.artifacts]) Object.assign(item, { bytes: resource(`generated/${item.file}`).bytes, sha256: resource(`generated/${item.file}`).sha256 });
  showcase.sourceCommit = manifest.commit;
  equal(sha(`${JSON.stringify(showcase, null, 2)}\n`), resource('generated/showcase.json').sha256, 'showcase metadata equals baseline apart from APKG identities and commit');
  out.results.artifacts = { baselineCommit: manifest.commit, baselineInputSha256: manifest.inputSha256, files: actual.size, html: html.length, identical: actual.size - differ.length, differ };
}
out.checks = checks;
out.passed = true;
console.log(JSON.stringify(out, null, 2));
