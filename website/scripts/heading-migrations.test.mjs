import test from 'node:test';
import assert from 'node:assert/strict';
import { validateHeadingMigrations, assertValidatedHeadingMigrations } from './heading-migrations.mjs';

const inputs = { sourceHash: 'a'.repeat(64), targetHash: 'b'.repeat(64), technicalDependencyHash: 'c'.repeat(64) };
const axis = { status: 'reviewed', reviewerType: 'ai', reviewerRun: 'independent-fixture-review', openIssues: [] };
const context = {
  pages: [{ id: 'home', route: '/' }, { id: 'json-to-anki', route: '/docs/json-to-anki/' }, { id: 'anki-forge-vs-genanki', route: '/docs/anki-forge-vs-genanki/' }],
  reviews: { entries: { home: { en: { ...inputs, technical: axis, linguistic: axis } } } },
  currentInputs: { home: { en: inputs } },
};
const entry = { pageId: 'home', locale: 'en', route: '/', anchors: ['workflow-heading', 'next-heading'], fromHeading: 'From your first deck to the next update', toHeading: 'Choose your next Anki task', reason: 'Independent source review approved clearer task wording while retaining both destinations.', containerModule: 'next', requiredLinks: ['/docs/json-to-anki/', '/docs/anki-forge-vs-genanki/'], inputs };
const document = () => ({ schemaVersion: 1, entries: [structuredClone(entry)] });

test('only validated source-bound entries can authorize a surface migration', () => {
  const candidate = document();
  assert.throws(() => assertValidatedHeadingMigrations(candidate.entries), /validated against independent/);
  const approved = validateHeadingMigrations(candidate, context);
  assert.doesNotThrow(() => assertValidatedHeadingMigrations(approved));
  assert.ok(Object.isFrozen(approved[0].inputs));
  candidate.entries[0].toHeading = 'Unreviewed replacement';
  assert.equal(approved[0].toHeading, entry.toHeading);
});
test('missing reviews, failed review axes and stale migration or record hashes fail', () => {
  for (const mutate of [
    value => { delete value.reviews.entries.home.en; },
    value => { value.reviews.entries.home.en.technical.status = 'pending'; },
    value => { value.reviews.entries.home.en.linguistic.openIssues = [{ severity: 'major' }]; },
    value => { value.reviews.entries.home.en.targetHash = 'd'.repeat(64); },
    value => { value.currentInputs.home.en.sourceHash = 'd'.repeat(64); },
  ]) {
    const changed = structuredClone(context);
    mutate(changed);
    assert.throws(() => validateHeadingMigrations(document(), changed), /review|stale/);
  }
  const stale = document();
  stale.entries[0].inputs.technicalDependencyHash = 'd'.repeat(64);
  assert.throws(() => validateHeadingMigrations(stale, context), /stale technicalDependencyHash/);
});
test('migration scope cannot cross locales, overlap anchors or omit protected old destinations', () => {
  for (const [mutate, expected] of [
    [value => { value.entries[0].route = '/zh-cn/'; }, /outside its page and locale/],
    [value => { value.entries[0].requiredLinks[0] = '/zh-cn/docs/json-to-anki/'; }, /outside registered page and locale/],
    [value => { value.entries[0].requiredLinks.pop(); }, /original local JSON and comparison links/],
    [value => { value.entries.push(structuredClone(value.entries[0])); }, /overlapping/],
  ]) {
    const changed = document();
    mutate(changed);
    assert.throws(() => validateHeadingMigrations(changed, context), expected);
  }
});
test('reviewed module migrations are not restricted to homepage identities', () => {
  const other = structuredClone(context);
  other.pages.push({ id: 'workflow', route: '/docs/workflow/' });
  other.reviews.entries.workflow = other.reviews.entries.home;
  other.currentInputs.workflow = other.currentInputs.home;
  const changed = document();
  Object.assign(changed.entries[0], { pageId: 'workflow', route: '/docs/workflow/', anchors: ['original-task'], containerModule: 'workflow' });
  assert.doesNotThrow(() => validateHeadingMigrations(changed, other));
});
