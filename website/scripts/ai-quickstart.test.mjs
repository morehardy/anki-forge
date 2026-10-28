import assert from 'node:assert/strict';
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { checkAiQuickstart, generateAiQuickstart, loadAiQuickstart, renderReadme, root } from './ai-quickstart.mjs';
import { renderPrompt } from '../src/lib/ai-quickstart.mjs';
import { importedDocs } from './content-links.mjs';
import { localeIds, pagePath } from '../site-model.mjs';

test('every language and locale renders the public setup and an empty task placeholder', async () => {
  const data = await loadAiQuickstart();
  for (const language of Object.keys(data.languages)) {
    for (const locale of localeIds) {
      const prompt = renderPrompt(data, language, ' \n ', locale);
      assert.ok(prompt.includes(data.languages[language].install));
      assert.ok(prompt.includes(data.languages[language].runtime));
      assert.ok(prompt.includes(new URL(pagePath(language === 'rust' ? 'quickstart' : `${language}-quickstart`, locale), data.languages[language].quickstart).href));
      assert.ok(prompt.includes(data.requestPlaceholders[locale]));
      assert.doesNotMatch(prompt, /\{(?:version|language|install|request)\}/);
    }
  }
});

test('user text is preserved literally and never expands template tokens or replacement syntax', async () => {
  const data = await loadAiQuickstart();
  const request = '  <img src=x onerror=alert(1)>\n</script> {version} {install} $& $$ $`\n中文  ';
  const prompt = renderPrompt(data, 'python', request);
  assert.ok(prompt.includes(`My goal:\n${request}\n\nPlease:`));
  assert.equal(prompt.split(request).length, 2);
  assert.throws(() => renderPrompt(data, 'unknown'), /Unknown/);
  assert.throws(() => renderPrompt(data, 'rust', '', 'unknown'), /Unknown/);
});

async function fixture(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'ankiforge-ai-contract-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const files = ['README.md', 'README.zh-CN.md', 'docs/ai-quickstart.json', 'docs/ai-assistants.md',
    'website/language-briefs.json', 'docs/installation.md', 'docs/compatibility.md', 'docs/media.md', 'docs/troubleshooting.md',
    'docs/node/quick-start.md', 'docs/python/quick-start.md',
    'website/src/content/docs/docs/quickstart.mdx', 'anki_forge/examples/target_api_basic.rs',
    'bindings/node/examples/quickstart.mjs', 'bindings/python/examples/quickstart.py',
    ...importedDocs.map(entry => entry.source)];
  for (const filename of files) {
    await mkdir(path.dirname(path.join(directory, filename)), { recursive: true });
    await cp(path.join(root, filename), path.join(directory, filename));
  }
  await cp(path.join(root, 'skills'), path.join(directory, 'skills'), { recursive: true });
  return directory;
}

test('check catches stale README, guide versions and Skill programs; sync changes marked content only', async t => {
  const directory = await fixture(t);
  await checkAiQuickstart({ directory });
  const filename = path.join(directory, 'README.md');
  const original = await readFile(filename, 'utf8');
  const stale = original.replace('You are helping me', 'STALE: You are helping me');
  await writeFile(filename, stale);
  await assert.rejects(checkAiQuickstart({ directory }), /stale AI brief/);
  await checkAiQuickstart({ directory, sync: true });
  assert.equal(await readFile(filename, 'utf8'), original);
  const guide = path.join(directory, 'docs/node/quick-start.md');
  const canonical = await readFile(guide, 'utf8');
  await writeFile(guide, canonical.replace(/ankiforge@\d+\.\d+\.\d+/, 'ankiforge@0.0.0'));
  await assert.rejects(checkAiQuickstart({ directory }), /public installation drift/);
  await writeFile(guide, canonical);
  const reference = path.join(directory, 'skills/anki-forge/references/python.md');
  await writeFile(reference, (await readFile(reference, 'utf8')).replace('Note.basic(', 'Note.invented('));
  await assert.rejects(checkAiQuickstart({ directory }), /quickstart differs/);
});

