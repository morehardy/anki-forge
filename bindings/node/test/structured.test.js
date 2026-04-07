import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { build, diff, inspect, normalize, ProtocolParseError } from '../src/index.js';

const bindingsNodeRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(bindingsNodeRoot, '../..');
const validAuthoringInput = path.join(
  repoRoot,
  'contracts/fixtures/valid/minimal-authoring-ir.json',
);
const invalidAuthoringInput = path.join(
  repoRoot,
  'contracts/fixtures/invalid/missing-document-id.json',
);
const validNormalizedInput = path.join(
  repoRoot,
  'contracts/fixtures/phase3/inputs/basic-normalized-ir.json',
);

function fakeLauncherScript(source) {
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-fake-'));
  const fakeScript = path.join(fakeDir, 'fake.js');
  fs.writeFileSync(fakeScript, source);
  return fakeScript;
}

test('structured normalize returns invalid result without throwing on contract-invalid output', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'normalization-result',
      result_status: 'invalid',
      tool_contract_version: 'phase2-v1',
      policy_refs: { identity_policy: 'identity-policy.default@1.0.0' },
      comparison_context: { kind: 'comparison-context', identity_mode: 'document-id' },
      diagnostics: { status: 'invalid', items: [] }
    }));
  `);

  const result = await normalize(
    { inputPath: invalidAuthoringInput },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(result.kind, 'normalization-result');
  assert.equal(result.result_status, 'invalid');
  assert.equal(result.helper.isInvalid, true);
  assert.equal(result.helper.warningCount >= 0, true);
});

test('structured build derives helper artifact paths from returned refs', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'package-build-result',
      result_status: 'success',
      tool_contract_version: 'phase3-v1',
      writer_policy_ref: 'writer-policy.default@1.0.0',
      build_context_ref: 'build-context.default@1.0.0',
      staging_ref: 'artifacts/alt/staging/manifest.json',
      artifact_fingerprint: 'artifact:demo',
      apkg_ref: 'artifacts/alt/package.apkg',
      diagnostics: { kind: 'build-diagnostics', items: [] }
    }));
  `);
  const artifactsDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-build-'));
  const result = await build(
    { inputPath: validNormalizedInput, artifactsDir },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(result.kind, 'package-build-result');
  assert.equal(result.result_status, 'success');
  assert.equal(typeof result.resolvedRuntime.bundleVersion, 'string');
  assert.match(result.helper.artifactPaths.stagingManifest, /alt\/staging\/manifest\.json$/);
  assert.match(result.helper.artifactPaths.apkg, /alt\/package\.apkg$/);
});

test('structured normalize raises ProtocolParseError for invalid json stdout', async () => {
  const fakeScript = fakeLauncherScript("process.stdout.write('{broken');");

  await assert.rejects(
    () =>
      normalize(
        { inputPath: validAuthoringInput },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
          launcherExecutable: process.execPath,
          launcherPrefix: [fakeScript],
        },
      ),
    (error) => error instanceof ProtocolParseError && error.parsePhase === 'json',
  );
});

test('structured normalize raises ProtocolParseError for contract-shape mismatch', async () => {
  const fakeScript = fakeLauncherScript(
    "process.stdout.write(JSON.stringify({ kind: 'normalization-result' }));",
  );

  await assert.rejects(
    () =>
      normalize(
        { inputPath: validAuthoringInput },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
          launcherExecutable: process.execPath,
          launcherPrefix: [fakeScript],
        },
      ),
    (error) => error instanceof ProtocolParseError && error.parsePhase === 'contract-shape',
  );
});

test('structured build raises ProtocolParseError for contract-version mismatch', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'package-build-result',
      result_status: 'success',
      tool_contract_version: 'phase3-v999',
      writer_policy_ref: 'writer-policy.default@1.0.0',
      build_context_ref: 'build-context.default@1.0.0',
      staging_ref: 'artifacts/staging/manifest.json',
      artifact_fingerprint: 'artifact:demo',
      diagnostics: { kind: 'build-diagnostics', items: [] }
    }));
  `);

  await assert.rejects(
    () =>
      build(
        {
          inputPath: validNormalizedInput,
          artifactsDir: fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-version-')),
        },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
          launcherExecutable: process.execPath,
          launcherPrefix: [fakeScript],
        },
      ),
    (error) => error instanceof ProtocolParseError && error.parsePhase === 'contract-version',
  );
});

test('structured inspect returns degraded result without throwing', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'inspect-report',
      observation_model_version: 'phase3-inspect-v1',
      source_kind: 'apkg',
      source_ref: 'artifacts/package-no-media.apkg',
      artifact_fingerprint: 'artifact:demo',
      observation_status: 'degraded',
      missing_domains: ['media'],
      degradation_reasons: ['media map unavailable'],
      observations: { notetypes: [], templates: [], fields: [], media: [], metadata: [], references: [] }
    }));
  `);

  const result = await inspect(
    { apkgPath: path.join(repoRoot, 'tmp/fake.apkg') },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(result.observation_status, 'degraded');
  assert.equal(result.helper.isDegraded, true);
});

test('structured diff returns partial result without throwing', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'diff-report',
      comparison_status: 'partial',
      left_fingerprint: 'artifact:left',
      right_fingerprint: 'artifact:right',
      left_observation_model_version: 'phase3-inspect-v1',
      right_observation_model_version: 'phase3-inspect-v1',
      summary: 'reference coverage reduced',
      uncompared_domains: ['references'],
      comparison_limitations: ['right report is degraded'],
      changes: []
    }));
  `);

  const result = await diff(
    {
      leftPath: path.join(repoRoot, 'tmp/left.inspect.json'),
      rightPath: path.join(repoRoot, 'tmp/right.inspect.json'),
    },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(result.comparison_status, 'partial');
  assert.equal(result.helper.isPartial, true);
});
