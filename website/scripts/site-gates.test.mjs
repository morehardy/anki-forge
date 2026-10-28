import test from 'node:test';
import assert from 'node:assert/strict';
import { cp, mkdir, mkdtemp, readFile, writeFile, rm, access } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import os from 'node:os';
import { withBase, siteConfig } from '../site.config.mjs';
import { generateMachineContent } from './machine-content.mjs';
import { publishedPages } from './published-pages.mjs';
import { machineGroups } from '../machine-content.config.mjs';
const dist = fileURLToPath(new URL('../dist/', import.meta.url));
const checker = fileURLToPath(new URL('./check-site.mjs', import.meta.url));
for (const [name, mutate, expected] of [
  ['missing action', html => html.replace(/data-update(?=[\s>])/, 'data-removed'), /update\/reset action|missing translated capability/],
  ['broken anchor', html => html.replace('id="hero-title"', 'id="removed-heading"').replace('href="#main"', 'href="#hero-title"'), /missing anchor/],
  ['wrong-language link', html => html.replace(`href="${withBase('/zh-cn/docs/languages/', siteConfig.base)}"`, `href="${withBase('/docs/languages/', siteConfig.base)}"`), /cross-language link/],
]) test(`built-site gate rejects ${name}`, async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-artifact-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const file = path.join(directory, 'zh-cn/index.html');
  const original = await readFile(file, 'utf8');
  const changed = mutate(original);
  assert.ok(changed !== original, 'Mutation must affect the real artifact');
  await writeFile(file, changed);
  assert.throws(() => execFileSync(process.execPath, [checker, directory], { encoding: 'utf8', stdio: 'pipe' }), expected);
});

test('built-site gate rejects an incomplete search artifact manifest', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-search-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const file = path.join(directory, 'generated/site-manifest.json');
  const manifest = JSON.parse(await readFile(file, 'utf8'));
  assert.ok(manifest.resources.some(resource => resource.route.includes('/pagefind/')));
  manifest.resources = manifest.resources.filter(resource => !resource.route.includes('/pagefind/'));
  await writeFile(file, JSON.stringify(manifest));
  assert.throws(() => execFileSync(process.execPath, [checker, directory], { encoding: 'utf8', stdio: 'pipe' }), /complete search index/);
});

