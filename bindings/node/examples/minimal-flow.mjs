import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { build, diff, inspect, normalize, resolveRuntime } from '../src/index.js';

const bindingsNodeRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(bindingsNodeRoot, '../..');
const runtime = resolveRuntime({ cwd: bindingsNodeRoot });
console.log('resolved runtime =>', runtime);

const normalized = await normalize(
  { inputPath: path.join(repoRoot, 'contracts/fixtures/valid/minimal-authoring-ir.json') },
  { cwd: bindingsNodeRoot },
);
console.log('normalize status =>', normalized.result_status);

const artifactsDir = path.join(repoRoot, 'tmp/phase4-node-example/basic');
const buildResult = await build(
  {
    inputPath: path.join(repoRoot, 'contracts/fixtures/phase3/inputs/basic-normalized-ir.json'),
    artifactsDir,
  },
  { cwd: bindingsNodeRoot },
);
console.log('build status =>', buildResult.result_status);

const stagingReport = await inspect(
  { stagingPath: path.join(artifactsDir, 'staging/manifest.json') },
  { cwd: bindingsNodeRoot },
);
const apkgReport = await inspect(
  { apkgPath: path.join(artifactsDir, 'package.apkg') },
  { cwd: bindingsNodeRoot },
);
console.log(
  'inspect statuses =>',
  stagingReport.observation_status,
  apkgReport.observation_status,
);

const diffResult = await diff(
  {
    leftPath: path.join(repoRoot, 'contracts/fixtures/phase3/expected/basic.inspect.json'),
    rightPath: path.join(repoRoot, 'contracts/fixtures/phase3/expected/basic.inspect.json'),
  },
  { cwd: bindingsNodeRoot },
);
console.log('diff status =>', diffResult.comparison_status);
