import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import {
  Project,
  Deck,
  Note,
  Field,
  Template,
  NoteType,
  GenerationRule,
  IdentityRecipe,
  BuildError,
  ProjectBusyError,
  firstUpdateSafeBuild,
  updateSafe,
} from "../dist/index.mjs";

const svg = Buffer.from(
  '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"/>',
);
function observe(operation, filename) {
  const result = spawnSync(
    process.env.ANKI_FORGE_TEST_OBSERVER,
    [operation, filename],
    { encoding: "utf8" },
  );
  if (result.error) throw result.error;
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
async function directory(t) {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), "anki-forge-parity-"));
  t.after(() => fs.rm(dir, { recursive: true, force: true }));
  return dir;
}
function custom(baseDir, fieldName = "Expression", reverse = false) {
  const p = new Project("Parity", { baseDir, stableId: "parity" });
  const templates = [
    new Template("Recognition", {
      key: "recognition",
      front: `{{${fieldName}}}`,
      back: "{{FrontSide}}<hr>{{Meaning}}",
      browserFront: `{{${fieldName}}}`,
      browserBack: "{{Meaning}}",
      targetDeck: "Parity::Target",
      generateWhen: GenerationRule.all(["expr"]),
    }),
    new Template("Reverse", {
      key: "reverse",
      front: "{{Meaning}}",
      back: `{{${fieldName}}}`,
      generateWhen: GenerationRule.any(["meaning"]),
    }),
  ];
  if (reverse) templates.reverse();
  p.addNoteType(
    NoteType.custom("vocabulary", {
      name: "Vocabulary",
      fields: [
        new Field(fieldName, { key: "expr", identity: true, required: true }),
        new Field("Meaning", { key: "meaning", sort: true, optional: true }),
      ],
      identity: IdentityRecipe.fields(["expr"]),
      css: ".card { color: navy; }",
      templates,
    }),
  );
  p.addNote(
    Note.custom("vocabulary")
      .text("expr", "<hola>")
      .html("meaning", "<b>hello</b>"),
  );
  return p;
}
function revision(baseDir, answer = "A", tags = []) {
  const p = new Project("Parity", { baseDir, stableId: "parity" });
  p.addNote(Note.basic("Question", answer, { stableId: "changed", tags }));
  p.addNote(
    Note.basic("Unchanged question", "Unchanged answer", {
      stableId: "unchanged",
    }),
  );
  return p;
}
async function project(caseName, baseDir) {
  const p = new Project("Parity", {
    baseDir,
    stableId: "parity",
    ...(caseName === "stock" ? { defaultDeck: "Parity::Default" } : {}),
  });
  switch (caseName) {
    case "stock":
      p.addNote(
        Note.basic("<hola>", "hello", {
          stableId: "hello",
          tags: ["language"],
        }),
      );
      p.addNote(
        Note.cloze("{{c1::one}} {{c2::two}}", {
          stableId: "numbers",
          backExtra: "extra",
          deckName: "Parity::Cloze",
        }),
      );
      break;
    case "normal":
    case "renamed":
    case "reordered":
      return custom(
        baseDir,
        caseName === "normal" ? "Expression" : "Prompt",
        caseName === "reordered",
      );
    case "custom-cloze":
      p.addNoteType(
        NoteType.customCloze("custom-cloze", "text", {
          fields: [
            new Field("Sentence", { key: "text", identity: true }),
            new Field("Extra", { key: "extra", optional: true }),
          ],
          templates: [
            new Template("Cloze", {
              front: "{{cloze:Sentence}}",
              back: "{{cloze:Sentence}} {{Extra}}",
              generateWhen: GenerationRule.cloze("text"),
            }),
          ],
        }),
      );
      p.addNote(
        Note.custom("custom-cloze", { stableId: "sentence" })
          .html("text", "{{c1::one}} {{c2::two}}")
          .text("extra", "extra"),
      );
      break;
    case "media":
    case "io": {
      const image = await p.media.addBytes("diagram.svg", svg);
      if (caseName === "io")
        p.addNote(
          Note.imageOcclusion(image, {
            stableId: "diagram",
            rects: [
              { x: 1, y: 2, width: 10, height: 20 },
              { x: 30, y: 40, width: 10, height: 20 },
            ],
            header: "Header",
            backExtra: "Extra",
            comments: "Comments",
            tags: ["image"],
          }),
        );
      else {
        const audio = await p.media.addBytes(
          "voice.mp3",
          Buffer.from("ID3\x04\0\0\0\0\0\0audio"),
        );
        p.addNote(
          Note.basic("", "", { stableId: "media" })
            .image("Front", image)
            .sound("Back", audio),
        );
      }
      break;
    }
    case "deck": {
      const deck = new Deck("Parity", { baseDir, stableId: "parity" });
      deck.basic("<hola>", "hello", { stableId: "hello", tags: ["language"] });
      deck.cloze("{{c1::one}} {{c2::two}}", {
        stableId: "numbers",
        extra: "extra",
      });
      return deck;
    }
    case "bundle":
      await p.importTemplateBundle(
        fileURLToPath(
          new URL(
            "../../../contracts/fixtures/template-bundle/custom-normal",
            import.meta.url,
          ),
        ),
      );
      p.addNote(
        Note.custom("language-card", { stableId: "bundle" })
          .text("prompt", "Question")
          .text("extra", "Answer"),
      );
      break;
    default:
      if (caseName.startsWith("revision-"))
        return revision(
          baseDir,
          caseName === "revision-1" ? "B" : "A",
          ["revision-3", "revision-4"].includes(caseName) ? ["new-tag"] : [],
        );
      throw new Error(`Unknown scenario ${caseName}`);
  }
  return p;
}
// Only execution time and the caller-selected test directory are non-semantic.
function normalize(value, root) {
  return JSON.parse(
    JSON.stringify(value, (key, item) => {
      if (key === "duration_ms") return 0;
      if (typeof item === "string") return item.replaceAll(root, "<root>");
      return item;
    }),
  );
}
function notes(inspected) {
  return Object.fromEntries(
    inspected.observations.references
      .filter((item) => item.fields)
      .map((item) => [item.id, item]),
  );
}

