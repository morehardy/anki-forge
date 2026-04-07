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

test('resolveRuntime discovers workspace metadata and keeps wrapper version separate', () => {
  const runtime = resolveRuntime({ cwd: bindingsNodeRoot });

  assert.equal(runtime.mode, 'workspace');
  assert.match(runtime.manifestPath, /contracts\/manifest\.yaml$/);
  assert.match(runtime.bundleRoot, /contracts$/);
  assert.equal(runtime.bundleVersion, '0.1.0');
  assert.equal(typeof WRAPPER_API_VERSION, 'string');
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
