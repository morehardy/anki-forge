// Diagnostic workload; RSS observations are not pass/fail thresholds.
import { randomBytes } from "node:crypto";
import {
  Project,
  Note,
  Media,
  BuildOptions,
  bindingMetadata,
} from "../dist/index.mjs";
if (!global.gc) throw new Error("Run with --expose-gc");
const samples = [];
async function sample(phase) {
  global.gc();
  await new Promise((r) => setImmediate(r));
  global.gc();
  samples.push({ phase, ...process.memoryUsage() });
}
await sample("loaded");
let project = new Project("resource-probe");
for (let i = 0; i < 1000; i++)
  project.add(`note-${i}`, Note.basic(`front-${i}`, "back".repeat(2048)));
const mib = Number(process.argv[2] ?? 8);
let payload = randomBytes(mib * 1024 * 1024);
const wav = Buffer.alloc(44);
wav.write("RIFF");
wav.writeUInt32LE(payload.length + 36, 4);
wav.write("WAVEfmt ", 8);
wav.writeUInt32LE(16, 16);
wav.writeUInt16LE(1, 20);
wav.writeUInt16LE(1, 22);
wav.writeUInt32LE(16000, 24);
wav.writeUInt32LE(32000, 28);
wav.writeUInt16LE(2, 32);
wav.writeUInt16LE(16, 34);
wav.write("data", 36);
wav.writeUInt32LE(payload.length, 40);
let media = await Media.bytes(Buffer.concat([wav, payload]), "audio/wav");
payload = null;
project.add("audio", Note.basic(media.sound(), "answer"));
media = null;
await sample("authored");
for (let i = 0; i < 3; i++) {
  const out = await project.build(BuildOptions.temporary());
  await out.artifact.close();
  await sample(`build-${i + 1}-closed`);
}
project = null;
await sample("project-collected");
console.log(
  JSON.stringify(
    { node: process.version, metadata: bindingMetadata(), samples },
    null,
    2,
  ),
);
