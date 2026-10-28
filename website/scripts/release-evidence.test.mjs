import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { sha256 } from './site-manifest.mjs';
import { assertCompleteChecks, assertEvidenceDirectory, assertEngineeringEvidenceMatches, assertEvidenceMatches, inspectReleaseArtifact, requiredChecks, runNamedCheck } from './release-evidence.mjs';

const identity = { commit: 'a'.repeat(40), inputSha256: 'b'.repeat(64), configuration: { site: 'https://example.test', base: '/', customDomain: '' }, releaseLocales: ['en', 'zh-CN'] };
async function fixture(t) {
  const root = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-release-evidence-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const dist = path.join(root, 'website/dist');
  await mkdir(path.join(dist, 'generated'), { recursive: true });
  const html = Buffer.from('<html lang="en"><main>Retain the original APKG.</main></html>');
  const style = Buffer.from('main { color: navy; }\n');
  await writeFile(path.join(dist, 'index.html'), html);
  await writeFile(path.join(dist, 'site.css'), style);
  const manifest = { schemaVersion: 1, ...structuredClone(identity), pages: [{ route: '/', sha256: sha256(html), bytes: html.length }], resources: [{ route: '/site.css', sha256: sha256(style), bytes: style.length }] };
  const manifestFile = path.join(dist, 'generated/site-manifest.json');
  await writeFile(manifestFile, JSON.stringify(manifest));
  return { root, dist, manifest, manifestFile };
}
const completed = () => requiredChecks.map(name => ({ name, command: ['npm', 'run', name], status: 'passed', exitCode: 0, signal: null, log: { file: name.replaceAll(':', '-') + '.log', sha256: 'c'.repeat(64), bytes: 0 } }));
function receipt(artifact) {
  return { schemaVersion: 1, scope: 'local-artifact-engineering', status: 'passed', engineeringStatus: 'passed', checks: completed(), identity: artifact.identity, manifestSha256: artifact.manifestSha256, artifactSha256: artifact.artifactSha256,
    postBuildReviews: { status: 'passed', currentManifestSha256: artifact.manifestSha256, inputSha256: artifact.identity.inputSha256, configuration: artifact.identity.configuration, releaseLocales: artifact.identity.releaseLocales, reports: [{ kind: 'usage-tasks', sha256: 'd'.repeat(64) }, { kind: 'machine-read', sha256: 'e'.repeat(64) }] },
  };
}

test('acceptance binds real manifest bytes, every delivered file and current inputs', async t => {
  const { dist } = await fixture(t);
  const artifact = await inspectReleaseArtifact(dist, identity);
  assert.equal(artifact.files.length, 3);
  assert.deepEqual(artifact.identity, identity);
  assert.doesNotThrow(() => assertEvidenceMatches(receipt(artifact), artifact));
});

test('changing source identity or release selection invalidates an existing artifact', async t => {
  const { dist } = await fixture(t);
  await assert.rejects(inspectReleaseArtifact(dist, { ...identity, inputSha256: 'd'.repeat(64) }), /Current source/);
  await assert.rejects(inspectReleaseArtifact(dist, { ...identity, releaseLocales: ['en', 'zh-CN', 'es'] }), /Current source/);
});

test('changing an actual resource or omitting it from manifest is rejected', async t => {
  const { dist, manifest, manifestFile } = await fixture(t);
  await writeFile(path.join(dist, 'site.css'), 'changed resource');
  await assert.rejects(inspectReleaseArtifact(dist, identity), /artifact bytes differ/);
  manifest.resources = [];
  await writeFile(manifestFile, JSON.stringify(manifest));
  await assert.rejects(inspectReleaseArtifact(dist, identity), /cover every delivered HTML and resource/);
});

test('a changed manifest cannot reuse an earlier passing receipt', async t => {
  const { dist, manifest, manifestFile } = await fixture(t);
  const artifact = await inspectReleaseArtifact(dist, identity);
  const report = receipt(artifact);
  manifest.extra = 'new manifest bytes';
  await writeFile(manifestFile, JSON.stringify(manifest));
  const changed = await inspectReleaseArtifact(dist, identity);
  assert.throws(() => assertEvidenceMatches(report, changed), /another manifest/);
});

