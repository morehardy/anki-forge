// Independent technical delta probe for the python-quickstart/zh-CN description edit ("生成 Anki 卡片" -> "生成 Anki 问答卡片"):
// HEAD 86095ce (last fully reviewed state) versus the uncommitted working tree.
// Usage:
//   node technical-python-qa-probe.mjs <HEAD root> --python <python with the published ankiforge 0.3.1>
// <HEAD root> is `git archive HEAD` extracted under /private/tmp, with website/node_modules symlinked to the checkout's
// (only module resolution for plain imports; nothing is built). The probe reads only; it runs the page's program with the
// published package in a disposable /private/tmp directory and removes it before exiting.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFile, mkdtemp, readFile, readdir, realpath, rm, writeFile } from 'node:fs/promises';
import { registerHooks } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const argv = process.argv.slice(2);
const flag = name => { const index = argv.indexOf(name); if (index < 0) return undefined; const [, value] = argv.splice(index, 2); return path.resolve(value); };
const python = flag('--python');
const [headRoot, workRoot = fileURLToPath(new URL('../../../../', import.meta.url))] = argv.map(value => path.resolve(value));
assert.ok(headRoot && python, 'missing arguments');
const E = 'docs/plans/evidence/multilingual-common-2026-10-27';
const sha = value => createHash('sha256').update(value).digest('hex');
const json = value => JSON.stringify(value);
const read = (root, file) => readFile(path.join(root, file), 'utf8');
const bytes = (root, file) => readFile(path.join(root, file));
const load = (root, file) => import(pathToFileURL(path.join(root, file)).href);
const git = (...args) => execFileSync('git', args, { cwd: workRoot, maxBuffer: 1 << 28 });
let checks = 0;
const equal = (actual, expected, message) => { assert.deepStrictEqual(actual, expected, message); checks++; };
const ok = (value, message) => { assert.ok(value, message); checks++; };
const out = { headRoot: 'git archive HEAD (temporary extraction under /private/tmp)', workRoot: 'working tree (uncommitted website/site-model.mjs edit)', results: {} };

registerHooks({ resolve(specifier, context, next) {
  if (specifier === '@astrojs/starlight') return { url: 'data:text/javascript,export default options => ({ name: "starlight-options-probe", options })', shortCircuit: true };
  if (specifier === 'astro/config') return { url: 'data:text/javascript,export const defineConfig = config => config', shortCircuit: true };
  return next(specifier, context);
} });
const side = async root => ({
  model: await load(root, 'website/site-model.mjs'),
  reviews: await load(root, 'website/scripts/content-review-inputs.mjs'),
  links: await load(root, 'website/scripts/content-links.mjs'),
  astro: (await load(root, 'website/astro.config.mjs')).default,
});
const H = await side(headRoot);
const N = await side(workRoot);
const localeIds = [...N.model.localeIds];
equal(localeIds, ['en', 'zh-CN'], 'release locales');
equal([...H.model.localeIds], localeIds, 'release locales unchanged');

