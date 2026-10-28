import test from 'node:test';
import assert from 'node:assert/strict';
import { load } from 'cheerio';
import { micromark } from 'micromark';
import { gfmTable, gfmTableHtml } from 'micromark-extension-gfm-table';
import { extractDocument, renderMarkdown, renderLlms } from './machine-content.mjs';

const page = { id: 'quickstart', type: 'docs', locale: 'en', canonical: 'https://example.org/project/docs/quickstart/', title: 'A guide', description: 'A description', htmlPath: 'docs/quickstart/index.html' };

test('rendered highlighted code and links survive Markdown export', () => {
  const html = `<div class="sl-markdown-content"><p>See <a href="../media/?size=2#sound">media</a>.</p>
    <div class="expressive-code"><figure><figcaption><span class="title">demo.rs</span></figcaption><pre data-language="rust"><code><div class="ec-line"><div class="code">中文 &lt;x&gt;  </div></div><div class="ec-line"><div class="code">\n</div></div><div class="ec-line"><div class="code">\`\`\`</div></div></code></pre><div class="copy"><button>Copy</button></div></figure></div></div>`;
  const document = extractDocument(html, page);
  const markdown = renderMarkdown(document);
  assert.deepEqual(document.codeBlocks, [{ language: 'rust', text: '中文 <x>  \n\n```', filename: 'demo.rs' }]);
  assert.match(markdown, /\[media\]\(https:\/\/example\.org\/project\/docs\/media\/\?size=2#sound\)/);
  assert.ok(markdown.includes('demo.rs'));
  assert.ok(markdown.includes('````rust\n中文 <x>  \n\n```\n````'));
  assert.ok(!markdown.includes('Copy'));
});

const indexContext = (overrides = {}) => ({
  siteConfig: { site: 'https://example.org', base: '/project', customDomain: '' },
  repository: 'https://github.com/example/project',
  publicInstall: { publicVersion: '1.2.3', languages: { python: { label: 'Python', install: 'pip install example', runtime: 'Python 3.11+' } } },
  buildLocales: ['en', 'zh-CN', 'es'],
  translator: (key, values = {}) => key === 'machine.optional' ? 'Optional' : `${key} ${values.version || values.language || ''}`,
  ...overrides,
});

test('a third-language index enumerates only the other build languages and keeps fixed protocol labels', () => {
  const output = renderLlms('es', [], indexContext());
  assert.ok(output.includes('[English](https://example.org/project/llms.txt)'));
  assert.ok(output.includes('[简体中文](https://example.org/project/zh-cn/llms.txt)'));
  assert.ok(!output.includes('https://example.org/project/es/llms.txt'));
  assert.ok(!output.includes('/fr/llms.txt'));
  assert.match(output, /\n## Optional\n/);
  assert.throws(() => renderLlms('es', [], indexContext({ translator: () => 'Opcional' })), /Optional protocol/);
  assert.throws(() => renderLlms('fr', [], indexContext()), /outside the build/);
  const markdown = renderMarkdown(extractDocument('<div class="sl-markdown-content"><p>Contenido.</p></div>', { ...page, locale: 'es', title: 'Título' }));
  for (const label of ['Canonical page:', 'Language: es', 'Description:']) assert.ok(markdown.includes(label));
});

test('the index limit counts UTF-8 bytes rather than translated character count', () => {
  const normal = indexContext();
  const large = indexContext({ translator: (key, values) => key === 'machine.intro' ? '日'.repeat(9000) : normal.translator(key, values) });
  assert.throws(() => renderLlms('es', [], large), /24 KiB/);
  const empty = renderLlms('es', [], indexContext({ translator: (key, values) => key === 'machine.intro' ? '' : normal.translator(key, values) }));
  const budget = 24 * 1024 - Buffer.byteLength(empty, 'utf8');
  const exact = indexContext({ translator: (key, values) => key === 'machine.intro' ? 'a'.repeat(budget) : normal.translator(key, values) });
  assert.equal(Buffer.byteLength(renderLlms('es', [], exact), 'utf8'), 24 * 1024);
  assert.throws(() => renderLlms('es', [], indexContext({ translator: (key, values) => key === 'machine.intro' ? 'a'.repeat(budget + 1) : normal.translator(key, values) })), /24 KiB/);
});

test('missing, duplicated and unsupported content fails with page context', () => {
  assert.throws(() => extractDocument('<main>other</main>', page), /quickstart.*en.*docs\/quickstart\/index\.html/);
  assert.throws(() => extractDocument('<div class="sl-markdown-content">a</div><div class="sl-markdown-content">b</div>', page), /quickstart.*en.*docs\/quickstart\/index\.html/);
  assert.throws(() => extractDocument('<div class="sl-markdown-content"><iframe src="video"></iframe></div>', page), /iframe/);
  assert.throws(() => extractDocument('<div class="sl-markdown-content"><div role="slider">Choose a value</div></div>', page), /slider/);
});

test('tables, hidden tabs, details, ordered lists and resources retain their meaning', () => {
  const html = `<div class="sl-markdown-content">
    <table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>x | y</td><td>z</td></tr></tbody></table>
    <table><tr><th colspan="2">Wide</th></tr><tr><td>left</td><td>right</td></tr></table>
    <starlight-tabs><ul role="tablist"><li><a role="tab" id="one">First</a></li><li><a role="tab" id="two">Second</a></li></ul><div role="tabpanel" aria-labelledby="one"><p>Visible</p></div><div role="tabpanel" aria-labelledby="two" hidden><p>Hidden panel</p></div></starlight-tabs>
    <aside class="starlight-aside"><p class="starlight-aside__title">Caution</p><p>Important text.</p></aside>
    <details><summary>More details</summary><p>Folded text</p></details>
    <ol start="3"><li>Third<ul><li>Nested</li></ul></li></ol>
    <a href="../download/"><img src="../image.png" alt="diagram" title="Image title"></a><audio controls><source src="../sound.wav"><a href="../sound.wav">Download audio</a></audio>
  </div>`;
  const markdown = renderMarkdown(extractDocument(html, page));
  for (const text of ['x \\| y', 'colspan="2"', 'left', 'right', 'First', 'Visible', 'Second', 'Hidden panel', '> Caution', 'Important text.', 'More details', 'Folded text', '3.', 'Nested', 'diagram', 'Image title', 'https://example.org/project/docs/image.png', 'https://example.org/project/docs/sound.wav', 'Download audio']) assert.ok(markdown.includes(text), `missing ${text}\n${markdown}`);
});

test('nested code and line breaks survive complex tables, quotes, and lists', () => {
  const html = `<div class="sl-markdown-content">
    <table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>line 1<br>line 2</td><td>ok</td></tr></tbody></table>
    <table><tr><th colspan="2">Example</th></tr><tr><td><pre class="language-python"><code>one\ntwo</code></pre></td><td>Second</td></tr></table>
    <table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td><p>First</p><p>Second</p></td><td>Other</td></tr></tbody></table>
    <blockquote><p>Explanation</p><pre class="language-rust"><code>quote  \nsecond</code></pre></blockquote>
    <ul><li><pre><code>list\nsecond</code></pre></li></ul>
    <div class="language-json"><pre><code>{}</code></pre></div>
  </div>`;
  const markdown = renderMarkdown(extractDocument(html, page));
  assert.ok(markdown.includes('| line 1<br>line 2 | ok |'), markdown);
  assert.ok(markdown.includes('<pre data-language="python"><code>one\ntwo</code></pre>'), markdown);
  assert.ok(markdown.includes('<td><p>First</p><p>Second</p></td><td>Other</td>'), markdown);
  assert.ok(markdown.includes('> ```rust\n> quote  \n> second\n> ```'), markdown);
  assert.ok(markdown.includes('-   ```text\n    list\n    second\n    ```'), markdown);
  assert.ok(markdown.includes('```json\n{}\n```'), markdown);
  assert.ok(!markdown.includes('AFMACHINEBLOCK'));
  const $ = load(micromark(markdown, { allowDangerousHtml: true }));
  assert.equal($('table pre code').text(), 'one\ntwo');
  assert.equal($('blockquote pre code').text(), 'quote  \nsecond\n');
  assert.equal($('ul li pre code').text(), 'list\nsecond\n');
});

test('images keep quoted titles and punctuation in their URLs and alternate text', () => {
  const html = `<div class="sl-markdown-content"><p>Literal &amp;copy; and &lt;word&gt;.</p><a href="../page/?x=1&amp;copy;" title="link &amp;copy;"><img src="../image(1).png" alt="a [diagram] **bold** &#96;code&#96; &amp;copy;" title="caption &quot;quoted&quot; &amp;copy;"></a></div>`;
  const markdown = renderMarkdown(extractDocument(html, page));
  const $ = load(micromark(markdown));
  assert.ok($('p').toArray().some(node => $(node).text() === 'Literal &copy; and <word>.'));
  assert.equal($('a img').attr('alt'), 'a [diagram] **bold** `code` &copy;');
  assert.equal($('a img').attr('title'), 'caption "quoted" &copy;');
  assert.equal($('a img').attr('src'), 'https://example.org/project/docs/image(1).png');
  assert.equal($('a').attr('href'), 'https://example.org/project/docs/page/?x=1&copy;');
  assert.equal($('a').attr('title'), 'link &copy;');
});

test('a pipe in a table image alt remains a pipe and does not split the cell', () => {
  const html = `<div class="sl-markdown-content"><table><thead><tr><th>Image</th><th>Label</th></tr></thead><tbody><tr><td><img src="../x.png" alt="a|b"></td><td>Other</td></tr></tbody></table></div>`;
  const markdown = renderMarkdown(extractDocument(html, page));
  const $ = load(micromark(markdown, { extensions: [gfmTable()], htmlExtensions: [gfmTableHtml()] }));
  assert.equal($('tbody tr td').length, 2);
  assert.equal($('tbody img').attr('alt'), 'a|b');
  assert.equal($('tbody td').last().text(), 'Other');
});

test('table code containing escaped pipes retains its original characters', () => {
  const html = `<div class="sl-markdown-content"><table><thead><tr><th>Code</th><th>Label</th></tr></thead><tbody><tr><td><code>a\\|b</code></td><td>Other</td></tr></tbody></table></div>`;
  const markdown = renderMarkdown(extractDocument(html, page));
  const $ = load(micromark(markdown, { allowDangerousHtml: true, extensions: [gfmTable()], htmlExtensions: [gfmTableHtml()] }));
  assert.equal($('tbody tr td').length, 2);
  assert.equal($('tbody code').text(), 'a\\|b');
});

test('image alt and title retain attribute newlines', () => {
  const html = `<div class="sl-markdown-content"><img src="image.png" alt="first&#10;second" title="first&#10;second"></div>`;
  const $ = load(micromark(renderMarkdown(extractDocument(html, page))));
  assert.equal($('img').attr('alt'), 'first\nsecond');
  assert.equal($('img').attr('title'), 'first\nsecond');
});

test('page metadata stays plain text when titles contain Markdown or HTML syntax', () => {
  const title = 'An **example** </script> #';
  const description = 'Literal &copy; and <details>.';
  const markdown = renderMarkdown(extractDocument('<div class="sl-markdown-content"><p>Body</p></div>', { ...page, title, description }));
  const $ = load(micromark(markdown, { allowDangerousHtml: true }));
  assert.equal($('h1').text(), title);
  assert.ok($('p').toArray().some(node => $(node).text().includes(`Description: ${description}`)));
  assert.equal($('script, details').length, 0);
});
