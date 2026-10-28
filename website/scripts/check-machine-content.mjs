import assert from 'node:assert/strict';
import { readFile, readdir, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import { micromark } from 'micromark';
import { gfmTable, gfmTableHtml } from 'micromark-extension-gfm-table';
import { localeIds, pagePath } from '../site-model.mjs';
import { repository, siteConfig, withBase } from '../site.config.mjs';
import { machineGroups } from '../machine-content.config.mjs';
import { createTranslator } from '../src/i18n/messages.mjs';
import { publishedPages } from './published-pages.mjs';
import { loadAiQuickstart } from './ai-quickstart.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const root = path.resolve(website, '..');
const dist = process.argv[2] ? path.resolve(process.argv[2]) : path.join(website, 'dist');
const topics = await publishedPages();
const install = await loadAiQuickstart();
const abs = route => new URL(withBase(route, siteConfig.base), siteConfig.site).href;
const mdRoute = (id, locale) => `${pagePath(id, locale)}index.md`;
const relative = route => route.replace(/^\//, '');
const text = bytes => new TextDecoder('utf-8', { fatal: true }).decode(bytes);
async function walk(directory) {
  return (await Promise.all((await readdir(directory, { withFileTypes: true })).map(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : [path.join(directory, entry.name)]))).flat();
}
const files = await walk(dist);
const expectedMd = topics.filter(page => ['docs', 'article'].includes(page.type)).flatMap(page => localeIds.map(locale => relative(mdRoute(page.id, locale)))).sort();
const actualMd = files.map(file => path.relative(dist, file).split(path.sep).join('/')).filter(file => /^(?:zh-cn\/)?(?:docs|blog)\/(?:.*\/)?index\.md$/.test(file)).sort();
assert.deepEqual(actualMd, expectedMd, 'Markdown coverage differs from published topics');
const details = new Map();

async function localTarget(value, from) {
  let url;
  try { url = new URL(value); } catch { throw new Error(`${from}: invalid or non-absolute link ${value}`); }
  assert.notEqual(url.protocol, 'javascript:', `${from}: javascript URL`);
  if (url.origin !== siteConfig.site) return;
  const prefix = withBase('/', siteConfig.base);
  assert.ok(url.pathname.startsWith(prefix), `${from}: link lost the configured base: ${value}`);
  let filename = path.resolve(dist, decodeURIComponent(url.pathname.slice(prefix.length)));
  assert.ok(filename === path.resolve(dist) || filename.startsWith(path.resolve(dist) + path.sep), `${from}: unsafe target`);
  if ((await stat(filename)).isDirectory()) filename = path.join(filename, 'index.html');
  await stat(filename);
  if (url.hash && filename.endsWith('.html')) {
    const $ = load(await readFile(filename, 'utf8'));
    assert.ok($('[id]').toArray().some(element => $(element).attr('id') === decodeURIComponent(url.hash.slice(1))), `${from}: missing anchor ${value}`);
  }
}

function htmlCodes($, body) {
  return body.find('pre').toArray().map(node => {
    const pre = $(node), code = pre.find('code').first();
    const copy = pre.closest('figure').find('.copy button[data-code]').attr('data-code');
    const lines = code.find('.ec-line > .code');
    const raw = lines.length ? lines.toArray().map(line => $(line).text() === '\n' ? '' : $(line).text()).join('\n') : code.text();
    if (copy !== undefined) assert.equal(raw.replace(/\r\n/g, '\n'), copy.replace(/\u007f/g, '\n').replace(/\r\n/g, '\n'), 'Rendered code differs from the copyable source');
    const classLanguage = [code, pre, ...pre.parents().toArray().map(node => $(node))].map(node => (node.attr('class') || '').match(/(?:^|\s)language-([^\s]+)/)?.[1]).find(Boolean);
    const language = pre.attr('data-language') || code.attr('data-language') || pre.closest('[data-language]').attr('data-language') || classLanguage || 'text';
    return { language, text: raw.replace(/\r\n/g, '\n') };
  });
}

const examples = {
  quickstart: (await readFile(path.join(root, 'anki_forge/examples/target_api_basic.rs'), 'utf8')).replace(/\r\n/g, '\n').replace(/\n$/, ''),
};
const source = await readFile(path.join(root, 'anki_forge/examples/website_showcase.rs'), 'utf8');
examples.update = source.split('// website:updates:start')[1].split('// website:updates:end')[0].replace(/^\n/, '').trimEnd().split('\n').map(line => line.replace(/^ {4}/, '')).join('\n');

for (const topic of topics) for (const locale of localeIds) {
  const route = pagePath(topic.id, locale);
  const label = `${topic.id}/${locale}`;
  const $ = load(await readFile(path.join(dist, relative(route), 'index.html'), 'utf8'));
  const links = $('link[rel="describedby"]');
  const llms = abs(locale === 'en' ? '/llms.txt' : '/zh-cn/llms.txt');
  assert.equal(links.length, 1, `${label}: discovery link count`);
  assert.equal(links.attr('href'), llms, `${label}: wrong index discovery`);
  assert.ok($('footer a[href]').toArray().some(node => $(node).attr('href') === llms), `${label}: missing visible index link`);
  const isDocument = ['docs', 'article'].includes(topic.type);
  const alternate = $('link[rel="alternate"][type="text/markdown"]');
  assert.equal(alternate.length, isDocument ? 1 : 0, `${label}: Markdown discovery count`);
  if (!isDocument) continue;
  const md = abs(mdRoute(topic.id, locale));
  details.set(md, { title: $('h1').text().trim(), description: $('meta[name="description"]').attr('content'), date: $('article time').attr('datetime') });
  assert.equal(alternate.attr('href'), md, `${label}: incorrect Markdown discovery`);
  assert.ok($('a[href]').toArray().some(node => $(node).attr('href') === md), `${label}: missing visible Markdown link`);
  const markdown = text(await readFile(path.join(dist, relative(mdRoute(topic.id, locale)))));
  assert.ok(markdown.endsWith('\n') && !markdown.includes('\r'), `${label}: Markdown encoding`);
  const rendered = load(micromark(markdown, { allowDangerousHtml: true, extensions: [gfmTable()], htmlExtensions: [gfmTableHtml()] }));
  assert.equal(rendered('h1').length, 1, `${label}: expected one H1`);
  assert.equal(rendered('h1').text(), $('h1').text().trim(), `${label}: incorrect title`);
  const lines = markdown.split('\n');
  assert.equal(lines[1], '', `${label}: incorrect metadata separator`);
  assert.equal(lines[2], `Canonical page: ${abs(route)}`, `${label}: incorrect canonical metadata`);
  assert.equal(lines[3], `Language: ${locale}`, `${label}: incorrect language metadata`);
  assert.equal(load(micromark(lines[4]))('p').text(), `Description: ${$('meta[name="description"]').attr('content')}`, `${label}: incorrect description metadata`);
  if (topic.type === 'article') assert.equal(lines[5], `Published: ${$('article time').attr('datetime')}`, `${label}: incorrect publication metadata`);
  assert.equal(lines[topic.type === 'article' ? 6 : 5], '', `${label}: missing body separator`);
  const body = $(topic.type === 'docs' ? '.sl-markdown-content' : '[data-machine-content]');
  assert.equal(body.length, 1, `${label}: content selector`);
  const codes = rendered('pre > code').toArray().map(node => {
    const code = rendered(node), pre = code.parent();
    const raw = pre.attr('data-language');
    const language = raw || (code.attr('class') || '').match(/(?:^|\s)language-([^\s]+)/)?.[1] || 'text';
    const content = code.text();
    return { language, text: raw ? content : content.replace(/\n$/, '') };
  });
  assert.deepEqual(codes, htmlCodes($, body), `${label}: Markdown code differs from HTML`);
  if (topic.id === 'quickstart' || topic.id === 'blog:your-first-anki-deck') assert.ok(codes.some(code => code.text === examples.quickstart), `${label}: quickstart source fixture differs`);
  if (topic.id === 'blog:stable-ids-and-deck-updates') assert.ok(codes.some(code => code.text === examples.update), `${label}: update source fixture differs`);
  const destinations = rendered('a[href], img[src], audio[src], video[src], source[src]').toArray().map(node => rendered(node).attr('href') || rendered(node).attr('src'));
  for (const destination of destinations) await localTarget(destination, label);
}

for (const locale of localeIds) {
  const t = createTranslator(locale);
  const filename = locale === 'en' ? 'llms.txt' : 'zh-cn/llms.txt';
  const bytes = await readFile(path.join(dist, filename));
  const content = text(bytes);
  assert.ok(bytes.length <= 24 * 1024 && content.endsWith('\n') && !content.endsWith('\n\n') && !content.includes('\r'), `${filename}: invalid size or newlines`);
  assert.equal((content.match(/^# /gm) || []).length, 1);
  assert.deepEqual([...content.matchAll(/^## (.+)$/gm)].map(match => match[1]), machineGroups.map(group => t(group.label)), `${filename}: group order`);
  assert.ok(content.includes(install.publicVersion), `${filename}: public version missing`);
  assert.ok(content.includes(t('machine.intro')) && content.includes(t('machine.constraints')), `${filename}: introduction or constraints missing`);
  for (const language of Object.values(install.languages)) assert.ok(content.includes(language.install) && content.includes(language.runtime), `${filename}: installation missing`);
  const index = load(micromark(content));
  const entries = index('li > a').toArray().map(node => ({ title: index(node).text(), href: index(node).attr('href'), description: index(node).parent().text().slice(index(node).text().length + 2) }));
  const links = entries.map(entry => entry.href);
  const expected = machineGroups.flatMap(group => [
    ...group.ids.filter(id => topics.some(topic => topic.id === id)).map(id => abs(mdRoute(id, locale))),
    ...(group.articles ? topics.filter(topic => topic.type === 'article').map(topic => abs(mdRoute(topic.id, locale))) : []),
    ...(group.repository ? [repository] : []),
    ...(group.otherLanguage ? [abs(locale === 'en' ? '/zh-cn/llms.txt' : '/llms.txt')] : []),
  ]);
  assert.deepEqual(links, expected, `${filename}: curated links or order differ`);
  for (const { title, href, description } of entries) {
    const page = details.get(href);
    if (!page) continue;
    assert.equal(title, page.title, `${filename}: link title differs from the rendered page`);
    assert.ok(description.includes(page.description), `${filename}: link description differs from the rendered page`);
    if (page.date) assert.ok(description.includes(page.date.slice(0, 10)) && description.includes(t('machine.history', { date: page.date.slice(0, 10) })), `${filename}: historical article date missing`);
  }
  if (topics.some(topic => topic.id === 'development')) assert.ok(content.includes(t('machine.sourceBuild')), `${filename}: source-build context missing`);
  for (const link of links) await localTarget(link, filename);
}
const errorPage = load(await readFile(path.join(dist, '404.html'), 'utf8'));
assert.equal(errorPage('link[rel="describedby"], link[type="text/markdown"]').length, 0, '404 has machine discovery links');
console.log(`Verified ${expectedMd.length} Markdown pages, code fixtures, links and bilingual AI indexes.`);
