import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, mkdtemp, rm, symlink } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import os from 'node:os';
import { pages, localeIds } from '../../../../website/site-model.mjs';
import { reviewInputs, reviewInputFiles, digest, rubricVersion, glossaryVersion } from '../../../../website/scripts/content-review-inputs.mjs';

const root = process.cwd();
const batch = 'docs/plans/evidence/website-pr66-portable-search-2026-11-02';
const baseline = 'e9fbc7b4a31dbad934455b0ec1b58695be4dc54e';
const reviewerRun = '/root/standards_review/pr66-portable-search-source-delta-2026-11-02';
const git = args => execFileSync('git', args, { encoding: 'utf8', maxBuffer: 64 << 20 });
const json = async file => JSON.parse(await readFile(file));
const proof = async (file, description) => ({ path: file, sha256: digest(await readFile(file)), description });
assert.equal(git(['rev-parse', 'HEAD']).trim(), baseline);
const candidate = await json(`${batch}/source-candidate.json`);
const priorIndex = JSON.parse(git(['show', `${baseline}:website/content-reviews.json`]));
const snapshots = {}, priorReports = {}, entries = {}, unchangedEntries = [];
const closure = new Set();
for (const page of pages) for (const locale of localeIds) {
  const files = await reviewInputFiles(page, locale, root);
  assert.deepEqual(files, candidate.entries[page.id][locale].inputFiles);
  files.forEach(file => closure.add(file));
}
const temp = await mkdtemp(path.join(os.tmpdir(), 'pr66-technical-source-portable-'));
try {
  const baselineRoot = path.join(temp, 'baseline');
  for (const filename of [...closure].sort()) {
    const exists = git(['ls-tree', '--name-only', baseline, '--', filename]).trim();
    const previousUtf8 = exists ? git(['show', `${baseline}:${filename}`]) : null;
    const currentUtf8 = await readFile(filename, 'utf8');
    snapshots[filename] = { previousSha256: previousUtf8 === null ? null : digest(previousUtf8), currentSha256: digest(currentUtf8), changed: previousUtf8 !== currentUtf8, previousUtf8, currentUtf8 };
    if (previousUtf8 !== null) {
      await mkdir(path.dirname(path.join(baselineRoot, filename)), { recursive: true });
      await writeFile(path.join(baselineRoot, filename), previousUtf8);
    }
  }
  const changedClosureInputs = Object.entries(snapshots).filter(([, value]) => value.changed).map(([file]) => file);
  assert.deepEqual(changedClosureInputs, ['website/astro.config.mjs', 'website/scripts/search-documents.mjs', 'website/vendor/pagefind/1.5.2/runtime.json']);
  // The closure definition itself gained two inputs. Reconstruct OLD triples
  // with the exact baseline algorithm, not the expanded current algorithm.
  const algorithmPath = 'website/scripts/content-review-inputs.mjs';
  const oldAlgorithm = git(['show', `${baseline}:${algorithmPath}`]);
  const currentAlgorithm = await readFile(algorithmPath, 'utf8');
  assert.equal(currentAlgorithm.replace(", 'website/astro.config.mjs', 'website/vendor/pagefind/1.5.2/runtime.json'", ''), oldAlgorithm);
  const sharedHelper = 'website/scripts/content-sections.mjs';
  assert.equal(await readFile(sharedHelper, 'utf8'), git(['show', `${baseline}:${sharedHelper}`]));
  await mkdir(path.join(baselineRoot, 'website/scripts'), { recursive: true });
  const resolvedAlgorithm = oldAlgorithm.replace("from './content-sections.mjs'", `from '${pathToFileURL(path.join(root, sharedHelper)).href}'`);
  await writeFile(path.join(baselineRoot, algorithmPath), resolvedAlgorithm);
  await symlink(path.join(root, 'website/node_modules'), path.join(baselineRoot, 'website/node_modules'));
  const previousFunctions = await import(pathToFileURL(path.join(baselineRoot, algorithmPath)));
  const previousClosure = new Set();
  let total = 0, affected = 0;
  for (const page of pages) for (const locale of localeIds) {
    total++;
    const beforeFiles = await previousFunctions.reviewInputFiles(page, locale, baselineRoot);
    beforeFiles.forEach(file => previousClosure.add(file));
    const before = await previousFunctions.reviewInputs(page, locale, baselineRoot);
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
      assert.equal(before[key], previous[key], `${page.id}/${locale} prior index ${key}`);
      assert.equal(before[key], inherited[key], `${page.id}/${locale} inherited report ${key}`);
      assert.equal(after[key], pending[key], `${page.id}/${locale} candidate ${key}`);
    }
    assert.equal(before.sourceHash, after.sourceHash);
    assert.equal(before.technicalDependencyHash, after.technicalDependencyHash);
    const changedInputs = pending.inputFiles.filter(file => snapshots[file].changed);
    const addedInputs = pending.inputFiles.filter(file => !beforeFiles.includes(file));
    if (before.targetHash === after.targetHash) {
      assert.notEqual(page.type, 'docs');
      assert.deepEqual(changedInputs, []);
      assert.deepEqual(addedInputs, []);
      unchangedEntries.push({ pageId: page.id, locale, inputs: after, previousReview: reference });
      continue;
    }
    assert.equal(page.type, 'docs');
    assert.deepEqual(changedInputs, changedClosureInputs);
    assert.deepEqual(addedInputs, ['website/astro.config.mjs', 'website/vendor/pagefind/1.5.2/runtime.json']);
    affected++;
    entries[page.id] ??= {};
    entries[page.id][locale] = { ...after, inputFiles: pending.inputFiles, previousInputs: before, previousReview: reference, inheritedOpenIssues: inherited.technical.openIssues, changedInputs, addedInputs };
  }
  assert.equal(total, 204); assert.equal(affected, 174); assert.equal(unchangedEntries.length, 30);
  assert.equal(previousClosure.size, 259); assert.equal(closure.size, 261);

  const migrationsPath = 'website/public-surface-migrations.json';
  const beforeMigrations = JSON.parse(git(['show', `${baseline}:${migrationsPath}`]));
  const migrations = await json(migrationsPath);
  assert.equal(migrations.entries.length, 12);
  const updatedMigrations = [];
  migrations.entries.forEach((migration, index) => {
    const previous = beforeMigrations.entries[index];
    const expected = candidate.entries[migration.pageId][migration.locale];
    assert.deepEqual(migration.inputs, Object.fromEntries(['sourceHash', 'targetHash', 'technicalDependencyHash'].map(key => [key, expected[key]])));
    const restored = structuredClone(migration); restored.inputs.targetHash = previous.inputs.targetHash;
    assert.deepEqual(restored, previous);
    if (migration.inputs.targetHash !== previous.inputs.targetHash) updatedMigrations.push(`${migration.pageId}/${migration.locale}`);
  });
  assert.deepEqual(updatedMigrations.sort(), ['build-guarantees/en', 'build-guarantees/zh-CN', 'python-api/en', 'python-api/zh-CN']);
  const configPath = 'website/post-build-reviews.config.json';
  assert.equal(await readFile(configPath, 'utf8'), git(['show', `${baseline}:${configPath}`]).replaceAll('website-pr66-rebuild-2026-11-02', 'website-pr66-portable-search-2026-11-02'));
  const packagePath = 'website/package.json', lockPath = 'website/package-lock.json';
  const pkg = await json(packagePath), lock = await json(lockPath);
  const oldPackage = JSON.parse(git(['show', `${baseline}:${packagePath}`]));
  const oldLock = JSON.parse(git(['show', `${baseline}:${lockPath}`]));
  assert.equal(pkg.devDependencies.pagefind, '1.5.2'); assert.equal(lock.packages[''].devDependencies.pagefind, '1.5.2');
  delete pkg.devDependencies.pagefind; delete lock.packages[''].devDependencies.pagefind;
  assert.deepEqual(pkg, oldPackage); assert.deepEqual(lock, oldLock);
  const boundFiles = [algorithmPath, 'website/astro.config.mjs', 'website/scripts/search-documents.mjs', 'website/scripts/search-ranking.test.mjs', packagePath, lockPath, migrationsPath, configPath, ...Object.keys((await json('website/vendor/pagefind/1.5.2/runtime.json')).files).map(file => `website/vendor/pagefind/1.5.2/${file}`), 'website/vendor/pagefind/1.5.2/runtime.json', 'website/vendor/pagefind/1.5.2/LICENSE', 'website/vendor/pagefind/1.5.2/LICENSE-vscode-ripgrep'];
  const implementationBindings = await Promise.all(boundFiles.map(async file => ({ path: file, sha256: digest(await readFile(file)) })));
  const snapshotPath = `${batch}/technical-input-snapshots.json`;
  await writeFile(snapshotPath, JSON.stringify({ schemaVersion: 1, reviewerRun, baseline, oldAlgorithmSha256: digest(oldAlgorithm), currentAlgorithmSha256: digest(currentAlgorithm), files: snapshots }, null, 2) + '\n');
  const auditPath = `${batch}/technical-source-input-audit.json`;
  await writeFile(auditPath, JSON.stringify({ schemaVersion: 1, reviewerRun, baseline, sourceCandidate: await proof(`${batch}/source-candidate.json`, 'Current candidate independently recalculated for every entry.'), snapshots: await proof(snapshotPath, 'Full previous/current UTF-8 closure bytes; runtime.json was absent at baseline; previous triples recomputed with the actual previous algorithm.'), verifiedCurrentAndPreviousTriples: total, previousInputFileCount: previousClosure.size, inputFileCount: closure.size, affectedEntries: affected, changedClosureInputs, unchangedSourceHashAndTechnicalDependencyHash: true, implementationBindings, previousReports: Object.entries(priorReports).map(([file, value]) => ({ path: file, sha256: value.sha256 })), entries, unchangedEntries }, null, 2) + '\n');
  const observationPath = `${batch}/technical-review-observations.json`;
  const observed = {
    schemaVersion: 1, reviewerRun, baseline, status: 'reviewed', implementationBindings,
    standards: ['website/README.md independent source/release contract: current input bindings, real review and reproducible artifacts remain mandatory.', 'Multilingual implementation specification sections 174–285 and remaining-locales plan sections 100–177: preserve reviewed source meaning and require fresh artifact evidence for changed build/runtime inputs.'],
    documentedViolations: [], possibleSmells: [],
    observations: [
      'Read all changed implementation hunks and actual callers. generatePagefind enumerates POSIX relative HTML filenames, applies code-unit sort and awaits each addHTMLFile before continuing. It retains upstream default index options, passes base-free URLs, removes stale output before writeFiles, canonicalizes only the languages object key order, checks generated version against the runtime manifest, verifies each required runtime length and SHA256, then copies unchanged official bytes. finally closes Pagefind on success/failure.',
      'Read installed Starlight integration and Page.astro, DocsHeader, DocsSearch and docs-search.ts. Global pagefind:false disables only the old automatic producer. Page-level data-pagefind-body remains based on non-404 and frontmatter pagefind; the custom header always renders DocsSearch and its independent ranking options/basePath/baseUrl remain intact. npm build invokes generate:search after Astro output and before machine content/manifests.',
      'Read upstream v1.5.2 service API, Fossicker and SearchState: addHTMLFile uses the same DOM parser as addDirectory and does not set force_inclusion (addCustomRecord does). The actual independent 205-file comparison produced equal complete 204 fragment values with six languages at 34 pages and 404 absent.',
      'Independently ran the actual new producer over reverse-created copies of all 205 reviewed HTML files. Complete decoded fragment values still equal the old reviewed artifact; stale shard removed; all 243 resulting file hashes match the coordinator actual macOS/native Linux comparison. Independently ran 20 search tests plus five isolated failure cases: version, missing inventory, wrong length, same-length byte tamper and missing file all reject.',
      'Verified official Linux-x64 tarball SHA512 against package-lock and inventory, exact extracted 58,802,000-byte binary SHA256, and all five vendored WASM files against official binary output byte-for-byte. Both bundled MIT license texts exactly match installed Pagefind license files. No wasm rewriting is performed. package.json/lock add direct pagefind1.5.2 only; all resolved dependency entries are unchanged.',
      'Expanded docs review closure binds astro.config and runtime.json, whose per-file hashes and lengths are enforced before runtime copying. Reconstructed all 204 old triples using the exact old closure algorithm and all current triples using current inputs. Of 261 current closure files, only astro.config, search-documents and the newly introduced runtime manifest differ. Exactly 174 docs targetHash values change; sourceHash and technicalDependencyHash are unchanged for every entry. No translated prose, API sample, section anchor or catalog bytes changed.',
      'Verified all 12 public surface migrations; only four current docs targetHash bindings advance and equal the candidate. All title/anchor/section/module/link semantics remain exact. Post-build configuration changes only report paths to the new batch. Previous technical open issues are carried forward without reclassification.'
    ],
    producerChecks: await proof(`${batch}/technical-producer-observations.json`, 'Independent actual production invocation, complete fragment values, complete output hashes and isolated fail-closed runtime checks.'),
    vendorChecks: await proof(`${batch}/technical-vendor-provenance.json`, 'Independent tarball integrity, binary identity, exact WASM byte and license checks.'),
    apiEquivalence: await proof(`${batch}/api-equivalence.json`, 'Earlier independent addDirectory versus sorted addHTMLFile real HTML comparison, copied unchanged into this batch.'),
    targetedTests: await proof(`${batch}/technical-search-tests.log`, 'Independent Node24 run: 20 tests passed, zero failed.'),
    crossPlatform: await proof(`${batch}/portable-cross-platform-results.json`, 'Coordinator actual macOS and native Linux tmpfs comparison; independently reproduced all listed hashes on this host.'),
    sourceInputAudit: await proof(auditPath, 'Full old/current inputs and recomputed triples with explicit inherited report references.'),
    migrationUpdates: updatedMigrations, openIssues: [],
    limitations: ['This is a source delta review; unchanged content semantics inherit previously sealed independent evidence and are not claimed as a new complete translation read.', 'New root/prefix release artifacts, real browser search and machine-read reports still require separate current-build acceptance. Cross-platform output verification concerns the recorded hosts/locked Pagefind1.5.2, not arbitrary future platform/version changes.', 'Unknown-language fallback can differ from older unordered indexes; equivalence is claimed for the six published exact locale keys.']
  };
  await writeFile(observationPath, JSON.stringify(observed, null, 2) + '\n');
  const reportEntries = {};
  for (const [id, locales] of Object.entries(entries)) {
    reportEntries[id] = {};
    for (const [locale, entry] of Object.entries(locales)) {
      const current = await reviewInputs(pages.find(page => page.id === id), locale, root);
      for (const key of Object.keys(current)) assert.equal(current[key], entry[key]);
      reportEntries[id][locale] = { ...current, technical: { status: 'reviewed', evidence: [
        `Delta scope ${id}/${locale}: deterministic shared Pagefind producer, removal of duplicate Starlight production, and added reviewed runtime inventory. Authored source, catalog/metadata, API examples and semantic anchors match baseline ${baseline}; previous complete semantics review is explicitly inherited, not claimed as a full prose reread.`,
        `Actual addHTMLFile/addDirectory comparison preserves all 204 fragment values, 404 exclusion and 34 pages per published language; the actual new producer independently matches all 243 coordinator macOS/Linux file hashes. Current ${locale} uses its exact Pagefind language key, with unchanged custom UI/ranking/base URL logic.`,
        'The five runtime files are exact official Linux-x64 Pagefind1.5.2 bytes; source tarball integrity, extracted binary, length/SHA inventory and license texts independently verified. Five isolated bad-version/inventory/bytes scenarios reject. No runtime instructions or strict artifact checks are rewritten.',
        `All current hashes were re-read from disk; sourceHash and technicalDependencyHash match the independently recomputed baseline. targetHash includes astro.config, producer and runtime inventory; binary hashes are bound by inventory and enforced by the producer. Previous technical evidence: ${entry.previousReview.path} sha256=${entry.previousReview.sha256}, entry=${entry.previousReview.entry}.`,
        await proof(auditPath, 'Full actual byte snapshots, independently reconstructed old/current triples, and entry-specific inherited evidence.'),
        await proof(observationPath, 'Concrete implementation review, full-site producer and failure tests, source package provenance and explicit acceptance boundaries.')
      ], openIssues: entry.inheritedOpenIssues } };
    }
  }
  for (const binding of implementationBindings) assert.equal(digest(await readFile(binding.path)), binding.sha256, `Source changed during seal: ${binding.path}`);
  const reportPath = `${batch}/technical-source-review.json`;
  await writeFile(reportPath, JSON.stringify({ schemaVersion: 1, rubricVersion, glossaryVersion, reviewerType: 'ai', reviewerRun, deltaScope: '174 affected docs entries only; 30 unchanged entries keep prior independent binding. No claim of rereading unchanged translations.', entries: reportEntries }, null, 2) + '\n');
  console.log(JSON.stringify({ reportPath, sha256: digest(await readFile(reportPath)), totalHashTriples: total, affected, unchanged: unchangedEntries.length, oldClosure: previousClosure.size, currentClosure: closure.size, changedClosureInputs, updatedMigrations }, null, 2));
} finally { await rm(temp, { recursive: true, force: true }); }
