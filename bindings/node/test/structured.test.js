import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  build,
  diff,
  inspect,
  normalize,
  productBuild,
  productValidate,
  templateValidate,
  ProtocolParseError,
  RuntimeInvocationError,
} from '../legacy/src/index.js';

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

test('legacy validate rejects publication options before invoking the runtime', async () => {
  for (const option of [{ apkgOut: 'must-not-exist.apkg' }, { reportJson: 'must-not-exist.json' }, { writeIdentityLockfile: true }]) {
    await assert.rejects(productValidate({ productDocument: {}, ...option }, { launcherExecutable: 'must-not-launch' }), TypeError);
  }
});

test('inline legacy documents resolve relative media against the supplied baseDir', async t => {
  const baseDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-relative-'));
  t.after(() => fs.rmSync(baseDir, { recursive: true, force: true }));
  const document = JSON.parse(fs.readFileSync(path.join(repoRoot, 'contracts/fixtures/product-v2/custom-typed-media.json'), 'utf8'));
  fs.writeFileSync(path.join(baseDir, 'hello.wav'), Buffer.from(document.media[0].source.data_base64, 'base64'));
  document.media[0].source = { kind: 'file', path: 'hello.wav' };
  const apkgOut = path.join(baseDir, 'result.apkg');
  const result = await productBuild({ productDocument: document, baseDir, apkgOut }, { cwd: bindingsNodeRoot });
  assert.equal(result.status, 'success', JSON.stringify(result.diagnostics)); assert.equal(result.counts.media, 1); assert.ok(fs.existsSync(apkgOut));
});

test('legacy malformed field types are protocol errors', async t => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-shape-'));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const script = path.join(temporary, 'fake.cjs');
  fs.writeFileSync(script, `process.stdout.write(JSON.stringify({kind:'anki-forge-build-report',schema_version:'phase4-build-report-v2',status:'success',comparison:'not_requested',counts:{notes:'1',cards:1,media:0},diagnostics:[],policy:{}}))`);
  await assert.rejects(productBuild({ productDocument: {}, apkgOut: path.join(temporary, 'result.apkg') }, {
    mode: 'installed', manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'), bundleRoot: path.join(repoRoot, 'contracts'), launcherExecutable: process.execPath, launcherPrefix: [script],
  }), error => error instanceof ProtocolParseError && error.parsePhase === 'contract-shape');
});

function fakeLauncherScript(source) {
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-fake-'));
  const fakeScript = path.join(fakeDir, 'fake.js');
  fs.writeFileSync(fakeScript, source);
  return fakeScript;
}

test('real product runtime protects the baseline and does not publish blocked candidates', async (t) => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-baseline-'));
  t.after(() => fs.rmSync(tempDir, { recursive: true, force: true }));
  const baseline = path.join(tempDir, 'previous.apkg');
  const output = path.join(tempDir, 'output.apkg');
  const identityLockfile = path.join(tempDir, 'identity.json');
  const runtime = { cwd: bindingsNodeRoot };
  const fixture = (name) => path.join(repoRoot, 'contracts/fixtures/product-v2', `${name}.json`);
  const first = await productBuild({
    inputPath: fixture('basic-stock'),
    apkgOut: baseline,
    identityLockfile,
    writeIdentityLockfile: true,
  }, runtime);
  assert.equal(first.status, 'success');
  const original = fs.readFileSync(baseline);
  const originalLockfile = fs.readFileSync(identityLockfile);

  const collision = await productBuild({
    inputPath: fixture('compare-risk'),
    apkgOut: baseline,
    compareTo: baseline,
    failOn: 'low',
  }, runtime);
  assert.equal(collision.status, 'invalid');
  assert.equal(collision.artifact, null);
  assert.ok(collision.diagnostics.some(({ code }) => code === 'PROJECT.PATH_COLLISION'));
  assert.ok(fs.readFileSync(baseline).equals(original), 'baseline bytes must stay unchanged');

  fs.writeFileSync(output, 'previous publication');
  const blocked = await productBuild({
    inputPath: fixture('compare-risk'),
    apkgOut: output,
    compareTo: baseline,
    failOn: 'low',
    identityLockfile,
    writeIdentityLockfile: true,
  }, runtime);
  assert.equal(blocked.status, 'blocked');
  assert.equal(blocked.artifact, null);
  assert.equal(blocked.update_safety.lockfile_written, false);
  assert.ok(blocked.diff.artifact_diff.changes.length > 0);
  assert.ok(blocked.risk.findings.length > 0);
  assert.equal(fs.readFileSync(output, 'utf8'), 'previous publication');
  assert.ok(fs.readFileSync(baseline).equals(original), 'baseline bytes must stay unchanged');
  assert.ok(fs.readFileSync(identityLockfile).equals(originalLockfile), 'lockfile bytes must stay unchanged');
});

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

