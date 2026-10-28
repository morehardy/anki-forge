import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createWriteStream } from 'node:fs';
import { mkdir, readFile, readdir, realpath, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildInputIdentity, sha256 } from './site-manifest.mjs';
import { verifyPostBuildReviews } from './post-build-reviews.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const repository = path.resolve(website, '..');
const defaultDist = path.join(website, 'dist');
export const requiredChecks = Object.freeze([
  'check:content', 'check:site', 'check:surface', 'check:machine',
  'check:schema', 'check:gates', 'test:browser', 'check:search',
]);
const inside = (parent, child) => child === parent || child.startsWith(parent + path.sep);
const identityKeys = ['commit', 'inputSha256', 'configuration', 'releaseLocales'];
const identityFrom = value => Object.fromEntries(identityKeys.map(key => [key, value[key]]));

async function physicalPath(filename) {
  try { return await realpath(filename); }
  catch (error) {
    if (error.code !== 'ENOENT') throw error;
    return path.join(await physicalPath(path.dirname(filename)), path.basename(filename));
  }
}

/** Evidence must stay outside the source and artifact it describes. */
export async function assertEvidenceDirectory(directory, root = repository) {
  const absolute = path.resolve(directory);
  const relative = path.relative(root, absolute).split(path.sep).join('/');
  assert.ok(relative === 'target' || relative.startsWith('target/') || relative === 'docs/plans/evidence' || relative.startsWith('docs/plans/evidence/'), 'Release evidence must be under target/ or docs/plans/evidence/, outside build inputs');
  const physical = await physicalPath(absolute);
  const physicalRoot = await realpath(root);
  assert.ok(!inside(path.join(physicalRoot, 'website'), physical), 'Release evidence cannot resolve into website sources or dist');
  if (inside(path.join(physicalRoot, 'docs'), physical)) assert.ok(inside(path.join(physicalRoot, 'docs/plans/evidence'), physical), 'Release evidence cannot resolve into documentation build inputs');
  return absolute;
}

function assertReleaseMode() {
  assert.notEqual(process.env.CONTENT_REVIEW_MODE, 'preview', 'An editorial preview cannot produce or pass release evidence');
}

