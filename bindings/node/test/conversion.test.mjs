import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { Deck, Project, Note, ProjectBusyError } from "../dist/index.mjs";

test("Deck snapshots become independently editable Projects while retaining the original Deck", async (t) => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "anki-forge-conversion-"),
  );
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const deck = new Deck("Imported", { baseDir: root, stableId: "imported" });
  deck.basic("<b>front</b>", "<i>back</i>", { stableId: "one" });
  const copying = Project.fromDeck(deck);
  assert.throws(() => deck.basic("busy", "note"), ProjectBusyError);
  const project = await copying;
  assert.equal(project.name, "Imported");
  assert.equal(project.baseDir, root);
  project.addNote(Note.cloze("{{c1::two}}", { stableId: "two" }));
  const imported = await project.writeApkg("imported.apkg");
  const original = await deck.writeApkg("original.apkg");
  assert.equal(imported.counts.notes, 2);
  assert.equal(original.counts.notes, 1);
  deck.basic("three", "3", { stableId: "three" });
  const modified = await deck.writeApkg("modified.apkg");
  assert.equal(modified.counts.notes, 2);
  for (const report of [imported, original, modified])
    await report.artifactHandle.close();
});

test("Deck media lookup and conversion retain registration fingerprints without reading the source again", async (t) => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "anki-forge-deck-media-"),
  );
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const source = path.join(root, "icon.svg");
  await fs.writeFile(source, '<svg xmlns="http://www.w3.org/2000/svg"/>');
  const deck = new Deck("Media", { baseDir: root });
  const registered = await deck.media.addFile("icon.svg");
  assert.equal(
    deck.media.get(registered.filename).filename,
    registered.filename,
  );
  assert.equal(deck.media.get("unknown.svg"), undefined);
  deck.basic(`<img src="${registered.filename}">`, "answer", {
    stableId: "image",
  });
  const baseline = await deck.build();
  await fs.writeFile(
    source,
    '<svg xmlns="http://www.w3.org/2000/svg"><path/></svg>',
  );
  // Both operations must preserve the original fingerprint, even after source mutation.
  assert.equal(
    deck.media.get(registered.filename).filename,
    registered.filename,
  );
  const copying = Project.fromDeck(deck);
  assert.throws(() => deck.media.get(registered.filename), ProjectBusyError);
  const project = await copying;
  for (const object of [project, deck]) {
    await assert.rejects(object.build(), (error) =>
      error.report.diagnosticCodes.includes("MEDIA.SOURCE_CHANGED"),
    );
  }
  await baseline.artifactHandle.close();
});