// ---------------------------------------------------------------------------------------------------------------
// 0. Inputs: the extraction is HEAD, HEAD equals the prior immutable snapshot, and only website/site-model.mjs changed.
const head = git('rev-parse', 'HEAD').toString().trim();
const trackedChanges = git('diff', '--name-only', 'HEAD').toString().trim().split('\n').filter(Boolean).sort();
equal(trackedChanges, ['website/post-build-reviews.config.json', 'website/site-model.mjs'], 'tracked working-tree changes against HEAD');
const untracked = git('ls-files', '--others', '--exclude-standard').toString().trim().split('\n').filter(Boolean);
ok(untracked.every(file => file.startsWith('docs/plans/evidence/')), 'every untracked file is evidence');
const candidateBytes = await bytes(workRoot, `${E}/source-review-python-qa-34.json`);
const candidate = JSON.parse(candidateBytes);
const beforeBytes = await bytes(workRoot, `${E}/technical-source-review-before-python-qa.json`);
const before = JSON.parse(beforeBytes);
equal(sha(candidateBytes), '019c3aa6f68418075b48da8bde9e4bb94597dfe4b97738fd6951664ae024f243', 'candidate sha256');
equal(sha(beforeBytes), '3007f99bfec220d0a5f54cf9fbca80a8717252a5762f18a221e6e20f03486011', 'prior report copy sha256');
equal(sha(git('show', `HEAD:${E}/technical-source-review.json`)), sha(beforeBytes), 'prior report copy equals the committed technical-source-review.json');
const snapshotBytes = await bytes(workRoot, `${E}/technical-source-input-snapshot-task-descriptions-34.json`);
equal(sha(snapshotBytes), '293e82dc61967d571e53884f5a0778c6c6c039a18a3ea47865092f8736a97cae', 'prior snapshot sha256');
const snapshot = JSON.parse(snapshotBytes);
const candidateFiles = [...new Set(Object.values(candidate.entries).flatMap(locales => Object.values(locales).flatMap(entry => entry.inputFiles)))].sort();
equal(candidateFiles, snapshot.files.map(file => file.path).sort(), 'candidate inputs are exactly the prior snapshot files');
const changedInputs = [];
for (const file of snapshot.files) {
  const headFile = await bytes(headRoot, file.path);
  equal(sha(headFile), file.sha256, `${file.path}: HEAD extraction equals the prior snapshot`);
  equal(sha(git('show', `HEAD:${file.path}`)), file.sha256, `${file.path}: HEAD blob equals the prior snapshot`);
  equal(sha(await bytes(workRoot, `${E}/${file.snapshotPath}`)), file.sha256, `${file.path}: content-addressed copy`);
  const current = sha(await bytes(workRoot, file.path));
  if (current !== file.sha256) changedInputs.push({ path: file.path, previousSha256: file.sha256, currentSha256: current });
}
equal(changedInputs.map(file => file.path), ['website/site-model.mjs'], 'only website/site-model.mjs differs among the candidate inputs');
const supporting = [];
for (const file of snapshot.supportingFiles) {
  const current = sha(await bytes(workRoot, file.path));
  const headSha = sha(await bytes(headRoot, file.path));
  equal(current, headSha, `${file.path}: supporting file unchanged in the working tree`);
  supporting.push({ path: file.path, equalsSnapshot: current === file.sha256 });
}
const supportingChangedSinceSnapshot = supporting.filter(file => !file.equalsSnapshot).map(file => file.path);
equal(supportingChangedSinceSnapshot, [], 'every supporting file still equals the prior snapshot');
for (const file of ['website/scripts/content-review-inputs.mjs', 'website/scripts/content-review-candidates.mjs', 'website/scripts/content-links.mjs', 'website/scripts/content-sections.mjs', 'website/astro.config.mjs', 'website/site.config.mjs', 'website/scripts/published-pages.mjs', 'website/search.config.mjs'])
  equal(sha(await bytes(workRoot, file)), sha(await bytes(headRoot, file)), `${file}: implementation byte-identical HEAD vs working tree`);
// post-build-reviews.config.json is not a review input and no review implementation reads it.
ok(!candidateFiles.includes('website/post-build-reviews.config.json'), 'post-build-reviews.config.json is not a candidate input');
ok(!(await read(workRoot, 'website/scripts/content-review-inputs.mjs')).includes('post-build-reviews'), 'the fingerprint implementation does not read post-build-reviews.config.json');
const postBuild = JSON.parse(await read(workRoot, 'website/post-build-reviews.config.json'));
const postBuildHead = JSON.parse(git('show', 'HEAD:website/post-build-reviews.config.json').toString());
const postBuildChanges = [];
for (const [environment, values] of Object.entries(postBuild.environments)) for (const [key, value] of Object.entries(values))
  if (postBuildHead.environments[environment][key] !== value) postBuildChanges.push({ key: `${environment}.${key}`, from: postBuildHead.environments[environment][key], to: value });
