import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { checkSurface, captureSurface, artifactPath } from './public-surface.mjs';
import { createHash } from 'node:crypto';
import { validateHeadingMigrations } from './heading-migrations.mjs';

const config = { site: 'https://example.test', base: '/' };
const html = '<html lang="zh-CN"><head><meta name="anki-forge:page-id" content="updates"><link rel="canonical" href="https://example.test/zh-cn/docs/updates/"></head><body><main><p><span id="old-update" data-section-alias="updates"></span></p><h2 id="updates">更新原始 APKG</h2><p>Preserve the original distributed APKG.</p><a download>APKG</a></main></body></html>';
const baseline = { schemaVersion: 1, source: { commit: 'previous-verified-release' }, locales: ['zh-CN'], pages: [{ route: '/zh-cn/docs/updates/', pageId: 'updates', locale: 'zh-CN', anchors: [{ id: 'old-update', task: 'updates', heading: '更新原始 APKG' }, { id: 'updates' }], capabilities: { downloads: 1 }, entities: [] }], machineResources: [{ route: '/zh-cn/docs/updates/index.md', kind: 'markdown' }] };
async function fixture(t, text = html) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-surface-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const file = artifactPath(directory, baseline.pages[0].route);
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, text);
  await writeFile(path.join(path.dirname(file), 'index.md'), '# Updates\n');
  return directory;
}
test('historic pages and machine resources remain independent of a mutable registry', async t => {
  const directory = await fixture(t);
  assert.equal((await checkSurface(directory, baseline, config)).pages, 1);
  // Removing the page from a new registry cannot change this historic expectation.
  await rm(artifactPath(directory, baseline.pages[0].route));
  await assert.rejects(checkSurface(directory, baseline, config), /ENOENT/);
});
for (const [name, mutate, message] of [
  ['missing anchor', text => text.replace('id="old-update"', 'id="removed"'), /historic anchor missing/],
  ['alias moved to another task', text => text.replace('<h2 id="updates">更新原始 APKG</h2>', '<h2 id="different-task">Other task</h2><h2 id="updates">更新原始 APKG</h2>'), /moved to another task/],
  ['noindex', text => text.replace('</head>', '<meta name="robots" content="noindex"></head>'), /restriction/],
  ['lost download', text => text.replace(' download', ''), /downloads capability removed/],
]) test(`historic surface rejects ${name}`, async t => { await assert.rejects(checkSurface(await fixture(t, mutate(html)), baseline, config), message); });
test('historic machine URLs cannot disappear even if HTML still works', async t => {
  const directory = await fixture(t);
  await rm(path.join(directory, 'zh-cn/docs/updates/index.md'));
  await assert.rejects(checkSurface(directory, baseline, config), /ENOENT/);
});
test('a legacy installation alias for the introduction stays before later tasks', async t => {
  const intro = html.replace('data-section-alias="updates"', 'data-section-alias="_top"');
  const expected = structuredClone(baseline);
  expected.pages[0].anchors[0] = { id: 'old-update', task: '_top', heading: null };
  await assert.doesNotReject(checkSurface(await fixture(t, intro), expected, config));
  const misplaced = intro.replace('<p><span id="old-update" data-section-alias="_top"></span></p>', '').replace('</h2>', '</h2><p><span id="old-update" data-section-alias="_top"></span></p>');
  await assert.rejects(checkSurface(await fixture(t, misplaced), expected, config), /introductory task moved/);
});
test('path resolution rejects artifact traversal and a missing deployment base', () => {
  assert.throws(() => artifactPath('/tmp/artifact', '/../../outside'), /outside artifact/);
  assert.throws(() => artifactPath('/tmp/artifact', '/docs/', '/project'), /outside configured base/);
});
test('a reviewed page title can change wording while its historic top anchor remains the H1', async t => {
  const directory = await fixture(t, html.replace('<main>', '<main><h1 id="page-title">A clearer page title</h1>'));
  const expected = structuredClone(baseline);
  expected.pages[0].anchors.push({ id: 'page-title', heading: 'Earlier page title', headingLevel: 'h1' });
  await assert.doesNotReject(checkSurface(directory, expected, config));
  await writeFile(artifactPath(directory, baseline.pages[0].route), html.replace('<main>', '<main><span id="page-title"></span><h1>Other title</h1>'));
  await assert.rejects(checkSurface(directory, expected, config), /page-title anchor moved away/);
});

