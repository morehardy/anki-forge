import assert from 'node:assert/strict';
import { readFile, realpath } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { sha256 } from './site-manifest.mjs';

const repository = fileURLToPath(new URL('../../', import.meta.url));
export const requiredTaskIds = Object.freeze(['rust-basic', 'python-basic', 'node-basic', 'json-media', 'original-baseline-update', 'troubleshooting']);
export const requiredMachineReadIds = Object.freeze(['python-install', 'node-api', 'json-update', 'compatibility-constraints']);
const digestPattern = /^[0-9a-f]{64}$/;
const nonempty = value => typeof value === 'string' && value.trim().length > 0;
const inside = (parent, child) => child === parent || child.startsWith(parent + path.sep);
const isEvidencePath = filename => filename.startsWith('docs/plans/evidence/') || filename.startsWith('target/');

function repositoryPath(filename) {
  assert.ok(nonempty(filename) && !filename.includes('\\') && !path.posix.isAbsolute(filename) && path.posix.normalize(filename) === filename && !filename.startsWith('../'), `Use an explicit repository-relative proof path: ${filename}`);
  return filename;
}

async function evidenceBytes(filename, root) {
  repositoryPath(filename);
  assert.ok(isEvidencePath(filename), `Post-build proof must be in docs/plans/evidence/ or target/: ${filename}`);
  const physicalRoot = await realpath(root);
  const physical = await realpath(path.join(root, filename));
  const allowed = await Promise.all(['docs/plans/evidence', 'target'].map(directory => realpath(path.join(root, directory)).catch(error => { if (error.code === 'ENOENT') return null; throw error; })));
  assert.ok(allowed.some(directory => directory && inside(directory, physical)), `${filename}: proof path escapes its excluded evidence directory`);
  assert.ok(!inside(path.join(physicalRoot, 'website'), physical), `${filename}: proof cannot resolve into website inputs`);
  if (inside(path.join(physicalRoot, 'docs'), physical)) assert.ok(inside(path.join(physicalRoot, 'docs/plans/evidence'), physical), `${filename}: proof cannot resolve into documentation build inputs`);
  return readFile(physical);
}

async function assertFileEvidence(evidence, root, label) {
  assert.ok(nonempty(evidence.description), `${label}: file evidence needs a concrete description`);
  assert.match(evidence.sha256 || '', digestPattern, `${label}: file evidence needs a SHA-256`);
  const bytes = await evidenceBytes(evidence.path, root);
  assert.equal(sha256(bytes), evidence.sha256, `${label}: evidence bytes changed: ${evidence.path}`);
}

async function assertSourceBindings(bindings, root, label) {
  assert.ok(Array.isArray(bindings) && bindings.length > 0, `${label}: current source/dependency bindings are required for execution reuse`);
  const seen = new Set();
  for (const binding of bindings) {
    const filename = repositoryPath(binding.path);
    assert.ok(!isEvidencePath(filename) && !filename.startsWith('docs/plans/') && !filename.startsWith('docs/superpowers/') && !filename.startsWith('website/dist/') && !filename.includes('/node_modules/'), `${label}: bind source inputs, not generated files or review evidence`);
    assert.ok(!seen.has(filename), `${label}: duplicate source binding ${filename}`);
    seen.add(filename);
    assert.match(binding.sha256 || '', digestPattern, `${label}: source binding needs a SHA-256`);
    const physical = await realpath(path.join(root, filename));
    assert.ok(inside(await realpath(root), physical), `${label}: source binding leaves the repository`);
    assert.equal(sha256(await readFile(physical)), binding.sha256, `${label}: execution source/dependency changed: ${filename}`);
  }
}

function assertNoSevereIssues(issues, label) {
  assert.ok(Array.isArray(issues), `${label}: openIssues must be explicit`);
  for (const issue of issues) {
    assert.ok(['critical', 'major', 'minor', 'preference'].includes(issue?.severity), `${label}: issue severity must be explicit`);
    assert.ok(!['critical', 'major'].includes(issue.severity), `${label}: unresolved ${issue.severity} issue`);
  }
}

