import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { load } from '../../../../website/node_modules/cheerio/dist/esm/index.js';
import { micromark } from '../../../../website/node_modules/micromark/index.js';
import { pages } from '../../../../website/site-model.mjs';
import { reviewInputs, digest } from '../../../../website/scripts/content-review-inputs.mjs';
import { headings, sectionMapping, legacyAliases } from '../../../../website/scripts/content-sections.mjs';
import { loadHeadingMigrations, validateHeadingMigrations } from '../../../../website/scripts/heading-migrations.mjs';
import { checkSurface } from '../../../../website/scripts/public-surface.mjs';

const root = process.cwd(), directory = 'docs/plans/evidence/website-pr66-final-2026-11-02';
const parse = async filename => JSON.parse(await readFile(filename));
const fileNames = ['website/scripts/heading-migrations.mjs', 'website/scripts/public-surface.mjs', 'website/scripts/public-surface.test.mjs', 'website/public-surface-migrations.json'];
const files = Object.fromEntries(await Promise.all(fileNames.map(async filename => [filename, { sha256: digest(await readFile(filename)), utf8: await readFile(filename, 'utf8') }])));
const configuration = await parse('website/public-surface-migrations.json');
const approved = await loadHeadingMigrations(root);
assert.equal(approved.length, 12);
assert.equal(approved.filter(entry => entry.sectionId).length, 5);
const reviews = await parse('website/content-reviews.json');
const sections = await parse('website/section-contracts.json'), legacy = await parse('website/legacy-sections.json');
const context = { pages, reviews, currentInputs: {}, currentSections: {} };
const baselinePaths = ['website/public-surface-baseline.json', 'website/machine-surface-baseline.json'];
const baselines = await Promise.all(baselinePaths.map(async filename => ({ path: filename, sha256: digest(await readFile(filename)), document: await parse(filename) })));
const observations = [];
const benchmark = await parse('docs/benchmark-presentation.json');
const expectedExternal = `https://github.com/morehardy/anki-forge/tree/${benchmark.evidenceCommit}/${benchmark.dataset}`;
for (const entry of approved) {
  const page = pages.find(page => page.id === entry.pageId);
  const current = await reviewInputs(page, entry.locale, root);
  assert.deepEqual(current, entry.inputs);
  context.currentInputs[page.id] ??= {};
  context.currentInputs[page.id][entry.locale] = current;
  const lineage = {};
  for (const axis of ['technical', 'linguistic']) {
    const proof = reviews.entries[page.id][entry.locale][axis].evidence;
    const bytes = await readFile(proof.path);
    assert.equal(digest(bytes), proof.sha256);
    const prior = JSON.parse(bytes).entries[page.id][entry.locale];
    for (const key of Object.keys(current)) assert.equal(prior[key], current[key]);
    lineage[axis] = proof;
  }
  const historical = [];
  for (const anchor of entry.anchors) {
    const matches = baselines.flatMap(baseline => baseline.document.pages.filter(item => item.pageId === entry.pageId && item.locale === entry.locale && item.route === entry.route).flatMap(item => item.anchors.filter(item => item.id === anchor && item.heading === entry.fromHeading).map(item => ({ baseline: baseline.path, ...item }))));
    assert.ok(matches.length, `${entry.route}#${anchor}: migration not found in actual historical baseline`);
    if (entry.containerModule) for (const match of matches) assert.equal(match.containerModule, entry.containerModule);
    historical.push(...matches);
  }
  let observation;
  if (entry.sectionId) {
    const source = await readFile(page.sources[entry.locale], 'utf8');
    const currentHeadings = headings(source).map(heading => ({ id: heading.id, title: load(micromark(heading.title)).text().replace(/\s+/g, ' ').trim() }));
    const mapping = sectionMapping(page.id, entry.locale, source, legacyAliases(legacy, page.id, entry.locale), sections);
    context.currentSections[page.id] ??= {};
    context.currentSections[page.id][entry.locale] = { mapping, headings: currentHeadings };
    assert.equal(currentHeadings.find(heading => heading.id === entry.headingId)?.title, entry.toHeading);
    observation = `Actual historical heading and all listed old IDs remain the same ${entry.sectionId} task in current reviewed source; the visible title is ${entry.toHeading}. Native heading ID ${entry.headingId} was independently checked.`;
  } else {
    const catalog = await parse(`website/src/i18n/${entry.locale}.json`);
    const expectedTitles = { install: catalog['home.choose'], performance: catalog['performance.title'], examples: catalog['home.try'], updates: catalog['home.evolve1'] + catalog['home.evolve2'] };
    assert.equal(entry.toHeading, expectedTitles[entry.containerModule]);
    if (entry.containerModule === 'performance') assert.deepEqual(entry.requiredExternalLinks, [expectedExternal]);
    observation = `${entry.containerModule} remains the original homepage task. New title exactly matches the current reviewed catalog consumed by HomePage/PerformanceEvidence. ${entry.containerModule === 'performance' ? 'The protected external URL is the exact evidenceCommit/dataset URL derived from the reviewed benchmark presentation, not an arbitrary HTTPS destination.' : 'Protected local destinations match the existing HomePage links within this module.'}`;
  }
  observations.push({ migration: entry, historical, lineage, observation });
}

