import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import { siteConfig, withBase } from '../site.config.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const root = path.dirname(website.slice(0, -1));
const dist = path.join(website, 'dist');
export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
async function walk(directory) {
  return (await Promise.all((await readdir(directory, { withFileTypes: true })).map(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : path.join(directory, entry.name)))).flat();
}

// This inventory observes the output, independently of the topic registry. It is
// uploaded beside the Pages artifact so deployment checks know which build won.
export async function writeSiteManifest() {
  const git = args => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
  const inputs = git(['ls-files', '--cached', '--others', '--exclude-standard', '--', 'website', 'docs', 'anki_forge', 'Cargo.toml', 'Cargo.lock', 'README.md', 'README.zh-CN.md', '.github/workflows/website.yml']).split('\n').filter(Boolean).sort();
  const fingerprints = [];
  for (const input of inputs) {
    // Implementation records and unrelated local audit drafts are not build inputs.
    if (input.startsWith('docs/plans/') || input.startsWith('docs/superpowers/')) continue;
    try { fingerprints.push([input, sha256(await readFile(path.join(root, input)))]); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  const pages = [], resources = [];
  for (const file of (await walk(dist)).sort()) {
    const relative = path.relative(dist, file).split(path.sep).join('/');
    if (relative === 'generated/site-manifest.json') continue;
    const bytes = await readFile(file);
    const route = withBase('/' + relative.replace(/index\.html$/, ''), siteConfig.base);
    const entry = { route, sha256: sha256(bytes), bytes: bytes.length };
    if (relative.endsWith('.html')) {
      const $ = load(bytes.toString());
      pages.push({ ...entry, id: $('meta[name="anki-forge:page-id"]').attr('content') ?? '404', locale: $('html').attr('lang'), modules: $('[data-module]').toArray().map(node => $(node).attr('data-module')), capabilities: {
        downloads: $('[download]').length, search: $('site-search').length, codeCopy: $('[data-copy], .expressive-code .copy button').length,
        flips: $('[data-flip]').length, audio: $('audio').length, updates: $('[data-update]').length,
      } });
    } else if (relative !== 'CNAME' && !relative.startsWith('.')) resources.push(entry);
  }
  const manifest = { schemaVersion: 1, commit: git(['rev-parse', 'HEAD']), inputSha256: sha256(JSON.stringify(fingerprints)), configuration: siteConfig, pages, resources };
  await writeFile(path.join(dist, 'generated/site-manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  console.log(`Recorded ${pages.length} pages and ${resources.length} resources for commit ${manifest.commit}.`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await writeSiteManifest();
