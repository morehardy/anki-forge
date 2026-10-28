import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { sha256 } from './site-manifest.mjs';
import { requiredTaskIds, requiredMachineReadIds, verifyPostBuildReviews } from './post-build-reviews.mjs';

// Synthetic files below exercise the validator; none are production approvals.
async function fixture(t, base = '/') {
  const root = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-post-review-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const evidence = 'docs/plans/evidence/validator-fixture';
  const taskFile = `${evidence}/usage.json`;
  const machineFile = `${evidence}/machine.json`;
  const originPath = `${evidence}/origin-manifest.json`;
  await mkdir(path.join(root, evidence), { recursive: true });
  await mkdir(path.join(root, 'website'), { recursive: true });
  await mkdir(path.join(root, 'anki_forge/examples'), { recursive: true });
  const source = 'fn main() { /* synthetic test input */ }\n';
  const dependency = '# synthetic lockfile for validator test\n';
  await writeFile(path.join(root, 'anki_forge/examples/fixture.rs'), source);
  await writeFile(path.join(root, 'Cargo.lock'), dependency);
  const executionPath = `${evidence}/execution.txt`;
  const execution = 'Synthetic unit-test execution evidence; not an actual SDK run.\n';
  await writeFile(path.join(root, executionPath), execution);
  const configuration = { site: 'https://example.test', base, customDomain: '' };
  const manifest = { schemaVersion: 1, commit: 'a'.repeat(40), inputSha256: 'b'.repeat(64), configuration, releaseLocales: ['en', 'zh-CN'], pages: [], resources: [] };
  const prefix = base === '/' ? '' : base;
  for (const locale of manifest.releaseLocales) {
    const local = `${prefix}${locale === 'en' ? '' : '/zh-cn'}`;
    for (const [id, route] of [['home', `${local}/`], ['quickstart', `${local}/docs/quickstart/`]]) manifest.pages.push({ id, route, locale, sha256: sha256(`HTML:${route}`) });
    for (const route of [`${local}/llms.txt`, `${local}/docs/quickstart/index.md`]) manifest.resources.push({ route, sha256: sha256(`machine:${route}`) });
  }
  const originalBytes = Buffer.from(JSON.stringify(manifest));
  await writeFile(path.join(root, originPath), originalBytes);
  const executionEvidence = { path: executionPath, sha256: sha256(execution), description: 'Synthetic fixture log for validator assertions', sourceBindings: [{ path: 'anki_forge/examples/fixture.rs', sha256: sha256(source) }], dependencyBindings: [{ path: 'Cargo.lock', sha256: sha256(dependency) }] };
  const reports = {};
  for (const [kind, ids, key] of [['usage-tasks', requiredTaskIds, 'taskResults'], ['machine-read', requiredMachineReadIds, 'aiMachineReadReview']]) {
    const report = { schemaVersion: 1, kind, mode: 'release-candidate', status: 'passed', reviewerType: 'ai', reviewerRun: 'synthetic-validator-fixture', inputSha256: manifest.inputSha256, configuration, releaseLocales: manifest.releaseLocales, readOrigin: 'http://127.0.0.1:4321', originManifest: { path: originPath, sha256: sha256(originalBytes), commit: manifest.commit }, resourceBindings: [...manifest.pages, ...manifest.resources].map(({ route, sha256 }) => ({ route, sha256 })), entries: {} };
    for (const locale of manifest.releaseLocales) {
      const page = manifest.pages.find(page => page.id === 'quickstart' && page.locale === locale);
      const home = manifest.pages.find(page => page.id === 'home' && page.locale === locale);
      const readRoutes = kind === 'machine-read' ? [`${home.route}llms.txt`, `${page.route}index.md`] : [page.route];
      report.entries[locale] = { [key]: ids.map(id => ({ id, status: 'passed', readUrls: readRoutes.map(route => `http://127.0.0.1:4321${route}`), ...(kind === 'machine-read' ? { answer: 'Synthetic answer used only to exercise validation.' } : { steps: ['Synthetic action and observation for validator test.'], ...(id === 'troubleshooting' ? {} : { executionEvidence: [structuredClone(executionEvidence)] }) }), evidence: [{ route: page.route, sha256: page.sha256, observation: 'Synthetic page observation for validator test.' }], openIssues: [] })) };
    }
    reports[kind] = report;
  }
  const config = { schemaVersion: 1, environments: Object.fromEntries(['root', 'prefix'].map(name => [name, { taskReview: taskFile, machineReview: machineFile }])) };
  await writeFile(path.join(root, 'website/post-build-reviews.config.json'), JSON.stringify(config));
  const save = async () => { await writeFile(path.join(root, taskFile), JSON.stringify(reports['usage-tasks'])); await writeFile(path.join(root, machineFile), JSON.stringify(reports['machine-read'])); };
  await save();
  const check = (current = manifest) => verifyPostBuildReviews(current, sha256(JSON.stringify(current)), { root });
  return { root, manifest, reports, config, taskFile, machineFile, executionPath, save, check };
}

for (const base of ['/', '/anki-forge']) test(`post-build proof covers six tasks and four machine readings in both locales at ${base}`, async t => {
  const setup = await fixture(t, base);
  const receipt = await setup.check();
  assert.equal(receipt.status, 'passed');
  assert.equal(receipt.environment, base === '/' ? 'root' : 'prefix');
  assert.equal(receipt.reports.length, 2);
  assert.ok(receipt.reports.every(report => !report.reused));
});

