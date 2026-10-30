import path from 'node:path';
import { readFile, readdir, mkdir, rm, stat, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import TurndownService from 'turndown';
import { gfm } from 'turndown-plugin-gfm';
import { machineGroups } from '../machine-content.config.mjs';
import { localeIds, pagePath, pages } from '../site-model.mjs';
import { configuredLocales } from '../locale-model.mjs';
import { repository, siteConfig, withBase } from '../site.config.mjs';
import { canonicalUrl, llmsPath, llmsUrl, markdownUrl, machineResourceKind } from '../src/lib/machine-urls.mjs';
import { createTranslator } from '../src/i18n/messages.mjs';
import { loadAiQuickstart } from './ai-quickstart.mjs';
import { publishedPages } from './published-pages.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const defaultDist = path.join(website, 'dist');

const allowed = new Set('a abbr aside audio b blockquote br caption code dd del details div dl dt em figcaption figure h2 h3 h4 h5 h6 hr i img kbd li ol p pre s samp section source span strike strong sub summary sup table tbody td tfoot th thead time tr ul video'.split(' '));
const context = page => `${page.id}/${page.locale} (${page.htmlPath || page.canonical})`;

function blockToken(index) { return `AFMACHINEBLOCK${index}END`; }
function plainText(value) {
  return String(value).replace(/([\\`*_[\]<>&#])/g, '\\$1').replace(/\r/g, '&#13;').replace(/\n/g, '&#10;');
}

/** Extract the reader-visible body and its ordered, rendered code blocks. */
export function extractDocument(html, page) {
  const $ = load(html);
  const selector = page.type === 'docs' ? '.sl-markdown-content' : '[data-machine-content]';
  const body = $(selector);
  if (body.length !== 1 || !page.title?.trim()) throw new Error(`${context(page)}: expected one ${selector} and a title (found ${body.length})`);

  // Only known theme decoration and controls are discarded; hidden panels remain.
  body.find('.sl-anchor-link, [data-copy], [data-copy-status], .expressive-code .copy, .expressive-code > link, .expressive-code > script, .expressive-code figcaption .sr-only, .starlight-aside__title svg').remove();
  body.find('script').each((_, element) => {
    if ($(element).text().includes('class StarlightTabsRestore') && $(element).next().is('starlight-tabs')) $(element).remove();
  });
  body.find('starlight-tabs').each((_, element) => {
    const tabs = $(element);
    tabs.children('[role="tabpanel"]').each((__, panel) => {
      const labelId = $(panel).attr('aria-labelledby');
      const label = labelId && tabs.children('.tablist-wrapper, [role="tablist"]').find('[role="tab"]').toArray().map(node => $(node)).find(node => node.attr('id') === labelId)?.text().trim();
      if (!label) throw new Error(`${context(page)}: tab panel is missing its label`);
      $(panel).prepend($('<p>').append($('<strong>').text(label)));
    });
    tabs.children('.tablist-wrapper, [role="tablist"], starlight-tabs-restore').remove();
    element.name = 'div';
  });

  const targets = [];
  for (const element of body.find('a[href], img[src], audio[src], video[src], source[src]').toArray()) {
    const node = $(element);
    const attr = element.name === 'a' ? 'href' : 'src';
    const value = node.attr(attr);
    let url;
    try { url = new URL(value, page.canonical); } catch { throw new Error(`${context(page)}: invalid ${attr} ${value}`); }
    if (url.protocol === 'javascript:') throw new Error(`${context(page)}: javascript URL is unsupported`);
    node.attr(attr, url.href);
    targets.push(url.href);
  }

  for (const element of body.find('*').toArray()) {
    const checkbox = element.name === 'input' && $(element).attr('type') === 'checkbox' && $(element).parent().is('li');
    if (!allowed.has(element.name) && !checkbox) throw new Error(`${context(page)}: unsupported <${element.name}> component`);
    const role = $(element).attr('role');
    if (role && /^(?:button|slider|spinbutton|combobox|menu|menuitem|switch|dialog|tab|tablist)$/.test(role)) throw new Error(`${context(page)}: unsupported interactive role ${role}`);
    if (Object.keys(element.attribs || {}).some(name => /^on[a-z]+$/i.test(name))) throw new Error(`${context(page)}: unsupported interactive <${element.name}> component`);
  }

  const blocks = [], codeBlocks = [];
  const cellBlocks = ['p', 'div', 'section', 'h2', 'h3', 'h4', 'h5', 'h6', 'dl', 'hr'].flatMap(tag => [`td ${tag}`, `th ${tag}`]).join(', ');
  const complexTable = element => {
    const table = $(element);
    if (table.find(`[rowspan], [colspan], table, pre, audio, video, blockquote, ul, ol, details, figure, ${cellBlocks}`).length) return true;
    if (!table.find('tr').first().find('th').length || table.text().includes('\\|')) return true;
    if (table.find('code').toArray().some(code => /[|\n]/.test($(code).text()))) return true;
    return table.find('img').toArray().some(image => ($(image).attr('alt') || '').includes('\\|'));
  };
  function replaceBlock(node, value) {
    const token = blockToken(blocks.length);
    if (body.text().includes(token)) throw new Error(`${context(page)}: reserved block marker in content`);
    blocks.push({ token, value });
    $(node).replaceWith($('<p>').text(token));
  }
  body.find('pre').each((_, element) => {
    const pre = $(element);
    const code = pre.find('code').first();
    if (!code.length) throw new Error(`${context(page)}: pre without code`);
    const lines = code.find('.ec-line > .code');
    const text = (lines.length ? lines.toArray().map(line => $(line).text() === '\n' ? '' : $(line).text()).join('\n') : code.text()).replace(/\r\n/g, '\n');
    const classLanguage = [code, pre, ...pre.parents().toArray().map(node => $(node))].map(node => (node.attr('class') || '').match(/(?:^|\s)language-([^\s]+)/)?.[1]).find(Boolean);
    const language = pre.attr('data-language') || code.attr('data-language') || pre.closest('[data-language]').attr('data-language') || classLanguage || 'text';
    if (!/^[A-Za-z0-9_+.-]+$/.test(language)) throw new Error(`${context(page)}: invalid code language ${language}`);
    const filename = pre.closest('figure').find('figcaption .title').first().text().trim();
    const codeBlock = { language, text, filename };
    codeBlocks.push(codeBlock);
    const longest = Math.max(2, ...[...text.matchAll(/`+/g)].map(match => match[0].length));
    const fence = '`'.repeat(longest + 1);
    if (pre.parents('table').toArray().some(complexTable) || pre.parents('audio, video').length) {
      pre.replaceWith($('<pre>').attr('data-language', language).append($('<code>').text(text)));
    } else replaceBlock(element, `${fence}${language}\n${text}\n${fence}`);
  });
  body.find('.expressive-code figcaption').each((_, element) => {
    const title = $(element).find('.title').first().text().trim();
    if (title) $(element).replaceWith($('<p>').text(title));
    else $(element).remove();
  });
  body.find('table').each((_, element) => {
    const table = $(element);
    if (!$.contains(body[0], element)) return;
    if (complexTable(element)) {
      table.find('*').addBack().removeAttr('class').removeAttr('style');
      replaceBlock(element, $.html(element));
    }
  });
  body.find('audio, video').each((_, element) => replaceBlock(element, $.html(element)));
  if (!body.text().trim() && !body.find('img, audio, video').length && !blocks.length) throw new Error(`${context(page)}: empty content`);
  return { ...page, html: body.html(), codeBlocks, blocks, targets };
}

