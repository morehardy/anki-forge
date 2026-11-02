import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { load } from 'cheerio';
import { micromark } from 'micromark';
import { configuredLocales, localePath } from '../locale-model.mjs';
import { headings, sectionMapping, legacyAliases } from './content-sections.mjs';

const approved = new WeakSet();
const inputKeys = ['sourceHash', 'targetHash', 'technicalDependencyHash'];
const repository = fileURLToPath(new URL('../../', import.meta.url));

/** Check a migration against current, independently reviewed source inputs.
 * The CLI first checks the complete source-review evidence, including reports.
 */
export function validateHeadingMigrations(document, { pages, reviews, currentInputs, currentSections }) {
  assert.equal(document.schemaVersion, 1, 'Unknown heading migration schema');
  assert.ok(Array.isArray(document.entries), 'Heading migrations must contain entries');
  const seen = new Set();
  const entries = document.entries.map(candidate => {
    const entry = structuredClone(candidate);
    const label = `${entry.pageId}/${entry.locale}`;
    const page = pages.find(page => page.id === entry.pageId);
    assert.ok(page && Object.hasOwn(configuredLocales, entry.locale), `${label}: unknown migration page or locale`);
    assert.equal(entry.route, localePath(page.route, entry.locale), `${label}: migration route is outside its page and locale`);
    const section = Object.hasOwn(entry, 'sectionId');
    assert.equal(section, Object.hasOwn(entry, 'headingId'), `${label}: document section and heading must be paired`);
    assert.notEqual(section, Object.hasOwn(entry, 'containerModule'), `${label}: choose exactly one migration scope`);
    if (section) {
      assert.ok(['docs', 'article'].includes(page.type), `${label}: section migration requires a document`);
      assert.ok(typeof entry.sectionId === 'string' && /^[a-z0-9][a-z0-9-]*$/.test(entry.sectionId), `${label}: semantic section required`);
      assert.ok(typeof entry.headingId === 'string' && /^[^\s"'<>]+$/u.test(entry.headingId), `${label}: native heading required`);
      assert.equal(entry.requiredLinks, undefined, `${label}: protected links belong to module scope`);
      assert.equal(entry.requiredExternalLinks, undefined, `${label}: protected external links belong to module scope`);
      const actual = currentSections?.[entry.pageId]?.[entry.locale];
      const heading = actual?.headings.find(heading => heading.id === entry.headingId);
      assert.ok(heading, `${label}: heading is absent from reviewed document source`);
      assert.equal(entry.toHeading, heading.title, `${label}: heading text differs from reviewed document source`);
      for (const anchor of [...(entry.anchors || []), entry.headingId]) {
        const target = actual.mapping[anchor] ?? (anchor === entry.sectionId && Object.values(actual.mapping).includes(anchor) ? anchor : undefined);
        assert.equal(target, entry.sectionId, `${label}#${anchor}: reviewed semantic section changed`);
      }
    } else {
      assert.ok(typeof entry.containerModule === 'string' && /^[a-z][a-z0-9-]*$/.test(entry.containerModule), `${label}: historic container module required`);
    }
    assert.ok(typeof entry.reason === 'string' && entry.reason.trim(), `${label}: reviewed migration reason required`);
    assert.ok(typeof entry.fromHeading === 'string' && entry.fromHeading.trim(), `${label}: old heading required`);
    assert.ok(typeof entry.toHeading === 'string' && entry.toHeading.trim() && entry.toHeading !== entry.fromHeading, `${label}: changed heading required`);
    assert.ok(Array.isArray(entry.anchors) && entry.anchors.length && new Set(entry.anchors).size === entry.anchors.length, `${label}: unique migrated anchors required`);
    for (const anchor of entry.anchors) {
      assert.ok(typeof anchor === 'string' && /^[^\s"'<>]+$/u.test(anchor), `${label}: invalid migrated anchor`);
      const key = `${label}#${anchor}`;
      assert.ok(!seen.has(key), `${key}: overlapping heading migrations`);
      seen.add(key);
    }
    if (!section) {
      assert.ok(Array.isArray(entry.requiredLinks) && new Set(entry.requiredLinks).size === entry.requiredLinks.length, `${label}: unique protected links required`);
      const external = entry.requiredExternalLinks ?? [];
      assert.ok(Array.isArray(external) && new Set(external).size === external.length, `${label}: unique protected external links required`);
      assert.ok(entry.requiredLinks.length + external.length > 0, `${label}: protected destinations required`);
      for (const link of entry.requiredLinks) assert.ok(pages.some(destination => localePath(destination.route, entry.locale) === link), `${label}: protected link is outside registered page and locale scope: ${link}`);
      for (const link of external) {
        const url = new URL(link);
        assert.ok(url.protocol === 'https:' && !url.username && !url.password && url.href === link, `${label}: canonical HTTPS external destination required`);
      }
    }
    if (entry.containerModule === 'next') for (const id of ['json-to-anki', 'anki-forge-vs-genanki']) {
      const destination = pages.find(page => page.id === id);
      assert.ok(destination && entry.requiredLinks.includes(localePath(destination.route, entry.locale)), `${label}: migration must retain the original local JSON and comparison links`);
    }
    const record = reviews.entries?.[entry.pageId]?.[entry.locale];
    const current = currentInputs[entry.pageId]?.[entry.locale];
    assert.ok(record && current, `${label}: reviewed current source inputs required`);
    for (const axis of ['technical', 'linguistic']) {
      assert.equal(record[axis]?.status, 'reviewed', `${label}: ${axis} source review required`);
      assert.equal(record[axis]?.reviewerType, 'ai', `${label}: independent ${axis} AI review required`);
      assert.ok(record[axis].reviewerRun?.trim(), `${label}: ${axis} reviewer identity required`);
      assert.ok(Array.isArray(record[axis].openIssues) && !record[axis].openIssues.some(issue => ['critical', 'major'].includes(issue.severity)), `${label}: unresolved ${axis} source review issues`);
    }
    assert.deepEqual(Object.keys(entry.inputs || {}).sort(), [...inputKeys].sort(), `${label}: source input hash triple required`);
    for (const key of inputKeys) {
      assert.match(entry.inputs[key], /^[0-9a-f]{64}$/, `${label}: invalid ${key}`);
      assert.equal(entry.inputs[key], current[key], `${label}: migration used stale ${key}`);
      assert.equal(record[key], current[key], `${label}: source review used stale ${key}`);
    }
    Object.freeze(entry.anchors);
    if (entry.requiredLinks) Object.freeze(entry.requiredLinks);
    if (entry.requiredExternalLinks) Object.freeze(entry.requiredExternalLinks);
    Object.freeze(entry.inputs);
    return Object.freeze(entry);
  });
  Object.freeze(entries);
  approved.add(entries);
  return entries;
}

export function assertValidatedHeadingMigrations(entries) {
  assert.ok(entries.length === 0 || approved.has(entries), 'Heading migrations must be validated against independent source reviews before surface checks');
}

export async function loadHeadingMigrations(root = repository) {
  const filename = path.join(root, 'website/public-surface-migrations.json');
  const bytes = await readFile(filename, 'utf8').catch(error => {
    if (error.code === 'ENOENT') return null;
    throw error;
  });
  if (bytes === null) return [];
  const document = JSON.parse(bytes);
  assert.equal(document.schemaVersion, 1, 'Unknown heading migration schema');
  assert.ok(Array.isArray(document.entries), 'Heading migrations must contain entries');
  if (!document.entries.length) return [];
  const [{ checkContent }, { pages }, { reviewInputs }] = await Promise.all([
    import('./check-content.mjs'), import('../site-model.mjs'), import('./content-review-inputs.mjs'),
  ]);
  // Never inherit CONTENT_REVIEW_MODE=preview for a compatibility exemption.
  await checkContent(root, { requireReviews: true });
  const reviews = JSON.parse(await readFile(path.join(root, 'website/content-reviews.json'), 'utf8'));
  const currentInputs = {};
  const currentSections = {};
  const sections = JSON.parse(await readFile(path.join(root, 'website/section-contracts.json'), 'utf8'));
  const legacy = JSON.parse(await readFile(path.join(root, 'website/legacy-sections.json'), 'utf8'));
  for (const entry of document.entries) {
    const page = pages.find(page => page.id === entry.pageId);
    assert.ok(page, `${entry.pageId}: unknown migration page`);
    currentInputs[entry.pageId] ||= {};
    currentInputs[entry.pageId][entry.locale] = await reviewInputs(page, entry.locale, root);
    if (Object.hasOwn(entry, 'sectionId')) {
      const source = await readFile(path.join(root, page.sources[entry.locale]), 'utf8');
      currentSections[entry.pageId] ||= {};
      currentSections[entry.pageId][entry.locale] = {
        mapping: sectionMapping(page.id, entry.locale, source, legacyAliases(legacy, page.id, entry.locale), sections),
        headings: headings(source).map(heading => ({ id: heading.id, title: load(micromark(heading.title)).text().replace(/\s+/g, ' ').trim() })),
      };
    }
  }
  return validateHeadingMigrations(document, { pages, reviews, currentInputs, currentSections });
}
