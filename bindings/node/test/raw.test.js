import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  resolveRuntime,
  runRaw,
  RuntimeInvocationError,
  WRAPPER_API_VERSION,
} from '../src/index.js';

const bindingsNodeRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(bindingsNodeRoot, '../..');
const validAuthoringInput = path.join(
  repoRoot,
  'contracts/fixtures/valid/minimal-authoring-ir.json',
);

function bundledContractVersion() {
  const manifest = fs.readFileSync(path.join(repoRoot, 'contracts/manifest.yaml'), 'utf8');
  const line = manifest.split(/\r?\n/).find((entry) => entry.trim().startsWith('bundle_version:'));
  assert.ok(line, 'bundled manifest must declare bundle_version');
  return line.split(':', 2)[1].trim().replace(/^['"]|['"]$/g, '');
}

test('resolveRuntime discovers workspace metadata and keeps wrapper version separate', () => {
  const runtime = resolveRuntime({ cwd: bindingsNodeRoot });

  assert.equal(runtime.mode, 'workspace');
  assert.match(runtime.manifestPath, /contracts\/manifest\.yaml$/);
  assert.match(runtime.bundleRoot, /contracts$/);
  assert.equal(runtime.bundleVersion, bundledContractVersion());
  assert.equal(typeof WRAPPER_API_VERSION, 'string');
});

test('resolveRuntime installed mode tolerates indented single-quoted bundle versions', () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-manifest-'));
  const manifestPath = path.join(tempRoot, 'manifest.yaml');
  fs.writeFileSync(manifestPath, "  bundle_version: '9.9.9'\n", 'utf8');

  const runtime = resolveRuntime({
    mode: 'installed',
    manifestPath,
    bundleRoot: tempRoot,
  });

  assert.equal(runtime.bundleVersion, '9.9.9');
});

test('runRaw normalize preserves stdout stderr exit status and argv', async () => {
  const result = await runRaw(
    'normalize',
    { inputPath: validAuthoringInput },
    { cwd: bindingsNodeRoot },
  );

  assert.equal(result.command, 'normalize');
  assert.equal(result.exitStatus, 0);
  assert.equal(typeof result.stdout, 'string');
  assert.equal(typeof result.stderr, 'string');
  assert.equal(Array.isArray(result.argv), true);
  assert.equal(result.resolvedRuntime.mode, 'workspace');
});

test('runRaw raises RuntimeInvocationError when launcher executable is missing', async () => {
  await assert.rejects(
    () =>
      runRaw(
        'normalize',
        { inputPath: validAuthoringInput },
        {
          cwd: bindingsNodeRoot,
          launcherExecutable: '/definitely-missing-anki-forge-binary',
        },
      ),
    (error) =>
      error instanceof RuntimeInvocationError &&
      error.command === 'normalize' &&
      error.resolvedRuntime.mode === 'workspace',
  );
});

test('runRaw installed mode defaults launcher executable and still classifies spawn failures', async () => {
  await assert.rejects(
    () =>
      runRaw(
        'normalize',
        { inputPath: validAuthoringInput },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
        },
      ),
    (error) =>
      error instanceof RuntimeInvocationError &&
      error.command === 'normalize' &&
      error.failurePhase === 'spawn' &&
      error.resolvedRuntime.mode === 'installed',
  );
});

test('runRaw wraps runtime discovery failures as RuntimeInvocationError', async () => {
  const detachedDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-detached-'));

  await assert.rejects(
    () => runRaw('normalize', { inputPath: validAuthoringInput }, { cwd: detachedDir }),
    (error) =>
      error instanceof RuntimeInvocationError &&
      error.command === 'normalize' &&
      error.failurePhase === 'runtime-resolution' &&
      error.resolvedRuntime === null,
  );
});