/** Render the extracted HTML body without changing fenced code bytes. */
export function renderMarkdown(document) {
  const converter = new TurndownService({ headingStyle: 'atx', codeBlockStyle: 'fenced', bulletListMarker: '-' });
  const escapeText = converter.escape.bind(converter);
  converter.escape = value => escapeText(value).replace(/&/g, '\\&').replace(/</g, '\\<');
  converter.use(gfm);
  converter.addRule('tableCell', { filter: ['th', 'td'], replacement: (content, node) => `${node.previousElementSibling ? ' ' : '| '}${content.replace(/(\\*)\|/g, (_, slashes) => `${slashes}${slashes.length % 2 ? '' : '\\'}|`)} |` });
  converter.addRule('tableBreak', { filter: node => node.nodeName === 'BR' && Boolean(node.closest('td, th')), replacement: () => '<br>' });
  converter.addRule('image', { filter: 'img', replacement: (_, node) => {
    const src = (node.getAttribute('src') || '').replace(/([\\()&])/g, '\\$1');
    const alt = (node.getAttribute('alt') || '').replace(/([!"#$%&'()*+,\-./:;<=>?@[\\\]^_`{|}~])/g, '\\$1').replace(/\r/g, '&#13;').replace(/\n/g, '&#10;');
    const title = (node.getAttribute('title') || '').replace(/([\\"&])/g, '\\$1').replace(/\r/g, '&#13;').replace(/\n/g, '&#10;');
    return src ? `![${alt}](${src}${title ? ` "${title}"` : ''})` : '';
  } });
  converter.addRule('link', { filter: node => node.nodeName === 'A' && node.getAttribute('href'), replacement: (content, node) => {
    const href = node.getAttribute('href').replace(/([\\()&])/g, '\\$1');
    const title = (node.getAttribute('title') || '').replace(/([\\"&])/g, '\\$1').replace(/\r/g, '&#13;').replace(/\n/g, '&#10;');
    return `[${content}](${href}${title ? ` "${title}"` : ''})`;
  } });
  converter.addRule('aside', { filter: node => node.nodeName === 'ASIDE' || /(?:^|\s)starlight-aside(?:\s|$)/.test(node.className || ''), replacement: content => `\n\n${content.trim().split('\n').map(line => `> ${line}`).join('\n')}\n\n` });
  converter.addRule('details', { filter: 'details', replacement: content => `\n\n${content.trim()}\n\n` });
  let body = converter.turndown(document.html);
  for (const block of [...document.blocks].reverse()) body = body.replace(block.token, (_token, offset, output) => {
    const prefix = output.slice(output.lastIndexOf('\n', offset - 1) + 1, offset);
    const continuation = /^(?:[ \t]*> ?)*[ \t]*(?:(?:[-+*]|\d+[.)])[ \t]+)?$/.test(prefix)
      ? prefix.replace(/(?:[-+*]|\d+[.)])(?=[ \t])/g, marker => ' '.repeat(marker.length)) : '';
    return block.value.replace(/\n/g, `\n${continuation}`);
  });
  const metadata = [`# ${plainText(document.title)}`, '', `Canonical page: ${document.canonical}`, `Language: ${document.locale}`, `Description: ${plainText(document.description)}`];
  if (document.publishedAt) metadata.push(`Published: ${document.publishedAt}`);
  return `${metadata.join('\n')}\n\n${body}\n`;
}

export function renderLlms(locale, documents, context) {
  const t = context.translator;
  const buildLocales = context.buildLocales ?? localeIds;
  if (!buildLocales.includes(locale)) throw new Error(`Index locale is outside the build: ${locale}`);
  const lines = ['# Anki Forge', '', `> ${t('machine.intro')}`, '', `${t('machine.version', { version: context.publicInstall.publicVersion })} ${t('machine.constraints')}`, ''];
  for (const language of Object.values(context.publicInstall.languages)) lines.push(`- ${language.label}: \`${language.install}\` (${language.runtime})`);
  for (const group of machineGroups) {
    if (group.label === 'machine.optional' && t(group.label) !== 'Optional') throw new Error(`${locale}: the Optional protocol heading must not be translated`);
    lines.push('', `## ${t(group.label)}`, '');
    for (const id of group.ids) {
      const document = documents.find(item => item.id === id && item.locale === locale);
      if (!document) {
        if (!pages.some(page => page.id === id)) throw new Error(`${id}/${locale}: curated topic is not registered`);
        continue;
      }
      const note = id === 'development' ? `${t('machine.sourceBuild')} ` : '';
      lines.push(`- [${plainText(document.title)}](${document.markdown}): ${note}${plainText(document.description)}`);
    }
    if (group.articles) for (const document of documents.filter(item => item.type === 'article' && item.locale === locale)) {
      lines.push(`- [${plainText(document.title)}](${document.markdown}): ${t('machine.history', { date: document.publishedAt.slice(0, 10) })} ${plainText(document.description)}`);
    }
    if (group.repository) lines.push(`- [GitHub](${context.repository}): ${t('machine.repository')}`);
    if (group.otherLanguages) for (const other of buildLocales.filter(id => id !== locale)) {
      const language = configuredLocales[other]?.label;
      if (!language) throw new Error(`Unknown index locale: ${other}`);
      lines.push(`- [${plainText(language)}](${llmsUrl(other, context.siteConfig)}): ${plainText(t('machine.otherLanguage', { language }))}`);
    }
  }
  const output = `${lines.join('\n')}\n`;
  if (Buffer.byteLength(output, 'utf8') > 24 * 1024) throw new Error(`${locale}: llms.txt exceeds 24 KiB`);
  return output;
}

async function walk(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  return (await Promise.all(entries.map(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : [path.join(directory, entry.name)]))).flat();
}
function distPath(dist, route) {
  const target = path.resolve(dist, `.${route}`);
  if (!target.startsWith(path.resolve(dist) + path.sep)) throw new Error(`Unsafe output path: ${route}`);
  return target;
}

async function validateTarget(value, from, dist, planned) {
  const url = new URL(value);
  if (url.origin !== siteConfig.site) return;
  const prefix = withBase('/', siteConfig.base);
  if (!url.pathname.startsWith(prefix)) throw new Error(`${from}: target lost the configured base: ${value}`);
  let file = path.resolve(dist, decodeURIComponent(url.pathname.slice(prefix.length)));
  if (file !== path.resolve(dist) && !file.startsWith(path.resolve(dist) + path.sep)) throw new Error(`${from}: unsafe target ${value}`);
  if (planned.has(file)) return;
  try {
    if ((await stat(file)).isDirectory()) file = path.join(file, 'index.html');
    await stat(file);
    if (url.hash && file.endsWith('.html')) {
      const $ = load(await readFile(file, 'utf8'));
      if (!$('[id]').toArray().some(element => $(element).attr('id') === decodeURIComponent(url.hash.slice(1)))) throw new Error('missing anchor');
    }
  } catch (error) { throw new Error(`${from}: invalid local target ${value}: ${error.message}`); }
}

export async function generateMachineContent(dist = defaultDist) {
  const registry = await publishedPages(undefined, pages, localeIds);
  const expected = new Set(registry.flatMap(page => localeIds.map(locale => pagePath(page.id, locale))));
  const htmlFiles = (await walk(dist)).filter(file => file.endsWith('.html') && path.basename(file) === 'index.html');
  const actual = htmlFiles.map(file => '/' + path.relative(dist, path.dirname(file)).split(path.sep).filter(part => part !== '.').join('/') + (path.dirname(file) === dist ? '' : '/'));
  if (expected.size !== actual.length || actual.some(route => !expected.has(route))) throw new Error('Published registry and actual HTML page sets differ');
  for (const file of await walk(path.join(website, 'public'))) {
    const relative = path.relative(path.join(website, 'public'), file).split(path.sep).join('/');
    if (machineResourceKind(relative)) throw new Error(`Manual resource occupies reserved machine namespace: ${relative}`);
  }

  const documents = [];
  for (const page of registry.filter(item => ['docs', 'article'].includes(item.type))) for (const locale of localeIds) {
    const route = pagePath(page.id, locale);
    const htmlPath = distPath(dist, `${route}index.html`);
    const html = await readFile(htmlPath, 'utf8');
    const $ = load(html);
    const canonical = canonicalUrl(page.id, locale, siteConfig);
    const title = $('h1').text().trim();
    const description = $('meta[name="description"]').attr('content');
    if ($('h1').length !== 1 || !title || !description || $('link[rel="canonical"]').attr('href') !== canonical || $('html').attr('lang') !== locale) throw new Error(`${page.id}/${locale} (${htmlPath}): invalid page metadata`);
    const metadata = { id: page.id, type: page.type, locale, canonical, title, description, htmlPath, publishedAt: page.type === 'article' ? $('article time').attr('datetime') : undefined };
    if (page.type === 'article' && !metadata.publishedAt) throw new Error(`${page.id}/${locale} (${htmlPath}): missing publication date`);
    const document = extractDocument(html, metadata);
    documents.push({ ...metadata, markdown: markdownUrl(metadata), output: renderMarkdown(document), targets: document.targets });
  }
  const publicInstall = await loadAiQuickstart();
  const outputs = documents.map(document => ({ file: distPath(dist, `${pagePath(document.id, document.locale)}index.md`), data: document.output }));
  for (const locale of localeIds) {
    const route = llmsPath(locale);
    outputs.push({ file: distPath(dist, route), data: renderLlms(locale, documents, { siteConfig, repository, publicInstall, buildLocales: localeIds, translator: createTranslator(locale) }) });
  }
  const planned = new Set(outputs.map(output => path.resolve(output.file)));
  for (const document of documents) for (const target of document.targets) await validateTarget(target, `${document.id}/${document.locale}`, dist, planned);
  for (const output of outputs.filter(item => item.file.endsWith('llms.txt'))) {
    for (const match of output.data.matchAll(/\]\(([^)]+)\)/g)) await validateTarget(match[1], output.file, dist, planned);
  }
  // Nothing is removed or written until every page has been extracted successfully.
  for (const file of await walk(dist)) {
    const relative = path.relative(dist, file).split(path.sep).join('/');
    if (machineResourceKind(relative)) await rm(file);
  }
  for (const output of outputs) { await mkdir(path.dirname(output.file), { recursive: true }); await writeFile(output.file, output.data, 'utf8'); }
  console.log(`Generated ${documents.length} Markdown pages and ${localeIds.length} AI indexes.`);
  return documents;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await generateMachineContent();
