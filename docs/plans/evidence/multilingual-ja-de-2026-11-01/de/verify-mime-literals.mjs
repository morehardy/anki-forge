// Inspect actual rendered API paragraphs, including their machine representation.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
const [artifact, output] = process.argv.slice(2), root = process.cwd();
const require = createRequire(path.join(root, 'website/package.json'));
const { load } = require('cheerio');
const sha = value => createHash('sha256').update(value).digest('hex');
const mb = await readFile(path.join(artifact, 'generated/site-manifest.json')), manifest = JSON.parse(mb);
const base = manifest.configuration.base === '/' ? '' : manifest.configuration.base;
let server;
if (process.argv.includes('--http')) {
  process.env.SITE_URL = manifest.configuration.site;
  process.env.SITE_BASE = manifest.configuration.base;
  process.env.SITE_CUSTOM_DOMAIN = manifest.configuration.customDomain || '';
  const { serveBuiltSite } = await import(pathToFileURL(path.join(root, 'website/scripts/serve-built-site.mjs')));
  server = await serveBuiltSite({ directory: artifact });
}
const captureDirectory = path.join(path.dirname(output), path.basename(output, '.json') + '-http');
const records = [];
try {
for (const id of ['python-api', 'node-api', 'rust-api', 'media', 'rust-guide']) {
  const page = manifest.pages.find(p => p.id === id && p.locale === 'de');
  const relative = page.route.slice(base.length).replace(/^\//, '');
  const html = await readFile(path.join(artifact, relative, 'index.html'));
  const md = await readFile(path.join(artifact, relative, 'index.md'));
  assert.equal(sha(html), page.sha256);
  assert.equal(sha(md), manifest.resources.find(r => r.route === page.route + 'index.md').sha256);
  const responses = [];
  if (server) {
    await mkdir(captureDirectory, { recursive: true });
    for (const [suffix, extension, expected] of [['', 'html', html], ['index.md', 'md', md]]) {
      const response = await fetch(server.origin + page.route + suffix, { redirect: 'manual' });
      const bytes = Buffer.from(await response.arrayBuffer());
      assert.equal(response.status, 200);
      assert.equal(sha(bytes), sha(expected));
      if (extension === 'md') {
        assert.match(response.headers.get('content-type'), /^text\/(?:markdown|plain); charset=utf-8$/);
        assert.equal(response.headers.get('content-disposition'), null);
      } else assert.match(response.headers.get('content-type'), /^text\/html/);
      const filename = path.join(captureDirectory, `${id}.${extension}`);
      await writeFile(filename, bytes);
      responses.push({ url: response.url, status: response.status, headers: Object.fromEntries(response.headers), path: path.relative(root, filename), sha256: sha(bytes), bytes: bytes.length });
    }
  }
  const $ = load(html);
  const paragraph = $('main p').filter((_, node) => $(node).text().includes('NOTE.MEDIA_USAGE_INVALID') && $(node).text().includes('image/')).first();
  const text = paragraph.text(), code = paragraph.find('code').map((_, node) => $(node).text()).get();
  const markdown = md.toString('utf8').split('\n').find(line => line.includes('NOTE.MEDIA_USAGE_INVALID') && line.includes('image/'));
  const checks = ['image/*', 'audio/*', 'video/*'].map(value => ({ value, htmlText: text.includes(value), htmlCode: code.includes(value), markdownCode: markdown?.includes('`' + value + '`') === true }));
  records.push({ id, route: page.route, htmlSha256: sha(html), markdownSha256: sha(md), text, code, markdown, checks, responses });
}
const passed = records.every(r => r.checks.every(c => c.htmlText && c.htmlCode && c.markdownCode));
await writeFile(output, JSON.stringify({ scope: 'Actual manifest-bound German documentation HTML and Markdown MIME-literal regression observations.', helper: { path: path.relative(root, process.argv[1]), sha256: sha(await readFile(process.argv[1])) }, readOrigin: server?.origin, manifestSha256: sha(mb), inputSha256: manifest.inputSha256, configuration: manifest.configuration, status: passed ? 'passed' : 'failed', records }, null, 2) + '\n');
console.log(`${passed ? 'PASS' : 'FAIL'}: ${records.length * 3} MIME literals in ${records.length} German pages, HTML text/code and machine Markdown checked.`);
assert.ok(passed, 'MIME wildcard literals must survive actual HTML and machine rendering');
} finally { await server?.close(); }