test('adding batch evidence outside the artifact leaves its receipt unchanged', async t => {
  const { root, dist } = await fixture(t);
  const before = await inspectReleaseArtifact(dist, identity);
  const evidence = path.join(root, 'docs/plans/evidence/release-check');
  await mkdir(evidence, { recursive: true });
  await writeFile(path.join(evidence, 'observation.json'), '{"source":"actual-check"}\n');
  assert.deepEqual(await inspectReleaseArtifact(dist, identity), before);
});

test('missing, failed, interrupted or different commands cannot become a passing report', async t => {
  const { dist } = await fixture(t);
  const artifact = await inspectReleaseArtifact(dist, identity);
  for (const mutate of [
    checks => checks.slice(1),
    checks => checks.map((check, index) => index === 0 ? { ...check, status: 'failed', exitCode: 1 } : check),
    checks => checks.map((check, index) => index === 0 ? { ...check, exitCode: null, signal: 'SIGTERM' } : check),
    checks => checks.map((check, index) => index === 0 ? { ...check, command: ['npm', 'run', 'build'] } : check),
  ]) {
    const report = receipt(artifact);
    report.checks = mutate(report.checks);
    assert.throws(() => assertEvidenceMatches(report, artifact), /required check|did not pass|exit status|command differs/);
  }
  assert.throws(() => assertCompleteChecks([]), /missing a required check/);
  assert.throws(() => assertEvidenceMatches({ ...receipt(artifact), status: 'running' }, artifact), /incomplete or still running/);
});

test('an engineering-only receipt never passes the complete release gate', async t => {
  const { dist } = await fixture(t);
  const artifact = await inspectReleaseArtifact(dist, identity);
  const report = receipt(artifact);
  report.status = 'engineering-passed-awaiting-review';
  delete report.postBuildReviews;
  assert.doesNotThrow(() => assertEngineeringEvidenceMatches(report, artifact));
  assert.throws(() => assertEvidenceMatches(report, artifact), /awaiting independent post-build review/);
  report.status = 'passed';
  assert.throws(() => assertEvidenceMatches(report, artifact), /task and machine reviews are missing/);
});

test('evidence outputs cannot enter source trees, dist, or point there through a symlink', async t => {
  const { root } = await fixture(t);
  await mkdir(path.join(root, 'target'), { recursive: true });
  assert.equal(await assertEvidenceDirectory(path.join(root, 'target/checks'), root), path.join(root, 'target/checks'));
  assert.equal(await assertEvidenceDirectory(path.join(root, 'docs/plans/evidence/checks'), root), path.join(root, 'docs/plans/evidence/checks'));
  for (const relative of ['website/dist/review', 'website/reviews', 'docs/reviews', 'target/../website/reviews']) await assert.rejects(assertEvidenceDirectory(path.join(root, relative), root), /outside build inputs/);
  await symlink(path.join(root, 'website'), path.join(root, 'target/unsafe'));
  await assert.rejects(assertEvidenceDirectory(path.join(root, 'target/unsafe/reviews'), root), /cannot resolve into website/);
});

test('an editorial preview is rejected before creating any release receipt', async t => {
  const { root } = await fixture(t);
  const script = fileURLToPath(new URL('./release-evidence.mjs', import.meta.url));
  const output = path.join(root, 'target/preview');
  assert.throws(() => execFileSync(process.execPath, [script, 'run', output], { env: { ...process.env, CONTENT_REVIEW_MODE: 'preview' }, stdio: 'pipe', encoding: 'utf8' }), /editorial preview/);
  await assert.rejects(readFile(path.join(output, 'release-evidence.json')), { code: 'ENOENT' });
});

test('a genuinely failed child process keeps its output and cannot be recorded as passed', async t => {
  const { root } = await fixture(t);
  const logFile = path.join(root, 'failed-command.log');
  const result = await runNamedCheck('check:content', logFile, { ...process.env, NODE_OPTIONS: '--anki-forge-invalid-release-test-option' });
  assert.equal(result.status, 'failed');
  assert.notEqual(result.exitCode, 0);
  assert.equal(result.signal, null);
  const bytes = await readFile(logFile);
  assert.match(bytes.toString(), /not allowed in NODE_OPTIONS|bad option/);
  assert.equal(result.log.sha256, sha256(bytes));
  assert.equal(result.log.bytes, bytes.length);
});
