import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";
import { deflateSync } from "node:zlib";
import * as sdk from "../dist/index.mjs";
const {
  Project,
  Note,
  Content,
  Media,
  Field,
  Template,
  NoteType,
  Mask,
  BuildOptions,
  CompareOptions,
  UpdatePolicy,
  BuildError,
  AddError,
  SchemaError,
  MediaError,
  CompareError,
  ImageOcclusionError,
  PersistError,
  ArtifactClosedError,
} = sdk;
function inspect(filename) {
  const r = spawnSync(
    process.env.ANKI_FORGE_TEST_OBSERVER,
    ["inspect", filename],
    { encoding: "utf8" },
  );
  assert.equal(r.status, 0, r.stderr);
  return JSON.parse(r.stdout);
}
async function temp(t) {
  const d = await fs.mkdtemp(path.join(os.tmpdir(), "node-public-"));
  t.after(() => fs.rm(d, { recursive: true, force: true }));
  return d;
}
function png(width = 64, height = 64) {
  function chunk(type, data) {
    const s = Buffer.concat([Buffer.from(type), data]);
    let crc = 0xffffffff;
    for (const b of s) {
      crc ^= b;
      for (let i = 0; i < 8; i++)
        crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
    const h = Buffer.alloc(4),
      c = Buffer.alloc(4);
    h.writeUInt32BE(data.length);
    c.writeUInt32BE((crc ^ 0xffffffff) >>> 0);
    return Buffer.concat([h, s, c]);
  }
  const h = Buffer.alloc(13);
  h.writeUInt32BE(width);
  h.writeUInt32BE(height, 4);
  h[8] = 8;
  h[9] = 6;
  return Buffer.concat([
    Buffer.from("89504e470d0a1a0a", "hex"),
    chunk("IHDR", h),
    chunk("IDAT", deflateSync(Buffer.alloc(height * (width * 4 + 1)))),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}
function model(name = "中文") {
  return NoteType.builder("vocab")
    .name(name)
    .field(new Field("front", { name: "正面", required: true }))
    .field(new Field("back", { name: "背面" }))
    .template(
      new Template("recognition", {
        name: "识别",
        front: "{{front}}",
        back: "{{FrontSide}}<hr>{{back}}",
        browserFront: "{{front}}",
        generation: sdk.GenerationRule.all(["front"]),
      }),
    )
    .build();
}

test("byte media shares container matching and canonical filenames with Rust", async () => {
  const cases = [
    [png(), "IMAGE/PNG", "image/png", ".png"],
    [Buffer.from("1a45dfa37765626d", "hex"), "AUDIO/WEBM; codecs=Opus", "audio/webm; codecs=Opus", ".webm"],
    [Buffer.from("OggSOpusHead"), "AUDIO/OPUS", "audio/opus", ".opus"],
    [Buffer.from("000000186674797069736f6d", "hex"), "audio/mp4", "audio/mp4", ".m4a"],
  ];
  const project = new Project("mime-containers").add("one", Note.basic("q", "a"));
  for (const [bytes, declared, canonical, extension] of cases) {
    const media = await Media.bytes(bytes, declared);
    const lower = await Media.bytes(bytes, canonical);
    assert.equal(media.mediaType, canonical);
    assert.equal(media.filename, lower.filename);
    assert.ok(media.filename.endsWith(extension));
    project.addAsset(media);
  }
  const output = await project.build(BuildOptions.temporary());
  try {
    assert.equal(output.report.counts.media, cases.length);
    assert.deepEqual(
      inspect(output.artifact.path).media.map((item) => Buffer.from(item.bytes).toString("hex")).sort(),
      cases.map(([bytes]) => bytes.toString("hex")).sort(),
    );
  } finally {
    await output.artifact.close();
  }
});

test("explicit keys and strings as Text; only Project authors publications", async (t) => {
  const p = new Project("parity").add(
    "hello",
    Note.basic("<hello>", Content.html("<b>world</b>")),
  );
  assert.throws(
    () => p.add("hello", Note.basic("a", "b")),
    (e) => e instanceof AddError && e.code === "NOTE.KEY_DUPLICATE",
  );
  assert.equal(p.length, 1);
  assert.throws(() => p.add("", Note.basic("a", "b")), AddError);
  const out = await p.build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  assert.equal(out.snapshot().result.status, "success");
  assert.equal(out.report.counts.notes, 1);
  assert.equal(
    inspect(out.artifact.path).notes[0].fields,
    "&lt;hello&gt;\x1f<b>world</b>",
  );
  assert.equal("result" in out.report.snapshot(), false);
  assert.equal("artifact" in out.report.snapshot(), false);
  for (const name of [
    "Deck",
    "MediaRegistry",
    "MediaRef",
    "IdentityRecipe",
    "firstUpdateSafeBuild",
    "updateSafe",
  ])
    assert.equal(name in sdk, false, name);
  const d = await temp(t),
    r = spawnSync(
      process.env.ANKI_FORGE_TEST_OBSERVER,
      ["produce", path.join(d, "rust.apkg")],
      { encoding: "utf8" },
    );
  assert.equal(r.status, 0, r.stderr);
  const rust = inspect(path.join(d, "rust.apkg"));
  assert.deepEqual(inspect(out.artifact.path).notes, rust.notes);
});
test("immutable validated model is reusable; key conflicts and failed add are atomic", async (t) => {
  const m = model();
  assert.equal(m.key, "vocab");
  assert.equal(m.displayName, "中文");
  assert.equal(m.field, undefined);
  assert.throws(
    () => NoteType.builder("bad").field(new Field("")).build(),
    SchemaError,
  );
  const n = m
    .note()
    .field("front", "词汇")
    .field("back", Content.html("<b>meaning</b>"));
  const p = new Project("custom").add("one", n);
  assert.throws(
    () => p.add("bad", n.field("missing", "x")),
    (e) => e instanceof AddError && e.code === "NOTE.FIELD_UNKNOWN",
  );
  assert.equal(p.length, 1);
  assert.throws(
    () => p.add("conflict", model("different").note().field("front", "x")),
    (e) => e.code === "NOTE.MODEL_CONFLICT",
  );
  assert.equal(p.length, 1);
  const out = await p.build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  assert.equal(
    inspect(out.artifact.path).notes[0].fields,
    "词汇\x1f<b>meaning</b>",
  );
  const other = await new Project("other")
    .add("reuse", n)
    .build(BuildOptions.temporary());
  t.after(() => other.artifact.close());
  assert.equal(other.report.counts.notes, 1);
});
test("media owns file snapshot and >64KiB bytes; typed dependencies are automatic", async (t) => {
  const d = await temp(t),
    imagePath = path.join(d, "source.png"),
    bytes = png();
  await fs.writeFile(imagePath, bytes);
  const image = await Media.file(imagePath);
  await fs.writeFile(imagePath, "changed");
  await fs.unlink(imagePath);
  const text = Buffer.from(
    '<svg xmlns="http://www.w3.org/2000/svg"><!--' +
      "x".repeat(80000) +
      "--></svg>",
  );
  const large = await Media.bytes(text, "image/svg+xml");
  assert.equal(large.byteLength, text.length);
  const fixed = image.withExportName("图片 badge.png");
  assert.notEqual(fixed.filename, image.filename);
  const p = new Project("media")
    .add(
      "image",
      Note.basic(Content.sequence(["before ", fixed.image()]), "after"),
    )
    .addAsset(large);
  const out = await p.build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  const seen = inspect(out.artifact.path);
  assert.equal(out.report.counts.media, 2);
  assert(seen.media.some((m) => Buffer.from(m.bytes).equals(bytes)));
  assert(seen.media.some((m) => Buffer.from(m.bytes).equals(text)));
  assert.match(seen.notes[0].fields, /<img/);
  const second = await new Project("media2")
    .add("same", Note.basic(fixed.image(), "x"))
    .build(BuildOptions.temporary());
  t.after(() => second.artifact.close());
  assert.equal(second.report.counts.media, 1);
});
test("media budgets, MIME and portable conflicts preserve atomic project state", async (t) => {
  await assert.rejects(Media.bytes(png(), "audio/wav"), MediaError);
  await assert.rejects(
    Media.bytes(png(), "image/png", { maxBytes: 0 }),
    (e) => e instanceof MediaError && e.details.limitExceeded.limit === 0,
  );
  const a = (await Media.bytes(png(), "image/png")).withExportName("Badge.png");
  assert.throws(() => a.withExportName("../bad.png"), MediaError);
  const b = a.withExportName("badge.png"),
    p = new Project("collision").addAsset(a);
  assert.throws(() => p.add("bad", Note.basic(b.image(), "x")), AddError);
  assert.equal(p.length, 0);
  p.add("ok", Note.basic(a.image(), "x"));
  const out = await p.build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  assert.equal(out.report.counts.media, 1);
});
test("media byte budgets reject before allocating binding copies", () => {
  // Fresh processes make peak RSS independent of other tests and retained heaps.
  // Fault in the caller's storage before measuring: only binding overhead counts.
  for (const useDefault of [false, true]) {
    const result = spawnSync(process.execPath, ["--input-type=module", "--eval", `
      import assert from "node:assert/strict";
      import { Media, MediaError } from ${JSON.stringify(new URL("../dist/index.mjs", import.meta.url).href)};
      await Media.bytes(new Uint8Array([1]), "application/octet-stream");
      const limit = ${useDefault ? 256 * 1024 * 1024 : 1024};
      const bytes = new Uint8Array(${useDefault ? 256 * 1024 * 1024 + 1 : 64 * 1024 * 1024});
      bytes.fill(1);
      const before = process.resourceUsage().maxRSS;
      await assert.rejects(
        Media.bytes(bytes, "application/octet-stream", ${useDefault ? "{}" : "{ maxBytes: BigInt(limit) }"}),
        error => error instanceof MediaError &&
          error.kind === "ResourceLimit" &&
          error.code === "MEDIA.RESOURCE_LIMIT_EXCEEDED" &&
          error.details.limitExceeded.resource === "media_bytes" &&
          error.details.limitExceeded.limit === limit &&
          error.details.limitExceeded.observed === bytes.length,
      );
      const overhead = (process.resourceUsage().maxRSS - before) * 1024;
      assert(overhead < 32 * 1024 * 1024,
        "rejecting over-budget bytes allocated " + overhead + " bytes of resident memory");
    `], { encoding: "utf8", timeout: 30000 });
    assert.equal(result.status, 0, result.stderr || String(result.error));
  }
});
test("media bytes snapshot a subarray before the caller can mutate it", async (t) => {
  const expected = png();
  const backing = new Uint8Array(expected.length + 14);
  const view = backing.subarray(7, backing.length - 7);
  view.set(expected);
  const pending = Media.bytes(view, "image/png", { maxBytes: view.byteLength });
  backing.fill(0);
  const media = await pending;
  assert.equal(media.byteLength, expected.length);
  const out = await new Project("snapshot")
    .add("image", Note.basic(media.image(), "answer"))
    .build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  assert.deepEqual(Buffer.from(inspect(out.artifact.path).media[0].bytes), expected);
});
test("bundle v2 returns reusable model whose assets survive directory deletion", async (t) => {
  const d = await temp(t);
  await fs.writeFile(path.join(d, "badge.png"), png());
  await fs.writeFile(
    path.join(d, "front.html"),
    '{{front}}<img src="badge.png">',
  );
  await fs.writeFile(path.join(d, "back.html"), "{{back}}");
  await fs.writeFile(
    path.join(d, "anki-template.yaml"),
    `format_version: template-bundle-v2\nnote_type:\n  key: bundled\n  name: Bundle\n  fields:\n    - key: front\n    - key: back\n  templates:\n    - key: card\n      front_file: front.html\n      back_file: back.html\nassets:\n  - path: badge.png\n    export_as: badge.png\n`,
  );
  const m = await NoteType.fromBundle(d);
  await fs.rm(d, { recursive: true });
  const out = await new Project("bundle")
    .add("a", m.note().field("front", "a").field("back", "b"))
    .build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  assert.equal(out.report.counts.media, 1);
  assert(
    inspect(out.artifact.path).media.some((m) =>
      Buffer.from(m.bytes).equals(png()),
    ),
  );
});
test("IO validates stable masks and both modes emit separate cards", async (t) => {
  const image = await Media.bytes(png(), "image/png");
  for (const mode of ["hide_all_guess_one", "hide_one_guess_one"]) {
    const n = Note.imageOcclusion(image)
      .mode(mode)
      .mask(Mask.rect("nucleus", 2, 2, 10, 10))
      .mask(Mask.rect("wall", 20, 20, 12, 12))
      .build()
      .field("header", "Cell");
    const out = await new Project(mode)
      .add("cell", n)
      .build(BuildOptions.temporary());
    t.after(() => out.artifact.close());
    assert.equal(inspect(out.artifact.path).cards, 2);
  }
  assert.throws(
    () =>
      Note.imageOcclusion(image)
        .mask(Mask.rect("bad", 60, 0, 10, 10))
        .build(),
    ImageOcclusionError,
  );
  assert.throws(
    () =>
      Note.imageOcclusion(image)
        .mask(Mask.rect("x", 0, 0, 2, 2))
        .mask(Mask.rect("x", 3, 3, 2, 2))
        .build(),
    ImageOcclusionError,
  );
});
test("non-finite mask coordinates retain structured native errors at build", async () => {
  const image = await Media.bytes(png(), "image/png");
  for (const value of [NaN, Infinity, -Infinity]) {
    for (const coordinate of [0, 1, 2, 3]) {
      const numbers = [0, 0, 2, 2];
      numbers[coordinate] = value;
      const mask = Mask.rect("invalid", ...numbers);
      const builder = Note.imageOcclusion(image).mask(mask);
      assert.throws(() => builder.build(), error => {
        assert(error instanceof ImageOcclusionError);
        assert.equal(error.code, "NOTE.IO_RECT_INVALID");
        assert.equal(error.kind, "InvalidMask");
        return true;
      });
    }
  }
});
test("compare keeps high-risk evidence and update blocks until explicit category allowance", async (t) => {
  const first = await new Project("updates")
    .add("a", Note.basic("a", "A"))
    .add("b", Note.basic("b", "B"))
    .build(BuildOptions.temporary());
  t.after(() => first.artifact.close());
  const next = new Project("updates").add("a", Note.basic("a", "edited"));
  const comparison = await next.compare(
    CompareOptions.against(first.artifact.path),
  );
  assert.equal(comparison.policy.allows_publication, false);
  assert(comparison.findings.some((f) => f.code === "RISK.NOTE_REMOVED"));
  await assert.rejects(
    next.build(BuildOptions.temporary().updateFrom(first.artifact.path)),
    (e) =>
      e instanceof BuildError &&
      e.snapshot().result.status === "failure" &&
      e.report.comparison.policy.allows_publication === false,
  );
  const policy = new UpdatePolicy().allow("RISK.NOTE_REMOVED");
  const accepted = await next.compare(
    CompareOptions.against(first.artifact.path).updatePolicy(policy),
  );
  assert.equal(accepted.policy.allows_publication, true);
  assert.equal(accepted.highestRisk, "high");
  const out = await next.build(
    BuildOptions.temporary()
      .updatePolicy(policy)
      .updateFrom(first.artifact.path),
  );
  t.after(() => out.artifact.close());
  assert.deepEqual(out.report.comparison.policy, accepted.policy);
  const before = inspect(first.artifact.path).notes.find((n) =>
      n.fields.startsWith("a\x1f"),
    ),
    after = inspect(out.artifact.path).notes[0];
  assert.equal(before.guid, after.guid);
  assert.equal(after.fields, "a\x1fedited");
});
test("invalid policy, Create policy and unavailable baseline retain machine errors", async (t) => {
  const p = new Project("errors").add("a", Note.basic("a", "b"));
  await assert.rejects(
    p.build(BuildOptions.temporary().updatePolicy(new UpdatePolicy())),
    (e) =>
      e instanceof BuildError &&
      e.code === "BUILD.UPDATE_POLICY_WITHOUT_BASELINE" &&
      e.kind === "Configuration",
  );
  await assert.rejects(
    p.compare(CompareOptions.against("/missing/nowhere.apkg")),
    (e) =>
      e instanceof CompareError && e.causes.length > 0 && Boolean(e.report),
  );
  const out = await p.build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  await assert.rejects(
    p.compare(
      CompareOptions.against(out.artifact.path).updatePolicy(
        new UpdatePolicy().allow("NOT.A.RISK"),
      ),
    ),
    sdk.PolicyError,
  );
  await assert.rejects(
    p.compare(
      CompareOptions.against(out.artifact.path).inspectLimits({
        maxArchiveBytes: 0,
      }),
    ),
    (e) => e instanceof CompareError && e.details.limitExceeded.limit === 0,
  );
});
test("artifact clones own temporary lifetime; snapshots do not; failed persistence preserves source", async (t) => {
  const d = await temp(t),
    out = await new Project("lifetime")
      .add("a", Note.basic("a", "b"))
      .build(BuildOptions.temporary()),
    snapshot = out.snapshot(),
    clone = out.artifact.clone();
  await out.artifact.close();
  await fs.access(clone.path);
  await assert.rejects(
    clone.persistTo(d),
    (e) => e instanceof PersistError && e.publication.stage === "not_published",
  );
  await fs.access(clone.path);
  const persisted = await clone.persistTo(path.join(d, "saved.apkg"));
  await clone.close();
  await assert.rejects(fs.access(snapshot.result.artifact));
  assert.equal(snapshot.result.status, "success");
  assert.throws(() => clone.clone(), ArtifactClosedError);
  await persisted.close();
  await fs.access(path.join(d, "saved.apkg"));
});
test("build captures project at invocation; clone remains independently mutable", async (t) => {
  const p = new Project("concurrency").add("one", Note.basic("one", "1"));
  const pending = p.build(BuildOptions.temporary());
  p.add("two", Note.basic("two", "2"));
  const clone = p.clone().add("three", Note.basic("three", "3"));
  assert.equal(p.length, 2);
  assert.equal(clone.length, 3);
  const out = await pending;
  t.after(() => out.artifact.close());
  assert.equal(out.report.counts.notes, 1);
  const results = await Promise.all([
    p.build(BuildOptions.temporary()),
    clone.build(BuildOptions.temporary()),
  ]);
  for (const o of results) t.after(() => o.artifact.close());
  assert.deepEqual(
    results.map((o) => o.report.counts.notes),
    [2, 3],
  );
});
test("Cloze strings escape HTML while retaining cloze syntax", async (t) => {
  const out = await new Project("cloze")
    .add("cloze", Note.cloze("{{c1::<b>cell</b>}}"))
    .build(BuildOptions.temporary());
  t.after(() => out.artifact.close());
  assert.match(
    inspect(out.artifact.path).notes[0].fields,
    /\{\{c1::&lt;b&gt;cell&lt;\/b&gt;\}\}/,
  );
  assert.equal(out.report.counts.cards, 1);
});
test("u64 budgets cross native boundary losslessly and unsafe numbers are rejected", async (t) => {
  const p = new Project("bigint").add("one", Note.basic("one", "1"));
  const out = await p.build(
    BuildOptions.temporary().inspectLimits({
      maxArchiveBytes: 18446744073709551615n,
    }),
  );
  t.after(() => out.artifact.close());
  assert.equal(out.report.counts.notes, 1);
  const image = await Media.bytes(png(), "image/png", {
    maxBytes: 18446744073709551615n,
  });
  assert.equal(image.byteLength, png().length);
  assert.throws(
    () =>
      BuildOptions.temporary().inspectLimits({
        maxEntries: Number.MAX_SAFE_INTEGER + 1,
      }),
    TypeError,
  );
  assert.throws(
    () =>
      BuildOptions.temporary().inspectLimits({
        maxEntries: 18446744073709551616n,
      }),
    TypeError,
  );
  for (const invalid of [-1, -1n, 1.5, "9007199254740993"]) {
    assert.throws(
      () => BuildOptions.temporary().inspectLimits({ maxEntries: invalid }),
      TypeError,
    );
  }
  await assert.rejects(
    Media.bytes(png(), "image/png", { maxBytes: 0n }),
    (error) =>
      error instanceof MediaError && error.details.limitExceeded.limit === 0,
  );
  await assert.rejects(
    p.build(BuildOptions.temporary().inspectLimits({ unregisteredBudget: 1 })),
    sdk.ConfigurationError,
  );
  assert.throws(() => new sdk.BuildOutput({}, {}), TypeError);
});
test("worker teardown safely releases in-flight native operations", async () => {
  const { Worker } = await import("node:worker_threads");
  const url = new URL("../dist/index.mjs", import.meta.url).href;
  for (let i = 0; i < 3; i++) {
    const worker = new Worker(
      `const {parentPort}=require('node:worker_threads');(async()=>{const {Project,Note,BuildOptions}=await import(${JSON.stringify(url)});const p=new Project('worker').add('a',Note.basic('a','b'));const pending=p.build(BuildOptions.temporary());parentPort.postMessage('started');await pending;})()`,
      { eval: true },
    );
    await new Promise((resolve, reject) => {
      worker.once("message", resolve);
      worker.once("error", reject);
    });
    await worker.terminate();
  }
});
test("domain errors retain machine codes, source metadata, location and budgets", async (t) => {
  const d = await temp(t);
  await assert.rejects(
    Media.file(path.join(d, "absent.png")),
    (e) =>
      e instanceof MediaError &&
      typeof e.kind === "string" &&
      e.code.startsWith("MEDIA.") &&
      e.causes.length > 0 &&
      e.sourceDetails.some((s) => s.type === "io" && s.kind === "NotFound") &&
      e.cause instanceof Error,
  );
  await fs.writeFile(path.join(d, "anki-template.yaml"), "format_version: [");
  await assert.rejects(
    NoteType.fromBundle(d),
    (e) =>
      e instanceof sdk.TemplateBundleError &&
      e.code &&
      e.causes.length > 0 &&
      e.details.path.endsWith("anki-template.yaml"),
  );
  assert.throws(
    () =>
      NoteType.builder("invalid-template")
        .field(new Field("front"))
        .template(
          new Template("card", { front: "{{unknown}}", back: "{{front}}" }),
        )
        .build(),
    (e) =>
      e instanceof SchemaError &&
      e.details.location.template === "card" &&
      Number.isInteger(e.details.location.byteRange.start),
  );
  await assert.rejects(
    Media.bytes(png(), "image/png", { maxBytes: 0 }),
    (e) =>
      e instanceof MediaError &&
      e.details.limitExceeded.resource === "media_bytes" &&
      e.details.limitExceeded.observed > 0,
  );
  await fs.writeFile(
    path.join(d, "anki-template.yaml"),
    "x".repeat(256 * 1024 + 1),
  );
  await assert.rejects(
    NoteType.fromBundle(d),
    (e) =>
      e instanceof sdk.TemplateBundleError &&
      e.sourceDetails.some(
        (s) => s.type === "bundle_limit" && s.limit === 256 * 1024,
      ),
  );
});
test("verified baseline observations survive a later candidate error", async (t) => {
  const first = await new Project("baseline-error")
    .add("a", Note.cloze("{{c1::cell}}"))
    .build(BuildOptions.temporary());
  t.after(() => first.artifact.close());
  const next = new Project("baseline-error").add(
    "a",
    Note.cloze("{{c501::cell}}"),
  );
  await assert.rejects(
    next.compare(CompareOptions.against(first.artifact.path)),
    (e) =>
      e instanceof CompareError &&
      e.report.baseline_counts.notes === 1 &&
      e.report.comparison === null,
  );
  await assert.rejects(
    next.build(BuildOptions.temporary().updateFrom(first.artifact.path)),
    (e) =>
      e instanceof BuildError &&
      e.report.baseline_counts.notes === 1 &&
      e.snapshot().result.status === "failure",
  );
});
test("unmatched allowance is a warning and preserves real successful outcome", async (t) => {
  const p = new Project("warning").add("a", Note.basic("a", "b")),
    first = await p.build(BuildOptions.temporary());
  t.after(() => first.artifact.close());
  const policy = new UpdatePolicy().allow("RISK.MEDIA_REMOVED");
  const comparison = await p.compare(
    CompareOptions.against(first.artifact.path).updatePolicy(policy),
  );
  assert.deepEqual(comparison.policy.unmatched_allowances, [
    "RISK.MEDIA_REMOVED",
  ]);
  assert.equal(comparison.policy.allows_publication, true);
  const out = await p.build(
    BuildOptions.temporary()
      .updateFrom(first.artifact.path)
      .updatePolicy(policy),
  );
  t.after(() => out.artifact.close());
  assert.equal(out.snapshot().result.status, "success");
  assert(out.report.diagnostics.length > 0);
  assert.equal(out.report.baselineCounts.notes, 1);
});
