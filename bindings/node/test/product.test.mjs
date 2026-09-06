import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { Writable } from "node:stream";
import { Worker } from "node:worker_threads";
import { fileURLToPath } from "node:url";
import {
  Project,
  Note,
  BuildError,
  ProjectAddError,
  ProjectBusyError,
  bindingMetadata,
} from "../dist/index.mjs";
import {
  Content,
  NoteType,
  Field,
  Template,
  GenerationRule,
  IdentityRecipe,
  MediaError,
  ProductNoteError,
  TemplateBundleError,
  validateTemplate,
  defaultInspectLimits,
  firstUpdateSafeBuild,
  updateSafe,
} from "../dist/index.mjs";
import { Deck, DeckError } from "../dist/index.mjs";

async function directory(t) {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), "anki-forge-sdk-"));
  t.after(() => fs.rm(dir, { recursive: true, force: true }));
  return dir;
}

test("ESM and CJS share classes and native runtime metadata", () => {
  assert.equal(
    Project,
    createRequire(import.meta.url)("../dist/cjs/index.js").Project,
  );
  assert.equal(bindingMetadata().bindingVersion, "0.2.0");
  assert.equal(bindingMetadata().nodeApiVersion, 8);
});

test("Basic and Cloze produce an inspected APKG through Rust", async (t) => {
  const baseDir = await directory(t);
  const project = new Project("Languages", {
    baseDir,
    stableId: "languages",
    defaultDeck: "Languages::Spanish",
  });
  project.addNote(
    Note.basic("hola", "hello", { stableId: "hola", tags: ["spanish"] }),
  );
  project.addNote(
    Note.cloze("{{c1::uno}} {{c2::dos}}", {
      stableId: "count",
      backExtra: "numbers",
    }),
  );
  (await project.validate()).ensureSuccess();
  assert.deepEqual(
    await fs.readdir(baseDir),
    [],
    "validation must not publish or create a lockfile",
  );
  const report = await project.writeApkg("spanish.apkg", { inspect: true });
  report.ensureSuccess();
  assert.deepEqual(report.counts, { notes: 2, cards: 3, media: 0 });
  assert.equal(report.inspect.cards, 3);
  assert.equal(report.artifact.path, path.join(baseDir, "spanish.apkg"));
  assert.equal(
    (await fs.readFile(report.artifact.path)).subarray(0, 2).toString(),
    "PK",
  );
  assert.ok(report.prettyReport().includes("2"));
  assert.ok(Object.isFrozen(report.raw.counts));
});

test("add errors are synchronous and leave the project unchanged", async (t) => {
  const project = new Project("Atomic", { baseDir: await directory(t) });
  project.addNote(Note.basic("one", "1", { stableId: "one" }));
  assert.throws(
    () => project.addNote(Note.basic("duplicate", "2", { stableId: "one" })),
    (error) =>
      error instanceof ProjectAddError &&
      error.code === "AFID.STABLE_ID_DUPLICATE",
  );
  assert.throws(
    () => project.addNote(Note.basic("blank", "3", { stableId: "  " })),
    ProjectAddError,
  );
  const report = await project.writeApkg("atomic.apkg");
  report.ensureSuccess();
  assert.equal(report.counts.notes, 1);
});

test("operations reserve the project immediately and restore it after a failed build", async (t) => {
  const project = new Project("State", { baseDir: await directory(t) });
  project.addNote(Note.basic("one", "1"));
  const validation = project.validate();
  assert.throws(
    () => project.addNote(Note.basic("busy", "2")),
    ProjectBusyError,
  );
  await assert.rejects(project.validate(), ProjectBusyError);
  await validation;
  project.addNote(Note.basic("two", "2"));
  await assert.rejects(
    project.build({ output: "same.apkg", compareTo: "same.apkg" }),
    (error) =>
      error instanceof BuildError &&
      error.report.diagnosticCodes.includes("PROJECT.PATH_COLLISION"),
  );
  const report = await project.writeApkg("good.apkg");
  report.ensureSuccess();
  assert.equal(report.counts.notes, 2);
});