test('built-site gate checks the mobile language switch as well as the header', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-mobile-language-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const file = path.join(directory, 'docs/index.html');
  const original = await readFile(file, 'utf8');
  const changed = original.replace(/(<div class="mobile-preferences[\s\S]*?data-language-switch[^>]*?href=")[^"]+/, `$1${withBase('/zh-cn/docs/cards/', siteConfig.base)}`);
  assert.notEqual(changed, original, 'Mutation must change the mobile control');
  await writeFile(file, changed);
  assert.throws(() => execFileSync(process.execPath, [checker, directory], { encoding: 'utf8', stdio: 'pipe' }), /switches must retain topic and release languages/);
});

test('built-site gate rejects an external footer destination on a different host', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-footer-host-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const file = path.join(directory, 'zh-cn/index.html');
  const original = await readFile(file, 'utf8');
  const changed = original.replace(/(<footer\b[\s\S]*?href=")https:\/\/github\.com\//, '$1https://example.invalid/');
  assert.notEqual(changed, original, 'Mutation must change the external footer host');
  await writeFile(file, changed);
  assert.throws(() => execFileSync(process.execPath, [checker, directory], { encoding: 'utf8', stdio: 'pipe' }), /footer destinations differ/);
});

const machineChecker = fileURLToPath(new URL('./check-machine-content.mjs', import.meta.url));
const schemaChecker = fileURLToPath(new URL('./check-structured-data.mjs', import.meta.url));
const topics = await publishedPages();
const sample = topics.find(page => page.type === 'docs' && page.id !== 'docs');
const indexTopic = machineGroups.flatMap(group => group.ids).map(id => topics.find(page => page.id === id)).find(Boolean);
if (!sample || !indexTopic) throw new Error('A published documentation article and curated topic are required for gate tests');
const sampleDirectory = sample.en.slice(1);
const indexTarget = `${indexTopic.en}index.md`;
for (const [name, file, mutate, checker, message] of [
  ['missing Markdown', `${sampleDirectory}index.md`, null, machineChecker, /Markdown coverage/],
  ['broken AI index', 'llms.txt', text => text.replace(indexTarget, '/missing-machine-gate.md'), machineChecker, /curated links/],
  ['wrong software version', 'index.html', text => text.replace(/"softwareVersion":"[^"]+"/, '"softwareVersion":"0.0.0"'), schemaChecker, /public version/],
  ['wrong article headline', `${sampleDirectory}index.html`, text => text.replace(/"headline":"[^"]+"/, '"headline":"Wrong title"'), schemaChecker, /headline/],
  ['wrong article language', `${sampleDirectory}index.html`, text => text.replace(/"inLanguage":"en"/, '"inLanguage":"zh-CN"'), schemaChecker, /language/],
  ['wrong canonical', `${sampleDirectory}index.html`, text => text.replace(/rel="canonical" href="[^"]+"/, 'rel="canonical" href="https://example.com/wrong/"'), schemaChecker, /canonical/],
]) test(`machine gate rejects ${name}`, async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-machine-gate-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const target = path.join(directory, file);
  if (mutate) {
    const original = await readFile(target, 'utf8');
    const changed = mutate(original);
    assert.notEqual(changed, original, 'Mutation must affect the artifact');
    await writeFile(target, changed);
  } else await rm(target);
  assert.throws(() => execFileSync(process.execPath, [checker, directory], { encoding: 'utf8', stdio: 'pipe' }), message);
});

test('valid links to the site root pass the machine gate', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-machine-root-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const file = path.join(directory, sampleDirectory, 'index.md');
  await writeFile(file, (await readFile(file, 'utf8')) + `\n[Home](${new URL(withBase('/', siteConfig.base), siteConfig.site).href})\n`);
  assert.doesNotThrow(() => execFileSync(process.execPath, [machineChecker, directory], { encoding: 'utf8', stdio: 'pipe' }));
});

test('generation leaves existing exports intact when an HTML link is broken', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-machine-invalid-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const html = path.join(directory, sampleDirectory, 'index.html');
  const markdown = path.join(directory, sampleDirectory, 'index.md');
  const before = await readFile(markdown);
  const original = await readFile(html, 'utf8');
  const changed = original.replace(/class="sl-markdown-content"[^>]*>/, match => `${match}<a href="${withBase('/missing-machine-target-62/', siteConfig.base)}">Broken</a>`);
  assert.notEqual(changed, original);
  await writeFile(html, changed);
  await assert.rejects(generateMachineContent(directory), /invalid local target|lost the configured base/);
  assert.deepEqual(await readFile(markdown), before);
});

test('repeated machine generation is byte-for-byte deterministic and removes stale exports', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-machine-repeat-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await cp(dist, directory, { recursive: true });
  const stale = path.join(directory, 'docs/removed-topic/index.md');
  await mkdir(path.dirname(stale), { recursive: true });
  await writeFile(stale, 'stale export');
  const first = await generateMachineContent(directory);
  await assert.rejects(access(stale), { code: 'ENOENT' });
  const names = first.map(document => path.join(directory, new URL(document.markdown).pathname.slice(withBase('/', siteConfig.base).length)));
  names.push(path.join(directory, 'llms.txt'), path.join(directory, 'zh-cn/llms.txt'));
  const before = await Promise.all(names.map(file => readFile(file)));
  await generateMachineContent(directory);
  const after = await Promise.all(names.map(file => readFile(file)));
  assert.deepEqual(after, before);
});
