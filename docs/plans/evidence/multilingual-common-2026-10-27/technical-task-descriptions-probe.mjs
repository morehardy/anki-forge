// Independent technical delta probe for the task-description change: d9f5354 (last fully reviewed) versus HEAD 109c254.
// Usage:
//   node technical-task-descriptions-probe.mjs <d9f5354 root> --before-dist <dir> --dist <dir> [--rebuild-dist <dir>]
//        --python <python with the published ankiforge 0.3.1> --node-package <published ankiforge 0.3.1 package directory>
// <d9f5354 root> is `git archive d9f5354` extracted with its own website/node_modules. The dists are
// CONTENT_REVIEW_MODE=preview builds of d9f5354 and of the working tree with the default (prefix-origin) site configuration.
// The probe reads only; it writes runtime scratch files to a disposable os.tmpdir() directory and removes it before exiting.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { createRequire, registerHooks } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { gunzipSync } from 'node:zlib';

const argv = process.argv.slice(2);
const flag = name => { const index = argv.indexOf(name); if (index < 0) return undefined; const [, value] = argv.splice(index, 2); return path.resolve(value); };
const beforeDist = flag('--before-dist'), dist = flag('--dist'), rebuildDist = flag('--rebuild-dist'), python = flag('--python'), nodePackage = flag('--node-package');
const [beforeRoot, workRoot = fileURLToPath(new URL('../../../../', import.meta.url))] = argv.map(value => path.resolve(value));
assert.ok(beforeRoot && beforeDist && dist && python && nodePackage, 'missing arguments');
const E = 'docs/plans/evidence/multilingual-common-2026-10-27';
const sha = value => createHash('sha256').update(value).digest('hex');
const json = value => JSON.stringify(value);
const read = (root, file) => readFile(path.join(root, file), 'utf8');
const load = (root, file) => import(pathToFileURL(path.join(root, file)).href);
const git = (...args) => execFileSync('git', args, { cwd: workRoot, maxBuffer: 1 << 28 });
let checks = 0;
const equal = (actual, expected, message) => { assert.deepStrictEqual(actual, expected, message); checks++; };
const ok = (value, message) => { assert.ok(value, message); checks++; };
const out = { beforeRoot: 'git archive d9f5354 (temporary extraction)', workRoot: 'working tree at HEAD 109c254', results: {} };

registerHooks({ resolve(specifier, context, next) {
  if (specifier === '@astrojs/starlight') return { url: 'data:text/javascript,export default options => ({ name: "starlight-options-probe", options })', shortCircuit: true };
  if (specifier === 'astro/config') return { url: 'data:text/javascript,export const defineConfig = config => config', shortCircuit: true };
  return next(specifier, context);
} });
const side = async root => ({
  model: await load(root, 'website/site-model.mjs'),
  sections: await load(root, 'website/scripts/content-sections.mjs'),
  reviews: await load(root, 'website/scripts/content-review-inputs.mjs'),
  astro: (await load(root, 'website/astro.config.mjs')).default,
  legacy: JSON.parse(await read(root, 'website/legacy-sections.json')),
  contracts: JSON.parse(await read(root, 'website/section-contracts.json')),
});
const H = await side(beforeRoot);
const N = await side(workRoot);
const links = await load(workRoot, 'website/scripts/content-links.mjs');
const { siteConfig } = await load(workRoot, 'website/site.config.mjs');
const { load: cheerio } = createRequire(path.join(workRoot, 'website/package.json'))('cheerio');
const localeIds = [...N.model.localeIds];
equal(localeIds, ['en', 'zh-CN'], 'release locales');
equal([...H.model.localeIds], localeIds, 'release locales unchanged');

// ---------------------------------------------------------------------------------------------------------------
// 0. Inputs: the extraction is d9f5354, the prior immutable snapshot equals it, and exactly 16 candidate inputs changed.
const EXPECTED_CHANGED = ['docs/cards.md', 'docs/image-occlusion.md', 'docs/media.md', 'docs/node/quick-start.md', 'docs/python/quick-start.md', 'docs/template-bundles.md', 'docs/updates.md', 'docs/zh-cn/cards.md', 'docs/zh-cn/image-occlusion.md', 'docs/zh-cn/media.md', 'docs/zh-cn/node-quickstart.md', 'docs/zh-cn/python-quickstart.md', 'docs/zh-cn/templates.md', 'docs/zh-cn/updates.md', 'website/site-model.mjs', 'website/src/content/docs/docs/quickstart.mdx'];
const priorSnapshot = JSON.parse(await read(workRoot, `${E}/technical-source-input-snapshot-locale-data-34.json`));
const snapshot = JSON.parse(await read(workRoot, `${E}/technical-source-input-snapshot-task-descriptions-34.json`));
const candidateBytes = await readFile(path.join(workRoot, `${E}/source-review-task-descriptions-34.json`));
const beforeReportBytes = await readFile(path.join(workRoot, `${E}/technical-source-review-before-task-descriptions.json`));
equal(sha(candidateBytes), 'c90a215e990eb6e1d4b5b40ab95e725b9d02adad9531ff93dff4a2972fc2be4e', 'candidate bytes');
equal(sha(beforeReportBytes), 'e7567230ef8d3111efc85f32dce12621811fa359ed646c9e7783c410f8acaa1a', 'prior cumulative report copy bytes');
equal(sha(await readFile(path.join(workRoot, `${E}/technical-source-input-snapshot-locale-data-34.json`))), '4d3607950df598a2aa6eb3ff9e6f9fbf03ca11285e3250c1cf7b0b52b0b03105', 'prior snapshot bytes');
const candidate = JSON.parse(candidateBytes), before = JSON.parse(beforeReportBytes);
const candidateFiles = [...new Set(Object.values(candidate.entries).flatMap(entry => Object.values(entry).flatMap(record => record.inputFiles)))].sort();
equal(candidateFiles, priorSnapshot.files.map(file => file.path).sort(), 'candidate input set equals the prior snapshot set');
const changedInputs = [];
for (const file of priorSnapshot.files) {
  const atCommit = git('show', `d9f5354:${file.path}`);
  equal(sha(await readFile(path.join(beforeRoot, file.path))), sha(atCommit), `${file.path}: extraction equals the d9f5354 blob`);
  equal(sha(atCommit), file.sha256, `${file.path}: prior snapshot equals d9f5354`);
  equal(sha(await readFile(path.join(workRoot, E, file.snapshotPath))), file.sha256, `${file.path}: prior immutable copy`);
  if (sha(await readFile(path.join(workRoot, file.path))) !== file.sha256) changedInputs.push(file.path);
}
equal(changedInputs.sort(), EXPECTED_CHANGED, 'exactly the 16 expected candidate inputs changed');
const implementationChanged = [], implementationCommittedAfterSnapshot = [];
for (const file of priorSnapshot.implementationFiles) {
  const atCommit = sha(git('show', `d9f5354:${file.path}`));
  equal(sha(await readFile(path.join(beforeRoot, file.path))), atCommit, `${file.path}: extraction equals the d9f5354 blob`);
  if (atCommit !== file.sha256) implementationCommittedAfterSnapshot.push(file.path);
  if (sha(await readFile(path.join(workRoot, file.path))) !== atCommit) implementationChanged.push(file.path);
}
equal(implementationChanged, [], 'no prior-snapshot implementation file changed d9f5354..HEAD');
// website/README.md was edited in 166abec after the locale-data snapshot: it now names language-briefs.json (that audit's third minor issue).
equal(implementationCommittedAfterSnapshot, ['website/README.md'], 'only website/README.md moved between the locale-data snapshot and d9f5354');
ok((await read(beforeRoot, 'website/README.md')).includes('its `language-briefs.json` entry'), 'website/README.md at d9f5354 lists language-briefs.json among locale data');
equal(snapshot.files.map(file => file.path), candidateFiles, 'new snapshot lists exactly the candidate inputs');
for (const file of [...snapshot.files, ...snapshot.supportingFiles]) {
  const bytes = await readFile(path.join(workRoot, file.path));
  equal(sha(bytes), file.sha256, `${file.path}: new snapshot equals the working tree`);
  equal(sha(await readFile(path.join(workRoot, E, file.snapshotPath))), file.sha256, `${file.path}: new immutable copy`);
}
for (const file of snapshot.supportingFiles) equal(sha(git('show', `d9f5354:${file.path}`)), file.d9f5354Sha256, `${file.path}: recorded d9f5354 blob`);
equal(snapshot.changedVsPriorSnapshot.map(item => item.path).sort(), EXPECTED_CHANGED, 'snapshot change list');
const committedDiff = git('diff', '--name-only', 'd9f5354', 'HEAD').toString().trim().split('\n').sort();
equal(committedDiff, ['.gitattributes', 'README.md', 'README.zh-CN.md', 'anki_forge/README.md', 'bindings/node/README.md', 'bindings/python/README.md', ...EXPECTED_CHANGED, 'docs/plans/2026-10-27-anki-search-discovery-implementation.md'].sort(), 'files changed d9f5354..HEAD');
equal(git('status', '--porcelain', '--', ...candidateFiles, ...snapshot.supportingFiles.map(file => file.path)).toString(), '', 'reviewed paths have no uncommitted changes');
out.results.inputs = { priorSnapshotFiles: priorSnapshot.files.length, candidateInputs: candidateFiles.length, changed: changedInputs, unchanged: candidateFiles.length - changedInputs.length, implementationFilesChanged: implementationChanged.length, implementationCommittedAfterPriorSnapshot: implementationCommittedAfterSnapshot, committedDiff };