test('product APIs share the Rust product-build contract', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'anki-forge-build-report',
      schema_version: 'phase4-build-report-v2',
      tool_version: 'test',
      status: 'success',
      comparison: 'not_requested',
      artifact: { path: 'deck.apkg' },
      counts: { notes: 1, cards: 2, media: 0 },
      media: {},
      diagnostics: [],
      metrics: { duration_ms: 1 },
      policy: { status: 'not_evaluated', blocking_findings: [] }
    }));
  `);
  const runtime = {
    mode: 'installed',
    manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
    bundleRoot: path.join(repoRoot, 'contracts'),
    launcherExecutable: process.execPath,
    launcherPrefix: [fakeScript],
  };
  const productDocument = {
    product_document_version: 'product-v3',
    document_id: 'node-custom-cloze',
    note_types: [],
    notes: [],
  };

  const apkgOut = path.join(
    fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-product-output-')),
    'deck.apkg',
  );
  const buildResult = await productBuild({ productDocument, apkgOut }, runtime);
  assert.equal(buildResult.status, 'success');

  for (const validate of [productValidate, templateValidate]) {
    const result = await validate({ productDocument }, runtime);
    assert.equal(result.status, 'success');
    assert.equal(result.counts.cards, 2);
    assert.equal(result.artifact, null);
    assert.equal(result.helper.isInvalid, false);
    assert.equal(result.rawCommand.command, 'product-build');
  }
});

test('productBuild requires an explicit retained artifact path', async () => {
  await assert.rejects(
    () => productBuild({ productDocument: {} }),
    (error) => error instanceof TypeError && /requires apkgOut/.test(error.message),
  );
});

test('productBuild retains the explicitly requested artifact', async () => {
  const fakeScript = fakeLauncherScript(`
    const fs = require('node:fs');
    const outputIndex = process.argv.indexOf('--apkg-out');
    const output = process.argv[outputIndex + 1];
    fs.writeFileSync(output, 'apkg');
    process.stdout.write(JSON.stringify({
      kind: 'anki-forge-build-report',
      schema_version: 'phase4-build-report-v2',
      status: 'success',
      comparison: 'not_requested',
      counts: { notes: 0, cards: 0, media: 0 },
      diagnostics: [],
      policy: {},
      artifact: { path: output }
    }));
  `);
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-retained-'));
  const apkgOut = path.join(outputDir, 'deck.apkg');

  await productBuild(
    { productDocument: {}, apkgOut },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(fs.existsSync(apkgOut), true);
});

test('productBuild preserves nonzero runtime failures without a valid report', async () => {
  const fakeScript = fakeLauncherScript("process.stderr.write('boom'); process.exit(2);");

  await assert.rejects(
    () =>
      productBuild(
        { productDocument: {}, apkgOut: path.join(os.tmpdir(), 'unused.apkg') },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
          launcherExecutable: process.execPath,
          launcherPrefix: [fakeScript],
        },
      ),
    (error) =>
      error instanceof RuntimeInvocationError &&
      error.exitStatus === 2 &&
      error.failurePhase === 'process-exit',
  );
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

test('structured observation versions support legacy, current, and mixed reports', async () => {
  const fixture = (name) => JSON.parse(fs.readFileSync(
    path.join(repoRoot, 'contracts/fixtures/phase3/expected', name), 'utf8',
  ));
  const runtimeFor = (payload) => ({
    mode: 'installed',
    manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
    bundleRoot: path.join(repoRoot, 'contracts'),
    launcherExecutable: process.execPath,
    launcherPrefix: [fakeLauncherScript(`process.stdout.write(${JSON.stringify(JSON.stringify(payload))});`)],
  });
  for (const version of ['phase3-inspect-v1', 'phase3-inspect-v2']) {
    const payload = fixture('basic.inspect.json');
    payload.observation_model_version = version;
    const result = await inspect({ apkgPath: 'unused.apkg' }, runtimeFor(payload));
    assert.equal(result.observation_model_version, version);
  }
  for (const [left, right] of [
    ['phase3-inspect-v1', 'phase3-inspect-v2'],
    ['phase3-inspect-v2', 'phase3-inspect-v1'],
    ['phase3-inspect-v2', 'phase3-inspect-v2'],
  ]) {
    const payload = fixture('basic.diff.json');
    payload.left_observation_model_version = left;
    payload.right_observation_model_version = right;
    payload.comparison_status = left === right ? 'complete' : 'partial';
    payload.comparison_limitations = left === right ? [] : ['observation model versions differ'];
    const result = await diff({ leftPath: 'left.json', rightPath: 'right.json' }, runtimeFor(payload));
    assert.equal(result.comparison_status, payload.comparison_status);
  }
  const unknown = fixture('basic.inspect.json');
  unknown.observation_model_version = 'phase3-inspect-v999';
  await assert.rejects(
    () => inspect({ apkgPath: 'unused.apkg' }, runtimeFor(unknown)),
    (error) => error instanceof ProtocolParseError && error.parsePhase === 'contract-version',
  );
});
