import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {
  Project,
  Deck,
  Note,
  BuildReport,
  BuildError,
  ApkgArtifact,
  ArtifactError,
} from "../dist/index.mjs";

async function directory(t) {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "anki-forge-artifact-test-"),
  );
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  return root;
}

test("published reports own independently clonable artifacts without changing JSON snapshots", async (t) => {
  const root = await directory(t);
  const project = new Project("Artifacts", { baseDir: root });
  project.addNote(Note.basic("one", "1"));
  const report = await project.writeApkg("published.apkg");
  report.ensureSuccess();
  const artifact = report.artifactHandle;
  assert.ok(artifact, "a native build must return an artifact owner");
  assert.equal(artifact.path, report.artifact.path);
  assert.equal(report.artifactHandle, artifact);
  assert.deepEqual(report.raw.artifact, { path: artifact.path });
  const detached = new BuildReport(
    JSON.parse(JSON.stringify(report.raw)),
    report.prettyReport(),
  );
  assert.equal(
    detached.artifactHandle,
    null,
    "JSON cannot create native ownership",
  );
  const clone = artifact.clone();
  await artifact.close();
  await artifact.close();
  const saved = await clone.persistTo("copy.apkg");
  assert.equal(saved.path, path.join(root, "copy.apkg"));
  assert.deepEqual(
    await fs.readFile(saved.path),
    await fs.readFile(clone.path),
  );
  await clone.close();
  await saved.close();
  assert.ok((await fs.stat(report.artifact.path)).isFile());
  assert.ok((await fs.stat(saved.path)).isFile());
  assert.throws(() => artifact.clone(), { name: "ArtifactClosedError" });
  await assert.rejects(artifact.persistTo("closed.apkg"), {
    name: "ArtifactClosedError",
  });
});

test("temporary builds stay readable until their last owner closes", async (t) => {
  const root = await directory(t);
  const project = new Project("Temporary", { baseDir: root });
  project.addNote(Note.basic("one", "1"));
  const deck = new Deck("Temporary Deck", { baseDir: root });
  deck.basic("one", "1");
  for (const source of [project, deck]) {
    const report = await source.build();
    report.ensureSuccess();
    const owner = report.artifactHandle;
    const clone = owner.clone();
    const bytes = await fs.readFile(owner.path);
    const detached = new BuildReport(report.raw, report.prettyReport());
    await owner.close();
    assert.deepEqual(await fs.readFile(clone.path), bytes);
    const saved = await clone.persistTo("saved.apkg");
    await clone.close();
    await assert.rejects(fs.stat(detached.artifact.path), { code: "ENOENT" });
    assert.deepEqual(await fs.readFile(saved.path), bytes);
    await saved.close();
    const again = await source.build({});
    again.ensureSuccess();
    await again.artifactHandle.close();
  }
});

test("an artifact cannot be constructed from an arbitrary path or forged handle", () => {
  assert.throws(
    () => new ApkgArtifact({ path: "/tmp/not-an-owned-artifact" }, "/tmp"),
    TypeError,
  );
});

test("artifacts-only builds remain persistent and temporary persistence failures preserve the source", async (t) => {
  const root = await directory(t);
  const project = new Project("Modes", { baseDir: root });
  project.addNote(Note.basic("one", "1"));
  const retained = await project.build({ artifactsDir: "retained" });
  retained.ensureSuccess();
  assert.equal(
    retained.artifact.path,
    path.join(root, "retained", "package.apkg"),
  );
  await retained.artifactHandle.close();
  assert.ok((await fs.stat(retained.artifact.path)).isFile());

  const temporary = await project.build();
  const owner = temporary.artifactHandle;
  t.after(() => owner.close());
  const bytes = await fs.readFile(owner.path);
  await assert.rejects(owner.persistTo(root), ArtifactError);
  await assert.rejects(owner.persistTo(owner.path), ArtifactError);
  const alias = path.join(root, "alias.apkg");
  await fs.link(owner.path, alias);
  await assert.rejects(owner.persistTo(alias), ArtifactError);
  assert.deepEqual(await fs.readFile(owner.path), bytes);
  const save = owner.persistTo("in-flight.apkg");
  await owner.close();
  const saved = await save;
  assert.deepEqual(await fs.readFile(saved.path), bytes);
  await saved.close();
  await assert.rejects(fs.stat(owner.path), { code: "ENOENT" });
});