function routeFile(directory, route, base) {
  assert.ok(typeof route === 'string' && route.startsWith('/') && !/[?#]/.test(route), `Invalid artifact route: ${route}`);
  const prefix = base === '/' ? '' : base.replace(/\/$/, '');
  assert.ok(route.startsWith(prefix + '/'), `Artifact route lost its deployment base: ${route}`);
  const relative = decodeURIComponent(route.slice(prefix.length));
  const file = path.resolve(directory, `.${relative}${relative.endsWith('/') ? 'index.html' : ''}`);
  assert.ok(inside(path.resolve(directory), file) && file !== path.resolve(directory), `Artifact route leaves dist: ${route}`);
  return file;
}

async function filesIn(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const filename = path.join(directory, entry.name);
    assert.ok(!entry.isSymbolicLink(), `A release artifact must contain concrete files: ${filename}`);
    if (entry.isDirectory()) files.push(...await filesIn(filename));
    else if (entry.isFile()) files.push(filename);
    else assert.fail(`Unsupported artifact entry: ${filename}`);
  }
  return files.sort();
}

/** Independently hash every delivered file and compare recorded files to manifest. */
export async function inspectReleaseArtifact(directory, currentIdentity) {
  const manifestFile = path.join(directory, 'generated/site-manifest.json');
  const manifestBytes = await readFile(manifestFile);
  const manifest = JSON.parse(manifestBytes);
  assert.equal(manifest.schemaVersion, 1, 'Unknown release manifest schema');
  assert.deepEqual(identityFrom(manifest), currentIdentity, 'Current source, configuration or release locales differ from the built artifact');
  assert.ok(Array.isArray(manifest.pages) && Array.isArray(manifest.resources), 'Incomplete release manifest');
  const files = await filesIn(directory);
  const snapshots = new Map(await Promise.all(files.map(async filename => {
    const bytes = await readFile(filename);
    return [filename, { file: path.relative(directory, filename).split(path.sep).join('/'), sha256: sha256(bytes), bytes: bytes.length }];
  })));
  const recorded = new Set();
  for (const entry of [...manifest.pages, ...manifest.resources]) {
    const filename = routeFile(directory, entry.route, manifest.configuration.base);
    assert.ok(!recorded.has(filename), `Duplicate artifact file in manifest: ${entry.route}`);
    recorded.add(filename);
    const actual = snapshots.get(filename);
    assert.ok(actual, `${entry.route}: artifact file is missing`);
    assert.equal(actual.sha256, entry.sha256, `${entry.route}: artifact bytes differ from verified manifest`);
    assert.equal(actual.bytes, entry.bytes, `${entry.route}: artifact byte count differs from verified manifest`);
  }
  const observed = files.filter(filename => {
    const relative = path.relative(directory, filename).split(path.sep).join('/');
    return relative !== 'generated/site-manifest.json' && relative !== 'CNAME' && !relative.startsWith('.');
  });
  assert.deepEqual([...recorded].sort(), observed.sort(), 'Manifest must cover every delivered HTML and resource file');
  if (manifest.configuration.customDomain) {
    assert.equal((await readFile(path.join(directory, 'CNAME'), 'utf8')).trim(), manifest.configuration.customDomain, 'Artifact CNAME differs from the release configuration');
  } else assert.ok(!snapshots.has(path.join(directory, 'CNAME')), 'A project-prefix artifact cannot contain a production CNAME');
  const inventory = [...snapshots.values()];
  return { manifestSha256: sha256(manifestBytes), artifactSha256: sha256(JSON.stringify(inventory)), identity: identityFrom(manifest), files: inventory };
}

export function assertCompleteChecks(checks) {
  assert.deepEqual(checks.map(check => check.name), requiredChecks, 'Release evidence is missing a required check or has a different check order');
  for (const check of checks) {
    assert.equal(check.status, 'passed', `${check.name}: check did not pass`);
    assert.equal(check.exitCode, 0, `${check.name}: nonzero or missing exit status`);
    assert.equal(check.signal, null, `${check.name}: check was terminated`);
    assert.deepEqual(check.command, ['npm', 'run', check.name], `${check.name}: command differs from the required check`);
    assert.ok(check.log?.file && /^[0-9a-f]{64}$/.test(check.log.sha256), `${check.name}: missing execution log evidence`);
  }
}

export function assertEngineeringEvidenceMatches(report, currentArtifact) {
  assert.equal(report.schemaVersion, 1, 'Unknown release evidence schema');
  assert.equal(report.scope, 'local-artifact-engineering', 'Evidence scope is not artifact engineering acceptance');
  assert.ok(['passed', 'engineering-passed-awaiting-review', 'failed'].includes(report.status), 'Release evidence is incomplete or still running');
  assert.equal(report.engineeringStatus, 'passed', 'Engineering checks did not complete successfully');
  assertCompleteChecks(report.checks);
  assert.deepEqual(report.identity, currentArtifact.identity, 'Release evidence used different source inputs or configuration');
  assert.equal(report.manifestSha256, currentArtifact.manifestSha256, 'Release evidence refers to another manifest');
  assert.equal(report.artifactSha256, currentArtifact.artifactSha256, 'Release evidence refers to changed artifact bytes');
}

export function assertEvidenceMatches(report, currentArtifact) {
  assertEngineeringEvidenceMatches(report, currentArtifact);
  assert.equal(report.status, 'passed', 'Release evidence is incomplete, failed, or awaiting independent post-build review');
  const review = report.postBuildReviews;
  assert.equal(review?.status, 'passed', 'Independent post-build task and machine reviews are missing');
  assert.equal(review.currentManifestSha256, currentArtifact.manifestSha256, 'Post-build review receipt refers to another artifact');
  assert.equal(review.inputSha256, currentArtifact.identity.inputSha256, 'Post-build review receipt used different source inputs');
  assert.deepEqual(review.configuration, currentArtifact.identity.configuration, 'Post-build review receipt used another deployment configuration');
  assert.deepEqual(review.releaseLocales, currentArtifact.identity.releaseLocales, 'Post-build review receipt has another locale set');
  assert.deepEqual(review.reports?.map(item => item.kind).sort(), ['machine-read', 'usage-tasks'], 'Both independent post-build review reports are required');
  for (const item of review.reports) assert.match(item.sha256 || '', /^[0-9a-f]{64}$/, 'Post-build review report digest is missing');
}

/** Run a real named npm check; capture combined output even when it fails. */
export async function runNamedCheck(name, logFile, environment = process.env) {
  assert.ok(requiredChecks.includes(name), `Unknown release check: ${name}`);
  const started = Date.now();
  const stream = createWriteStream(logFile, { flags: 'w' });
  let loggingError;
  stream.on('error', error => { loggingError = error; });
  await new Promise((resolve, reject) => { stream.once('open', resolve); stream.once('error', reject); });
  let outcome;
  try {
    outcome = await new Promise(resolve => {
      const child = spawn('npm', ['run', name], { cwd: website, env: { ...environment, PATH: `${path.dirname(process.execPath)}${path.delimiter}${environment.PATH || ''}` }, stdio: ['ignore', 'pipe', 'pipe'], detached: process.platform !== 'win32' });
      child.stdout.on('data', bytes => stream.write(bytes));
      child.stderr.on('data', bytes => stream.write(bytes));
      let timer;
      let forced;
      const terminate = signal => {
        try { if (process.platform === 'win32') child.kill(signal); else process.kill(-child.pid, signal); } catch (error) { if (error.code !== 'ESRCH') stream.write(`\nRunner termination error: ${error.message}\n`); }
      };
      child.once('spawn', () => {
        timer = setTimeout(() => { stream.write('\nRelease check exceeded its 8 minute limit.\n'); terminate('SIGTERM'); forced = setTimeout(() => terminate('SIGKILL'), 5000); forced.unref(); }, 8 * 60 * 1000);
        timer.unref();
      });
      child.once('error', error => { stream.write(`\nCould not run npm: ${error.message}\n`); });
      child.once('close', (exitCode, signal) => { clearTimeout(timer); clearTimeout(forced); resolve({ exitCode, signal }); });
    });
  } finally {
    if (loggingError) throw loggingError;
    await new Promise((resolve, reject) => { stream.once('error', reject); stream.end(resolve); });
  }
  const bytes = await readFile(logFile);
  return { name, command: ['npm', 'run', name], status: outcome.exitCode === 0 && outcome.signal === null ? 'passed' : 'failed', ...outcome, durationMs: Date.now() - started, log: { file: path.basename(logFile), sha256: sha256(bytes), bytes: bytes.length } };
}

function defaultEvidenceDirectory() {
  const base = process.env.SITE_BASE ?? '/anki-forge';
  return path.join(repository, 'target/website-localization', base === '/' ? 'root' : 'prefix');
}

export async function runReleaseEvidence(outputDirectory = process.env.RELEASE_EVIDENCE_DIR || defaultEvidenceDirectory(), { engineeringOnly = false } = {}) {
  assertReleaseMode();
  const output = await assertEvidenceDirectory(outputDirectory);
  await mkdir(output, { recursive: true });
  const reportFile = path.join(output, 'release-evidence.json');
  const report = { schemaVersion: 1, scope: 'local-artifact-engineering', status: 'running', engineeringStatus: 'running', engineeringOnly, startedAt: new Date().toISOString(), runtime: { node: process.version, platform: process.platform, arch: process.arch }, checks: [] };
  const save = () => writeFile(reportFile, JSON.stringify(report, null, 2) + '\n');
  await save();
  let initial;
  let failure;
  try {
    initial = await inspectReleaseArtifact(defaultDist, await buildInputIdentity());
    Object.assign(report, { identity: initial.identity, manifestSha256: initial.manifestSha256, artifactSha256: initial.artifactSha256, fileCount: initial.files.length });
    await save();
    for (const [index, name] of requiredChecks.entries()) {
      console.log(`Release acceptance: running ${name}`);
      const check = await runNamedCheck(name, path.join(output, `${String(index + 1).padStart(2, '0')}-${name.replaceAll(':', '-')}.log`), { ...process.env, RELEASE_EVIDENCE_DIR: output });
      report.checks.push(check);
      await save();
      assert.equal(check.status, 'passed', `${name} failed; inspect ${check.log.file}`);
    }
    assertCompleteChecks(report.checks);
    const searchBytes = await readFile(path.join(output, 'search-results.json'));
    const search = JSON.parse(searchBytes);
    assert.equal(search.status, 'passed', 'Search report did not pass');
    assert.equal(search.mode, 'release-candidate', 'Editorial search evidence cannot authorize release');
    assert.equal(search.manifestSha256, initial.manifestSha256, 'Search report is for another artifact');
    assert.deepEqual(search.configuration, initial.identity.configuration, 'Search report used another deployment configuration');
    report.attachments = [{ file: 'search-results.json', sha256: sha256(searchBytes), bytes: searchBytes.length }];
    report.engineeringStatus = 'passed';
    if (!engineeringOnly) report.postBuildReviews = await reviewCurrentArtifact(initial);
  } catch (error) { failure = error; }
  finally {
    if (initial) {
      try {
        const final = await inspectReleaseArtifact(defaultDist, await buildInputIdentity());
        assert.deepEqual(final, initial, 'Checks changed the artifact they were meant to validate');
        report.artifactUnchanged = true;
      } catch (error) { report.artifactUnchanged = false; report.engineeringStatus = 'failed'; report.integrityError = error.message; failure ||= error; }
    }
    if (report.engineeringStatus !== 'passed') report.engineeringStatus = 'failed';
    report.status = failure ? 'failed' : engineeringOnly ? 'engineering-passed-awaiting-review' : 'passed';
    report.finishedAt = new Date().toISOString();
    if (failure) report.error = failure.message;
    await save();
  }
  if (failure) throw failure;
  console.log(engineeringOnly ? `Engineering checks passed; independent post-build reviews are still required: ${report.manifestSha256}` : `Artifact acceptance and independent post-build reviews passed without rebuilding: ${report.manifestSha256}`);
  return report;
}

async function checkEngineeringAttachments(report, artifact, output) {
  assertEngineeringEvidenceMatches(report, artifact);
  assert.equal(report.artifactUnchanged, true, 'Artifact integrity was not established after checks');
  const attachments = [...report.checks.map(check => check.log), ...(report.attachments || [])];
  assert.ok(report.attachments?.some(file => file.file === 'search-results.json'), 'Search result attachment is missing');
  for (const attachment of attachments) {
    assert.equal(path.basename(attachment.file), attachment.file, 'Evidence attachments must stay in their batch directory');
    const bytes = await readFile(path.join(output, attachment.file));
    assert.equal(sha256(bytes), attachment.sha256, `${attachment.file}: execution evidence changed`);
    assert.equal(bytes.length, attachment.bytes, `${attachment.file}: execution evidence byte count changed`);
  }
  const search = JSON.parse(await readFile(path.join(output, 'search-results.json'), 'utf8'));
  assert.equal(search.status, 'passed', 'Search evidence did not pass');
  assert.equal(search.mode, 'release-candidate', 'Editorial search evidence cannot authorize release');
  assert.equal(search.manifestSha256, artifact.manifestSha256, 'Search evidence is for another artifact');
  assert.deepEqual(search.configuration, artifact.identity.configuration, 'Search evidence used another configuration');
}

async function reviewCurrentArtifact(artifact) {
  const bytes = await readFile(path.join(defaultDist, 'generated/site-manifest.json'));
  assert.equal(sha256(bytes), artifact.manifestSha256, 'Artifact manifest changed before independent review verification');
  return verifyPostBuildReviews(JSON.parse(bytes), artifact.manifestSha256);
}

/** Attach real reviews after an explicit engineering-only run, without rebuilding. */
export async function finishReleaseEvidence(outputDirectory = process.env.RELEASE_EVIDENCE_DIR || defaultEvidenceDirectory()) {
  assertReleaseMode();
  const output = await assertEvidenceDirectory(outputDirectory);
  const filename = path.join(output, 'release-evidence.json');
  const report = JSON.parse(await readFile(filename, 'utf8'));
  try {
    const artifact = await inspectReleaseArtifact(defaultDist, await buildInputIdentity());
    await checkEngineeringAttachments(report, artifact, output);
    report.postBuildReviews = await reviewCurrentArtifact(artifact);
    const final = await inspectReleaseArtifact(defaultDist, await buildInputIdentity());
    assert.deepEqual(final, artifact, 'Review verification changed the artifact or source inputs');
    report.status = 'passed';
    report.engineeringOnly = false;
    report.reviewAttachedAt = new Date().toISOString();
    report.finishedAt = report.reviewAttachedAt;
    delete report.error;
    assertEvidenceMatches(report, final);
  } catch (error) {
    report.status = 'failed';
    report.error = error.message;
    await writeFile(filename, JSON.stringify(report, null, 2) + '\n');
    throw error;
  }
  await writeFile(filename, JSON.stringify(report, null, 2) + '\n');
  console.log(`Independent six-task and four-machine reviews attached to the unchanged artifact: ${report.manifestSha256}`);
  return report;
}

export async function checkReleaseEvidence(outputDirectory = process.env.RELEASE_EVIDENCE_DIR || defaultEvidenceDirectory()) {
  assertReleaseMode();
  const output = await assertEvidenceDirectory(outputDirectory);
  const report = JSON.parse(await readFile(path.join(output, 'release-evidence.json'), 'utf8'));
  const artifact = await inspectReleaseArtifact(defaultDist, await buildInputIdentity());
  assertEvidenceMatches(report, artifact);
  await checkEngineeringAttachments(report, artifact, output);
  const reviews = await reviewCurrentArtifact(artifact);
  assert.deepEqual(report.postBuildReviews, reviews, 'Independent post-build review reports or their evidence changed');
  assert.deepEqual(await inspectReleaseArtifact(defaultDist, await buildInputIdentity()), artifact, 'Artifact or inputs changed while checking release evidence');
  console.log(`Release evidence still matches current inputs and all ${artifact.files.length} delivered files: ${artifact.manifestSha256}`);
  return report;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const mode = args.shift() || 'run';
  const engineeringOnly = args.includes('--engineering-only');
  const values = args.filter(value => value !== '--engineering-only');
  assert.ok(['run', 'finish', 'check'].includes(mode) && values.length <= 1 && !values.some(value => value.startsWith('--')), 'Use release-evidence.mjs run [--engineering-only]|finish|check [evidence-directory]');
  assert.ok(!engineeringOnly || mode === 'run', '--engineering-only is allowed only for an explicit run; it never passes check:release');
  if (mode === 'run') await runReleaseEvidence(values[0], { engineeringOnly });
  else if (mode === 'finish') await finishReleaseEvidence(values[0]);
  else await checkReleaseEvidence(values[0]);
}