// ---------------------------------------------------------------------------------------------------------------
// 1. Fingerprints: d9f5354 code on d9f5354 bytes reproduces the prior report; current code on current bytes reproduces the candidate.
const regenerated = execFileSync(process.execPath, [path.join(workRoot, 'website/scripts/content-review-candidates.mjs')], { cwd: path.join(workRoot, 'website'), maxBuffer: 1 << 28 });
equal(sha(regenerated), sha(candidateBytes), 'candidate regenerated to stdout is byte-identical');
const recordedAtBefore = JSON.parse(await read(beforeRoot, 'website/content-reviews.json'));
equal(sha(await readFile(path.join(workRoot, 'website/content-reviews.json'))), sha(await readFile(path.join(beforeRoot, 'website/content-reviews.json'))), 'content-reviews.json unchanged d9f5354..HEAD');
const pick = value => ({ sourceHash: value.sourceHash, targetHash: value.targetHash, technicalDependencyHash: value.technicalDependencyHash });
const stale = [], current = [];
const changedHashes = { sourceHash: 0, targetHash: 0, technicalDependencyHash: 0 };
for (const page of N.model.pages) for (const locale of localeIds) {
  const beforePage = H.model.pages.find(item => item.id === page.id);
  const oldHashes = await H.reviews.reviewInputs(beforePage, locale, beforeRoot);
  equal(oldHashes, pick(before.entries[page.id][locale]), `${page.id}/${locale}: d9f5354 inputs reproduce the prior report`);
  equal(oldHashes, pick(recordedAtBefore.entries[page.id][locale]), `${page.id}/${locale}: d9f5354 inputs equal the recorded content-reviews.json`);
  const newHashes = await N.reviews.reviewInputs(page, locale, workRoot);
  equal(newHashes, pick(candidate.entries[page.id][locale]), `${page.id}/${locale}: current inputs reproduce the candidate`);
  const files = await N.reviews.reviewInputFiles(page, locale, workRoot);
  equal(files, candidate.entries[page.id][locale].inputFiles, `${page.id}/${locale}: candidate input files`);
  equal(await H.reviews.reviewInputFiles(beforePage, locale, beforeRoot), before.entries[page.id][locale].inputFiles, `${page.id}/${locale}: prior input files`);
  equal(files, before.entries[page.id][locale].inputFiles, `${page.id}/${locale}: input file list unchanged`);
  let differs = false;
  for (const key of Object.keys(changedHashes)) if (oldHashes[key] !== newHashes[key]) { changedHashes[key]++; differs = true; }
  (differs ? stale : current).push(`${page.id}/${locale}`);
}
const docsIds = N.model.pages.filter(page => page.type === 'docs').map(page => page.id);
equal(stale, docsIds.flatMap(id => localeIds.map(locale => `${id}/${locale}`)), 'stale records are exactly every docs page in both locales');
equal(current, ['home', 'examples', 'blog', 'blog:your-first-anki-deck', 'blog:stable-ids-and-deck-updates'].flatMap(id => localeIds.map(locale => `${id}/${locale}`)), 'the 10 home/examples/blog/article records are unchanged');
equal(changedHashes, { sourceHash: 58, targetHash: 58, technicalDependencyHash: 0 }, 'hash change counts');
out.results.fingerprints = { records: stale.length + current.length, stale: stale.length, current, changed: changedHashes };