const testsLog = execFileSync(process.execPath, ['--test', 'website/scripts/heading-migrations.test.mjs', 'website/scripts/public-surface.test.mjs'], { encoding: 'utf8' });
const testsPath = `${directory}/technical-heading-tests.log`;
await writeFile(testsPath, testsLog);
assert.match(testsLog, /pass 17/);
const checks = [];
function invalid(label, entry) {
  assert.throws(() => validateHeadingMigrations({ schemaVersion: 1, entries: [entry] }, context));
  checks.push({ label, result: 'rejected' });
}
const section = approved.find(entry => entry.pageId === 'build-guarantees' && entry.locale === 'zh-CN');
const performance = approved.find(entry => entry.containerModule === 'performance' && entry.locale === 'en');
invalid('section rejects a local-links field', { ...section, requiredLinks: [] });
invalid('section rejects an external-links field', { ...section, requiredExternalLinks: [expectedExternal] });
invalid('module rejects empty destinations', { ...performance, requiredExternalLinks: [] });
invalid('module rejects duplicate external destinations', { ...performance, requiredExternalLinks: [expectedExternal, expectedExternal] });
invalid('module rejects noncanonical HTTPS spelling', { ...performance, requiredExternalLinks: ['https://GITHUB.com/morehardy/anki-forge'] });
invalid('external-only module still rejects stale reviewed hashes', { ...performance, inputs: { ...performance.inputs, targetHash: '0'.repeat(64) } });
invalid('module local destinations remain scoped to locale', { ...performance, requiredLinks: ['/es/docs/updates/'] });