test("risk-blocked candidates preserve published APKG and identity lockfile", async (t) => {
  const baseDir = await directory(t);
  const baseline = new Project("Stable", { baseDir, stableId: "stable" });
  baseline.addNote(Note.basic("hello", "one", { stableId: "hello" }));
  (
    await baseline.writeApkg("baseline.apkg", {
      identityLockfile: "identity.json",
      writeIdentityLockfile: true,
    })
  ).ensureSuccess();
  const lockfile = await fs.readFile(path.join(baseDir, "identity.json"));
  const bytes = await fs.readFile(path.join(baseDir, "baseline.apkg"));
  await fs.writeFile(
    path.join(baseDir, "published.apkg"),
    "previous publication",
  );
  const changed = new Project("Stable", { baseDir, stableId: "stable" });
  changed.addNote(Note.basic("changed", "two", { stableId: "hello" }));
  await assert.rejects(
    changed.writeApkg("published.apkg", {
      compareTo: "baseline.apkg",
      failOn: "low",
      identityLockfile: "identity.json",
      writeIdentityLockfile: true,
    }),
    (error) =>
      error instanceof BuildError &&
      error.report.status === "blocked" &&
      !error.report.updateSafety.lockfile_written,
  );
  assert.deepEqual(
    await fs.readFile(path.join(baseDir, "identity.json")),
    lockfile,
  );
  assert.deepEqual(
    await fs.readFile(path.join(baseDir, "baseline.apkg")),
    bytes,
  );
  assert.equal(
    await fs.readFile(path.join(baseDir, "published.apkg"), "utf8"),
    "previous publication",
  );
});

test("runtime checks reject unknown options before entering Rust", async (t) => {
  const project = new Project("Options", { baseDir: await directory(t) });
  await assert.rejects(
    project.build({ output: "deck.apkg", timeout: 1 }),
    TypeError,
  );
  assert.throws(() => Note.basic("a", "b", { tags: "wrong" }), TypeError);
});

function vocabulary() {
  return NoteType.custom("vocabulary", {
    name: "Vocabulary",
    css: ".card { color: navy; }",
    fields: [
      new Field("Expression", { key: "expr", identity: true, required: true }),
      new Field("Meaning", { key: "meaning", sort: true, optional: true }),
      new Field("Audio", { key: "audio" }),
    ],
    identity: IdentityRecipe.fields(["expr"]),
    templates: [
      new Template("Recognition", {
        key: "recognition",
        front: "{{Expression}} {{Audio}}",
        back: "{{FrontSide}}<hr>{{Meaning}}",
        browserFront: "{{Expression}}",
        browserBack: "{{Meaning}}",
        targetDeck: "Vocabulary::Cards",
        generateWhen: GenerationRule.all(["expr"]),
      }),
    ],
  });
}

test("custom fields, templates, identity, content and generation use Rust authoring", async (t) => {
  const project = new Project("Custom", { baseDir: await directory(t) });
  project.addNoteType(vocabulary());
  assert.throws(
    () => project.addNoteType(vocabulary()),
    (error) =>
      error instanceof ProjectAddError &&
      error.code === "NOTETYPE.ID_DUPLICATE",
  );
  assert.throws(() => project.addNote(Note.custom("unknown")), ProjectAddError);
  const note = Note.custom("vocabulary", { stableId: "hello" })
    .text("expr", "<hello>")
    .html("meaning", "<b>world</b>");
  project.addNote(note);
  const report = await project.writeApkg("custom.apkg");
  report.ensureSuccess();
  assert.equal(report.inspect.notes, 1);
  assert.equal(report.inspect.cards, 1);
  assert.equal(Content.text("<b>&").render(), "&lt;b&gt;&amp;");
  assert.equal(Content.html("<b>&").render(), "<b>&");
  assert.ok(Object.isFrozen(note));
});

test("custom Cloze and semantic template diagnostics preserve byte offsets", async (t) => {
  const project = new Project("Custom cloze", { baseDir: await directory(t) });
  project.addNoteType(
    NoteType.customCloze("custom-cloze", "sentence", {
      fields: [
        new Field("Sentence", { key: "sentence", identity: true }),
        new Field("Extra", { key: "extra", optional: true }),
      ],
      templates: [
        new Template("Cloze", {
          front: "{{cloze:Sentence}}",
          back: "{{cloze:Sentence}} {{Extra}}",
          generateWhen: GenerationRule.cloze("sentence"),
        }),
      ],
    }),
  );
  project.addNote(
    Note.custom("custom-cloze", { stableId: "two" })
      .html("sentence", "{{c1::first}} {{c2::second}}")
      .text("extra", "hint"),
  );
  const report = await project.writeApkg("cloze.apkg");
  report.ensureSuccess();
  assert.equal(report.counts.cards, 2);
  const invalid = validateTemplate("你好 {{Missing}}", ["Sentence"]);
  assert.ok(invalid.hasErrors);
  assert.ok(invalid.diagnostics.some((item) => item.span?.byte_start >= 7));
});