// ---------------------------------------------------------------------------------------------------------------
// 2. Site model: only the eight intended titles/descriptions changed; groups, navigation and routes are identical.
const NEW = {
  quickstart: { title: { en: 'Generate Anki flashcards with Rust', 'zh-CN': '用 Rust 生成 Anki 卡片' } },
  'node-quickstart': { title: { en: 'Generate Anki flashcards with Node.js', 'zh-CN': '用 Node.js 生成 Anki 卡片' }, description: { en: 'Install ankiforge from npm, generate Anki flashcards with Node.js or TypeScript, and export an APKG deck you can import into Anki.', 'zh-CN': '从 npm 安装 ankiforge，用 Node.js 或 TypeScript 生成 Anki 卡片，并导出可导入 Anki 的 APKG 牌组。' } },
  'python-quickstart': { title: { en: 'Generate Anki flashcards with Python', 'zh-CN': '用 Python 生成 Anki 卡片' }, description: { en: 'Install ankiforge from PyPI, generate Anki flashcards with a short Python script, and export an APKG deck you can import into Anki.', 'zh-CN': '从 PyPI 安装 ankiforge，用简短的 Python 脚本生成 Anki 卡片，并导出可导入 Anki 的 APKG 牌组。' } },
  templates: { description: { en: 'Load a reusable Anki note type from one bundle of card templates, CSS and media, for normal or Cloze cards, with Rust, Python or Node.js.', 'zh-CN': '从一个模板包加载可复用的 Anki 笔记类型，包含卡片模板、CSS 和媒体，支持普通与 Cloze 卡片，可使用 Rust、Python 或 Node.js。' } },
  cards: { description: { en: 'Generate Anki Basic question-and-answer and Cloze fill-in-the-blank cards from code. A verified Rust example builds two notes and three cards.', 'zh-CN': '用代码批量生成 Anki 问答（Basic）卡片与 Cloze 填空卡片。经过验证的 Rust 示例用两条笔记生成三张卡片。' } },
  media: { description: { en: 'Add images and audio to generated Anki cards with Rust, Python or Node.js. Import files, compose card content, and manage export names, MIME types and budgets.', 'zh-CN': '用 Rust、Python 或 Node.js 为生成的 Anki 卡片添加图片和音频：导入文件、组合卡片内容，并管理导出名称、MIME 类型与媒体预算。' } },
  'image-occlusion': { description: { en: 'Create Anki Image Occlusion cards programmatically from an image and the rectangle masks you supply, in either occlusion mode, with Rust, Python or Node.js.', 'zh-CN': '用代码批量生成 Anki 图片遮挡卡片：提供图片和矩形遮罩坐标，支持两种遮挡模式，可使用 Rust、Python 或 Node.js。' } },
  updates: { description: { en: 'Release a new version of a generated Anki deck: keep stable note IDs, build from the previous original APKG and review the comparison before publishing.', 'zh-CN': '发布生成牌组的新版本：保持稳定笔记 ID，以上一版原始 APKG 为基线构建，并在分发前检查比较结果。' } },
};
const paths = (a, b, prefix = '') => {
  if (Object.is(a, b)) return [];
  if (a && b && typeof a === 'object' && typeof b === 'object' && Array.isArray(a) === Array.isArray(b)) return [...new Set([...Object.keys(a), ...Object.keys(b)])].flatMap(key => paths(a[key], b[key], prefix ? `${prefix}.${key}` : key));
  return [prefix];
};
const OLD = {};
const modelChanges = {};
equal(N.model.pages.map(page => page.id), H.model.pages.map(page => page.id), 'page order');
for (const page of N.model.pages) {
  const old = H.model.pages.find(item => item.id === page.id);
  const changed = paths(old, page).sort();
  if (!changed.length) continue;
  modelChanges[page.id] = changed;
  OLD[page.id] = {};
  for (const field of Object.keys(NEW[page.id] ?? {})) {
    equal(page[field], NEW[page.id][field], `${page.id}: new ${field}`);
    OLD[page.id][field] = old[field];
  }
  const expected = Object.keys(NEW[page.id]).flatMap(field => localeIds.flatMap(locale => [`${field}.${locale}`, `metadata.${locale}.${field}`])).sort();
  equal(changed, expected, `${page.id}: only the intended title/description fields (and their metadata mirrors) changed`);
}
equal(Object.keys(modelChanges).sort(), Object.keys(NEW).sort(), 'pages with model changes');
for (const name of ['docGroups', 'navigation', 'footerNavigation', 'nextTasks', 'homepageModules']) equal(json(N.model[name]), json(H.model[name]), `${name} identical`);
for (const page of N.model.pages) for (const locale of localeIds) {
  equal(N.model.pagePath(page.id, locale), H.model.pagePath(page.id, locale), `${page.id}/${locale} route`);
  equal(page.sources?.[locale], H.model.pages.find(item => item.id === page.id).sources?.[locale], `${page.id}/${locale} source path`);
}
const entryDiff = paths(H.model.documentationEntries(), N.model.documentationEntries());
ok(entryDiff.every(item => /^\d+\.(title|description)$/.test(item)), 'documentation entries differ only in title/description');
// Starlight sidebar actually built by astro.config.mjs: only the three quickstart item labels changed.
const starlight = config => config.integrations.find(item => item.name === 'starlight-options-probe').options;
const sidebarDiff = paths(starlight(H.astro).sidebar, starlight(N.astro).sidebar);
const sidebarChanged = sidebarDiff.map(item => { const [group, , index, ...rest] = item.split('.'); const entry = starlight(N.astro).sidebar[group].items[index]; return `${entry.link ?? entry.slug}:${rest.join('.')}`; }).sort();
equal(sidebarChanged, ['docs/node-quickstart:label', 'docs/node-quickstart:translations.zh-CN', 'docs/python-quickstart:label', 'docs/python-quickstart:translations.zh-CN', 'docs/quickstart:label', 'docs/quickstart:translations.zh-CN'].sort(), 'only the three quickstart sidebar labels changed');
const starlightRest = config => { const { sidebar, ...rest } = starlight(config); return json(rest); };
equal(starlightRest(N.astro), starlightRest(H.astro), 'all other Starlight options identical');
const quickstartMdx = await read(workRoot, 'website/src/content/docs/docs/quickstart.mdx');
equal(quickstartMdx.split('\n')[1], `title: ${NEW.quickstart.title.en}`, 'quickstart.mdx frontmatter title equals the model title');
out.results.model = { changedPages: modelChanges, old: OLD, sidebarItemsChanged: sidebarChanged };

