import test from "node:test";
import assert from "node:assert/strict";
import { randomBytes, createHash } from "node:crypto";
import { Writable } from "node:stream";
import { Project, Note } from "../dist/index.mjs";

test("Writable export transfers bounded chunks with backpressure and leaves the target open", async () => {
  const project = new Project("Bounded export", { stableId: "bounded-export" });
  const samples = randomBytes(256 * 1024);
  const header = Buffer.from(
    "UklGRiQAAABXQVZFZm10IBAAAAABAAEAESsAACJWAAACABAAZGF0YQAAAAA=",
    "base64",
  );
  header.writeUInt32LE(samples.length + 36, 4);
  header.writeUInt32LE(samples.length, 40);
  const media = await project.media.addBuffer(
    "noise.wav",
    Buffer.concat([header, samples]),
  );
  project.addNote(
    Note.basic("front", "back", { stableId: "one" }).sound("Back", media),
  );
  const expected = createHash("sha256")
    .update(await project.toApkgBuffer())
    .digest("hex");
  const actual = createHash("sha256");
  const sizes = [];
  const target = new Writable({
    highWaterMark: 1024,
    write(chunk, encoding, callback) {
      sizes.push(chunk.length);
      actual.update(chunk);
      setTimeout(callback, 1);
    },
  });
  await project.writeTo(target);
  assert.ok(
    sizes.length > 1,
    "the complete archive must not be sent as one Buffer",
  );
  assert.ok(sizes.every((size) => size <= 64 * 1024));
  assert.equal(actual.digest("hex"), expected);
  assert.equal(target.writableEnded, false);
  assert.equal(target.listenerCount("error"), 0);
  assert.equal(target.listenerCount("close"), 0);
  target.end();
});

test("Writable failures at generation and transfer release temporary files and listeners", async (t) => {
  const fs = await import("node:fs/promises");
  const os = await import("node:os");
  const path = await import("node:path");
  const { execFile } = await import("node:child_process");
  const { promisify } = await import("node:util");
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "anki-forge-stream-cleanup-"),
  );
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const entry = new URL("../dist/index.mjs", import.meta.url).href;
  const script = `
    import { Project, Note } from ${JSON.stringify(entry)};
    import { Writable } from 'node:stream';
    import assert from 'node:assert/strict';
    import fs from 'node:fs/promises';
    import os from 'node:os';
    const project = new Project('Failing streams'); project.addNote(Note.basic('front','answer'));
    for (const mode of ['generation-error', 'generation-close', 'callback-error', 'write-close', 'write-throw']) {
      const failure = new Error(mode);
      const stream = new Writable({ highWaterMark: 1, write(chunk, encoding, done) {
        if (mode === 'write-throw') throw failure;
        if (mode === 'write-close') { this.destroy(); return; }
        if (mode === 'callback-error') { done(failure); return; }
        done();
      }});
      const writing = project.writeTo(stream);
      if (mode === 'generation-error') stream.destroy(failure);
      if (mode === 'generation-close') stream.destroy();
      await assert.rejects(writing, error => mode.includes('close') ? /closed/.test(error.message) : error === failure);
      await new Promise(resolve=>setImmediate(resolve));
      assert.equal(stream.listenerCount('error'), 0);
      assert.equal(stream.listenerCount('close'), 0);
      assert.equal(stream.listenerCount('drain'), 0);
      assert.deepEqual(await fs.readdir(os.tmpdir()), []);
    }
    const report = await project.build(); report.ensureSuccess(); await report.artifactHandle.close();
  `;
  await promisify(execFile)(
    process.execPath,
    ["--input-type=module", "-e", script],
    {
      env: { ...process.env, TMPDIR: root, TMP: root, TEMP: root },
      timeout: 30000,
    },
  );
});
