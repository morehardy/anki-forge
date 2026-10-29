import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, mkdir, readFile, writeFile, copyFile, rm, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { rewriteMarkdown, stripDocumentTitle } from './content-links.mjs';
import { documentationEntries, pages, localeIds, configuredLocales } from '../site-model.mjs';
import { legacyAliases, sectionMapping, addSectionAliases } from './content-sections.mjs';
import legacySections from '../legacy-sections.json' with { type: 'json' };
import { siteConfig, repository } from '../site.config.mjs';
import { prepareBrandAssets } from './social-image.mjs';
import { loadBenchmarkPresentation } from './benchmark-presentation.mjs';
import { checkContent } from './check-content.mjs';
import { checkDocumentation } from './documentation.mjs';
import { checkAiQuickstart, generateAiQuickstart } from './ai-quickstart.mjs';
import { frontmatter } from './published-pages.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const root = path.resolve(website, '..');
const output = path.join(website, 'public/generated');
const generated = path.join(website, 'src/generated');
await mkdir(generated, { recursive: true });
await checkContent();
await checkDocumentation();
await generateAiQuickstart(await checkAiQuickstart());
await writeFile(path.join(generated, 'benchmark.json'), JSON.stringify(await loadBenchmarkPresentation(), null, 2) + '\n');
execFileSync('cargo', ['run', '--locked', '--quiet', '-p', 'ankiforge', '--example', 'docs_workflow', '--', path.join(root, 'target/docs-examples')], { cwd: root, stdio: 'inherit' });
execFileSync('cargo', ['run', '--locked', '--quiet', '-p', 'ankiforge', '--example', 'website_showcase', '--', output], { cwd: root, stdio: 'inherit' });

const recipe = path.join(output, 'recipes/json-to-anki');
await mkdir(recipe, { recursive: true });
await cp(path.join(root, 'docs/recipes/json-to-anki'), recipe, { recursive: true });
for (const source of ['anki_forge/examples/json_to_anki.rs', 'bindings/python/examples/json_to_anki.py', 'bindings/node/examples/json_to_anki.mts']) {
  await copyFile(path.join(root, source), path.join(recipe, path.basename(source)));
}
const csvRecipe = path.join(output, 'recipes/csv-to-anki');
await mkdir(csvRecipe, { recursive: true });
await copyFile(path.join(root, 'docs/recipes/csv-to-anki/cards.csv'), path.join(csvRecipe, 'cards.csv'));
await copyFile(path.join(root, 'bindings/python/examples/csv_to_anki.py'), path.join(csvRecipe, 'csv_to_anki.py'));

const sourcePath = 'anki_forge/examples/website_showcase.rs';
const source = await readFile(path.join(root, sourcePath), 'utf8');
const data = JSON.parse(await readFile(path.join(output, 'showcase.json'), 'utf8'));
for (const example of [...data.examples, { id: 'updates' }]) {
  const start = `// website:${example.id}:start`;
  const end = `// website:${example.id}:end`;
  if (!source.includes(start) || !source.includes(end)) throw new Error(`Missing source region: ${example.id}`);
  const code = source.split(start)[1].split(end)[0].replace(/^\n/, '').trimEnd().split('\n').map(line => line.replace(/^ {4}/, '')).join('\n');
  await writeFile(path.join(generated, `${example.id}.rs`), code);
}
data.update.artifacts = [];
for (const file of [data.update.previous, data.update.next]) {
  const bytes = await readFile(path.join(output, file));
  data.update.artifacts.push({ file, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
}
for (const example of [...data.examples, data.combined]) {
  const bytes = await readFile(path.join(output, example.file));
  example.bytes = bytes.length;
  example.sha256 = createHash('sha256').update(bytes).digest('hex');
}
data.sourceCommit = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
data.sourceModified = execFileSync('git', ['status', '--porcelain', '--', sourcePath], { cwd: root, encoding: 'utf8' }).trim().length > 0;
data.sourceSha256 = createHash('sha256').update(source).digest('hex');
await writeFile(path.join(generated, 'showcase.json'), `${JSON.stringify(data, null, 2)}\n`);
await writeFile(path.join(output, 'showcase.json'), `${JSON.stringify(data, null, 2)}\n`);
await copyFile(path.join(root, sourcePath), path.join(output, 'website_showcase.rs'));
await prepareBrandAssets(website, data.examples[0]);

// Only generated .md files in registered documentation namespaces are owned here.
// Authored English .mdx files are never removed; withdrawn candidates are cleaned too.
for (const config of Object.values(configuredLocales)) {
  const directory = path.join(website, `src/content/docs${config.prefix}/docs`);
  const entries = await readdir(directory, { withFileTypes: true }).catch(error => {
    if (error.code === 'ENOENT') return [];
    throw error;
  });
  for (const entry of entries) if (entry.isFile() && entry.name.endsWith('.md')) await rm(path.join(directory, entry.name));
}
const sections = {};
for (const page of pages.filter(page => page.type === 'docs' || page.type === 'article')) {
  sections[page.id] = {};
  for (const locale of localeIds) {
    const source = await readFile(path.join(root, page.sources[locale]), 'utf8');
    sections[page.id][locale] = sectionMapping(page.id, locale, source, legacyAliases(legacySections, page.id, locale));
  }
}
await writeFile(path.join(generated, 'sections.json'), JSON.stringify(sections, null, 2) + '\n');
for (const entry of documentationEntries()) {
  const original = await readFile(path.join(root, entry.source), 'utf8');
  const draft = frontmatter(original).draft === true;
  const withoutFrontmatter = original.replace(/^---\s*\n[\s\S]*?\n---(?:\n|$)/, '');
  let content = rewriteMarkdown(stripDocumentTitle(withoutFrontmatter).replace(/^```rust,[^\n]+$/gm, '```rust'), entry.source, siteConfig.base, entry.locale);
  content = addSectionAliases(content, sections[entry.id][entry.locale]);
  for (const [alias, target] of Object.entries(sections[entry.id][entry.locale])) {
    if (target === '_top') content = `<span id="${alias}" data-section-alias="_top"></span>\n\n` + content;
  }
  const header = `---\ntitle: ${JSON.stringify(entry.title)}\ndescription: ${JSON.stringify(entry.description)}\neditUrl: ${repository}/edit/main/${entry.source}\n${draft ? 'draft: true\n' : ''}---\n\n`;
  const destination = path.join(website, entry.output);
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(destination, header + content);
}

const cname = path.join(website, 'public/CNAME');
if (siteConfig.customDomain) await writeFile(cname, `${siteConfig.customDomain}\n`);
else await rm(cname, { force: true });
console.log('Prepared verified examples, downloads, and source-backed documentation.');