ok(postBuildChanges.every(change => change.from.replace('multilingual-locale-data-2026-10-28', 'multilingual-task-descriptions-2026-10-29') === change.to), 'post-build config only repoints evidence paths');
// The site-model edit is a single inserted "问答" in one line.
const headModelText = await read(headRoot, 'website/site-model.mjs');
const workModelText = await read(workRoot, 'website/site-model.mjs');
const headLines = headModelText.split('\n'), workLines = workModelText.split('\n');
equal(workLines.length, headLines.length, 'site-model.mjs line count');
const changedLines = headLines.map((line, index) => line === workLines[index] ? null : index + 1).filter(Boolean);
equal(changedLines, [182], 'site-model.mjs changes exactly line 182');
const OLD_ZH = '从 PyPI 安装 ankiforge，用简短的 Python 脚本生成 Anki 卡片，并导出可导入 Anki 的 APKG 牌组。';
const NEW_ZH = '从 PyPI 安装 ankiforge，用简短的 Python 脚本生成 Anki 问答卡片，并导出可导入 Anki 的 APKG 牌组。';
equal(workLines[181], headLines[181].replace(OLD_ZH, NEW_ZH), 'line 182 differs only by the zh-CN description');
equal(headLines[181].split(OLD_ZH).length, 2, 'old description occurs once on line 182');
equal(headModelText.split(OLD_ZH).length, 2, 'old description occurs once in site-model.mjs');
equal(workModelText, headModelText.replace(OLD_ZH, NEW_ZH), 'the whole file edit is the one description replacement');
equal(NEW_ZH, OLD_ZH.replace('生成 Anki 卡片', '生成 Anki 问答卡片'), 'the replacement inserts only 问答');
out.results.inputs = { head, trackedChanges, untrackedEvidence: untracked.length, candidateInputs: candidateFiles.length, changed: changedInputs, unchanged: candidateFiles.length - changedInputs.length, supportingFiles: supporting.length, supportingChangedSinceSnapshot, postBuildConfig: postBuildChanges, siteModelChangedLines: changedLines };

// ---------------------------------------------------------------------------------------------------------------
// 1. Fingerprints: HEAD bytes reproduce the prior report; current bytes reproduce the candidate; one targetHash moves.
const regenerated = execFileSync(process.execPath, [path.join(workRoot, 'website/scripts/content-review-candidates.mjs')], { cwd: path.join(workRoot, 'website'), maxBuffer: 1 << 28 });
equal(sha(regenerated), sha(candidateBytes), 'candidate regenerated to stdout is byte-identical');
const recorded = JSON.parse(await read(headRoot, 'website/content-reviews.json'));
const pick = value => ({ sourceHash: value.sourceHash, targetHash: value.targetHash, technicalDependencyHash: value.technicalDependencyHash });
const stale = [], changedHashes = { sourceHash: 0, targetHash: 0, technicalDependencyHash: 0 };
let records = 0;
for (const page of N.model.pages) for (const locale of localeIds) {
  records++;
  const headPage = H.model.pages.find(item => item.id === page.id);
  const oldHashes = await H.reviews.reviewInputs(headPage, locale, headRoot);
  equal(oldHashes, pick(before.entries[page.id][locale]), `${page.id}/${locale}: HEAD inputs reproduce the prior report`);
  equal(oldHashes, pick(recorded.entries[page.id][locale]), `${page.id}/${locale}: HEAD inputs equal the recorded content-reviews.json`);
  equal(recorded.entries[page.id][locale].technical.evidence.sha256, sha(beforeBytes), `${page.id}/${locale}: recorded technical evidence is the prior report`);
  const newHashes = await N.reviews.reviewInputs(page, locale, workRoot);
  equal(newHashes, pick(candidate.entries[page.id][locale]), `${page.id}/${locale}: current inputs reproduce the candidate`);
  const files = await N.reviews.reviewInputFiles(page, locale, workRoot);
  equal(files, candidate.entries[page.id][locale].inputFiles, `${page.id}/${locale}: candidate input files`);
  equal(await H.reviews.reviewInputFiles(headPage, locale, headRoot), before.entries[page.id][locale].inputFiles, `${page.id}/${locale}: prior input files`);
  equal(files, before.entries[page.id][locale].inputFiles, `${page.id}/${locale}: input file list unchanged`);
  const keys = Object.keys(changedHashes).filter(key => oldHashes[key] !== newHashes[key]);
  for (const key of keys) changedHashes[key]++;
  if (keys.length) stale.push(`${page.id}/${locale}`);
}
equal(stale, ['python-quickstart/zh-CN'], 'exactly one stale record');
equal(changedHashes, { sourceHash: 0, targetHash: 1, technicalDependencyHash: 0 }, 'only one targetHash changes');
equal(json(Object.keys(candidate.entries)), json(Object.keys(before.entries)), 'record order');
out.results.fingerprints = { records, stale, changed: changedHashes, prior: pick(before.entries['python-quickstart']['zh-CN']), current: pick(candidate.entries['python-quickstart']['zh-CN']) };