test('Rust Skill cannot replace the canonical quickstart with an identical different source', async t => {
  const directory = await fixture(t);
  const canonical = 'anki_forge/examples/target_api_basic.rs';
  const alternate = 'anki_forge/examples/alternate.rs';
  await cp(path.join(directory, canonical), path.join(directory, alternate));
  const reference = path.join(directory, 'skills/anki-forge/references/rust.md');
  await writeFile(reference, (await readFile(reference, 'utf8')).replace(`source: ${canonical}`, `source: ${alternate}`));
  await assert.rejects(checkAiQuickstart({ directory }), /incorrect quickstart source/);
});

test('invalid installs, URLs and missing prompt requirements fail before generation', async t => {
  const directory = await fixture(t);
  const filename = path.join(directory, 'docs/ai-quickstart.json');
  const original = JSON.parse(await readFile(filename, 'utf8'));
  const invalid = [
    [data => { data.languages.node.install = 'npm install ankiforge'; }, /incorrect public install/],
    [data => { data.languages.rust.quickstart = 'https://other.example/docs/quickstart/'; }, /inconsistent public docs origin/],
    [data => { data.prompts.en = data.prompts.en.replace('persistent', 'temporary'); }, /incomplete prompt contract/],
    [data => { data.prompts.en += '\n<script>alert(1)</script>'; }, /Unsafe prompt template/],
  ];
  for (const [mutate, error] of invalid) {
    const data = structuredClone(original);
    mutate(data);
    await writeFile(filename, JSON.stringify(data));
    await assert.rejects(loadAiQuickstart(directory), error);
  }
});

test('site data generation leaves tracked README untouched and uses the same rendered brief', async t => {
  const directory = await fixture(t);
  const data = await checkAiQuickstart({ directory });
  const before = await readFile(path.join(directory, 'README.md'), 'utf8');
  await generateAiQuickstart(data, directory);
  assert.equal(await readFile(path.join(directory, 'README.md'), 'utf8'), before);
  const generated = JSON.parse(await readFile(path.join(directory, 'website/src/generated/ai-quickstart.json'), 'utf8'));
  assert.deepEqual(generated, data);
  assert.ok(renderReadme(data, 'en').includes(renderPrompt(generated, 'rust')));
});

test('published Spanish AI prompts and empty task placeholders cannot silently fall back', { skip: !localeIds.includes('es') }, async t => {
  const directory = await fixture(t);
  const filename = path.join(directory, 'docs/ai-quickstart.json');
  const original = JSON.parse(await readFile(filename, 'utf8'));
  for (const [field, expected] of [['prompts', /es: missing published AI prompt/], ['requestPlaceholders', /es: missing localized task placeholder/]]) {
    const changed = structuredClone(original);
    delete changed[field].es;
    await writeFile(filename, JSON.stringify(changed));
    await assert.rejects(loadAiQuickstart(directory), expected);
    assert.throws(() => renderPrompt(changed, 'python', '', 'es'), /Unknown AI quickstart language or locale/);
  }
});

test('every independently specified Spanish AI requirement is enforced before generation', { skip: !localeIds.includes('es') }, async t => {
  const directory = await fixture(t);
  const filename = path.join(directory, 'docs/ai-quickstart.json');
  const original = JSON.parse(await readFile(filename, 'utf8'));
  const briefFile = path.join(directory, 'website/language-briefs.json');
  const briefs = JSON.parse(await readFile(briefFile, 'utf8'));
  for (const [requirement, phrase] of Object.entries(briefs.es.aiPromptContract)) {
    assert.ok(original.prompts.es.includes(phrase), `${requirement}: mutation must remove a real Spanish requirement`);
    const changed = structuredClone(original);
    changed.prompts.es = changed.prompts.es.replaceAll(phrase, '[requisito omitido]');
    await writeFile(filename, JSON.stringify(changed));
    await assert.rejects(loadAiQuickstart(directory), /rust\/es: incomplete prompt contract/, requirement);
  }
  await writeFile(filename, JSON.stringify(original));
  const missing = structuredClone(briefs);
  delete missing.es.aiPromptContract.baseline;
  await writeFile(briefFile, JSON.stringify(missing));
  await assert.rejects(loadAiQuickstart(directory), /es: missing AI prompt requirements/);
});