test("media paths capture baseDir, preserve registration fingerprints and remain atomic", async (t) => {
  const baseDir = await directory(t);
  const source = path.join(baseDir, "声音.svg");
  await fs.writeFile(source, '<svg xmlns="http://www.w3.org/2000/svg"/>');
  const project = new Project("Media", { baseDir });
  const media = await project.media.addFile("声音.svg", {
    exportAs: "icon.svg",
  });
  assert.equal(media.image().render(), '<img src="icon.svg">');
  project.addNote(Note.basic("image", "answer").image("Front", media));
  (await project.writeApkg("valid.apkg")).ensureSuccess();
  await fs.writeFile(
    source,
    '<svg xmlns="http://www.w3.org/2000/svg"><path/></svg>',
  );
  await assert.rejects(
    project.writeApkg("changed.apkg"),
    (error) =>
      error instanceof BuildError &&
      error.report.diagnosticCodes.includes("MEDIA.SOURCE_CHANGED"),
  );
  await assert.rejects(fs.access(path.join(baseDir, "changed.apkg")));
  await assert.rejects(project.media.addFile("missing.png"), MediaError);
  await assert.rejects(
    project.media.addBytes("invalid", Buffer.from("data"), {
      exportAs: "../escape.png",
    }),
    MediaError,
  );
});

test("media references follow filename semantics across projects and missing sources are diagnosed by Rust", async (t) => {
  const baseDir = await directory(t);
  const a = new Project("A", { baseDir }),
    b = new Project("B", { baseDir });
  const bytes = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"/>');
  const ref = await a.media.addBytes("icon.svg", bytes);
  b.addNote(Note.basic("image", "answer").image("Front", ref));
  await assert.rejects(b.writeApkg("missing.apkg"), BuildError);
  await b.media.addBytes("icon.svg", bytes);
  const report = await b.writeApkg("present.apkg");
  report.ensureSuccess();
  assert.equal(report.counts.media, 1);
  const again = await b.media.addBytes("icon.svg", bytes);
  assert.equal(again.filename, ref.filename);
  await assert.rejects(
    b.media.addBytes("icon.svg", Buffer.from("different")),
    MediaError,
  );
});

test("bytes are snapshotted and large buffers retain private files through repeated builds", async (t) => {
  const project = new Project("Buffers", { baseDir: await directory(t) });
  const small = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"/>');
  const pending = project.media.addBytes("small.svg", small);
  small.fill(0);
  const smallRef = await pending;
  const large = Buffer.from(
    `<svg xmlns="http://www.w3.org/2000/svg"><!--${"x".repeat(70_000)}--></svg>`,
  );
  await assert.rejects(
    project.media.addBytes("large.svg", large),
    (error) => error instanceof MediaError && /INLINE/.test(error.code),
  );
  const largeRef = await project.media.addBuffer("large.svg", large);
  large.fill(0);
  project.addNote(
    Note.basic("one", "").image("Front", smallRef).image("Back", largeRef),
  );
  for (const output of ["first.apkg", "second.apkg"]) {
    const report = await project.writeApkg(output);
    report.ensureSuccess();
    assert.equal(report.counts.media, 2);
  }
});

test("Project image occlusion validates rectangles and emits cards", async (t) => {
  const project = new Project("Occlusion", { baseDir: await directory(t) });
  const image = await project.media.addBytes(
    "image.svg",
    Buffer.from(
      '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"/>',
    ),
  );
  assert.throws(
    () =>
      project.addNote(
        Note.imageOcclusion(image, { stableId: "invalid", rects: [] }),
      ),
    ProductNoteError,
  );
  project.addNote(
    Note.imageOcclusion(image, {
      stableId: "io-0",
      mode: "hide-all-guess-one",
      rects: [{ x: 1, y: 2, width: 10, height: 10 }],
      header: "Header",
      backExtra: "Extra",
      comments: "Comments",
    }),
  );
  const report = await project.writeApkg("io.apkg");
  report.ensureSuccess();
  assert.equal(report.counts.notes, 1);
  assert.equal(report.counts.cards, 1);
  // Current Rust stock renderer emits grouped c1,2 syntax, which its writer
  // rejects. Keep that core diagnostic visible until the core supports it.
  project.addNote(
    Note.imageOcclusion(image, {
      stableId: "io-1",
      mode: "hide-one-guess-one",
      rects: [{ x: 1, y: 2, width: 10, height: 10 }],
    }),
  );
  await assert.rejects(
    project.writeApkg("io-grouped.apkg"),
    (error) =>
      error instanceof BuildError &&
      error.code === "PRODUCT.CLOZE_MARKER_MALFORMED",
  );
});