// ---------------------------------------------------------------------------------------------------------------
// 2. Site model: only the python-quickstart zh-CN description (and its metadata mirror) changed.
const paths = (a, b, prefix = '') => {
  if (Object.is(a, b)) return [];
  if (a && b && typeof a === 'object' && typeof b === 'object' && Array.isArray(a) === Array.isArray(b)) return [...new Set([...Object.keys(a), ...Object.keys(b)])].flatMap(key => paths(a[key], b[key], prefix ? `${prefix}.${key}` : key));
  return [prefix];
};
equal(N.model.pages.map(page => page.id), H.model.pages.map(page => page.id), 'page order');
const modelChanges = {};
for (const page of N.model.pages) {
  const changed = paths(H.model.pages.find(item => item.id === page.id), page).sort();
  if (changed.length) modelChanges[page.id] = changed;
}
equal(modelChanges, { 'python-quickstart': ['description.zh-CN', 'metadata.zh-CN.description'] }, 'only the python-quickstart zh-CN description changed');
const NP = N.model.pages.find(page => page.id === 'python-quickstart'), HP = H.model.pages.find(page => page.id === 'python-quickstart');
equal(HP.description['zh-CN'], OLD_ZH, 'previous zh-CN description');
equal(NP.description['zh-CN'], NEW_ZH, 'new zh-CN description');
equal(NP.metadata['zh-CN'], { title: '用 Python 生成 Anki 卡片', description: NEW_ZH }, 'zh-CN metadata');
equal(NP.metadata.en, HP.metadata.en, 'en metadata unchanged');
equal(NP.metadata.en, { title: 'Generate Anki flashcards with Python', description: 'Install ankiforge from PyPI, generate Anki flashcards with a short Python script, and export an APKG deck you can import into Anki.' }, 'en metadata');
equal(NP.title, HP.title, 'titles unchanged');
equal([NP.route, NP.type, NP.section, NP.order, json(NP.sources), json(NP.capabilities)], [HP.route, HP.type, HP.section, HP.order, json(HP.sources), json(HP.capabilities)], 'route, type, section, order, sources and capabilities unchanged');
for (const name of ['docGroups', 'navigation', 'footerNavigation', 'nextTasks', 'homepageModules']) equal(json(N.model[name]), json(H.model[name]), `${name} identical`);
for (const page of N.model.pages) for (const locale of localeIds) equal(N.model.pagePath(page.id, locale), H.model.pagePath(page.id, locale), `${page.id}/${locale} route`);
const entryDiff = paths(H.model.documentationEntries(), N.model.documentationEntries());
const entryIndex = N.model.documentationEntries().findIndex(entry => entry.id === 'python-quickstart' && entry.locale === 'zh-CN');
equal(entryDiff, [`${entryIndex}.description`], 'documentation entries differ only in the python-quickstart zh-CN description');
const starlight = config => config.integrations.find(item => item.name === 'starlight-options-probe').options;
equal(json(starlight(N.astro)), json(starlight(H.astro)), 'Starlight options (sidebar, locales, labels) identical');
const group = N.model.docGroups.find(item => item.ids.includes('python-quickstart'));
out.results.model = { changedPages: modelChanges, previous: { 'zh-CN': HP.description['zh-CN'] }, current: { 'zh-CN': NP.description['zh-CN'], en: NP.description.en }, title: NP.title, route: N.model.pagePath('python-quickstart', 'zh-CN'), group: group.labels['zh-CN'], documentationEntryIndex: entryIndex, starlightOptionsIdentical: true };

// ---------------------------------------------------------------------------------------------------------------
// 3. Page sources: the zh-CN page and its program are unchanged and support every clause of the description.
const zhSource = await read(workRoot, 'docs/zh-cn/python-quickstart.md');
const enSource = await read(workRoot, 'docs/python/quick-start.md');
const program = await read(workRoot, 'bindings/python/examples/quickstart.py');
for (const [file, text] of [['docs/zh-cn/python-quickstart.md', zhSource], ['docs/python/quick-start.md', enSource], ['bindings/python/examples/quickstart.py', program]])
  equal(text, await read(headRoot, file), `${file}: unchanged`);
