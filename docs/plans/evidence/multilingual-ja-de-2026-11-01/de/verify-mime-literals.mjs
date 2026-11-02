// Inspect actual rendered API paragraphs, including their machine representation.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import path from 'node:path';
const [artifact, output] = process.argv.slice(2), root = process.cwd();
const require = createRequire(path.join(root, 'website/package.json'));
const { load } = require('cheerio');
const sha = value => createHash('sha256').update(value).digest('hex');
const mb = await readFile(path.join(artifact, 'generated/site-manifest.json')), manifest = JSON.parse(mb);
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
const records = [];
for (const id of ['python-api', 'node-api']) {
  const page = manifest.pages.find(p => p.id === id && p.locale === 'de');
  const relative = page.route.slice(base.length).replace(/^\//, '');
  const html = await readFile(path.join(artifact, relative, 'index.html'));
  const md = await readFile(path.join(artifact, relative, 'index.md'));
  assert.equal(sha(html), page.sha256);
  assert.equal(sha(md), manifest.resources.find(r => r.route === page.route + 'index.md').sha256);
  const $ = load(html);
  const paragraph = $('main p').filter((_, node) => $(node).text().includes('NOTE.MEDIA_USAGE_INVALID') && $(node).text().includes('image/')).first();
  const text = paragraph.text(), code = paragraph.find('code').map((_, node) => $(node).text()).get();
  const markdown = md.toString('utf8').split('\n').find(line => line.includes('NOTE.MEDIA_USAGE_INVALID') && line.includes('image/'));
  const checks = ['image/*', 'audio/*', 'video/*'].map(value => ({ value, htmlText: text.includes(value), htmlCode: code.includes(value), markdownCode: markdown?.includes('`' + value + '`') === true }));
  records.push({ id, route: page.route, htmlSha256: sha(html), markdownSha256: sha(md), text, code, markdown, checks });
}
const passed = records.every(r => r.checks.every(c => c.htmlText && c.htmlCode && c.markdownCode));
await writeFile(output, JSON.stringify({ scope: 'Actual manifest-bound German API HTML and Markdown MIME-literal regression observations.', manifestSha256: sha(mb), inputSha256: manifest.inputSha256, configuration: manifest.configuration, status: passed ? 'passed' : 'failed', records }, null, 2) + '\n');
console.log(`${passed ? 'PASS' : 'FAIL'}: six MIME literals in two German API pages, HTML text/code and machine Markdown checked.`);
assert.ok(passed, 'MIME wildcard literals must survive actual HTML and machine rendering');