test("late build failures retain temporary artifacts in their error reports", async (t) => {
  const root = await directory(t);
  const project = new Project("Failed publication", {
    baseDir: root,
    stableId: "failed-publication",
  });
  project.addNote(Note.basic("one", "1", { stableId: "one" }));
  let failure;
  await assert.rejects(
    project.build({
      identityLockfile: root,
      writeIdentityLockfile: true,
      updateSafety: "disabled",
    }),
    (error) => {
      failure = error;
      return (
        error instanceof BuildError &&
        error.report.diagnosticCodes.includes("UPDATE.LOCKFILE_WRITE_FAILED")
      );
    },
  );
  const artifact = failure.report.artifactHandle;
  assert.ok(artifact);
  assert.ok((await fs.stat(artifact.path)).isFile());
  const independent = artifact.clone();
  await artifact.close();
  assert.ok((await fs.stat(independent.path)).isFile());
  await independent.close();
  await assert.rejects(fs.stat(artifact.path), { code: "ENOENT" });
  const recovered = await project.build();
  recovered.ensureSuccess();
  await recovered.artifactHandle.close();
});

test("JSON reporting without a persistent destination keeps the core failure and no artifact owner", async (t) => {
  const root = await directory(t);
  const project = new Project("Report", { baseDir: root });
  project.addNote(Note.basic("one", "1"));
  await assert.rejects(
    project.build({ reportJson: "report.json" }),
    (error) => {
      assert.ok(error instanceof BuildError);
      assert.ok(
        error.report.diagnosticCodes.includes(
          "PROJECT.REPORT_JSON_WRITE_FAILED",
        ),
      );
      assert.equal(error.report.artifactHandle, null);
      return true;
    },
  );
  const json = JSON.parse(
    await fs.readFile(path.join(root, "report.json"), "utf8"),
  );
  assert.equal(json.artifact, null);
});

test("artifact persistence captures baseDir across chdir and rejects symbolic aliases", async (t) => {
  const root = await directory(t);
  const project = new Project("Unicode", { baseDir: root });
  project.addNote(Note.basic("one", "1"));
  const report = await project.build();
  const owner = report.artifactHandle;
  t.after(() => owner.close());
  if (process.platform !== "win32") {
    const alias = path.join(root, "symbolic.apkg");
    await fs.symlink(owner.path, alias);
    await assert.rejects(owner.persistTo(alias), ArtifactError);
  }
  const previous = process.cwd();
  try {
    process.chdir(os.tmpdir());
    const saved = await owner.persistTo("副本 with spaces.apkg");
    assert.equal(saved.path, path.join(root, "副本 with spaces.apkg"));
    await saved.close();
    assert.ok((await fs.stat(saved.path)).isFile());
  } finally {
    process.chdir(previous);
  }
  assert.throws(() => ApkgArtifact.prototype.clone.call({}), TypeError);
  await assert.rejects(ApkgArtifact.prototype.close.call({}), TypeError);
});

test("reports outlive collected projects, GC releases temporary artifacts and worker teardown releases owners", async (t) => {
  const { execFile } = await import("node:child_process");
  const { promisify } = await import("node:util");
  const root = await directory(t);
  const entry = new URL("../dist/index.mjs", import.meta.url).href;
  const script = `
    import { Project, Note } from ${JSON.stringify(entry)};
    import fs from 'node:fs/promises';
    import assert from 'node:assert/strict';
    import { Worker } from 'node:worker_threads';
    const collect = async () => { for(let i=0;i<15;i++) { global.gc(); await new Promise(r=>setTimeout(r,10)); } };
    let project = new Project('GC'); project.addNote(Note.basic('one','1'));
    let report = await project.build(); const temporary = report.artifact.path;
    project = null; await collect(); await fs.access(temporary);
    report = null; await collect();
    await assert.rejects(fs.access(temporary), { code: 'ENOENT' });
    const worker = new Worker(\`
      const {parentPort}=require('node:worker_threads');
      (async()=>{ const {Project,Note}=await import(${JSON.stringify(entry)}); const p=new Project('Worker'); p.addNote(Note.basic('one','1')); global.owner=await p.build(); parentPort.postMessage(global.owner.artifact.path); })();
      setInterval(()=>{},1000);
    \`, {eval:true, execArgv: []});
    const filename = await new Promise((resolve,reject)=>{ worker.once('message',resolve); worker.once('error',reject); });
    await fs.access(filename); await worker.terminate();
    await collect(); await assert.rejects(fs.access(filename), {code:'ENOENT'});
  `;
  await promisify(execFile)(
    process.execPath,
    ["--expose-gc", "--input-type=module", "-e", script],
    {
      env: { ...process.env, TMPDIR: root, TMP: root, TEMP: root },
      timeout: 30000,
    },
  );
});