for (const [locale, text] of [['zh-CN', zhSource], ['en', enSource]]) {
  const match = text.match(/<!-- source: bindings\/python\/examples\/quickstart\.py -->\n```python\n([\s\S]*?)```\n<!-- \/source -->/);
  ok(match, `${locale}: page embeds the shared quickstart program`);
  equal(match[1], program, `${locale}: embedded program equals bindings/python/examples/quickstart.py`);
}
const programLines = program.trim().split('\n').filter(line => line.trim());
equal(programLines.length, 5, 'the program is five non-blank lines');
equal(programLines.filter(line => /Note\.(\w+)\(/.test(line)).map(line => line.match(/Note\.(\w+)\(/)[1]), ['basic'], 'the program creates exactly one note, with Note.basic');
for (const needle of ['# 用 Python 生成 Anki 卡片', '[公开 PyPI 包](https://pypi.org/project/ankiforge/)', 'python -m pip install ankiforge==0.3.1', '将以下程序保存为 `main.py`', "Note.basic('hola', 'hello')", "BuildOptions.to('spanish.apkg')", 'python main.py', '在 Anki 中打开持久保存的 `spanish.apkg`，其中 **Spanish** 牌组包含 **hola → hello**', '导出不需要安装 Anki'])
  ok(zhSource.includes(needle), `zh-CN page contains ${json(needle)}`);
const headings = text => [...text.matchAll(/^(#{1,6}) (.+)$/gm)].map(match => `${match[1]} ${match[2]}`);
equal(headings(zhSource), ['# 用 Python 生成 Anki 卡片', '## 导出牌组', '## 继续学习'], 'zh-CN headings');
const base = '/anki-forge';
const rewrittenWork = N.links.rewriteMarkdown(zhSource, 'docs/zh-cn/python-quickstart.md', base, 'zh-CN');
const rewrittenHead = H.links.rewriteMarkdown(zhSource, 'docs/zh-cn/python-quickstart.md', base, 'zh-CN');
equal(rewrittenWork, rewrittenHead, 'zh-CN page links rewrite identically');
const pageLinks = [...rewrittenWork.matchAll(/\[([^\]\n]+)\]\(([^\s)]+)\)/g)].map(match => ({ text: match[1], href: match[2] }));
for (const link of pageLinks.filter(link => link.href.startsWith(`${base}/`))) {
  const [pathname] = link.href.split('#');
  const resolved = N.model.resolvePage(pathname, base);
  ok(resolved && resolved.locale === 'zh-CN', `${link.href}: resolves to a zh-CN page`);
}
const searchTerms = JSON.parse(await read(workRoot, 'website/search-terms.json')).locales['zh-CN']['python-quickstart'];
out.results.sources = { zhHeadings: headings(zhSource), programLines: programLines.length, noteConstructors: ['Note.basic'], links: pageLinks, searchTermsUnchanged: true, searchTerms };

// ---------------------------------------------------------------------------------------------------------------
// 4. API truth at the 0.3.1 release tag: Note.basic is the stock Basic note type, one question/answer template.
const tag = git('rev-parse', 'python-v0.3.1^{commit}').toString().trim();
equal([git('rev-parse', 'anki-forge-v0.3.1^{commit}').toString().trim(), git('rev-parse', 'npm-v0.3.1^{commit}').toString().trim()], [tag, tag], 'release tags name one commit');
const atTag = file => git('show', `${tag}:${file}`).toString();
const needles = {
  'bindings/python/src/ankiforge/note.py': ['def basic(front: str | Content, back: str | Content) -> Note:', 'return Note(_native.NativeNote.basic(_content(front)._handle, _content(back)._handle))'],
  'anki_forge/src/note/mod.rs': ['/// Creates a Basic note. Ordinary strings are text, on both sides.', 'pub fn basic(front: impl Into<Content>, back: impl Into<Content>) -> Self {', 'crate::schema::stock::basic()', '.field("front", front)', '.field("back", back)'],
  'anki_forge/src/schema/stock.rs': ['MODEL.get_or_init(|| stock("basic", &["front", "back"], None))'],
  'anki_forge/src/authoring_core/stock.rs': ['"basic" => Ok(StockLoweringDefaults {', 'kind: "normal".into(),', 'name: "Basic".into(),', 'authoring_field("Front", Some(0), None, false),', 'authoring_field("Back", Some(1), None, false),', '"Card 1",', '"{{Front}}",', '"{{FrontSide}}\\n\\n<hr id=answer>\\n\\n{{Back}}",'],
};
for (const [file, list] of Object.entries(needles)) { const text = atTag(file); for (const needle of list) ok(text.includes(needle), `${file}@0.3.1 contains ${json(needle)}`); }
const basicBlock = atTag('anki_forge/src/authoring_core/stock.rs').split('"basic" => Ok(StockLoweringDefaults {')[1].split('"cloze" => Ok(')[0];
equal((basicBlock.match(/authoring_template\(/g) || []).length, 1, 'Basic has exactly one template');
equal(atTag('bindings/python/src/ankiforge/note.py'), await read(workRoot, 'bindings/python/src/ankiforge/note.py'), 'note.py unchanged since the tag');
equal(atTag('bindings/python/examples/quickstart.py'), program, 'quickstart.py unchanged since the tag');
out.results.apiSources = { tag, files: Object.fromEntries(Object.entries(needles).map(([file, list]) => [file, list.length])) };

// ---------------------------------------------------------------------------------------------------------------
// 5. Runtime with the published wheel: the page's program writes one Basic note with one question/answer card.
const INSPECT = String.raw`import json, os, sqlite3, sys, tempfile, zipfile
from compression import zstd
def varints(buf):
    out, i = {}, 0
    while i < len(buf):
        key = 0; shift = 0
        while True:
            b = buf[i]; i += 1; key |= (b & 0x7f) << shift; shift += 7
            if b < 0x80: break
        field, wire = key >> 3, key & 7
        if wire == 0:
            v = 0; shift = 0
            while True:
                b = buf[i]; i += 1; v |= (b & 0x7f) << shift; shift += 7
                if b < 0x80: break
            out.setdefault(field, []).append(v)
        elif wire == 2:
            n = 0; shift = 0
            while True:
                b = buf[i]; i += 1; n |= (b & 0x7f) << shift; shift += 7
                if b < 0x80: break
            out.setdefault(field, []).append(bytes(buf[i:i + n])); i += n
        elif wire == 1: out.setdefault(field, []).append(bytes(buf[i:i + 8])); i += 8
        elif wire == 5: out.setdefault(field, []).append(bytes(buf[i:i + 4])); i += 4
        else: raise ValueError(wire)
    return out
apkg, scratch = sys.argv[1], sys.argv[2]
z = zipfile.ZipFile(apkg)
path = os.path.join(scratch, "collection.sqlite")
with open(path, "wb") as f: f.write(zstd.decompress(z.read("collection.anki21b")))
db = sqlite3.connect(path)
notetypes = []
for ntid, name, config in db.execute("select id, name, config from notetypes"):
    c = varints(config)
    notetypes.append({"name": name, "kind": c.get(1, [0])[0], "originalStockKind": c.get(9, [0])[0],
        "fields": [r[0] for r in db.execute("select name from fields where ntid = ? order by ord", (ntid,))],
        "templates": [{"name": n, "question": varints(t)[1][0].decode(), "answer": varints(t)[2][0].decode()} for n, t in db.execute("select name, config from templates where ntid = ? order by ord", (ntid,))]})
decks = dict(db.execute("select id, name from decks"))
result = {"members": z.namelist(), "notetypes": notetypes,
    "notes": [{"fields": flds.split("\x1f"), "tags": tags} for flds, tags in db.execute("select flds, tags from notes")],
    "cards": [{"ord": o, "deck": decks[d]} for o, d in db.execute("select ord, did from cards")],
    "identityModels": {k: {"kind": v["kind"], "templates": sorted(v["templates"])} for k, v in json.loads(z.read("ankiforge-identity.json"))["identity"]["models"].items()},
    "identityNotes": {k: {"model": v["model"], "cards": v["cards"]} for k, v in json.loads(z.read("ankiforge-identity.json"))["identity"]["notes"].items()}}
db.close(); os.remove(path)
print(json.dumps(result, ensure_ascii=False, sort_keys=True))
`;
const temporary = await realpath(await mkdtemp('/private/tmp/anki-forge-python-qa-probe-'));
try {
  const run = path.join(temporary, 'run'), scripts = path.join(temporary, 'scripts');
  execFileSync('mkdir', [run, scripts]);
  await copyFile(path.join(workRoot, 'bindings/python/examples/quickstart.py'), path.join(run, 'main.py'));
  await writeFile(path.join(scripts, 'inspect_apkg.py'), INSPECT);
  const version = execFileSync(python, ['-I', '-c', 'import importlib.metadata as m, sys; print(m.version("ankiforge")); print(sys.version.split()[0])'], { cwd: run }).toString().trim().split('\n');
  equal(version[0], '0.3.1', 'published package version');
  const printed = execFileSync(python, ['-I', 'main.py'], { cwd: run }).toString().trim();
  equal(printed, path.join(run, 'spanish.apkg'), 'main.py prints the persistent spanish.apkg path');
  const apkg = await bytes(run, 'spanish.apkg');
  const inspected = JSON.parse(execFileSync(python, ['-I', path.join(scripts, 'inspect_apkg.py'), path.join(run, 'spanish.apkg'), scripts], { cwd: scripts }).toString());
  equal(inspected.notetypes, [{ name: 'Basic', kind: 0, originalStockKind: 1, fields: ['Front', 'Back'], templates: [{ name: 'Card 1', question: '{{Front}}', answer: '{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}' }] }], 'one stock Basic (normal, not Cloze) note type with one question/answer template');
  equal(inspected.notes, [{ fields: ['hola', 'hello'], tags: '' }], 'one note: front hola, back hello');
  equal(inspected.cards, [{ ord: 0, deck: 'Spanish' }], 'one card from template Card 1 in deck Spanish');
  equal(inspected.identityModels, { basic: { kind: 'normal', templates: ['card'] } }, 'identity: model basic is a normal note type with one template');
  equal(inspected.identityNotes, { 'es:hola': { model: 'basic', cards: { 'template:card': 0 } } }, 'identity: note es:hola uses model basic');
  ok(inspected.members.includes('collection.anki21b'), 'APKG carries the modern collection');
  // Installed Python files equal the wheel's RECORD; the recorded pip report names the PyPI wheel.
  const site = execFileSync(python, ['-I', '-c', 'import ankiforge, os; print(os.path.dirname(os.path.dirname(ankiforge.__file__)))']).toString().trim();
  const distInfo = (await readdir(site)).find(name => /^ankiforge-0\.3\.1\.dist-info$/.test(name));
  const record = (await read(site, `${distInfo}/RECORD`)).trim().split('\n').map(line => line.split(',')).filter(([file, digest]) => file.startsWith('ankiforge/') && digest);
  for (const [file, digest] of record) equal(createHash('sha256').update(await readFile(path.join(site, file))).digest('base64url'), digest.replace(/^sha256=/, ''), `python wheel RECORD ${file}`);
  equal(sha(await bytes(site, 'ankiforge/note.py')), sha(atTag('bindings/python/src/ankiforge/note.py')), 'installed note.py equals the 0.3.1 tag');
  const pipReport = JSON.parse(await read(workRoot, 'docs/plans/evidence/multilingual-locale-data-2026-10-28/executions/basic/python/pip-report.json'));
  const wheel = pipReport.install.find(item => item.metadata.name === 'ankiforge');
  equal([wheel.metadata.version, new URL(wheel.download_info.url).host, wheel.download_info.archive_info.hashes.sha256], ['0.3.1', 'files.pythonhosted.org', '632c2dc0da33234ad0d9615092be9e218389077489723a0c00b90099c62dd5c1'], 'pip report: ankiforge 0.3.1 wheel from PyPI');
  out.results.runtime = { python: version[1], ankiforge: version[0], printed: 'spanish.apkg (absolute path in the scratch directory)', apkgBytes: apkg.length, inspected, pythonRecordFiles: record.length, wheel: { url: wheel.download_info.url, sha256: wheel.download_info.archive_info.hashes.sha256 } };
} finally {
  await rm(temporary, { recursive: true, force: true });
}

out.checks = checks;
out.passed = true;
console.log(JSON.stringify(out, null, 2));
