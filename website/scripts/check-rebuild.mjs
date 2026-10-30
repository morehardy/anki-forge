import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assertEvidenceDirectory, inspectReleaseArtifact } from './release-evidence.mjs';
import { artifactPath } from './public-surface.mjs';
import { buildInputIdentity, sha256 } from './site-manifest.mjs';

const website = fileURLToPath(new URL('../', import.meta.url));
const repository = path.resolve(website, '..');
const scoped = file => file.endsWith('.html') || file.endsWith('.md') || path.posix.basename(file) === 'llms.txt';

/** Observe real files after independently checking every manifest hash. */
export async function captureRebuildSnapshot(directory, identity) {
  const artifact = await inspectReleaseArtifact(directory, identity);
  const manifestBytes = await readFile(path.join(directory, 'generated/site-manifest.json'));
  assert.equal(sha256(manifestBytes), artifact.manifestSha256, 'Manifest changed while taking the rebuild snapshot');
  const manifest = JSON.parse(manifestBytes);
  const filename = entry => path.relative(directory, artifactPath(directory, entry.route, identity.configuration.base)).split(path.sep).join('/');
  const pages = manifest.pages.map(filename).sort();
  assert.ok(pages.length && pages.every(file => file.endsWith('.html')), 'Rebuild scope requires actual HTML page rows');
  assert.ok(pages.includes('404.html'), 'Rebuild scope must include the real 404 HTML');
  assert.deepEqual(pages, artifact.files.filter(entry => entry.file.endsWith('.html')).map(entry => entry.file).sort(), 'Every delivered HTML file must have a manifest page row');
  const resources = manifest.resources.map(filename).filter(scoped);
  assert.ok(resources.some(file => file.endsWith('.md')) && resources.some(file => path.posix.basename(file) === 'llms.txt'), 'Rebuild scope requires Markdown and llms resources');
  const files = artifact.files.filter(entry => scoped(entry.file));
  assert.deepEqual([...pages, ...resources].sort(), files.map(entry => entry.file).sort(), 'Every delivered HTML and machine file must be in the rebuild scope');
  return { identity: artifact.identity, files };
}

export function assertRebuildMatches(before, after) {
  assert.deepEqual(after.identity, before.identity, 'Rebuild changed source input identity, configuration, locales or commit');
  assert.deepEqual(after.files.map(entry => entry.file), before.files.map(entry => entry.file), 'Rebuild changed the delivered HTML or machine file set');
  for (const [index, entry] of before.files.entries()) {
    assert.deepEqual(after.files[index], entry, `${entry.file}: same-input rebuild changed delivered bytes`);
  }
}

export async function checkRebuild() {
  assert.notEqual(process.env.CONTENT_REVIEW_MODE, 'preview', 'An editorial preview cannot establish release rebuild stability');
  const identity = await buildInputIdentity();
  const output = await assertEvidenceDirectory(process.env.RELEASE_EVIDENCE_DIR || path.join(repository, 'target/website-localization', identity.configuration.base === '/' ? 'root' : 'prefix'));
  await mkdir(output, { recursive: true });
  const report = { schemaVersion: 1, scope: 'same-input-html-and-machine-rebuild', status: 'running', startedAt: new Date().toISOString(), command: ['npm', 'run', 'build'] };
  const save = () => writeFile(path.join(output, 'rebuild-results.json'), JSON.stringify(report, null, 2) + '\n');
  await save();
  try {
    report.before = await captureRebuildSnapshot(path.join(website, 'dist'), identity);
    await save();
    console.log(`Rebuilding the same inputs to compare ${report.before.files.length} delivered HTML and machine files.`);
    const result = await new Promise((resolve, reject) => {
      const child = spawn('npm', ['run', 'build'], { cwd: website, env: { ...process.env, PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH || ''}` }, stdio: 'inherit' });
      child.once('error', reject);
      child.once('close', (exitCode, signal) => resolve({ exitCode, signal }));
    });
    Object.assign(report, result);
    assert.equal(result.exitCode, 0, 'The actual rebuild failed');
    assert.equal(result.signal, null, 'The actual rebuild was interrupted');
    report.after = await captureRebuildSnapshot(path.join(website, 'dist'), await buildInputIdentity());
    assertRebuildMatches(report.before, report.after);
    report.status = 'passed';
    console.log(`Same-input rebuild preserved all ${report.after.files.length} HTML and machine files byte for byte; the second artifact is ready for release checks.`);
  } catch (error) {
    report.status = 'failed';
    report.error = error.message;
    throw error;
  } finally {
    report.finishedAt = new Date().toISOString();
    await save();
  }
  return report;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await checkRebuild();
