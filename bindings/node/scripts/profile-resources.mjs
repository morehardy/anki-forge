// Diagnostic-only subprocess workload. Output distinguishes Rust build allocations
// from JavaScript transfer buffers; no RSS threshold belongs in the test suite.
import { createRequire } from "node:module";
import { randomBytes } from "node:crypto";
import { Writable } from "node:stream";
import { Deck, Project, Note, bindingMetadata } from "../dist/index.mjs";

if (!global.gc) throw new Error("Run with --expose-gc");
const mode = process.argv[2];
const samples = [];
const binding = createRequire(import.meta.url)(
  process.env.ANKI_FORGE_NATIVE_PATH,
);
const collect = async () => {
  global.gc();
  await new Promise((resolve) => setImmediate(resolve));
  global.gc();
  await new Promise((resolve) => setImmediate(resolve));
};
async function sample(phase) {
  await collect();
  samples.push({
    phase,
    ...process.memoryUsage(),
    ...(binding.allocationStats ? JSON.parse(binding.allocationStats()) : {}),
  });
}
await sample("loaded");
if (mode === "deck") {
  let deck = new Deck("Resource probe", { stableId: "resources" });
  for (let i = 0; i < 1000; i++)
    deck.basic(`front-${i}`, "back".repeat(2048), { stableId: `note-${i}` });
  for (let i = 0; i < 64; i++) {
    const image = await deck.media.addBytes(
      `image-${i}.svg`,
      Buffer.from(
        `<svg xmlns="http://www.w3.org/2000/svg"><!--${"x".repeat(16384)}--></svg>`,
      ),
    );
    deck.basic(`<img src="${image.filename}">`, "media", {
      stableId: `media-${i}`,
    });
  }
  await sample("authored");
  for (let i = 0; i < 3; i++) {
    const report = await deck.build();
    report.ensureSuccess();
    await report.artifactHandle.close();
    await sample(`build-${i + 1}-closed`);
  }
  deck = null;
  await sample("deck-collected");
} else if (mode === "stream") {
  const mib = Number(process.argv[3] ?? 8);
  const project = new Project("Transfer probe");
  let payload = randomBytes(mib * 1024 * 1024);
  const header = Buffer.from(
    "UklGRiQAAABXQVZFZm10IBAAAAABAAEAESsAACJWAAACABAAZGF0YQAAAAA=",
    "base64",
  );
  header.writeUInt32LE(payload.length + 36, 4);
  header.writeUInt32LE(payload.length, 40);
  const media = await project.media.addBuffer(
    "noise.wav",
    Buffer.concat([header, payload]),
  );
  payload = null;
  project.addNote(Note.basic("media", "answer").sound("Back", media));
  await sample("authored");
  let total = 0,
    count = 0,
    maxChunk = 0,
    peakArrayBuffers = 0,
    peakExternal = 0;
  const target = new Writable({
    highWaterMark: 1024,
    write(chunk, encoding, callback) {
      total += chunk.length;
      count++;
      maxChunk = Math.max(maxChunk, chunk.length);
      const use = process.memoryUsage();
      peakArrayBuffers = Math.max(peakArrayBuffers, use.arrayBuffers);
      peakExternal = Math.max(peakExternal, use.external);
      if (count === 1)
        sample("transfer-start").then(() => callback(), callback);
      else setImmediate(callback);
    },
  });
  await project.writeTo(target);
  await sample("transfer-complete");
  samples.push({
    phase: "transfer-summary",
    inputMiB: mib,
    archiveBytes: total,
    count,
    maxChunk,
    peakArrayBuffers,
    peakExternal,
    writableEnded: target.writableEnded,
  });
  target.end();
} else throw new Error("Expected deck or stream");
console.log(
  JSON.stringify(
    {
      node: process.version,
      platform: `${process.platform}/${process.arch}`,
      metadata: bindingMetadata(),
      mode,
      samples,
    },
    null,
    2,
  ),
);