test("template bundle import is atomic and retains source locations", async (t) => {
  const baseDir = await directory(t);
  const fixtures = new URL(
    "../../../contracts/fixtures/template-bundle/",
    import.meta.url,
  );
  const project = new Project("Bundles", { baseDir });
  await assert.rejects(
    project.importTemplateBundle("missing"),
    TemplateBundleError,
  );
  const normal = path.join(baseDir, "normal");
  await fs.cp(new URL("custom-normal/", fixtures), normal, { recursive: true });
  await fs.rename(
    path.join(normal, "assets/icon.svg"),
    path.join(normal, "assets/hidden.svg"),
  );
  await assert.rejects(
    project.importTemplateBundle("normal"),
    TemplateBundleError,
  );
  await fs.rename(
    path.join(normal, "assets/hidden.svg"),
    path.join(normal, "assets/icon.svg"),
  );
  await project.importTemplateBundle("normal");
  project.addNote(
    Note.custom("language-card", { stableId: "bundle" })
      .text("prompt", "Question")
      .text("extra", "Answer"),
  );
  const report = await project.writeApkg("bundle.apkg");
  report.ensureSuccess();
  assert.equal(report.counts.media, 1);
  await assert.rejects(
    project.importTemplateBundle("normal"),
    (error) =>
      error instanceof TemplateBundleError &&
      error.code === "NOTETYPE.ID_DUPLICATE",
  );
});

test("buffer and Writable outputs support backpressure, keep streams open and propagate write errors", async (t) => {
  const baseDir = await directory(t);
  const project = new Project("Bytes", { baseDir });
  project.addNote(Note.basic("front", "back"));
  const bytes = await project.toApkgBuffer();
  assert.ok(Buffer.isBuffer(bytes));
  assert.equal(bytes.subarray(0, 2).toString(), "PK");
  assert.deepEqual(await fs.readdir(baseDir), []);
  const chunks = [];
  const output = new Writable({
    highWaterMark: 1,
    write(chunk, encoding, callback) {
      setTimeout(() => {
        chunks.push(Buffer.from(chunk));
        callback();
      }, 10);
    },
  });
  await project.writeTo(output);
  assert.equal(output.writableEnded, false);
  assert.equal(Buffer.concat(chunks).subarray(0, 2).toString(), "PK");
  output.end();
  const broken = new Writable({
    write(chunk, encoding, callback) {
      callback(new Error("consumer write failed"));
    },
  });
  await assert.rejects(project.writeTo(broken), /consumer write failed/);
  const closedDuringBuild = new Writable({
    write(chunk, encoding, callback) {
      callback();
    },
  });
  const writing = project.writeTo(closedDuringBuild);
  closedDuringBuild.destroy(new Error("consumer closed during build"));
  await assert.rejects(writing, /consumer closed during build/);
});