test('captured native heading anchors cannot move to a page-top span or another heading', async t => {
  const directory = await fixture(t);
  const filename = artifactPath(directory, baseline.pages[0].route);
  const bytes = await readFile(filename);
  const manifestFile = path.join(directory, 'manifest.json');
  await writeFile(manifestFile, JSON.stringify({ configuration: { base: '/' }, commit: 'historic', pages: [{ route: baseline.pages[0].route, id: 'updates', locale: 'zh-CN', sha256: createHash('sha256').update(bytes).digest('hex') }], resources: [] }));
  const captured = await captureSurface(directory, manifestFile, {});
  assert.equal(captured.pages[0].anchors.find(anchor => anchor.id === 'updates').heading, '更新原始 APKG');
  await assert.doesNotReject(checkSurface(directory, captured, config));
  const withoutLegacy = html.replace('<p><span id="old-update" data-section-alias="updates"></span></p>', '');
  const nativeOnly = structuredClone(captured);
  nativeOnly.pages[0].anchors = nativeOnly.pages[0].anchors.filter(anchor => anchor.id !== 'old-update');
  for (const changed of [
    withoutLegacy.replace('<main>', '<main><span id="updates"></span>').replace('<h2 id="updates">', '<h2 id="new-update">'),
    withoutLegacy.replace('<h2 id="updates">更新原始 APKG</h2>', '<h2 id="updates">Other task</h2><h2 id="new-update">更新原始 APKG</h2>'),
  ]) {
    await writeFile(filename, changed);
    await assert.rejects(checkSurface(directory, nativeOnly, config), /historic anchor.*task|historic anchor.*heading/);
  }
  await writeFile(filename, withoutLegacy.replace('<h2 id="updates">', '<span id="updates" data-section-alias="new-update"></span><h2 id="new-update">'));
  await assert.doesNotReject(checkSurface(directory, nativeOnly, config));
});

test('a source-reviewed heading rewrite retains its module, aliases and old local destinations', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-heading-migration-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const oldHeading = 'From your first deck to the next update';
  const newHeading = 'Choose your next Anki task';
  const source = `<html lang="en"><head><meta name="anki-forge:page-id" content="home"><link rel="canonical" href="https://example.test/"></head><main><section data-module="next"><span id="workflow-heading" data-section-alias="next-heading"></span><h2 id="next-heading">${oldHeading}</h2><a href="/docs/json-to-anki/">JSON</a><a href="/docs/anki-forge-vs-genanki/">Compare</a></section></main></html>`;
  const filename = path.join(directory, 'index.html');
  await writeFile(filename, source);
  const manifestFile = path.join(directory, 'manifest.json');
  await writeFile(manifestFile, JSON.stringify({ configuration: { base: '/' }, commit: 'independent-history', pages: [{ route: '/', id: 'home', locale: 'en', sha256: createHash('sha256').update(source).digest('hex') }], resources: [] }));
  const historic = await captureSurface(directory, manifestFile, {});
  assert.ok(historic.pages[0].anchors.every(anchor => anchor.containerModule === 'next'));
  const inputs = { sourceHash: 'a'.repeat(64), targetHash: 'b'.repeat(64), technicalDependencyHash: 'c'.repeat(64) };
  const axis = { status: 'reviewed', reviewerType: 'ai', reviewerRun: 'independent-fixture', openIssues: [] };
  const entry = { pageId: 'home', locale: 'en', route: '/', anchors: ['workflow-heading', 'next-heading'], fromHeading: oldHeading, toHeading: newHeading, reason: 'Independently reviewed task wording', containerModule: 'next', requiredLinks: ['/docs/json-to-anki/', '/docs/anki-forge-vs-genanki/'], inputs };
  const context = { pages: [{ id: 'home', route: '/' }, { id: 'json-to-anki', route: '/docs/json-to-anki/' }, { id: 'anki-forge-vs-genanki', route: '/docs/anki-forge-vs-genanki/' }], reviews: { entries: { home: { en: { ...inputs, technical: axis, linguistic: axis } } } }, currentInputs: { home: { en: inputs } } };
  const approved = validateHeadingMigrations({ schemaVersion: 1, entries: [entry] }, context);
  const changed = source.replace(oldHeading, newHeading);
  await writeFile(filename, changed);
  await assert.rejects(checkSurface(directory, historic, config), /historic alias moved/);
  await assert.rejects(checkSurface(directory, historic, config, [entry]), /validated against independent/);
  await assert.doesNotReject(checkSurface(directory, historic, config, approved));
  for (const [mutate, expected] of [
    [value => value.replace('data-module="next"', 'data-module="other"'), /another module/],
    [value => value.replace('<span id="workflow-heading" data-section-alias="next-heading"></span>', '').replace('<main>', '<main><span id="workflow-heading" data-section-alias="next-heading"></span>'), /detached|another module/],
    [value => value.replace('<a href="/docs/json-to-anki/">JSON</a>', ''), /lost protected local link/],
    [value => value.replace('<a href="/docs/json-to-anki/">JSON</a>', '').replace('</section>', '</section><a href="/docs/json-to-anki/">JSON</a>'), /lost protected local link/],
    [value => value.replace('href="/docs/anki-forge-vs-genanki/"', 'href="https://unrelated.test/docs/anki-forge-vs-genanki/"'), /lost protected local link/],
    [value => value.replace(newHeading, 'Another unreviewed heading'), /historic alias moved/],
  ]) {
    await writeFile(filename, mutate(changed));
    await assert.rejects(checkSurface(directory, historic, config, approved), expected);
  }
  const prefix = { site: 'https://example.test', base: '/project' };
  await writeFile(filename, changed.replace('href="https://example.test/"', 'href="https://example.test/project/"').replaceAll('href="/docs/', 'href="/project/docs/'));
  await assert.doesNotReject(checkSurface(directory, historic, prefix, approved));
});