test("independent Rust and Node product constructors preserve full APKG observations and reports", async (t) => {
  let root;
  if (process.env.ANKI_FORGE_NODE_EVIDENCE_DIR) {
    await fs.mkdir(process.env.ANKI_FORGE_NODE_EVIDENCE_DIR, {
      recursive: true,
    });
    root = await fs.mkdtemp(
      path.join(process.env.ANKI_FORGE_NODE_EVIDENCE_DIR, "scenarios-"),
    );
  } else root = await directory(t);
  const rustDir = path.join(root, "rust"),
    nodeDir = path.join(root, "node");
  await fs.mkdir(nodeDir);
  const expected = observe("suite", rustDir),
    actual = {};
  for (const [caseName, rust] of Object.entries(expected)) {
    // serde_json's map is sorted; build dependencies have an explicit order below.
    assert.ok(rust.report, caseName);
  }
  for (const caseName of [
    "stock",
    "normal",
    "renamed",
    "reordered",
    "custom-cloze",
    "media",
    "io",
    "deck",
    "bundle",
    "revision-0",
    "revision-1",
    "revision-2",
    "revision-3",
    "revision-4",
  ]) {
    const p = await project(caseName, nodeDir);
    let options = {};
    if (caseName === "renamed") options = { compareTo: "normal.apkg" };
    if (caseName === "reordered") options = { compareTo: "renamed.apkg" };
    if (caseName === "revision-0")
      options = firstUpdateSafeBuild("identity.json");
    if (caseName.startsWith("revision-") && caseName !== "revision-0")
      options = { ...updateSafe("identity.json"), writeIdentityLockfile: true };
    const report = await p.writeApkg(`${caseName}.apkg`, options);
    report.ensureSuccess();
    const inspected = observe(
      "inspect",
      path.join(nodeDir, `${caseName}.apkg`),
    );
    actual[caseName] = { report: report.raw, inspect: inspected };
    assert.deepEqual(
      normalize(actual[caseName], nodeDir),
      normalize(expected[caseName], rustDir),
      caseName,
    );
  }
  const history = [0, 1, 2, 3, 4].map((index) =>
    notes(actual[`revision-${index}`].inspect),
  );
  for (let index = 1; index < history.length; index++) {
    assert.equal(
      history[index].changed.revision.mtime_secs,
      history[index - 1].changed.revision.mtime_secs + (index === 4 ? 0 : 1),
    );
    assert.deepEqual(
      history[index].unchanged.revision,
      history[0].unchanged.revision,
    );
    assert.deepEqual(
      actual[`revision-${index}`].inspect.identity.notes.map(
        (item) => item.anki_guid,
      ),
      actual["revision-0"].inspect.identity.notes.map((item) => item.anki_guid),
    );
  }
  assert.equal(
    history[2].changed.revision.content_hash,
    history[0].changed.revision.content_hash,
  );
  assert.ok(
    actual.renamed.report.diagnostics.some(
      (item) => item.code === "UPDATE.FIELD_RENAMED",
    ),
  );
  assert.ok(
    actual.reordered.report.diagnostics.some(
      (item) => item.code === "UPDATE.TEMPLATE_ORD_CHANGED",
    ),
  );
  for (const template of actual.renamed.inspect.observations.templates) {
    const changed = actual.reordered.inspect.observations.templates.find(
      (item) => item.name === template.name,
    );
    assert.equal(changed.config_id, template.config_id);
    assert.equal(changed.ord, 1 - template.ord);
  }
  if (process.env.ANKI_FORGE_NODE_EVIDENCE_DIR) {
    const manifest = [];
    for (const caseName of Object.keys(actual)) {
      const file = path.join(nodeDir, `${caseName}.apkg`);
      manifest.push({
        case: caseName,
        file,
        sha256: createHash("sha256")
          .update(await fs.readFile(file))
          .digest("hex"),
        counts: actual[caseName].report.counts,
        desktop: "pending",
      });
    }
    await fs.writeFile(
      path.join(root, "evidence.json"),
      JSON.stringify(
        {
          node: process.version,
          platform: process.platform,
          arch: process.arch,
          rust: expected,
          sdk: actual,
          artifacts: manifest,
        },
        null,
        2,
      ),
    );
    await fs.writeFile(
      path.join(root, "DESKTOP-CHECK.md"),
      `# Node SDK Desktop verification\n\nAutomated Rust/Node semantic comparisons passed. Desktop import and rendering are pending.\n\nAnki version: pending\nPlatform: pending\nReviewer/date: pending\n\nUse a disposable Anki profile.\n\n- Import stock, custom-cloze, media, io, deck, and bundle APKGs into separate empty profiles. Verify note/card counts against evidence.json and front/back rendering, audio, image, and browser appearance.\n- Import normal, renamed, and reordered into the same profile in that order. Verify field/template changes and preservation of existing notes and scheduling.\n- Import revision-0 through revision-4 in order into one profile. Review a card before upgrading; verify answer changes, content revert, tag changes, no duplicate notes, and preserved review history.\n- The hide-one-guess-one core failure is still pending and is not covered by the successful io package.\n\nRecord results and screenshots here. Do not mark Desktop validation complete from inspect results alone.\n`,
    );
    t.diagnostic(`Reviewable Desktop packages and evidence: ${root}`);
  }
});

