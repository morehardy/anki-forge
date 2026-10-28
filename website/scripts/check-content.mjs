import assert from 'node:assert/strict';
import { readFile, readdir, stat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { pages, docPages, localeIds, locales, docGroups, navigation, footerNavigation, nextTasks } from '../site-model.mjs';
import { configuredLocales } from '../locale-model.mjs';
import { validateMessages } from '../src/i18n/messages.mjs';
import { sectionMapping } from './content-sections.mjs';
import { frontmatter } from './published-pages.mjs';
import { digest, reviewInputs, rubricVersion, glossaryVersion } from './content-review-inputs.mjs';
export { digest } from './content-review-inputs.mjs';

const repository = fileURLToPath(new URL('../../', import.meta.url));
const sorted = values => [...values].sort();
async function files(directory) {
  return (await Promise.all((await readdir(directory, { withFileTypes: true })).map(entry => entry.isDirectory() ? files(path.join(directory, entry.name)) : path.join(directory, entry.name)))).flat();
}
const prose = text => text.replace(/^---\n[\s\S]*?\n---\n/, '').replace(/```[\s\S]*?```/g, '').replace(/^import .+;$/gm, '');
const code = text => [...text.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g)].map(match => match[1]).sort();

export async function checkContent(root = repository, { requireReviews = process.env.CONTENT_REVIEW_MODE !== 'preview' } = {}) {
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
  const sections = JSON.parse(await read('website/section-contracts.json'));
  const legacy = JSON.parse(await read('website/legacy-sections.json'));
  assert.equal(reviews.schemaVersion, 2, 'Unsupported source review schema');
  async function publishedOrRegistered(input, registered) {
    const registeredSet = new Set(registered);
    const result = [];
    for (const file of input) {
      const name = path.relative(root, file);
      if (registeredSet.has(name) || frontmatter(await readFile(file, 'utf8')).draft !== true) result.push(file);
    }
    return result;
  }
  for (const locale of localeIds.filter(locale => locale !== 'en')) {
    const expected = docPages.map(page => page.sources[locale]);
    const docs = await publishedOrRegistered((await files(path.join(root, `docs${locales[locale].prefix}`))).filter(file => /\.mdx?$/.test(file)), expected);
    assert.deepEqual(sorted(docs.map(file => path.relative(root, file))), sorted(expected), `Unregistered or missing ${locale} documentation`);
  }
  const expectedBlogs = pages.filter(page => page.type === 'article').flatMap(page => localeIds.map(locale => page.sources[locale]));
  const blogFiles = (await files(path.join(root, 'website/src/content/blog'))).filter(file => /\.mdx?$/.test(file)).filter(file => {
    const relative = path.relative(path.join(root, 'website/src/content/blog'), file).split(path.sep).join('/');
    const configured = Object.entries(configuredLocales).find(([, locale]) => locale.prefix && relative.startsWith(locale.prefix.slice(1) + '/'));
    return !configured || localeIds.includes(configured[0]);
  });
  const blogs = await publishedOrRegistered(blogFiles, expectedBlogs);
  assert.deepEqual(sorted(blogs.map(file => path.relative(root, file))), sorted(expectedBlogs), 'Unregistered or missing blog translation');
  // Authored Starlight entry pages are independently discovered; imported pages are generated .md.
  const entries = await publishedOrRegistered((await files(path.join(root, 'website/src/content/docs/docs'))).filter(file => file.endsWith('.mdx')), docPages.map(page => page.sources.en).filter(source => source.startsWith('website/')));
  assert.deepEqual(sorted(entries.map(file => path.relative(root, file))), sorted(docPages.map(page => page.sources.en).filter(source => source.startsWith('website/'))), 'Unregistered documentation entry');
  for (const page of pages) {
    for (const locale of localeIds) await stat(path.join(root, page.sources[locale]));
    if (page.type !== 'article') for (const locale of localeIds) {
      assert.ok(page.title?.[locale]?.trim(), `${page.id}/${locale}: missing title`);
      assert.ok(page.description?.[locale]?.trim(), `${page.id}/${locale}: missing description`);
    }
    const en = await read(page.sources.en);
    for (const locale of localeIds) {
      const target = await read(page.sources[locale]);
      if (['docs', 'article'].includes(page.type)) {
        sectionMapping(page.id, locale, target, locale === 'zh-CN' ? legacy[page.id] : {}, sections);
        if (locale !== 'en') {
          assert.notEqual(prose(en), prose(target), `${page.id}/${locale}: fallback content`);
          if (page.sources.en.endsWith('.md')) assert.deepEqual(code(target), code(en), `${page.id}/${locale}: protected executable code differs between languages`);
          if (page.type === 'article') for (const field of ['translationKey', 'date', 'category']) assert.deepEqual(frontmatter(target)[field], frontmatter(en)[field], `${page.id}/${locale}: ${field} differs`);
        }
      }
      if (requireReviews) await checkSourceReview(page, locale, reviews.entries?.[page.id]?.[locale], root);
    }
  }
  assert.deepEqual(sorted(Object.keys(reviews.entries)), sorted(pages.map(page => page.id)), 'Missing or stale content review topics');
  for (const [id, entries] of Object.entries(reviews.entries)) assert.deepEqual(sorted(Object.keys(entries)), sorted(localeIds), `${id}: missing or stale review locales`);
  console.log(`Checked ${pages.length} paired topics, real content sources, semantic sections, protected code and localized messages.`);
  if (!requireReviews) console.log('PREVIEW ONLY: independent source reviews were not accepted; this does not authorize release.');
}