test("inspection limits, media policy, report JSON and comparison have observable core behavior", async (t) => {
  const baseDir = await directory(t);
  const project = new Project("Options", { baseDir, stableId: "options" });
  project.addNote(Note.basic("front", "back", { stableId: "one" }));
  assert.equal(Object.keys(defaultInspectLimits()).length, 11);
  const first = await project.writeApkg("first.apkg", {
    inspect: false,
    ...firstUpdateSafeBuild("identity.json"),
    reportJson: "report.json",
    artifactsDir: "artifacts",
    mediaMode: "self-contained",
    mediaStoreDir: "media-store",
  });
  first.ensureSuccess();
  assert.equal(first.inspect, null);
  assert.equal(first.updateSafety.lockfile_written, true);
  assert.equal(
    JSON.parse(await fs.readFile(path.join(baseDir, "report.json"), "utf8"))
      .status,
    "success",
  );
  const lockfile = await fs.readFile(path.join(baseDir, "identity.json"));
  (
    await project.writeApkg("next.apkg", updateSafe("identity.json"))
  ).ensureSuccess();
  assert.deepEqual(
    await fs.readFile(path.join(baseDir, "identity.json")),
    lockfile,
  );
  const comparison = await project.diffAgainstApkg("first.apkg");
  comparison.ensureSuccess();
  assert.equal(comparison.comparison, "complete");
  assert.deepEqual(
    await fs.readFile(path.join(baseDir, "identity.json")),
    lockfile,
  );
  await assert.rejects(
    project.writeApkg("limited.apkg", {
      inspectLimits: { maxArchiveBytes: 1 },
    }),
    (error) =>
      error instanceof BuildError &&
      error.report.diagnosticCodes.includes("INSPECT.RESOURCE_LIMIT_EXCEEDED"),
  );
  await project.media.addBytes(
    "unused.svg",
    Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"/>'),
  );
  await assert.rejects(
    project.writeApkg("unused.apkg", {
      mediaPolicy: { unusedBinding: "error" },
    }),
    BuildError,
  );
  (
    await project.writeApkg("ignored.apkg", {
      mediaPolicy: { unusedBinding: "ignore" },
    })
  ).ensureSuccess();
});

test("every inspection budget reaches the Rust inspector and rejects an over-budget artifact", async (t) => {
  const baseDir = await directory(t);
  const project = new Project("Budgets", { baseDir });
  const image = await project.media.addBytes(
    "icon.svg",
    Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"/>'),
  );
  project.addNote(Note.basic("image", "").image("Back", image));
  for (const key of Object.keys(defaultInspectLimits())) {
    await assert.rejects(
      project.writeApkg(`${key}.apkg`, { inspectLimits: { [key]: 0 } }),
      (error) => {
        assert.ok(error instanceof BuildError, key);
        assert.ok(
          error.report.diagnosticCodes.includes(
            "INSPECT.RESOURCE_LIMIT_EXCEEDED",
          ),
          key,
        );
        assert.equal(error.report.artifact, null, key);
        return true;
      },
    );
    await assert.rejects(fs.access(path.join(baseDir, `${key}.apkg`)));
  }
  (await project.writeApkg("within-budget.apkg")).ensureSuccess();
});

test("media policy preserves Rust octet-stream fallback and declared MIME mismatch severity", async (t) => {
  const baseDir = await directory(t);
  const unknown = new Project("Unknown MIME", { baseDir });
  const ref = await unknown.media.addBytes(
    "unknown.dat",
    Buffer.from("unknown media format"),
  );
  unknown.addNote(Note.basic("asset", "").image("Back", ref));
  // Rust Product media always declares a MIME derived from the export name,
  // falling back to application/octet-stream. That is not an unknown MIME.
  (
    await unknown.writeApkg("octet-stream.apkg", {
      mediaPolicy: { unknownMime: "error" },
    })
  ).ensureSuccess();
  (
    await unknown.writeApkg("unknown-ignored.apkg", {
      mediaPolicy: { unknownMime: "ignore" },
    })
  ).ensureSuccess();
  const mismatch = new Project("MIME mismatch", { baseDir });
  const wrong = await mismatch.media.addBytes(
    "wrong.jpg",
    Buffer.from(
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLttAAAAABJRU5ErkJggg==",
      "base64",
    ),
  );
  mismatch.addNote(Note.basic("asset", "").image("Back", wrong));
  await assert.rejects(
    mismatch.writeApkg("mismatch-error.apkg", {
      mediaPolicy: { declaredMimeMismatch: "error" },
    }),
    (error) =>
      error instanceof BuildError &&
      error.report.diagnosticCodes.includes("MEDIA.DECLARED_MIME_MISMATCH"),
  );
  const warned = await mismatch.writeApkg("mismatch-warning.apkg", {
    mediaPolicy: { declaredMimeMismatch: "warning" },
  });
  warned.ensureSuccess();
  assert.ok(
    warned.diagnostics.some(
      (item) =>
        item.code === "MEDIA.DECLARED_MIME_MISMATCH" &&
        item.severity === "warning",
    ),
  );
});

