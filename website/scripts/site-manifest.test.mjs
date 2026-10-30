import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { buildInputIdentity } from './site-manifest.mjs';

test('real binding, Cargo and contract changes invalidate an artifact identity while external evidence does not', async t => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'anki-forge-build-inputs-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const git = args => execFileSync('git', args, { cwd: root, stdio: 'pipe' });
  git(['init', '-q']);
  const files = ['website/src/page.astro', 'bindings/python/MIGRATION.md', 'bindings/python/examples/json_to_anki.py', 'bindings/node/examples/json_to_anki.mts', 'bindings/node/native/src/lib.rs', 'contract_tools/src/lib.rs', 'contracts/manifest.yaml', 'scripts/packaged_consumer/src/inspection.rs', '.cargo/config.toml', 'rust-toolchain.toml'];
  for (const filename of files) {
    await mkdir(path.dirname(path.join(root, filename)), { recursive: true });
    await writeFile(path.join(root, filename), 'original\n');
  }
  git(['add', '.']);
  git(['-c', 'user.email=fixture@example.test', '-c', 'user.name=Fixture', 'commit', '-qm', 'Input fixture']);
  const before = await buildInputIdentity(root);
  for (const filename of files) {
    await writeFile(path.join(root, filename), 'changed\n');
    const changed = await buildInputIdentity(root);
    assert.notEqual(changed.inputSha256, before.inputSha256, `${filename}: real build input must invalidate the receipt`);
    assert.equal(changed.commit, before.commit);
    await writeFile(path.join(root, filename), 'original\n');
  }
  await mkdir(path.join(root, 'docs/plans/evidence/batch'), { recursive: true });
  await writeFile(path.join(root, 'docs/plans/evidence/batch/proof.json'), '{"status":"passed"}\n');
  assert.deepEqual(await buildInputIdentity(root), before);
  await writeFile(path.join(root, 'contracts/new-fixture.json'), '{}\n');
  assert.notEqual((await buildInputIdentity(root)).inputSha256, before.inputSha256, 'Untracked real contract inputs must be included');
});