async function checkSourceReview(page, locale, record, root) {
  const label = `${page.id}/${locale}`;
  assert.ok(record, `${label}: source review required`);
  const inputs = await reviewInputs(page, locale, root);
  for (const [key, expected] of Object.entries(inputs)) assert.equal(record[key], expected, `${label}: ${key} changed; independent source review required`);
  assert.equal(record.rubricVersion, rubricVersion, `${label}: stale rubric`);
  assert.equal(record.glossaryVersion, glossaryVersion, `${label}: stale glossary`);
  for (const axis of ['technical', 'linguistic']) {
    const result = record[axis];
    assert.equal(result?.status, 'reviewed', `${label}: ${axis} review required`);
    assert.equal(result.reviewerType, 'ai', `${label}: independent AI review required`);
    assert.ok(typeof result.reviewerRun === 'string' && result.reviewerRun.trim(), `${label}: reviewer run required`);
    assert.ok(Array.isArray(result.openIssues) && !result.openIssues.some(issue => ['critical', 'major'].includes(issue.severity)), `${label}: unresolved ${axis} issues`);
    const reference = result.evidence;
    assert.ok(reference?.path?.startsWith('docs/plans/evidence/') && !reference.path.split('/').includes('..'), `${label}: immutable source review evidence required`);
    const evidence = await readFile(path.join(root, reference.path));
    assert.equal(digest(evidence), reference.sha256, `${label}: ${axis} evidence digest changed`);
    const report = JSON.parse(evidence);
    assert.equal(report.reviewerRun, result.reviewerRun, `${label}: reviewer identity mismatch`);
    assert.equal(report.reviewerType, 'ai');
    const entry = report.entries?.[page.id]?.[locale];
    for (const [key, expected] of Object.entries(inputs)) assert.equal(entry?.[key], expected, `${label}: ${axis} evidence used stale ${key}`);
    const actual = entry?.[axis];
    assert.equal(actual?.status, 'reviewed', `${label}: ${axis} evidence incomplete`);
    assert.ok(Array.isArray(actual.evidence) && actual.evidence.length, `${label}: concrete ${axis} evidence required`);
    assert.deepEqual(actual.openIssues, result.openIssues, `${label}: ${axis} issues differ from evidence`);
    if (axis === 'linguistic') for (const key of ['terminology', 'syntax', 'register', 'usability']) {
      assert.equal(actual.score?.[key], result.score?.[key], `${label}: score differs from evidence`);
      assert.ok(actual.score[key] >= 4 && actual.score[key] <= 5, `${label}: inadequate ${key} score`);
    }
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await checkContent(process.argv[2] ? path.resolve(process.argv[2]) : repository);
