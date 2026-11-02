import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, mkdtemp, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import os from 'node:os';
import { canonicalizePagefindEntry } from '../../../../website/scripts/search-documents.mjs';
import { pages, localeIds } from '../../../../website/site-model.mjs';
import { reviewInputs, reviewInputFiles, digest, rubricVersion, glossaryVersion } from '../../../../website/scripts/content-review-inputs.mjs';

const root = process.cwd();
const batch = 'docs/plans/evidence/website-pr66-rebuild-2026-11-02';
const previousBatch = 'docs/plans/evidence/website-pr66-final-2026-11-02';
const baseline = '0001985be86f39713adb7f7dfb5d29af0df614b7';
const reviewerRun = '/root/standards_review/pr66-pagefind-rebuild-delta-2026-11-02';
const git = args => execFileSync('git', args, { encoding: 'utf8', maxBuffer: 64 << 20 });
const json = async filename => JSON.parse(await readFile(filename, 'utf8'));
const proof = async (filename, description) => ({ path: filename, sha256: digest(await readFile(filename)), description });
assert.equal(git(['rev-parse', 'HEAD']).trim(), baseline);
const candidatePath = `${batch}/source-candidate.json`;
const candidate = await json(candidatePath);
const priorIndex = JSON.parse(git(['show', `${baseline}:website/content-reviews.json`]));
const snapshots = {}, priorReports = {}, entries = {}, unchangedEntries = [];
const closure = new Set();
for (const page of pages) for (const locale of localeIds) {
  const files = await reviewInputFiles(page, locale, root);
  assert.deepEqual(files, candidate.entries[page.id][locale].inputFiles);
  files.forEach(file => closure.add(file));
}
const temporary = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-technical-pagefind-review-'));
try {
  const baselineRoot = path.join(temporary, 'baseline');
  for (const filename of [...closure].sort()) {
    const previousUtf8 = git(['show', `${baseline}:${filename}`]);
    const currentUtf8 = await readFile(filename, 'utf8');
    snapshots[filename] = { previousSha256: digest(previousUtf8), currentSha256: digest(currentUtf8), changed: previousUtf8 !== currentUtf8, previousUtf8, currentUtf8 };
    await mkdir(path.dirname(path.join(baselineRoot, filename)), { recursive: true });
    await writeFile(path.join(baselineRoot, filename), previousUtf8);
  }
  assert.deepEqual(Object.entries(snapshots).filter(([, value]) => value.changed).map(([name]) => name), ['website/scripts/search-documents.mjs']);
  let total = 0, affected = 0;
  for (const page of pages) for (const locale of localeIds) {
    total++;
    const before = await reviewInputs(page, locale, baselineRoot);
    const after = await reviewInputs(page, locale, root);
    const pending = candidate.entries[page.id][locale];
    const previous = priorIndex.entries[page.id][locale];
    const reference = previous.technical.evidence;
    if (!priorReports[reference.path]) {
      const bytes = await readFile(reference.path);
      assert.equal(digest(bytes), reference.sha256);
      priorReports[reference.path] = { sha256: digest(bytes), report: JSON.parse(bytes) };
    }
    const inherited = priorReports[reference.path].report.entries[page.id][locale];
    assert.equal(inherited.technical.status, 'reviewed');
    for (const key of Object.keys(after)) {
      assert.equal(before[key], previous[key]);
      assert.equal(before[key], inherited[key]);
      assert.equal(after[key], pending[key]);
    }
    assert.equal(before.sourceHash, after.sourceHash);
    assert.equal(before.technicalDependencyHash, after.technicalDependencyHash);
    const changedInputs = pending.inputFiles.filter(filename => snapshots[filename].changed);
    if (before.targetHash === after.targetHash) {
      assert.notEqual(page.type, 'docs');
      assert.deepEqual(changedInputs, []);
      unchangedEntries.push({ pageId: page.id, locale, inputs: after, previousReview: reference });
      continue;
    }
    assert.equal(page.type, 'docs');
    assert.deepEqual(changedInputs, ['website/scripts/search-documents.mjs']);
    affected++;
    entries[page.id] ??= {};
    entries[page.id][locale] = { ...after, inputFiles: pending.inputFiles, previousInputs: before, previousReview: reference, inheritedOpenIssues: inherited.technical.openIssues, changedInputs };
  }
  assert.equal(total, 204);
  assert.equal(affected, 174);
  assert.equal(unchangedEntries.length, 30);

  // Exercise the actual production function on both real producer outputs and
  // all 6! insertion orders. This never touches website/dist or an archive.
  const fixtureRoot = path.join(temporary, 'fixture');
  await mkdir(path.join(fixtureRoot, 'pagefind'), { recursive: true });
  const fixture = path.join(fixtureRoot, 'pagefind/pagefind-entry.json');
  const originalBytes = await readFile(`${batch}/pagefind-original.json`, 'utf8');
  const rebuiltBytes = await readFile(`${batch}/pagefind-commit-rebuild.json`, 'utf8');
  const original = JSON.parse(originalBytes), rebuilt = JSON.parse(rebuiltBytes);
  assert.notEqual(originalBytes, rebuiltBytes);
  assert.deepEqual(original, rebuilt);
  const runtimeFile = 'website/dist/pagefind/pagefind.js';
  const runtime = await readFile(runtimeFile, 'utf8');
  const method = runtime.match(/findIndex\(language\)\{([\s\S]*?)\}async loadMeta\(index\)/)?.[1];
  assert.ok(method);
  const findIndex = new Function('language', method);
  assert.ok(runtime.includes('getAttribute("lang")||"unknown").toLowerCase()'));
  assert.ok(runtime.includes('language:detectLanguage(),primary:true'));
  const selected = Object.fromEntries(localeIds.map(locale => [locale, findIndex.call({ languages: original.languages }, locale.toLowerCase())]));
  let canonicalBytes;
  function* permutations(values) {
    if (values.length === 0) { yield []; return; }
    for (let index = 0; index < values.length; index++) {
      for (const rest of permutations(values.filter((_, i) => i !== index))) yield [values[index], ...rest];
    }
  }
  let permutationCount = 0;
  for (const order of permutations(Object.keys(original.languages))) {
    const input = { ...original, languages: Object.fromEntries(order.map(locale => [locale, original.languages[locale]])) };
    await writeFile(fixture, JSON.stringify(input));
    await canonicalizePagefindEntry(fixtureRoot);
    const bytes = await readFile(fixture, 'utf8');
    canonicalBytes ??= bytes;
    assert.equal(bytes, canonicalBytes);
    const value = JSON.parse(bytes);
    assert.deepEqual(value, original);
    for (const locale of localeIds) assert.deepEqual(findIndex.call({ languages: value.languages }, locale.toLowerCase()), selected[locale]);
    permutationCount++;
  }
  assert.equal(permutationCount, 720);
  await writeFile(fixture, rebuiltBytes);
  await canonicalizePagefindEntry(fixtureRoot);
  assert.equal(await readFile(fixture, 'utf8'), canonicalBytes);
  await canonicalizePagefindEntry(fixtureRoot);
  assert.equal(await readFile(fixture, 'utf8'), canonicalBytes);
  for (const invalid of [{}, { languages: null }, { languages: [] }, { languages: 'en' }]) {
    await writeFile(fixture, JSON.stringify(invalid));
    await assert.rejects(canonicalizePagefindEntry(fixtureRoot), /Pagefind language map required/);
  }
  const canonical = JSON.parse(canonicalBytes);
  const runtimeObservation = {
    runtime: { path: runtimeFile, sha256: digest(runtime), scope: 'Actual existing Pagefind 1.5.2 bundle read only for detectLanguage, createInstance and findIndex code, not treated as the pending rebuilt release artifact.' },
    actualFindIndexSource: method,
    actualLanguageDetection: 'createInstance supplies language: detectLanguage(); detectLanguage lowercases the document html lang. All six release languages have exact Pagefind map keys.',
    permutations: permutationCount, canonicalSha256: digest(canonicalBytes),
    preservedFields: ['version', 'all language hash/wasm/page_count values', 'include_characters array and order'],
    selectedPerReleaseLocale: selected,
    unknownLanguageBoundary: {
      previous: findIndex.call({ languages: original.languages }, 'unsupported-language'),
      canonical: findIndex.call({ languages: canonical.languages }, 'unsupported-language'),
      observation: 'Unknown-language fallback sorts by page_count and uses map insertion order to break equal counts. Sorting intentionally makes that formerly random fallback deterministic; it can select a different language. This review claims selection preservation only for the six published language keys, all exact matches. The site does not serve an unknown-locale search page.'
    },
    idempotence: true, invalidMapCasesRejected: 4,
  };

  const migrationsPath = 'website/public-surface-migrations.json';
  const beforeMigrations = JSON.parse(git(['show', `${baseline}:${migrationsPath}`]));
  const migrations = await json(migrationsPath);
  assert.equal(migrations.entries.length, 12);
  const updatedMigrations = [];
  migrations.entries.forEach((migration, index) => {
    const previous = beforeMigrations.entries[index];
    const expected = candidate.entries[migration.pageId][migration.locale];
    assert.deepEqual(migration.inputs, Object.fromEntries(['sourceHash', 'targetHash', 'technicalDependencyHash'].map(key => [key, expected[key]])));
    const restored = structuredClone(migration);
    restored.inputs.targetHash = previous.inputs.targetHash;
    assert.deepEqual(restored, previous, 'Only current targetHash may change in a previously reviewed migration');
    if (migration.inputs.targetHash !== previous.inputs.targetHash) updatedMigrations.push(`${migration.pageId}/${migration.locale}`);
  });
  assert.deepEqual(updatedMigrations.sort(), ['build-guarantees/en', 'build-guarantees/zh-CN', 'python-api/en', 'python-api/zh-CN']);
  const configPath = 'website/post-build-reviews.config.json';
  const previousConfig = git(['show', `${baseline}:${configPath}`]);
  assert.equal(await readFile(configPath, 'utf8'), previousConfig.replaceAll('website-pr66-final-2026-11-02', 'website-pr66-rebuild-2026-11-02'));
  const boundFiles = ['website/scripts/search-documents.mjs', 'website/scripts/search-ranking.test.mjs', migrationsPath, configPath];
  const implementationBindings = await Promise.all(boundFiles.map(async filename => ({ path: filename, previousSha256: digest(git(['show', `${baseline}:${filename}`])), sha256: digest(await readFile(filename)), currentUtf8: await readFile(filename, 'utf8') })));
  const snapshotPath = `${batch}/technical-input-snapshots.json`;
  await writeFile(snapshotPath, JSON.stringify({ schemaVersion: 1, reviewerRun, baseline, files: snapshots }, null, 2) + '\n');
  const auditPath = `${batch}/technical-source-input-audit.json`;
  const audit = {
    schemaVersion: 1, reviewerRun, baseline,
    sourceCandidate: await proof(candidatePath, 'Single pending candidate generated from current disk inputs.'),
    snapshots: await proof(snapshotPath, 'Actual git baseline and current UTF-8 bytes for every reviewInputFiles closure file; previous triples independently recomputed from a temporary baseline snapshot.'),
    verifiedCurrentAndPreviousTriples: total, inputFileCount: closure.size, affectedEntries: affected,
    onlyChangedClosureInput: 'website/scripts/search-documents.mjs',
    unchangedSourceHashAndTechnicalDependencyHash: true, implementationBindings,
    previousReports: Object.entries(priorReports).map(([filename, value]) => ({ path: filename, sha256: value.sha256 })),
    entries, unchangedEntries,
  };
  await writeFile(auditPath, JSON.stringify(audit, null, 2) + '\n');
  const observationPath = `${batch}/technical-pagefind-observations.json`;
  const observation = {
    schemaVersion: 1, reviewerRun, baseline, status: 'reviewed', implementationBindings,
    standards: ['website/README.md source review/release contract: current source hashes require genuine independent delta review and traceable unchanged-byte lineage.', 'Multilingual implementation specification 174–285: independent source review precedes artifact acceptance; changes must retain language semantics and API constraints.'],
    documentedViolations: [], possibleSmells: [],
    observations: [
      'Read the actual added function and its production call. It parses the Pagefind entry, requires a non-null object language map, sorts only its locale keys using deterministic code-unit ordering, retains each value and unrelated field, and emits stable JSON plus newline.',
      'Read website/package.json build sequencing and Astro Pagefind integration: generate:search runs after astro build produces Pagefind, and canonicalizes before machine generation and site-manifest hashing. The official artifact reader remains strict; no hash comparison is weakened or bypassed.',
      'Read the actual UI caller and generated Pagefind 1.5.2 language detection/selection. Six published html lang values map to exact lowercase keys. Hash/WASM/count fields, ranking configuration and all document/search engine bytes remain unchanged by this source delta.',
      'The targeted production regression passed all 19 tests. Independently tested all 720 permutations of the six-language real producer fixture through the actual production helper, complete parsed value preservation, repeated invocation, the second real commit-rebuild fixture, six exact runtime selections and four invalid language-map inputs.',
      'All 259 source-review closure files were read against git baseline. Only search-documents.mjs differs; all 204 current and baseline hash triples were recomputed. Exactly 174 docs targetHash values change; all sourceHash and technicalDependencyHash values remain equal. No translated prose, catalogs, headings, code samples, API dependencies or semantic anchors changed.',
      'The 12 previously reviewed public-surface migrations are identical after restoring only four docs targetHash fields. New hashes match the recomputed candidate. All old/new titles, anchors, section/module constraints, local/external destinations, reasons and other hashes are unchanged. Post-build config only changes the four report paths to this new batch.'
    ],
    runtimeObservation,
    migrationUpdates: updatedMigrations,
    priorHeadingReview: await proof(`${previousBatch}/technical-heading-review.json`, 'Immutable independent review of the unchanged migration semantics and checker implementation; only the four separately verified target bindings advance.'),
    sourceInputAudit: await proof(auditPath, 'Actual complete input-byte and per-entry hash lineage audit.'),
    targetedTestLog: await proof(`${batch}/technical-search-tests.log`, 'Independent Node 24 search-ranking.test.mjs run: 19 passed, zero failed.'),
    openIssues: [],
    limitations: ['New root/prefix builds and machine reports are still separate post-build acceptance; this source report does not approve artifacts before they exist.', 'Unknown-language fallback with equal page counts can change, as explicitly demonstrated. Only six published exact-language search paths are claimed semantically equivalent.']
  };
  await writeFile(observationPath, JSON.stringify(observation, null, 2) + '\n');
  const reportEntries = {};
  for (const [id, locales] of Object.entries(entries)) {
    reportEntries[id] = {};
    for (const [locale, entry] of Object.entries(locales)) {
      // Re-read disk once more at report sealing, after all tests and audits.
      const current = await reviewInputs(pages.find(page => page.id === id), locale, root);
      for (const key of Object.keys(current)) assert.equal(current[key], entry[key]);
      const previous = priorReports[entry.previousReview.path].report.entries[id][locale].technical;
      reportEntries[id][locale] = {
        ...current,
        technical: {
          status: 'reviewed',
          evidence: [
            `Delta scope: ${id}/${locale} changes only the shared website/scripts/search-documents.mjs producer input. Actual authored source, metadata/catalog, API examples, semantic anchors and technical dependency bytes match reviewed baseline ${baseline}. The prior complete semantics review is inherited explicitly, not claimed as a full prose reread.`,
            `Independently read canonicalizePagefindEntry and generateSearchDocuments call order. The real six-language fixture passed all 720 insertion permutations with equal canonical bytes and unchanged parsed data; actual Pagefind exact-language hash/WASM selection is preserved for ${locale}. Unknown-language tie fallback is explicitly outside that equivalence claim.`,
            `Current sourceHash ${current.sourceHash} and technicalDependencyHash ${current.technicalDependencyHash} equal their independently reconstructed baseline; targetHash ${current.targetHash} was recalculated from disk and matched the candidate. All previous technical open issues are retained.`,
            `Previous independent technical evidence: ${entry.previousReview.path} sha256=${entry.previousReview.sha256}, entry=${entry.previousReview.entry}.`,
            await proof(auditPath, 'Full old/current closure byte snapshots and recomputed per-entry triples, including all unchanged inputs.'),
            await proof(observationPath, 'Actual implementation/runtime reading, permutation and invalid-input results, migration/config checks, and explicit review boundaries.'),
          ],
          openIssues: previous.openIssues,
        },
      };
    }
  }
  const report = { schemaVersion: 1, rubricVersion, glossaryVersion, reviewerType: 'ai', reviewerRun, deltaScope: 'Only the 174 docs entries affected by Pagefind producer canonicalization. Thirty unchanged entries retain their existing independent report binding. No translation rereview is asserted.', entries: reportEntries };
  const reportPath = `${batch}/technical-source-review.json`;
  await writeFile(reportPath, JSON.stringify(report, null, 2)+'\n');
  console.log(JSON.stringify({ reportPath, sha256: digest(await readFile(reportPath)), totalHashTriples: total, affected, unchanged: unchangedEntries.length, inputFiles: closure.size, permutations: permutationCount, updatedMigrations }, null, 2));
} finally {
  await rm(temporary, { recursive: true, force: true });
}
