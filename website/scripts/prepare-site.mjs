import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile, copyFile, rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { importedDocs, rewriteMarkdown } from './content-links.mjs';
import { siteConfig } from '../site.config.mjs';
import { prepareBrandAssets } from './social-image.mjs';
import { checkDocumentation, packageVersions } from './documentation.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const root = path.resolve(website, '..');
const output = path.join(website, 'public/generated');
const generated = path.join(website, 'src/generated');
await mkdir(generated, { recursive: true });
await checkDocumentation();
await writeFile(path.join(generated, 'versions.json'), JSON.stringify(await packageVersions(), null, 2) + '\n');
execFileSync('cargo', ['run', '--locked', '--quiet', '-p', 'anki_forge', '--example', 'docs_workflow', '--', path.join(root, 'target/docs-examples')], { cwd: root, stdio: 'inherit' });
execFileSync('cargo', ['run', '--locked', '--quiet', '-p', 'anki_forge', '--example', 'website_showcase', '--', output], { cwd: root, stdio: 'inherit' });

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
for (const example of data.examples) {
  const bytes = await readFile(path.join(output, example.file));
  example.bytes = bytes.length;
  example.sha256 = createHash('sha256').update(bytes).digest('hex');
}
data.sourceSha256 = createHash('sha256').update(source).digest('hex');
await writeFile(path.join(generated, 'showcase.json'), `${JSON.stringify(data, null, 2)}\n`);
await writeFile(path.join(output, 'showcase.json'), `${JSON.stringify(data, null, 2)}\n`);
await copyFile(path.join(root, sourcePath), path.join(output, 'website_showcase.rs'));
await prepareBrandAssets(website, data.examples[0]);

await mkdir(path.join(website, 'src/content/docs/docs'), { recursive: true });
for (const entry of importedDocs) {
  const original = await readFile(path.join(root, entry.source), 'utf8');
  const content = rewriteMarkdown(original.replace(/^# .+\r?\n+/, '').replace(/^```rust,[^\n]+$/gm, '```rust'), entry.source, siteConfig.base);
  const frontmatter = `---\ntitle: ${JSON.stringify(entry.title)}\ndescription: ${JSON.stringify(entry.description)}\neditUrl: https://github.com/morehardy/anki-forge/edit/main/${entry.source}\n---\n\n`;
  await writeFile(path.join(website, `src/content/docs/docs/${entry.name}.md`), frontmatter + content);
}

const cname = path.join(website, 'public/CNAME');
if (siteConfig.customDomain) await writeFile(cname, `${siteConfig.customDomain}\n`);
else await rm(cname, { force: true });
console.log('Prepared verified examples, downloads, and source-backed documentation.');