test("legacy lockfiles block strict updates and migrate only with APKG evidence", async (t) => {
  const baseDir = await directory(t),
    lock = path.join(baseDir, "identity.json");
  const first = await revision(baseDir).writeApkg(
    "first.apkg",
    firstUpdateSafeBuild("identity.json"),
  );
  first.ensureSuccess();
  const original = JSON.parse(await fs.readFile(lock, "utf8"));
  for (const kind of ["revision", "model"]) {
    const legacy = structuredClone(original);
    if (kind === "revision")
      for (const note of legacy.identity_index.notes) delete note.revision;
    else
      for (const type of legacy.identity_index.notetypes)
        type.anki_model_id = null;
    const bytes = JSON.stringify(legacy);
    await fs.writeFile(lock, bytes);
    const code =
      kind === "revision"
        ? "UPDATE.NOTE_REVISION_MISSING"
        : "UPDATE.NOTETYPE_MODEL_ID_MISSING";
    await assert.rejects(
      revision(baseDir, "B").writeApkg("blocked.apkg", {
        ...updateSafe("identity.json"),
        writeIdentityLockfile: true,
      }),
      (error) =>
        error instanceof BuildError &&
        error.report.diagnosticCodes.includes(code),
    );
    assert.equal(await fs.readFile(lock, "utf8"), bytes);
    await assert.rejects(fs.access(path.join(baseDir, "blocked.apkg")));
    const warned = await revision(baseDir, "B").writeApkg("warned.apkg", {
      identityLockfile: "identity.json",
      updateSafety: "report-only",
      writeIdentityLockfile: true,
    });
    warned.ensureSuccess();
    assert.ok(
      warned.diagnostics.some(
        (item) => item.code === code && item.severity === "warning",
      ),
    );
    assert.equal(warned.risk.highest_level, "high");
    assert.equal(warned.updateSafety.lockfile_written, false);
    assert.equal(await fs.readFile(lock, "utf8"), bytes);
    const migrated = await revision(baseDir, "B").writeApkg("migrated.apkg", {
      ...updateSafe("identity.json"),
      compareTo: "first.apkg",
      writeIdentityLockfile: true,
    });
    migrated.ensureSuccess();
    assert.equal(migrated.updateSafety.lockfile_written, true);
    assert.equal(
      notes(observe("inspect", path.join(baseDir, "migrated.apkg"))).changed
        .revision.mtime_secs,
      notes(observe("inspect", path.join(baseDir, "first.apkg"))).changed
        .revision.mtime_secs + 1,
    );
  }
});

