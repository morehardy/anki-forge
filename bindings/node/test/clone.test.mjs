import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { Deck, Project, Note, ProjectBusyError } from "../dist/index.mjs";

test("Project and Deck clones preserve identity and can mutate and build independently", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "anki-forge-clone-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  for (const source of [
    new Project("Variants", { baseDir: root, stableId: "variants" }),
    new Deck("Variants", { baseDir: root, stableId: "variants" }),
  ]) {
    const add = (target, id) =>
      target instanceof Deck
        ? target.basic(id, "back", { stableId: id })
        : target.addNote(Note.basic(id, "back", { stableId: id }));
    add(source, "one");
    const copying = source.clone();
    assert.throws(() => add(source, "busy"), ProjectBusyError);
    const clone = await copying;
    assert.equal(clone.constructor, source.constructor);
    assert.equal(clone.baseDir, root);
    assert.equal(clone.name, source.name);
    assert.deepEqual(await source.toApkgBuffer(), await clone.toApkgBuffer());
    add(clone, "two");
    const building = clone.build();
    add(source, "three");
    add(source, "four");
    const [a, b] = await Promise.all([building, source.build()]);
    assert.equal(a.counts.notes, 2);
    assert.equal(b.counts.notes, 3);
    await a.artifactHandle.close();
    await b.artifactHandle.close();
  }
});

test("cloned large Buffer media survives collection of its original Project and cleans up after the last owner", async (t) => {
  const { execFile } = await import("node:child_process");
  const { promisify } = await import("node:util");
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "anki-forge-clone-gc-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const entry = new URL("../dist/index.mjs", import.meta.url).href;
  const script = `
    import { Project, Note } from ${JSON.stringify(entry)};
    import assert from 'node:assert/strict';
    import fs from 'node:fs/promises';
    import os from 'node:os';
    const collect = async () => { for(let i=0;i<10;i++) { global.gc(); await new Promise(r=>setTimeout(r,10)); } };
    let original = new Project('Large');
    const bytes = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"><!--' + 'x'.repeat(2*1024*1024) + '--></svg>');
    const ref = await original.media.addBuffer('large.svg', bytes);
    original.addNote(Note.basic('large', 'answer').image('Front', ref));
    let clone = await original.clone();
    assert.equal((await fs.readdir(os.tmpdir())).filter(n=>n.startsWith('anki-forge-node-media-')).length,1);
    original = null;
    await collect();
    for(let i=0;i<2;i++) { const report=await clone.build(); assert.equal(report.counts.notes,1); await report.artifactHandle.close(); }
    clone = null;
    await collect();
    assert.equal((await fs.readdir(os.tmpdir())).filter(n=>n.startsWith('anki-forge-node-media-')).length,0);
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