// Additional real HTML checks use the actual reviewed Chinese migration, where
// the historic localized alias and canonical section anchor are distinct.
const fixture = path.join(root, directory, 'technical-heading-fixture');
await mkdir(path.join(fixture, 'zh-cn/docs/build-guarantees'), { recursive: true });
const filename = path.join(fixture, 'zh-cn/docs/build-guarantees/index.html');
const oldAlias = section.anchors.find(anchor => anchor !== section.sectionId);
const heading = `<h2 id="${section.headingId}">${section.toHeading}</h2>`;
const canonical = `<span id="${section.sectionId}" data-section-alias="${section.sectionId}"></span>`;
const alias = `<span id="${oldAlias}" data-section-alias="${section.sectionId}"></span>`;
const valid = `<html lang="zh-CN"><head><meta name="anki-forge:page-id" content="build-guarantees"><link rel="canonical" href="https://example.test${section.route}"></head><main>${canonical}${alias}${heading}</main></html>`;
const baseline = { schemaVersion: 1, source: { commit: 'independent-counterexample-fixture' }, locales: ['zh-CN'], pages: [{ route: section.route, pageId: section.pageId, locale: section.locale, anchors: [{ id: oldAlias, heading: section.fromHeading, headingLevel: 'h2' }], capabilities: {}, entities: [] }], machineResources: [] };
const config = { site: 'https://example.test', base: '/' };
try {
  await writeFile(filename, valid);
  await checkSurface(fixture, baseline, config, approved);
  checks.push({ label: 'actual two-anchor migration accepts same adjacent native heading', result: 'accepted' });
  for (const [label, mutate] of [
    ['canonical section redirected to a different adjacent heading', html => html.replace(canonical, `${canonical}<h2 id="other">Other task</h2>`) ],
    ['duplicate canonical semantic anchor', html => html.replace('<main>', `<main>${canonical}`) ],
    ['historical alias moved beyond reviewed heading', html => html.replace(alias, '').replace('</h2>', `</h2>${alias}`) ],
    ['reviewed heading moved to a new module while old aliases remain outside', html => html.replace(heading, `<section data-module="other">${heading}</section>`) ],
  ]) {
    await writeFile(filename, mutate(valid));
    await assert.rejects(checkSurface(fixture, baseline, config, approved));
    checks.push({ label, result: 'rejected' });
  }
} finally { await rm(fixture, { recursive: true, force: true }); }
for (const [filename, expected] of Object.entries(files)) assert.equal(digest(await readFile(filename)), expected.sha256, `${filename}: changed during review`);
const priorPath = `${directory}/technical-source-review.json`;
const output = {
  schemaVersion: 1, reviewerType: 'ai', reviewerRun: '/root/standards_review/pr66-heading-migration-2026-11-02', status: 'reviewed',
  scope: 'Independent review of the four frozen heading-migration implementation/test/config files, all 12 real migrations, and source/artifact failure boundaries. This does not authorize a new built artifact or alter the frozen source review.',
  files, sourceReview: { path: priorPath, sha256: digest(await readFile(priorPath)), description: 'Frozen independent 204-entry technical review; real current triples and per-entry report digests were rechecked for every migration.' },
  baselines: baselines.map(({ path, sha256 }) => ({ path, sha256 })),
  observations, regressionTests: { command: `${process.execPath} --test website/scripts/heading-migrations.test.mjs website/scripts/public-surface.test.mjs`, passed: 17, log: { path: testsPath, sha256: digest(await readFile(testsPath)) } },
  independentCounterexamples: checks,
  resolvedReviewConcerns: ['toHeading is tied to normalized actual reviewed document source, not just a declared section ID.', 'The actual migrated H2–H6 must retain the historical module, including absence of a module; keeping only the alias outside a new module no longer passes.'],
  conclusions: ['Document and module migrations are mutually exclusive; only docs/article can use section migrations, and both source-review axes/current hash triples remain mandatory.', 'Historic alias, native heading and canonical section IDs must be unique and target the same actual adjacent task heading.', 'Module migrations retain unique historical containers, exact local destinations, and optional canonical HTTPS external destinations inside that same module. The mandatory next-module JSON/comparison links remain enforced.', 'All 12 configured old titles/anchors occur in the actual independent baselines; current titles match reviewed source/catalog content. Performance URLs are fixed to the reviewed benchmark evidence commit and dataset.'],
  documentedStandardViolations: [], possibleSmells: [], openIssues: []
};
const reportPath = `${directory}/technical-heading-review.json`;
await writeFile(reportPath, JSON.stringify(output, null, 2) + '\n');
console.log(JSON.stringify({ path: reportPath, sha256: digest(await readFile(reportPath)), migrations: observations.length, unitTests: 17, additionalChecks: checks.length, files: Object.fromEntries(Object.entries(files).map(([name, value]) => [name, value.sha256])) }, null, 2));