test("64-bit baseline model IDs and existing GUIDs survive native builds without JS rounding", async (t) => {
  const baseDir = await directory(t),
    lock = path.join(baseDir, "identity.json");
  (
    await revision(baseDir).writeApkg(
      "first.apkg",
      firstUpdateSafeBuild("identity.json"),
    )
  ).ensureSuccess();
  const value = JSON.parse(await fs.readFile(lock, "utf8"));
  value.identity_index.notetypes[0].anki_model_id = "__large_model_id__";
  const note = value.identity_index.notes.find(
    (item) => item.stable_id === "changed",
  );
  assert.ok(note);
  note.anki_guid = "legacy-guid";
  await fs.writeFile(
    lock,
    JSON.stringify(value).replace('"__large_model_id__"', "9007199254740993"),
  );
  const report = await revision(baseDir, "B").writeApkg(
    "large.apkg",
    updateSafe("identity.json"),
  );
  report.ensureSuccess();
  const inspected = observe("inspect", path.join(baseDir, "large.apkg"));
  assert.equal(
    inspected.identity.notes.find((item) => item.stable_id === "changed")
      .anki_guid,
    "legacy-guid",
  );
  assert.equal(
    inspected.identity.notetypes[0].anki_model_id,
    "9007199254740993",
  );
  assert.equal(report.updateSafety.notes_preserved, 2);
});

test("corrupt baselines retain partial reports and protect existing publications", async (t) => {
  const baseDir = await directory(t),
    p = revision(baseDir);
  await fs.writeFile(path.join(baseDir, "corrupt.apkg"), "not a ZIP");
  await fs.writeFile(path.join(baseDir, "published.apkg"), "old publication");
  await assert.rejects(
    p.writeApkg("published.apkg", { compareTo: "corrupt.apkg" }),
    (error) => {
      assert.ok(error instanceof BuildError);
      assert.equal(error.report.artifact, null);
      assert.ok(
        error.report.diagnosticCodes.includes(
          "UPDATE.BASELINE_APKG_UNREADABLE",
        ),
      );
      assert.ok(
        error.report.updateSafety.baseline_sources.some(
          (item) => item.status !== "loaded",
        ),
      );
      assert.ok(error.failureCause);
      return true;
    },
  );
  assert.equal(
    await fs.readFile(path.join(baseDir, "published.apkg"), "utf8"),
    "old publication",
  );
  assert.equal(
    await fs.readFile(path.join(baseDir, "corrupt.apkg"), "utf8"),
    "not a ZIP",
  );
  (await p.writeApkg("recovered.apkg")).ensureSuccess();
});

test("path aliases cannot overwrite APKG baselines, outputs or identity lockfiles", async (t) => {
  const baseDir = await directory(t),
    p = revision(baseDir);
  (
    await p.writeApkg("baseline.apkg", firstUpdateSafeBuild("identity.json"))
  ).ensureSuccess();
  const baseline = path.join(baseDir, "baseline.apkg"),
    lock = path.join(baseDir, "identity.json");
  const before = await fs.readFile(baseline),
    lockBefore = await fs.readFile(lock);
  const links = ["hard"];
  if (process.platform !== "win32") links.push("symbolic");
  for (const kind of links) {
    const alias = path.join(baseDir, `${kind}.apkg`);
    if (kind === "hard") await fs.link(baseline, alias);
    else await fs.symlink(baseline, alias);
    for (const options of [
      { output: alias, compareTo: baseline },
      { output: "output.apkg", compareTo: baseline, reportJson: alias },
      {
        output: "output.apkg",
        compareTo: baseline,
        identityLockfile: alias,
        writeIdentityLockfile: true,
      },
    ])
      await assert.rejects(
        p.build(options),
        (error) =>
          error instanceof BuildError &&
          error.report.diagnosticCodes.includes("PROJECT.PATH_COLLISION"),
      );
    assert.deepEqual(await fs.readFile(baseline), before);
  }
  const lockAlias = path.join(baseDir, "lock-alias.json");
  await fs.link(lock, lockAlias);
  await assert.rejects(
    p.build({ output: lockAlias, ...updateSafe(lock) }),
    (error) =>
      error instanceof BuildError &&
      error.report.diagnosticCodes.includes("PROJECT.PATH_COLLISION"),
  );
  assert.deepEqual(await fs.readFile(lock), lockBefore);
});

test("independent native instances run concurrently and input limits reject unsafe integers", async (t) => {
  const baseDir = await directory(t),
    a = revision(baseDir),
    b = revision(baseDir, "B");
  const first = a.writeApkg("a.apkg"),
    second = b.writeApkg("b.apkg");
  assert.throws(() => a.addNote(Note.basic("busy", "state")), ProjectBusyError);
  for (const report of await Promise.all([first, second]))
    report.ensureSuccess();
  for (const value of [Number.MAX_SAFE_INTEGER + 1, -1, Infinity, NaN, 1.5]) {
    await assert.rejects(
      a.writeApkg("invalid.apkg", {
        inspectLimits: { maxArchiveBytes: value },
      }),
      TypeError,
    );
  }
  await assert.rejects(fs.access(path.join(baseDir, "invalid.apkg")));
});