// ---------------------------------------------------------------------------------------------------------------
// 3. Sources: changes are confined to the H1/title line or the opening paragraphs before the first ## heading.
const prelude = text => { const lines = text.replace(/\r\n/g, '\n').split('\n'); const start = lines[0] === '---' ? lines.indexOf('---', 1) + 1 : lines[0].startsWith('# ') ? 1 : 0; const end = lines.findIndex((line, index) => index >= start && /^## /.test(line)); return lines.slice(start, end < 0 ? lines.length : end).join('\n'); };
const paragraphs = text => text.split(/\n\s*\n/).map(item => item.trim()).filter(Boolean);
const changedSources = {};
for (const page of N.model.pages.filter(page => ['docs', 'article'].includes(page.type))) for (const locale of localeIds) {
  const source = page.sources[locale];
  const a = await read(beforeRoot, source), b = await read(workRoot, source);
  const strip = list => list.map(({ line, ...rest }) => rest);
  equal(strip(N.sections.headings(b)), strip(H.sections.headings(a)), `${page.id}/${locale}: ## headings and section markers unchanged`);
  const legacyA = H.sections.legacyAliases(H.legacy, page.id, locale), legacyB = N.sections.legacyAliases(N.legacy, page.id, locale);
  equal(N.sections.sectionMapping(page.id, locale, b, legacyB), H.sections.sectionMapping(page.id, locale, a, legacyA), `${page.id}/${locale}: stable section mapping unchanged`);
  equal([...b.matchAll(/<!-- source: ([^\n]+) -->/g)].map(match => match[1]), [...a.matchAll(/<!-- source: ([^\n]+) -->/g)].map(match => match[1]), `${page.id}/${locale}: source-backed program references unchanged`);
  if (a === b) continue;
  const tailA = a.slice(a.search(/^## /m)), tailB = b.slice(b.search(/^## /m));
  equal(tailB, tailA, `${page.id}/${locale}: everything from the first ## heading is byte-identical`);
  const pa = paragraphs(prelude(a)), pb = paragraphs(prelude(b));
  let common = 0; while (common < pa.length && common < pb.length && pa[pa.length - 1 - common] === pb[pb.length - 1 - common]) common++;
  const record = { source, firstLine: a.split('\n')[0] === b.split('\n')[0] && a.split('\n')[1] === b.split('\n')[1] ? 'unchanged' : 'changed', removed: pa.slice(0, pa.length - common), added: pb.slice(0, pb.length - common) };
  if (record.firstLine === 'changed') {
    equal(prelude(b), prelude(a), `${page.id}/${locale}: a title-line change leaves the opening text unchanged`);
    const title = b.startsWith('---') ? b.split('\n')[1].replace(/^title: /, '') : b.split('\n')[0].replace(/^# /, '');
    equal(title, page.title[locale], `${page.id}/${locale}: source title line equals the model title`);
  }
  record.links = record.added.flatMap(item => [...item.matchAll(/\[([^\]]+)\]\(([^)\s]+)\)/g)].map(match => ({ text: match[1], target: match[2] })));
  changedSources[`${page.id}/${locale}`] = record;
}
equal(Object.values(changedSources).map(item => item.source).sort(), EXPECTED_CHANGED.filter(file => file !== 'website/site-model.mjs').sort(), 'changed page sources');
out.results.sources = changedSources;

// ---------------------------------------------------------------------------------------------------------------
// 4. New links resolve to the matching API section in the same locale (rewritten URL, built anchor, section content).
const htmlCache = new Map();
const html = async (root, relative) => { const key = `${root}|${relative}`; if (!htmlCache.has(key)) htmlCache.set(key, await readFile(path.join(root, relative), 'utf8')); return htmlCache.get(key); };
const distFile = route => { const relative = route.replace(new RegExp(`^${siteConfig.base === '/' ? '' : siteConfig.base}/`), ''); return relative.endsWith('/') || relative === '' ? `${relative}index.html` : relative; };
const sectionText = ($, heading) => { let text = '', node = heading.closest('.sl-heading-wrapper').next(); while (node.length && !node.is('.sl-heading-wrapper.level-h2') && !(node.is('p') && node.children('span[data-section-alias]').length && node.next().is('.sl-heading-wrapper.level-h2'))) { text += ` ${node.text()}`; node = node.next(); } return text; };
const EXPECTED_LINKS = {
  cards: [['Python API', 'python-api', 'project-and-note-values', ['Note.basic(front, back)', 'Note.cloze(text)']], ['Node API', 'node-api', 'project-and-notes', ['Note.basic(front, back)', 'Note.cloze(text)']]],
  'image-occlusion': [['Python API', 'python-api', 'media-and-occlusion', ['Note.image_occlusion(media)', 'Mask.rect(', 'OcclusionMode.HIDE_ONE_GUESS_ONE', 'HIDE_ALL_GUESS_ONE']], ['Node API', 'node-api', 'media-and-image-occlusion', ['Note.imageOcclusion(image)', 'Mask.rect(', 'hide_one_guess_one', 'hide_all_guess_one']]],
  templates: [['Python API', 'python-api', 'models-and-templates', ['NoteType.from_bundle(']], ['Node API', 'node-api', 'immutable-models', ['NoteType.fromBundle(']]],
};
const linkResults = [];
for (const [pageId, expectedLinks] of Object.entries(EXPECTED_LINKS)) for (const locale of localeIds) {
  const record = changedSources[`${pageId}/${locale}`];
  equal(record.links.map(item => item.text), expectedLinks.map(item => item[0]), `${pageId}/${locale}: new link texts`);
  for (const [index, [text, targetId, anchor, identifiers]] of expectedLinks.entries()) {
    const { target } = record.links[index];
    const href = links.rewriteTarget(target, record.source, siteConfig.base, locale);
    const expectedHref = `${siteConfig.base === '/' ? '' : siteConfig.base}${N.model.pagePath(targetId, locale)}#${anchor}`;
    equal(href, expectedHref, `${pageId}/${locale}: ${target} rewrites to the same-locale API page`);
    const $source = cheerio(await html(dist, distFile(`${siteConfig.base}${N.model.pagePath(pageId, locale)}`)));
    const anchorEl = $source('.sl-markdown-content a').filter((_, el) => $source(el).attr('href') === href);
    equal(anchorEl.length, 1, `${pageId}/${locale}: built page contains the rewritten link once`);
    equal(anchorEl.text(), text, `${pageId}/${locale}: built link text`);
    const $ = cheerio(await html(dist, distFile(`${siteConfig.base}${N.model.pagePath(targetId, locale)}`)));
    const targets = $(`[id="${anchor}"]`);
    equal(targets.length, 1, `${targetId}/${locale}: exactly one #${anchor}`);
    let heading;
    const contract = N.contracts.entries[targetId];
    if (locale === 'en') {
      ok(targets.is('h2'), `${targetId}/en: #${anchor} is the h2 heading itself`);
      heading = targets;
    } else {
      ok(targets.is(`span[data-section-alias="${anchor}"]`), `${targetId}/zh-CN: #${anchor} is a stable-section alias`);
      heading = targets.parent('p').next('.sl-heading-wrapper.level-h2').children('h2');
      equal(heading.length, 1, `${targetId}/zh-CN: alias immediately precedes an h2 section`);
      equal(contract.locales['zh-CN'][heading.attr('id')], anchor, `${targetId}/zh-CN: that heading's reviewed contract maps to ${anchor}`);
    }
    const sourceHeading = N.sections.headings(await read(workRoot, N.model.pages.find(item => item.id === targetId).sources[locale])).find(item => (contract.locales[locale]?.[item.id] ?? item.id) === anchor);
    equal(heading.text(), sourceHeading.title, `${targetId}/${locale}: anchor lands on “${sourceHeading.title}”`);
    const body = sectionText($, heading);
    for (const identifier of identifiers) ok(body.includes(identifier), `${targetId}/${locale}#${anchor}: section documents ${identifier}`);
    const markdown = await read(dist, distFile(`${siteConfig.base}${N.model.pagePath(pageId, locale)}index.md`));
    ok(markdown.includes(`[${text}](${siteConfig.site}${href})`), `${pageId}/${locale}: Markdown export keeps the absolute same-locale link`);
    linkResults.push({ page: `${pageId}/${locale}`, target, href, heading: heading.text(), identifiers });
  }
}
// The media opening says the batch section shows the Python and Node.js calls; check it in both built locales.
for (const locale of localeIds) {
  const $ = cheerio(await html(dist, distFile(`${siteConfig.base}${N.model.pagePath('media', locale)}`)));
  const heading = $('h2').filter((_, el) => (N.contracts.entries.media.locales[locale]?.[$(el).attr('id')] ?? $(el).attr('id')) === 'batch-media-import');
  equal(heading.length, 1, `media/${locale}: batch media import section exists`);
  const body = sectionText($, heading);
  ok(body.includes("await Media.files(['first.png', 'second.png'])") && body.includes("Media.files(['first.png', 'second.png'])") && /Node/.test(body) && /Python/.test(body), `media/${locale}: batch section shows the Node.js and Python calls`);
  const intro = $('.sl-markdown-content').children().first();
  ok(intro.is('p') && $('.sl-markdown-content').children().index(intro) < $('.sl-markdown-content').children().index(heading.closest('.sl-heading-wrapper')), `media/${locale}: the new opening precedes ("below") the batch section`);
}
// Every same-site link in the main content of all built pages resolves, including fragments.
const walk = async directory => (await Promise.all((await readdir(directory, { withFileTypes: true })).map(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : path.join(directory, entry.name)))).flat();
const linkCheck = async root => {
  const failures = []; let count = 0, fragments = 0;
  for (const file of (await walk(root)).filter(item => item.endsWith('.html')).sort()) {
    const relative = path.relative(root, file).split(path.sep).join('/');
    const pageUrl = new URL(`${siteConfig.base === '/' ? '' : siteConfig.base}/${relative.replace(/index\.html$/, '')}`, siteConfig.site);
    const $ = cheerio(await html(root, relative));
    for (const element of $('.sl-markdown-content a[href], main a[href]').toArray()) {
      const value = $(element).attr('href');
      if (/^(?:mailto:|tel:|javascript:|data:)/i.test(value)) continue;
      const url = new URL(value, pageUrl);
      if (url.origin !== pageUrl.origin) continue;
      count++;
      const targetRelative = decodeURIComponent(url.pathname.slice(siteConfig.base === '/' ? 1 : siteConfig.base.length + 1));
      const targetFile = targetRelative === '' || targetRelative.endsWith('/') ? `${targetRelative}index.html` : targetRelative;
      let bytes;
      try { bytes = await html(root, targetFile); } catch { failures.push(`${relative}: ${value} (missing target)`); continue; }
      if (url.hash && url.hash !== '#' && url.hash !== '#_top') {
        fragments++;
        const id = decodeURIComponent(url.hash.slice(1));
        if (!targetFile.endsWith('.html') || !cheerio(bytes)(`[id="${id.replace(/"/g, '\\"')}"]`).length) failures.push(`${relative}: ${value} (missing #${id})`);
      }
    }
  }
  return { count, fragments, failures: [...new Set(failures)].sort() };
};
const headLinks = await linkCheck(dist), beforeLinks = await linkCheck(beforeDist);
equal(headLinks.failures, beforeLinks.failures, 'no new unresolved same-site link or fragment');
equal(headLinks.failures, [], 'every same-site content link and fragment resolves');
out.results.links = { newLinks: linkResults, sameSiteLinks: { head: { links: headLinks.count, fragments: headLinks.fragments }, before: { links: beforeLinks.count, fragments: beforeLinks.fragments }, failures: headLinks.failures.length } };

// ---------------------------------------------------------------------------------------------------------------
// 5. Public API facts behind the new sentences, in the 0.3.1 release-tag sources (identical commit for all three tags).
const tagCommits = ['anki-forge-v0.3.1', 'npm-v0.3.1', 'python-v0.3.1'].map(tag => git('rev-parse', `${tag}^{commit}`).toString().trim());
equal(new Set(tagCommits).size, 1, 'the three 0.3.1 release tags name one commit');
const atTag = file => git('show', `${tagCommits[0]}:${file}`).toString();
const API = [
  ['bindings/python/src/ankiforge/note.py', ['def basic(front: str | Content, back: str | Content) -> Note:', 'def cloze(text: str | Content) -> Note:', 'def image_occlusion(image: Media) -> ImageOcclusionBuilder:', 'class ImageOcclusionBuilder:', 'def mask(self, mask: Mask)', 'def mode(self, mode: OcclusionMode)', 'HIDE_ALL_GUESS_ONE = "hide_all_guess_one"', 'HIDE_ONE_GUESS_ONE = "hide_one_guess_one"', 'def rect(key: str, x: float, y: float, width: float, height: float) -> Mask:']],
  ['bindings/python/src/ankiforge/notetype.py', ['def from_bundle(path: PathInput, *, limits: MediaLimits = MediaLimits()) -> NoteType:']],
  ['bindings/python/src/ankiforge/media.py', ['def files(paths: Iterable[PathInput], *, limits: MediaLimits = MediaLimits()) -> list[Media]:', 'def file(path: PathInput, *, limits: MediaLimits = MediaLimits()) -> Media:']],
  ['bindings/node/src/index.ts', ['static basic(front: ContentLike, back: ContentLike): Note {', 'static cloze(text: ContentLike): Note {', 'static imageOcclusion(image: Media): ImageOcclusionBuilder {', 'export type OcclusionMode = "hide_all_guess_one" | "hide_one_guess_one";', 'static async fromBundle(', 'static async files(']],
  ['anki_forge/src/note/occlusion.rs', ['pub fn image_occlusion(image: Media) -> ImageOcclusionBuilder', 'HideAllGuessOne,', 'HideOneGuessOne,', 'let ordinal = ordinals[&mask.key];']],
  ['anki_forge/src/schema/bundle.rs', ['pub fn from_bundle(path: impl AsRef<Path>) -> Result<Self, Error>']],
];
for (const [file, needles] of API) {
  const text = atTag(file);
  equal(text, await read(workRoot, file), `${file}: working tree equals the 0.3.1 release tag`);
  for (const needle of needles) ok(text.includes(needle), `${file}@0.3.1 contains ${needle}`);
}
const pythonApi = await read(workRoot, 'docs/python/api.md');
ok(pythonApi.includes('`NoteType.from_bundle(path, limits=MediaLimits())`'), 'docs/python/api.md documents from_bundle with a positional-or-keyword limits parameter');
out.results.apiSources = { tag: tagCommits[0], files: API.map(([file, needles]) => ({ file, needles: needles.length })) };

// ---------------------------------------------------------------------------------------------------------------
// 6. Runtime: the published Python and Node 0.3.1 packages behave as the new sentences say.
const temporary = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-task-descriptions-probe-'));
try {
  const fixtures = [
    path.join(workRoot, 'contracts/fixtures/template-bundle/custom-cloze'), path.join(workRoot, 'contracts/fixtures/template-bundle/custom-normal'),
    path.join(workRoot, 'anki_forge/tests/fixtures/public-api/occlusion.png'), path.join(workRoot, 'anki_forge/tests/fixtures/public-api/pixel.png'), path.join(workRoot, 'anki_forge/tests/fixtures/public-api/silence.wav'),
  ];
  await writeFile(path.join(temporary, 'python_probe.py'), `import inspect, json, sys
import ankiforge
from ankiforge import Project, Note, NoteType, Media, MediaLimits, Mask, OcclusionMode, BuildOptions
cloze, normal, occlusion, pixel, wav = sys.argv[1:6]
def build(project):
    output = project.build(BuildOptions.temporary())
    counts = output.report.counts
    result = {"notes": counts.notes, "cards": counts.cards, "media": counts.media}
    output.artifact.close()
    return result
out = {"version": ankiforge.__version__}
p = Project("probe-cards", default_deck="Probe")
p.add("basic", Note.basic("hola", "hello"))
p.add("capital", Note.cloze("{{c1::Madrid}} is in {{c2::Spain}}.").field("back_extra", "A city and its country."))
out["basicAndCloze"] = build(p)
p = Project("probe-bundle-cloze"); p.add("capital", NoteType.from_bundle(cloze).note().field("text", "{{c1::Madrid}} is in {{c2::Spain}}.").field("extra", "x")); out["bundleCloze"] = build(p)
p = Project("probe-bundle-normal"); p.add("hola", NoteType.from_bundle(normal).note().field("prompt", "hola")); out["bundleNormal"] = build(p)
for name, mode in [("default", None), ("hide_one_guess_one", OcclusionMode.HIDE_ONE_GUESS_ONE)]:
    builder = Note.image_occlusion(Media.file(occlusion)).mask(Mask.rect("nucleus", 10, 10, 20, 20)).mask(Mask.rect("wall", 50, 50, 20, 20)).mask(Mask.rect("label", 10, 50, 20, 20))
    if mode: builder = builder.mode(mode)
    p = Project("probe-occlusion"); p.add("cell", builder.build().field("header", "Cell")); out["occlusion_" + name] = build(p)
images = Media.files([pixel, wav])
p = Project("probe-media"); p.add("cell", Note.basic(images[0].with_export_name("cell.png").image(), images[1].sound())); out["batchMedia"] = build(p)
try:
    NoteType.from_bundle(normal, MediaLimits())
    out["fromBundlePositionalLimits"] = "accepted"
except TypeError as error:
    out["fromBundlePositionalLimits"] = "TypeError: " + str(error)
out["fromBundleKeywordLimits"] = NoteType.from_bundle(normal, limits=MediaLimits()).key
out["signatures"] = {name: str(inspect.signature(value)) for name, value in [("Note.basic", Note.basic), ("Note.cloze", Note.cloze), ("Note.image_occlusion", Note.image_occlusion), ("NoteType.from_bundle", NoteType.from_bundle), ("Media.files", Media.files)]}
print(json.dumps(out, sort_keys=True))
`);
  const pythonResult = JSON.parse(execFileSync(python, ['-I', path.join(temporary, 'python_probe.py'), ...fixtures], { cwd: temporary }).toString());
  await writeFile(path.join(temporary, 'node_probe.mjs'), `import { pathToFileURL } from 'node:url';
import path from 'node:path';
const [pkg, cloze, normal, occlusion, pixel, wav] = process.argv.slice(2);
const af = await import(pathToFileURL(path.join(pkg, 'dist/index.mjs')).href);
const { Project, Note, NoteType, Media, Mask, BuildOptions } = af;
const build = async project => { const output = await project.build(BuildOptions.temporary()); const { notes, cards, media } = output.report.counts; await output.artifact.close(); return { notes, cards, media }; };
const out = {};
let p = new Project('probe-cards').defaultDeck('Probe');
p.add('basic', Note.basic('hola', 'hello'));
p.add('capital', Note.cloze('{{c1::Madrid}} is in {{c2::Spain}}.').field('back_extra', 'A city and its country.'));
out.basicAndCloze = await build(p);
p = new Project('probe-bundle-cloze'); p.add('capital', (await NoteType.fromBundle(cloze)).note().field('text', '{{c1::Madrid}} is in {{c2::Spain}}.').field('extra', 'x')); out.bundleCloze = await build(p);
p = new Project('probe-bundle-normal'); p.add('hola', (await NoteType.fromBundle(normal)).note().field('prompt', 'hola')); out.bundleNormal = await build(p);
for (const mode of [undefined, 'hide_one_guess_one']) {
  let builder = Note.imageOcclusion(await Media.file(occlusion)).mask(Mask.rect('nucleus', 10, 10, 20, 20)).mask(Mask.rect('wall', 50, 50, 20, 20)).mask(Mask.rect('label', 10, 50, 20, 20));
  if (mode) builder = builder.mode(mode);
  p = new Project('probe-occlusion'); p.add('cell', builder.build().field('header', 'Cell')); out['occlusion_' + (mode ?? 'default')] = await build(p);
}
const images = await Media.files([pixel, wav]);
p = new Project('probe-media'); p.add('cell', Note.basic(images[0].withExportName('cell.png').image(), images[1].sound())); out.batchMedia = await build(p);
out.version = JSON.parse(await (await import('node:fs/promises')).readFile(path.join(pkg, 'package.json'), 'utf8')).version;
console.log(JSON.stringify(out));
`);
  const nodeResult = JSON.parse(execFileSync(process.execPath, [path.join(temporary, 'node_probe.mjs'), nodePackage, ...fixtures], { cwd: temporary }).toString());
  const expectedRuntime = { basicAndCloze: { notes: 2, cards: 3, media: 0 }, bundleCloze: { notes: 1, cards: 2, media: 0 }, bundleNormal: { notes: 1, cards: 1, media: 1 }, occlusion_default: { notes: 1, cards: 3, media: 1 }, occlusion_hide_one_guess_one: { notes: 1, cards: 3, media: 1 }, batchMedia: { notes: 1, cards: 1, media: 2 } };
  for (const [name, result] of [['python', pythonResult], ['node', nodeResult]]) {
    equal(result.version, '0.3.1', `${name}: published package version`);
    for (const [key, value] of Object.entries(expectedRuntime)) equal(result[key], value, `${name}: ${key}`);
  }
  ok(/^TypeError: /.test(pythonResult.fromBundlePositionalLimits), 'python: from_bundle rejects a positional limits argument');
  equal(pythonResult.fromBundleKeywordLimits, 'language-card', 'python: from_bundle accepts limits= by keyword');
  // Installed Python files equal the wheel's RECORD; the Node lockfile pins the npm registry tarballs.
  const site = execFileSync(python, ['-I', '-c', 'import ankiforge, os; print(os.path.dirname(os.path.dirname(ankiforge.__file__)))']).toString().trim();
  const distInfo = (await readdir(site)).find(name => /^ankiforge-0\.3\.1\.dist-info$/.test(name));
  const record = (await read(site, `${distInfo}/RECORD`)).trim().split('\n').map(line => line.split(',')).filter(([file, digest]) => file.startsWith('ankiforge/') && digest);
  for (const [file, digest] of record) equal(Buffer.from(createHash('sha256').update(await readFile(path.join(site, file))).digest()).toString('base64url'), digest.replace(/^sha256=/, ''), `python wheel RECORD ${file}`);
  const lock = JSON.parse(await readFile(path.join(nodePackage, '../../package-lock.json'), 'utf8'));
  equal(lock.packages['node_modules/ankiforge'].resolved, 'https://registry.npmjs.org/ankiforge/-/ankiforge-0.3.1.tgz', 'node package resolved from the npm registry');
  out.results.runtime = { python: pythonResult, node: nodeResult, pythonRecordFiles: record.length, nodeIntegrity: lock.packages['node_modules/ankiforge'].integrity };
} finally {
  await rm(temporary, { recursive: true, force: true });
}

// ---------------------------------------------------------------------------------------------------------------
// 7. Artifacts: d9f5354's preview build reproduces its release manifest; HEAD differs from it only by the reviewed text.
const manifest = JSON.parse(await read(workRoot, 'docs/plans/evidence/multilingual-locale-data-2026-10-28/prefix-origin-manifest.json'));
const recomputedIdentity = JSON.parse(execFileSync(process.execPath, ['--input-type=module', '-e', `const { buildInputIdentity } = await import(${json(pathToFileURL(path.join(workRoot, 'website/scripts/site-manifest.mjs')).href)}); console.log(JSON.stringify(await buildInputIdentity(${json(beforeRoot)})));`]).toString());
equal(recomputedIdentity.inputSha256, manifest.inputSha256, 'd9f5354 build-input identity equals the locale-data release manifest');
const inventory = async root => new Map(await Promise.all((await walk(root)).map(async file => [path.relative(root, file).split(path.sep).join('/'), sha(await readFile(file))])).then(items => items.filter(([relative]) => relative !== 'generated/site-manifest.json' && relative !== 'CNAME' && !relative.startsWith('.'))));
const B = await inventory(beforeDist), W = await inventory(dist);
const volatile = relative => /^generated\/[^/]+\.apkg$/.test(relative) || ['generated/showcase.json', 'pagefind/pagefind-entry.json'].includes(relative);
const expectedManifest = new Map([...manifest.pages, ...manifest.resources].map(entry => { const relative = entry.route.slice(manifest.configuration.base.length + 1); return [relative === '' || relative.endsWith('/') ? `${relative}index.html` : relative, entry.sha256]; }));
const manifestDiffer = [...B].filter(([relative, digest]) => expectedManifest.get(relative) !== digest).map(([relative]) => relative).sort();
equal(manifestDiffer.filter(relative => !volatile(relative)), [], 'd9f5354 preview build equals its release manifest apart from volatile artifacts');
const missingFromBefore = [...expectedManifest.keys()].filter(relative => !B.has(relative)).sort();
equal(missingFromBefore, ['generated/cell-diagram.png'], 'only the stale untracked cell-diagram.png in the main checkout is absent from the clean d9f5354 build');
const onlyBefore = [...B.keys()].filter(relative => !W.has(relative)).sort(), onlyHead = [...W.keys()].filter(relative => !B.has(relative)).sort();
equal(onlyHead.filter(relative => relative !== 'generated/cell-diagram.png').every(relative => /^pagefind\/(fragment\/|index\/|pagefind\.)/.test(relative)) && onlyBefore.every(relative => /^pagefind\/(fragment\/|index\/|pagefind\.)/.test(relative)), true, 'only content-addressed Pagefind files are renamed');
const differ = [...W].filter(([relative, digest]) => B.has(relative) && B.get(relative) !== digest).map(([relative]) => relative).sort();
const htmlDiffer = differ.filter(relative => relative.endsWith('.html'));
const expectedHtml = docsIds.flatMap(id => localeIds.map(locale => distFile(`${siteConfig.base}${N.model.pagePath(id, locale)}`))).sort();
equal(htmlDiffer, expectedHtml, 'exactly the 58 docs HTML pages changed');
const changedMarkdown = Object.keys(NEW).flatMap(id => localeIds.map(locale => distFile(`${siteConfig.base}${N.model.pagePath(id, locale)}index.md`))).sort();
equal(differ.filter(relative => !relative.endsWith('.html') && !volatile(relative)).sort(), [...changedMarkdown, 'generated/search-en.json', 'generated/search-zh-cn.json', 'llms.txt', 'zh-cn/llms.txt'].sort(), 'other changed artifacts');
// Normalize HTML: Starlight sidebar hash, the three titles and the seven page descriptions; then everything must be byte-identical
// except the opening paragraphs of the ten pages whose opening changed, which must equal the reviewed new text.
const titlePairs = Object.entries(NEW).filter(([, value]) => value.title).flatMap(([id, value]) => localeIds.map(locale => [value.title[locale], OLD[id].title[locale]]));
const replaceAll = (text, from, to) => text.split(from).join(to);
const plain = markdown => markdown.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').replace(/`([^`]+)`/g, '$1').replace(/'/g, '’').replace(/\s+/g, ' ').trim();
const htmlResults = { identicalAfterNormalization: 0, openingChanged: [] };
for (const relative of htmlDiffer) {
  const page = N.model.pages.find(item => docsIds.includes(item.id) && localeIds.some(locale => distFile(`${siteConfig.base}${N.model.pagePath(item.id, locale)}`) === relative));
  const locale = localeIds.find(item => distFile(`${siteConfig.base}${N.model.pagePath(page.id, item)}`) === relative);
  let a = await html(beforeDist, relative), b = await html(dist, relative);
  const hash = text => text.replace(/<sl-sidebar-state-persist data-hash="[^"]*"/g, '<sl-sidebar-state-persist data-hash=""');
  a = hash(a); b = hash(b);
  if (NEW[page.id]?.description) b = replaceAll(b, NEW[page.id].description[locale], OLD[page.id].description[locale]);
  for (const [next, previous] of titlePairs) b = replaceAll(b, next, previous);
  const record = changedSources[`${page.id}/${locale}`];
  if (!record?.added.length) { equal(b, a, `${relative}: identical after title/description/sidebar-hash normalization`); htmlResults.identicalAfterNormalization++; continue; }
  const $a = cheerio(a), $b = cheerio(b);
  const childrenA = $a('.sl-markdown-content').children().toArray().map(el => $a.html(el)), childrenB = $b('.sl-markdown-content').children().toArray().map(el => $b.html(el));
  let common = 0; while (common < childrenA.length && common < childrenB.length && childrenA[childrenA.length - 1 - common] === childrenB[childrenB.length - 1 - common]) common++;
  const leadA = $a('.sl-markdown-content').children().toArray().slice(0, childrenA.length - common), leadB = $b('.sl-markdown-content').children().toArray().slice(0, childrenB.length - common);
  ok([...leadA, ...leadB].every(el => el.tagName === 'p'), `${relative}: only opening paragraphs differ`);
  equal(leadB.map(el => $b(el).text().replace(/\s+/g, ' ').trim()), record.added.map(plain), `${relative}: built opening equals the reviewed new source text`);
  equal(leadA.map(el => $a(el).text().replace(/\s+/g, ' ').trim()), record.removed.map(plain), `${relative}: replaced opening equals the d9f5354 source text`);
  $a('.sl-markdown-content').empty(); $b('.sl-markdown-content').empty();
  equal($b.html(), $a.html(), `${relative}: everything outside the content is identical after normalization`);
  htmlResults.openingChanged.push(relative);
}
equal(htmlResults.identicalAfterNormalization + htmlResults.openingChanged.length, 58, 'all 58 HTML differences explained');
// Markdown exports, llms.txt and search directories change exactly the reviewed titles, descriptions and openings.
const markdownResults = {};
for (const relative of changedMarkdown) {
  const page = N.model.pages.find(item => localeIds.some(locale => distFile(`${siteConfig.base}${N.model.pagePath(item.id, locale)}index.md`) === relative));
  const locale = localeIds.find(item => distFile(`${siteConfig.base}${N.model.pagePath(page.id, item)}index.md`) === relative);
  const a = (await read(beforeDist, relative)).split('\n'), b = (await read(dist, relative)).split('\n');
  const removed = a.filter(line => !b.includes(line)), added = b.filter(line => !a.includes(line));
  const record = changedSources[`${page.id}/${locale}`] ?? { added: [], removed: [] };
  const expectedAdded = [...(NEW[page.id].title ? [`# ${NEW[page.id].title[locale]}`] : []), ...(NEW[page.id].description ? [`Description: ${NEW[page.id].description[locale]}`] : []), ...record.added.map(plain)];
  const unlink = line => line.startsWith('# ') || line.startsWith('Description: ') ? line : plain(line.replace(new RegExp(`${siteConfig.site}${siteConfig.base === '/' ? '' : siteConfig.base}`, 'g'), ''));
  equal(added.filter(Boolean).map(unlink), expectedAdded, `${relative}: added lines`);
  const expectedRemoved = [...(OLD[page.id].title ? [`# ${OLD[page.id].title[locale]}`] : []), ...(OLD[page.id].description ? [`Description: ${OLD[page.id].description[locale]}`] : []), ...record.removed.map(plain)];
  equal(removed.filter(Boolean).map(unlink), expectedRemoved, `${relative}: removed lines`);
  markdownResults[relative] = { added: added.filter(Boolean).length, removed: removed.filter(Boolean).length };
}
for (const [relative, locale] of [['llms.txt', 'en'], ['zh-cn/llms.txt', 'zh-CN']]) {
  const a = (await read(beforeDist, relative)).split('\n'), b = (await read(dist, relative)).split('\n');
  const added = b.filter(line => !a.includes(line));
  const expected = Object.keys(NEW).map(id => { const page = N.model.pages.find(item => item.id === id); return `- [${page.title[locale]}](${siteConfig.site}${siteConfig.base === '/' ? '' : siteConfig.base}${N.model.pagePath(id, locale)}index.md): ${page.description[locale]}`; });
  equal(added.sort(), expected.sort(), `${relative}: exactly the eight reviewed entries changed`);
  equal(a.length, b.length, `${relative}: same entry count`);
}
for (const locale of localeIds) {
  const file = `generated/search-${locale.toLowerCase()}.json`;
  const a = JSON.parse(await read(beforeDist, file)), b = JSON.parse(await read(dist, file));
  ok(paths(a, b).every(item => /^documents\.\d+\.(title|pageTitle|description)$/.test(item)), `${file}: only titles and descriptions changed`);
  equal(paths(a, b).length, 13, `${file}: 3 titles, 3 page titles and 7 descriptions`);
}
const entryA = JSON.parse(await read(beforeDist, 'pagefind/pagefind-entry.json')), entryB = JSON.parse(await read(dist, 'pagefind/pagefind-entry.json'));
ok(paths(entryA, entryB).every(item => /^languages\.[^.]+\.hash$/.test(item)), 'pagefind-entry.json differs only in the per-language index hashes');
const fragments = async root => Object.fromEntries(await Promise.all((await readdir(path.join(root, 'pagefind/fragment'))).map(async name => { const text = gunzipSync(await readFile(path.join(root, 'pagefind/fragment', name))).toString(); const data = JSON.parse(text.slice('pagefind_dcd'.length)); return [data.url, data]; })));
const fragmentsA = await fragments(beforeDist), fragmentsB = await fragments(dist);
equal(Object.keys(fragmentsB).sort(), Object.keys(fragmentsA).sort(), 'same Pagefind page set');
const fragmentChanged = Object.keys(fragmentsB).filter(url => json(fragmentsA[url]) !== json(fragmentsB[url])).sort();
equal(fragmentChanged, Object.keys(NEW).flatMap(id => localeIds.map(locale => N.model.pagePath(id, locale))).sort(), 'Pagefind fragments changed for exactly the 16 retitled or reopened pages');
for (const url of fragmentChanged) {
  const changed = paths(fragmentsA[url], fragmentsB[url]);
  ok(changed.every(item => /^(content|word_count|meta\.title|anchors\.\d+\.location)$/.test(item) || (item === 'anchors.0.text' && fragmentsB[url].anchors[0].id === '_top' && fragmentsB[url].anchors[0].text === fragmentsB[url].meta.title)), `${url}: fragment changed only in content, word count, title (also the _top anchor text) and anchor offsets`);
  if (changed.includes('meta.title')) ok(titlePairs.some(([next, previous]) => fragmentsB[url].meta.title === next && fragmentsA[url].meta.title === previous), `${url}: Pagefind title is the reviewed retitle`);
}
const showcaseA = JSON.parse(await read(beforeDist, 'generated/showcase.json')), showcaseB = JSON.parse(await read(dist, 'generated/showcase.json'));
ok(paths(showcaseA, showcaseB).every(item => /(\.bytes|\.sha256|^sourceCommit)$/.test(item)), 'showcase.json differs only in generated APKG identities and the source commit');
let rebuild;
if (rebuildDist) {
  const R = await inventory(rebuildDist);
  equal([...R.keys()].sort(), [...W.keys()].sort(), 'rebuild route set');
  const rebuildDiffer = [...R].filter(([relative, digest]) => W.get(relative) !== digest).map(([relative]) => relative).sort();
  equal(rebuildDiffer.filter(relative => !volatile(relative)), [], 'HEAD rebuild is byte-identical apart from volatile artifacts');
  rebuild = { files: R.size, differ: rebuildDiffer };
}
out.results.artifacts = { baselineManifest: { commit: manifest.commit, inputSha256: manifest.inputSha256, recomputedFromD9f5354: recomputedIdentity.inputSha256 }, before: { files: B.size, differFromManifest: manifestDiffer, missingFromManifest: missingFromBefore }, head: { files: W.size }, renamedPagefind: { before: onlyBefore.length, head: onlyHead.filter(relative => relative !== 'generated/cell-diagram.png').length }, differ, html: htmlResults, markdown: markdownResults, pagefindFragmentsChanged: fragmentChanged.length, rebuild };

// ---------------------------------------------------------------------------------------------------------------
// 8. README link additions (supporting review): every added link resolves to the intended page, locale and section.
const READMES = { 'README.md': 'en', 'README.zh-CN.md': 'zh-CN', 'anki_forge/README.md': 'en', 'bindings/node/README.md': 'en', 'bindings/python/README.md': 'en' };
const readmeResults = [];
for (const [file, locale] of Object.entries(READMES)) {
  const added = git('diff', '-U0', 'd9f5354', 'HEAD', '--', file).toString().split('\n').filter(line => line.startsWith('+') && !line.startsWith('+++')).join('\n');
  const found = [...added.matchAll(/\[([^\]]+)\]\(([^)\s]+)\)/g)].map(match => ({ text: match[1], url: match[2] }));
  ok(found.length >= 2, `${file}: added links found`);
  for (const { text, url } of found) {
    ok(url.startsWith('https://ankiforge.dev/'), `${file}: ${url} is an absolute production link`);
    const parsed = new URL(url);
    const resolved = N.model.resolvePage(parsed.pathname, '/');
    ok(resolved, `${file}: ${url} names a published page`);
    equal(resolved.locale, locale, `${file}: ${url} stays in the README's locale`);
    equal(N.model.pagePath(resolved.page.id, locale), parsed.pathname, `${file}: ${url} is the canonical route`);
    const $ = cheerio(await html(dist, distFile(`${siteConfig.base}${parsed.pathname}`)));
    let section = null;
    if (parsed.hash) {
      const id = parsed.hash.slice(1), element = $(`[id="${id}"]`);
      equal(element.length, 1, `${file}: ${url} fragment exists`);
      ok(element.is('h2'), `${file}: ${url} fragment is a section heading`);
      section = { heading: element.text(), text: sectionText($, element) };
    }
    readmeResults.push({ file, text, url, page: resolved.page.id, title: resolved.page.title[locale], section: section?.heading ?? null, sectionText: section?.text.replace(/\s+/g, ' ').trim().slice(0, 160) ?? null });
  }
}
const byText = text => readmeResults.filter(item => item.text === text);
for (const item of byText('JSON to Anki with TypeScript')) ok(item.section === 'Node' && /compiles TypeScript/.test(item.sectionText), 'Node README: #node is the TypeScript program');
for (const item of byText('JSON to Anki with Rust')) equal(item.section, 'Rust', 'crate README: #rust');
for (const item of byText('CSV to Anki with Python')) ok(/Python/.test(item.title), 'CSV guide is the Python recipe');
equal(readmeResults.length, 15, 'README links checked');
out.results.readmes = readmeResults;

out.checks = checks;
out.passed = true;
console.log(JSON.stringify(out, null, 2));
