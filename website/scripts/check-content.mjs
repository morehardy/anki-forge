import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, readdir, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { pages, docPages, localeIds, docGroups, navigation, footerNavigation, nextTasks } from '../site-model.mjs';
import { validateMessages } from '../src/i18n/messages.mjs';
import { headings } from './content-sections.mjs';

const repository = fileURLToPath(new URL('../../', import.meta.url));
export const digest = value => createHash('sha256').update(value).digest('hex');
const sorted = values => [...values].sort();
async function files(directory) {
  return (await Promise.all((await readdir(directory, { withFileTypes: true })).map(entry => entry.isDirectory() ? files(path.join(directory, entry.name)) : path.join(directory, entry.name)))).flat();
}
const prose = text => text.replace(/^---\n[\s\S]*?\n---\n/, '').replace(/```[\s\S]*?```/g, '').replace(/^import .+;$/gm, '');
const code = text => [...text.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g)].map(match => match[1]);

export async function checkContent(root = repository) {
  const read = name => readFile(path.join(root, name), 'utf8');
  const catalogs = Object.fromEntries(await Promise.all(localeIds.map(async locale => [locale, JSON.parse(await read(`website/src/i18n/${locale}.json`))])));
  validateMessages(catalogs);
  validateMessages(Object.fromEntries(await Promise.all(localeIds.map(async locale => [locale, JSON.parse(await read(`website/src/content/i18n/${locale}.json`))]))));
  assert.equal(new Set(pages.map(page => page.id)).size, pages.length, 'Duplicate topic identity');
  assert.deepEqual(sorted(docGroups.flatMap(group => group.ids)), sorted(docPages.map(page => page.id)), 'Sidebar must contain every document exactly once');
  for (const item of [...navigation, ...footerNavigation, ...nextTasks]) {
    if (item.pageId) assert.ok(pages.some(page => page.id === item.pageId), `Unregistered action: ${item.pageId}`);
    if (typeof item.label === 'string') for (const locale of localeIds) assert.ok(catalogs[locale][item.label], `${locale}: missing action message ${item.label}`);
  }
  const reviews = JSON.parse(await read('website/content-reviews.json'));
  const docs = await files(path.join(root, 'docs/zh-cn'));
  assert.deepEqual(sorted(docs.map(file => path.relative(root, file))), sorted(docPages.map(page => page.sources['zh-CN'])), 'Unregistered or missing Chinese documentation');
  const blogs = (await files(path.join(root, 'website/src/content/blog'))).filter(file => /\.mdx?$/.test(file));
  assert.deepEqual(sorted(blogs.map(file => path.relative(root, file))), sorted(pages.filter(page => page.type === 'article').flatMap(page => Object.values(page.sources))), 'Unregistered or missing blog translation');
  // Authored Starlight entry pages are independently discovered; imported pages are generated .md.
  const entries = (await files(path.join(root, 'website/src/content/docs/docs'))).filter(file => file.endsWith('.mdx'));
  assert.deepEqual(sorted(entries.map(file => path.relative(root, file))), sorted(docPages.map(page => page.sources.en).filter(source => source.startsWith('website/'))), 'Unregistered documentation entry');
  for (const page of pages) {
    for (const source of Object.values(page.sources)) await stat(path.join(root, source));
    if (page.type !== 'article') for (const locale of localeIds) {
      assert.ok(page.title?.[locale]?.trim(), `${page.id}/${locale}: missing title`);
      assert.ok(page.description?.[locale]?.trim(), `${page.id}/${locale}: missing description`);
    }
    if (!['docs', 'article'].includes(page.type)) continue;
    const en = await read(page.sources.en), zh = await read(page.sources['zh-CN']);
    assert.match(prose(zh), /[\u3400-\u9fff]/, `${page.id}: English fallback is not a translation`);
    assert.notEqual(prose(en), prose(zh), `${page.id}: fallback content`);
    assert.deepEqual(headings(zh).map(h => h.depth), headings(en).map(h => h.depth), `${page.id}: translated section tasks differ`);
    if (page.sources.en.endsWith('.md')) assert.deepEqual(code(zh), code(en), `${page.id}: executable code differs between languages`);
    const tableShape = text => prose(text).split('\n').filter(line => /^\|/.test(line)).map(line => (line.match(/\|/g) ?? []).length);
    assert.deepEqual(tableShape(zh), tableShape(en), `${page.id}: translated table rows/columns differ`);
    if (page.type === 'article') {
      for (const field of ['translationKey', 'date', 'category']) assert.equal(zh.match(new RegExp(`^${field}: (.+)$`, 'm'))?.[1], en.match(new RegExp(`^${field}: (.+)$`, 'm'))?.[1], `${page.id}: ${field} differs`);
    }
    const review = reviews[page.id];
    assert.equal(review?.status, 'reviewed', `${page.id}: translation review required`);
    assert.equal(review.en, digest(en), `${page.id}: English source changed; review both languages`);
    assert.equal(review['zh-CN'], digest(zh), `${page.id}: Chinese source changed; review both languages`);
  }
  assert.deepEqual(sorted(Object.keys(reviews)), sorted(pages.filter(page => ['docs', 'article'].includes(page.type)).map(page => page.id)), 'Missing or stale content review records');
  console.log(`Checked ${pages.length} paired topics, real content sources, sections, tables, code, messages and translation reviews.`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await checkContent(process.argv[2] ? path.resolve(process.argv[2]) : repository);