test("Worker termination before build and persist promises settle releases task-owned temporary artifacts", async (t) => {
  const { execFile } = await import("node:child_process");
  const { promisify } = await import("node:util");
  const root = await directory(t);
  const entry = new URL("../dist/index.mjs", import.meta.url).href;
  const script = `
    import fs from 'node:fs/promises';
    import os from 'node:os';
    import path from 'node:path';
    import assert from 'node:assert/strict';
    import { createHash } from 'node:crypto';
    import { Worker } from 'node:worker_threads';
    const until = async predicate => {
      const deadline = Date.now() + 20000;
      while (!(await predicate())) {
        assert.ok(Date.now() < deadline, 'native task did not clean up in time');
        await new Promise(resolve => setTimeout(resolve, 20));
      }
    };
    const queued = worker => new Promise((resolve,reject) => { worker.once('message',resolve); worker.once('error',reject); });
    const building = new Worker(\`
      const {parentPort}=require('node:worker_threads');
      (async()=>{
        const {Project,Note}=await import(${JSON.stringify(entry)});
        const p=new Project('Pending build');
        for(let i=0;i<1000;i++) p.addNote(Note.basic('front-'+i,'back'.repeat(4096),{stableId:'n'+i}));
        p.build(); parentPort.postMessage('queued');
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0);
      })();
    \`, {eval:true,execArgv:[]});
    await queued(building);
    // Witness native filesystem work before teardown, not merely an empty tempdir.
    await until(async () => (await fs.readdir(os.tmpdir())).length > 0);
    await building.terminate();
    await until(async () => (await fs.readdir(os.tmpdir())).length === 0);
    const destination=path.join(os.tmpdir(),'persistent.apkg');
    const persisting = new Worker(\`
      const {parentPort}=require('node:worker_threads');
      (async()=>{
        const {Project,Note}=await import(${JSON.stringify(entry)});
        const fs=require('node:fs/promises'), crypto=require('node:crypto');
        const p=new Project('Pending copy'); p.addNote(Note.basic('front','answer'));
        const report=await p.build();
        const hash=crypto.createHash('sha256').update(await fs.readFile(report.artifact.path)).digest('hex');
        report.artifactHandle.persistTo(${JSON.stringify(path.join(root, "persistent.apkg"))});
        parentPort.postMessage({source:report.artifact.path,hash});
        // Keep the native result queued even if copying finishes before termination.
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0);
      })();
    \`, {eval:true,execArgv:[]});
    const {source,hash}=await queued(persisting);
    await persisting.terminate();
    await until(async () => {
      const entries=await fs.readdir(os.tmpdir());
      return entries.length===1 && entries[0]==='persistent.apkg';
    });
    await assert.rejects(fs.access(source),{code:'ENOENT'});
    assert.equal(createHash('sha256').update(await fs.readFile(destination)).digest('hex'),hash);
  `;
  await promisify(execFile)(
    process.execPath,
    ["--input-type=module", "-e", script],
    {
      env: { ...process.env, TMPDIR: root, TMP: root, TEMP: root },
      timeout: 50000,
    },
  );
});

test("writeApkg still requires a filename when build output is optional", async () => {
  const project = new Project("Required filename");
  project.addNote(Note.basic("one", "1"));
  for (const output of [undefined, null, 123])
    await assert.rejects(async () => project.writeApkg(output), TypeError);
  const report = await project.build();
  report.ensureSuccess();
  await report.artifactHandle.close();
});