function manifestRecords(manifest) {
  assert.ok(Array.isArray(manifest.pages) && Array.isArray(manifest.resources), 'Post-build proof requires a complete manifest');
  const records = new Map();
  for (const entry of [...manifest.pages, ...manifest.resources]) {
    assert.ok(typeof entry.route === 'string' && entry.route.startsWith('/') && !/[?#]/.test(entry.route), 'Manifest routes must be site-relative');
    assert.ok(!records.has(entry.route), `Duplicate manifest route: ${entry.route}`);
    records.set(entry.route, entry);
  }
  return records;
}

function proofIdentity(manifest) {
  return { inputSha256: manifest.inputSha256, configuration: manifest.configuration, releaseLocales: manifest.releaseLocales };
}

function assertReadOrigin(value, configuration) {
  const url = new URL(value || configuration.site);
  assert.ok(url.pathname === '/' && !url.search && !url.hash && !url.username && !url.password, 'readOrigin must be the actual origin without a path or credentials');
  assert.ok(url.origin === configuration.site || (['127.0.0.1', 'localhost', '[::1]'].includes(url.hostname) && url.protocol === 'http:'), 'readOrigin must be the configured site or actual loopback preview origin');
  return url.origin;
}

async function validateReport(filename, expectedKind, manifest, manifestSha256, root) {
  const bytes = await evidenceBytes(filename, root);
  const report = JSON.parse(bytes);
  assert.equal(report.schemaVersion, 1, `${filename}: unknown post-build review schema`);
  assert.equal(report.kind, expectedKind, `${filename}: wrong independent review kind`);
  assert.equal(report.status, 'passed', `${filename}: post-build review did not pass`);
  assert.equal(report.reviewerType, 'ai', `${filename}: an independent AI review is required`);
  assert.ok(nonempty(report.reviewerRun), `${filename}: independent reviewer run is required`);
  assertNoSevereIssues(report.openIssues || [], filename);
  assert.equal(report.mode, 'release-candidate', `${filename}: preview or unclassified review cannot authorize release`);
  assert.deepEqual(proofIdentity(report), proofIdentity(manifest), `${filename}: review used different inputs, deployment configuration or release locales`);
  assert.match(report.inputSha256 || '', digestPattern, `${filename}: inputSha256 is required`);
  const origin = report.originManifest;
  assert.ok(origin && nonempty(origin.path), `${filename}: archive the actual manifest that was reviewed`);
  assert.match(origin.sha256 || '', digestPattern, `${filename}: original manifest SHA-256 is required`);
  assert.match(origin.commit || '', /^[0-9a-f]{40}$/, `${filename}: original manifest commit is required`);
  const originBytes = await evidenceBytes(origin.path, root);
  assert.equal(sha256(originBytes), origin.sha256, `${filename}: original reviewed manifest bytes changed`);
  const original = JSON.parse(originBytes);
  assert.equal(original.schemaVersion, 1, `${filename}: unknown original manifest schema`);
  assert.equal(original.commit, origin.commit, `${filename}: original manifest commit does not match its bytes`);
  assert.deepEqual(proofIdentity(original), proofIdentity(manifest), `${filename}: original artifact source inputs or configuration differ`);
  const currentRecords = manifestRecords(manifest);
  const originalRecords = manifestRecords(original);
  assert.ok(Array.isArray(report.resourceBindings) && report.resourceBindings.length > 0, `${filename}: actual artifact resource bindings are required`);
  const bindings = new Map();
  for (const binding of report.resourceBindings) {
    assert.ok(!bindings.has(binding.route), `${filename}: duplicate reviewed resource ${binding.route}`);
    assert.match(binding.sha256 || '', digestPattern, `${filename}: reviewed resource SHA-256 is required`);
    assert.equal(originalRecords.get(binding.route)?.sha256, binding.sha256, `${filename}: binding was not part of the actual reviewed artifact: ${binding.route}`);
    assert.equal(currentRecords.get(binding.route)?.sha256, binding.sha256, `${filename}: reviewed resource changed: ${binding.route}`);
    bindings.set(binding.route, binding.sha256);
  }
  const machineResources = manifest.resources.filter(resource => /\/(?:index\.md|llms\.txt)$/.test(resource.route));
  assert.ok(machineResources.length > 0, `${filename}: machine-readable review scope is missing`);
  for (const resource of machineResources) assert.equal(bindings.get(resource.route), resource.sha256, `${filename}: incomplete Markdown/llms review scope: ${resource.route}`);
  assert.deepEqual(Object.keys(report.entries || {}).sort(), [...manifest.releaseLocales].sort(), `${filename}: review must cover every current release locale, without fallback`);
  const readOrigin = assertReadOrigin(report.readOrigin, manifest.configuration);
  const urlRoutes = new Map([...currentRecords.keys()].map(route => [new URL(route, manifest.configuration.site).pathname, route]));
  const owners = new Map(manifest.pages.filter(page => page.id !== '404').map(page => [page.route, page.locale]));
  for (const page of manifest.pages.filter(page => page.id !== '404')) {
    if (currentRecords.has(`${page.route}index.md`)) owners.set(`${page.route}index.md`, page.locale);
    if (page.id === 'home') owners.set(`${page.route}llms.txt`, page.locale);
  }
  for (const locale of manifest.releaseLocales) {
    const entry = report.entries[locale];
    const key = expectedKind === 'usage-tasks' ? 'taskResults' : 'aiMachineReadReview';
    const expectedIds = expectedKind === 'usage-tasks' ? requiredTaskIds : requiredMachineReadIds;
    const results = entry?.[key];
    assert.ok(Array.isArray(results), `${filename}/${locale}: ${key} is required`);
    assert.deepEqual(results.map(result => result.id).sort(), [...expectedIds].sort(), `${filename}/${locale}: incomplete or duplicate ${key}`);
    assertNoSevereIssues(entry.openIssues || [], `${filename}/${locale}`);
    for (const result of results) {
      const label = `${filename}/${locale}/${result.id}`;
      assert.equal(result.status, 'passed', `${label}: independent review did not pass`);
      assertNoSevereIssues(result.openIssues, label);
      assert.ok(Array.isArray(result.readUrls) && result.readUrls.length > 0, `${label}: actual read URLs are required`);
      const readRoutes = result.readUrls.map(value => {
        const url = new URL(value);
        assert.ok(!url.username && !url.password && [readOrigin, manifest.configuration.site].includes(url.origin), `${label}: read URL is outside the reviewed artifact origin`);
        const route = urlRoutes.get(url.pathname);
        assert.ok(route && bindings.has(route), `${label}: actual read URL has no artifact binding: ${value}`);
        assert.ok(!owners.has(route) || owners.get(route) === locale, `${label}: another language was read as fallback`);
        return route;
      });
      if (expectedKind === 'machine-read') {
        const home = manifest.pages.find(page => page.id === 'home' && page.locale === locale);
        assert.ok(home && readRoutes.includes(`${home.route}llms.txt`), `${label}: task must start from this locale's llms entry`);
        assert.ok(readRoutes.some(route => route.endsWith('/index.md') && owners.get(route) === locale), `${label}: actual localized Markdown reading is required`);
        assert.ok(nonempty(result.answer), `${label}: retain the actual answer and cited conclusions`);
      } else {
        assert.ok(readRoutes.some(route => manifest.pages.some(page => page.route === route && page.locale === locale && page.id !== '404')), `${label}: actual localized page reading is required`);
        assert.ok(Array.isArray(result.steps) && result.steps.length > 0 && result.steps.every(nonempty), `${label}: actual actions and observations are required`);
        if (result.id !== 'troubleshooting') {
          assert.ok(Array.isArray(result.executionEvidence) && result.executionEvidence.length > 0, `${label}: real execution evidence or an explicitly bound reuse is required`);
        }
        for (const execution of result.executionEvidence || []) {
          await assertFileEvidence(execution, root, label);
          await assertSourceBindings(execution.sourceBindings, root, `${label}/execution source`);
          await assertSourceBindings(execution.dependencyBindings, root, `${label}/execution dependencies`);
        }
      }
      assert.ok(Array.isArray(result.evidence) && result.evidence.length > 0, `${label}: concrete review evidence is required`);
      for (const evidence of result.evidence) {
        if (evidence.path) await assertFileEvidence(evidence, root, label);
        else {
          assert.ok(nonempty(evidence.observation), `${label}: resource evidence needs an actual observation`);
          assert.equal(bindings.get(evidence.route), evidence.sha256, `${label}: observation is not bound to the reviewed resource`);
          assert.ok(bindings.has(evidence.route), `${label}: observed resource is missing`);
        }
      }
    }
  }
  return { kind: expectedKind, path: filename, sha256: sha256(bytes), bytes: bytes.length, reviewerRun: report.reviewerRun, originManifestSha256: origin.sha256, originCommit: origin.commit, currentManifestSha256: manifestSha256, reused: origin.sha256 !== manifestSha256, resourceScopeSha256: sha256(JSON.stringify([...bindings].sort(([left], [right]) => left.localeCompare(right)))) };
}

/** Join actual independent reports to the current artifact, without rewriting them. */
export async function verifyPostBuildReviews(manifest, manifestSha256, { root = repository, configPath = 'website/post-build-reviews.config.json' } = {}) {
  assert.notEqual(process.env.CONTENT_REVIEW_MODE, 'preview', 'An editorial preview cannot pass post-build reviews');
  assert.match(manifestSha256 || '', digestPattern, 'Current artifact manifest SHA-256 is required');
  assert.ok(Array.isArray(manifest.releaseLocales) && manifest.releaseLocales.length > 0, 'Current artifact has no release locale snapshot');
  const configBytes = await readFile(path.join(root, repositoryPath(configPath)));
  const config = JSON.parse(configBytes);
  assert.equal(config.schemaVersion, 1, 'Unknown post-build review configuration');
  const environment = manifest.configuration.base === '/' ? 'root' : 'prefix';
  const selected = config.environments?.[environment];
  assert.ok(selected?.taskReview && selected?.machineReview, `${environment}: both task and machine review paths must be fixed before building`);
  const reports = await Promise.all([
    validateReport(selected.taskReview, 'usage-tasks', manifest, manifestSha256, root),
    validateReport(selected.machineReview, 'machine-read', manifest, manifestSha256, root),
  ]);
  return { schemaVersion: 1, status: 'passed', environment, inputSha256: manifest.inputSha256, configuration: manifest.configuration, releaseLocales: manifest.releaseLocales, currentManifestSha256: manifestSha256, configSha256: sha256(configBytes), reports };
}