test('pure commit changes reuse actual unchanged review scope and preserve its origin', async t => {
  const { manifest, check } = await fixture(t);
  const changed = { ...manifest, commit: 'c'.repeat(40) };
  const receipt = await check(changed);
  assert.ok(receipt.reports.every(report => report.reused && report.originCommit === manifest.commit && report.currentManifestSha256 === sha256(JSON.stringify(changed))));
});

test('an absent independent report blocks post-build acceptance', async t => {
  const { root, machineFile, check } = await fixture(t);
  await rm(path.join(root, machineFile));
  await assert.rejects(check(), /ENOENT/);
});

test('missing locale, task, machine answer or concrete evidence blocks acceptance', async t => {
  for (const mutate of [
    reports => { delete reports['usage-tasks'].entries['zh-CN']; },
    reports => { reports['usage-tasks'].entries.en.taskResults.pop(); },
    reports => { reports['machine-read'].entries.en.aiMachineReadReview.pop(); },
    reports => { reports['machine-read'].entries.en.aiMachineReadReview[0].answer = ''; },
    reports => { reports['machine-read'].entries.en.aiMachineReadReview[0].evidence = []; },
  ]) {
    const setup = await fixture(t);
    mutate(setup.reports); await setup.save();
    await assert.rejects(setup.check(), /every current release locale|incomplete or duplicate|actual answer|concrete review evidence/);
  }
});

test('unchanged input hash does not authorize changed or incompletely bound Markdown', async t => {
  const setup = await fixture(t);
  const changed = structuredClone(setup.manifest);
  changed.resources[1].sha256 = 'e'.repeat(64);
  await assert.rejects(setup.check(changed), /reviewed resource changed/);
  setup.reports['machine-read'].resourceBindings = setup.reports['machine-read'].resourceBindings.filter(binding => binding.route !== setup.manifest.resources[1].route);
  await setup.save();
  await assert.rejects(setup.check(), /incomplete Markdown\/llms review scope/);
});

test('source inputs or deployment configuration cannot change under an old AI report', async t => {
  const setup = await fixture(t);
  await assert.rejects(setup.check({ ...setup.manifest, inputSha256: 'd'.repeat(64) }), /different inputs/);
  await assert.rejects(setup.check({ ...setup.manifest, configuration: { ...setup.manifest.configuration, site: 'https://different.test' } }), /different inputs/);
});

test('execution reuse requires current source and dependency hashes and actual evidence files', async t => {
  const setup = await fixture(t);
  await writeFile(path.join(setup.root, 'anki_forge/examples/fixture.rs'), 'changed program');
  await assert.rejects(setup.check(), /execution source\/dependency changed/);
  const second = await fixture(t);
  await writeFile(path.join(second.root, 'Cargo.lock'), 'changed dependency');
  await assert.rejects(second.check(), /execution source\/dependency changed/);
  const third = await fixture(t);
  await writeFile(path.join(third.root, third.executionPath), 'changed evidence');
  await assert.rejects(third.check(), /evidence bytes changed/);
});

test('execution claims without a reusable run cannot satisfy the five executable tasks', async t => {
  const setup = await fixture(t);
  delete setup.reports['usage-tasks'].entries.en.taskResults[0].executionEvidence;
  await setup.save();
  await assert.rejects(setup.check(), /real execution evidence/);
});

test('preview, non-AI reviews and severe open issues never authorize a release', async t => {
  for (const mutate of [
    report => { report.mode = 'editorial-preview'; },
    report => { report.reviewerType = 'author-self-check'; },
    report => { report.entries.en.aiMachineReadReview[0].openIssues = [{ severity: 'major', description: 'Wrong version.' }]; },
    report => { report.openIssues = [{ severity: 'critical', description: 'Wrong original baseline.' }]; },
  ]) {
    const setup = await fixture(t);
    mutate(setup.reports['machine-read']); await setup.save();
    await assert.rejects(setup.check(), /preview or unclassified|independent AI|unresolved major|unresolved critical/);
  }
});

test('cross-language fallback and fabricated reading targets fail', async t => {
  const setup = await fixture(t);
  setup.reports['machine-read'].entries['zh-CN'].aiMachineReadReview[0].readUrls[0] = 'http://127.0.0.1:4321/llms.txt';
  await setup.save();
  await assert.rejects(setup.check(), /another language was read/);
  const second = await fixture(t);
  second.reports['machine-read'].entries.en.aiMachineReadReview[0].readUrls[0] = 'http://127.0.0.1:4321/not-an-actual-file';
  await second.save();
  await assert.rejects(second.check(), /no artifact binding/);
});

test('the exact original artifact must be retained rather than merely named', async t => {
  const setup = await fixture(t);
  const origin = setup.reports['machine-read'].originManifest;
  await writeFile(path.join(setup.root, origin.path), '{}');
  await assert.rejects(setup.check(), /original reviewed manifest bytes changed/);
});

test('report paths inside build inputs or outside the evidence roots are rejected', async t => {
  const setup = await fixture(t);
  setup.config.environments.root.machineReview = 'website/self-approval.json';
  await writeFile(path.join(setup.root, 'website/post-build-reviews.config.json'), JSON.stringify(setup.config));
  await assert.rejects(setup.check(), /Post-build proof must be in/);
});