test("Deck retains native identity selection, addition errors, Cloze and build state", async (t) => {
  const baseDir = await directory(t);
  assert.throws(() => new Deck("Invalid", { basicIdentity: [] }), DeckError);
  const deck = new Deck("Deck", {
    baseDir,
    stableId: "deck",
    basicIdentity: ["front", "back"],
  });
  deck.basic("same", "one");
  deck.basic("same", "two");
  assert.throws(() => deck.basic("same", "one"), DeckError);
  assert.throws(
    () =>
      deck.basic("override", "one", {
        identityOverride: { fields: ["front"], reasonCode: "" },
      }),
    DeckError,
  );
  deck.basic("override", "one", {
    identityOverride: { fields: ["front"], reasonCode: "source-key" },
  });
  deck.cloze("{{c1::first}} {{c2::second}}", { extra: "hint" });
  const pending = deck.writeApkg("deck.apkg");
  assert.throws(() => deck.basic("busy", "one"), ProjectBusyError);
  const report = await pending;
  report.ensureSuccess();
  assert.equal(report.counts.notes, 4);
  assert.equal(report.counts.cards, 5);
});

test("Deck image occlusion enforces dimensions and bounds through its own Rust API", async (t) => {
  const deck = new Deck("Deck IO", { baseDir: await directory(t) });
  const png = Buffer.from([
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0,
    0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120,
    156, 99, 248, 15, 4, 0, 9, 251, 3, 253, 167, 102, 129, 94, 0, 0, 0, 0, 73,
    69, 78, 68, 174, 66, 96, 130,
  ]);
  const image = await deck.media.addBytes("pixel.png", png);
  assert.throws(
    () =>
      deck.imageOcclusion(image, {
        rects: [{ x: 0, y: 0, width: 2, height: 2 }],
      }),
    (error) => error instanceof DeckError && /OUT_OF_BOUNDS/.test(error.code),
  );
  deck.imageOcclusion(image, {
    rects: [{ x: 0, y: 0, width: 1, height: 1 }],
    header: "Pixel",
  });
  const report = await deck.writeApkg("pixel.apkg");
  report.ensureSuccess();
  assert.equal(report.counts.cards, 1);
  assert.equal(report.counts.media, 1);
});

test("baseDir stays fixed after chdir and builds do not block the event loop", async (t) => {
  const baseDir = await directory(t);
  const project = new Project("Async", { baseDir });
  for (let index = 0; index < 500; index++)
    project.addNote(Note.basic(`front-${index}`, "back"));
  const previous = process.cwd();
  let ticks = 0;
  const timer = setInterval(() => {
    ticks++;
  }, 1);
  try {
    process.chdir(os.tmpdir());
    const report = await project.writeApkg("async.apkg");
    report.ensureSuccess();
    assert.equal(report.artifact.path, path.join(baseDir, "async.apkg"));
    assert.ok(ticks > 0);
  } finally {
    clearInterval(timer);
    process.chdir(previous);
  }
});

test("pending builds own the native project after JavaScript collection", async (t) => {
  const baseDir = await directory(t);
  let project = new Project("Collected", { baseDir });
  for (let index = 0; index < 100; index++)
    project.addNote(Note.basic(`note-${index}`, "back"));
  const pending = project.writeApkg("collected.apkg");
  project = null;
  global.gc();
  const report = await pending;
  report.ensureSuccess();
  assert.equal(report.counts.notes, 100);
});

test("worker teardown during a native task does not retire another project", async (t) => {
  const baseDir = await directory(t);
  const sdk = fileURLToPath(new URL("../dist/cjs/index.js", import.meta.url));
  for (const output of ["path", "buffer"]) {
    const worker = new Worker(
      `
    const { parentPort, workerData } = require('node:worker_threads');
    const { Project, Note } = require(workerData.sdk);
    const project = new Project('Worker', { baseDir: workerData.baseDir });
    for (let index = 0; index < 500; index++) project.addNote(Note.basic('front-' + index, 'back'));
    const task = workerData.output === 'buffer'
      ? project.toApkgBuffer()
      : project.writeApkg('worker.apkg');
    task.catch(() => {});
    parentPort.postMessage('queued');
  `,
      { eval: true, workerData: { sdk, baseDir, output } },
    );
    t.after(() => worker.terminate());
    await new Promise((resolve, reject) => {
      worker.once("message", resolve);
      worker.once("error", reject);
    });
    await worker.terminate();
    const project = new Project("After worker", { baseDir });
    project.addNote(Note.basic("still", "working"));
    (await project.writeApkg(`after-${output}.apkg`)).ensureSuccess();
  }
});
