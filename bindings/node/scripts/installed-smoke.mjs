import fs from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import http from "node:http";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { deflateSync } from "node:zlib";
import { spawn } from "node:child_process";
import { root, targets } from "./platforms.mjs";

const npmCli =
  process.env.npm_execpath ??
  path.resolve(
    path.dirname(process.execPath),
    "../lib/node_modules/npm/bin/npm-cli.js",
  );
const temporary = await fs.mkdtemp(
  path.join(os.tmpdir(), "anki-forge-installed-"),
);
const cache = path.join(temporary, "cache");
const tarballs = path.join(temporary, "tarballs");
await fs.mkdir(tarballs);
const metadata = new Map();
const packages = new Map();
const allPlatforms = process.argv.includes("--all");
function run(args, cwd, env = process.env) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, args, {
      cwd,
      env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "",
      stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", reject);
    child.on("exit", (code) =>
      code === 0
        ? resolve(stdout)
        : reject(
            new Error(`${args.join(" ")} exited ${code}\n${stderr}\n${stdout}`),
          ),
    );
  });
}
const server = http.createServer((request, response) => {
  const name = decodeURIComponent((request.url ?? "").slice(1));
  if (packages.has(name)) {
    response.end(packages.get(name));
    return;
  }
  const manifest = metadata.get(name);
  if (manifest) {
    response.setHeader("content-type", "application/json");
    response.end(
      JSON.stringify({
        name,
        "dist-tags": { latest: manifest.version },
        versions: { [manifest.version]: manifest },
      }),
    );
    return;
  }
  response.writeHead(404);
  response.end("{}");
});
try {
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const registry = `http://127.0.0.1:${server.address().port}`;
  const host = targets.find(
    (item) => item.os === process.platform && item.cpu === process.arch,
  );
  if (!host) throw new Error("Unsupported smoke-test host");
  await fs.access(path.join(root, "npm", host.suffix, "anki-forge.node"));
  // Pack real distributable files and let npm resolve optional dependencies from
  // this disposable registry. No npm link, source-tree module fallback or CLI.
  const packedTargets = allPlatforms ? targets : [host];
  for (const item of packedTargets)
    await fs.access(path.join(root, "npm", item.suffix, "anki-forge.node"));
  for (const directory of [
    root,
    ...packedTargets.map((item) => path.join(root, "npm", item.suffix)),
  ]) {
    const [packed] = JSON.parse(
      await run(
        [
          npmCli,
          "pack",
          "--json",
          "--ignore-scripts",
          "--cache",
          cache,
          "--pack-destination",
          tarballs,
        ],
        directory,
      ),
    );
    const manifest = JSON.parse(
      await fs.readFile(path.join(directory, "package.json"), "utf8"),
    );
    const bytes = await fs.readFile(path.join(tarballs, packed.filename));
    packages.set(packed.filename, bytes);
    manifest.dist = {
      tarball: `${registry}/${packed.filename}`,
      integrity: `sha512-${createHash("sha512").update(bytes).digest("base64")}`,
    };
    metadata.set(manifest.name, manifest);
    if (directory === root) {
      assert.ok(packed.files.some((file) => file.path === "dist/index.d.mts"));
      assert.ok(
        !packed.files.some(
          (file) =>
            file.path.startsWith("native/") || file.path.endsWith(".node"),
        ),
      );
    }
  }
  // Supply metadata for non-host packages, so npm performs OS/CPU filtering.
  for (const item of targets.filter((item) => !packedTargets.includes(item))) {
    const manifest = JSON.parse(
      await fs.readFile(
        path.join(root, "npm", item.suffix, "package.json"),
        "utf8",
      ),
    );
    metadata.set(manifest.name, {
      ...manifest,
      dist: { tarball: `${registry}/must-not-fetch-${item.suffix}.tgz` },
    });
  }
  const consumer = path.join(temporary, "consumer 安装 path");
  await fs.mkdir(consumer);
  await fs.writeFile(
    path.join(consumer, "package.json"),
    '{"private":true,"type":"module"}\n',
  );
  const version = metadata.get("anki-forge-node").version;
  await run(
    [
      npmCli,
      "install",
      `anki-forge-node@${version}`,
      "--registry",
      registry,
      "--cache",
      cache,
      "--include=optional",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
    ],
    consumer,
  );
  await run(
    [
      npmCli,
      "ci",
      "--registry",
      registry,
      "--cache",
      path.join(temporary, "fresh-ci-cache"),
      "--omit=dev",
      "--include=optional",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
    ],
    consumer,
  );
  assert.ok(
    (
      await fs.stat(
        path.join(
          consumer,
          "node_modules",
          `anki-forge-node-${host.suffix}`,
          "anki-forge.node",
        ),
      )
    ).size > 0,
  );
  for (const item of targets.filter((item) => item !== host)) {
    await assert.rejects(
      fs.access(
        path.join(consumer, "node_modules", `anki-forge-node-${item.suffix}`),
      ),
    );
  }
  await fs.writeFile(
    path.join(consumer, "smoke.mjs"),
    `
    import assert from 'node:assert/strict';
    import { createRequire } from 'node:module';
    import { Project, Note, bindingMetadata } from 'anki-forge-node';
    const cjs = createRequire(import.meta.url)('anki-forge-node');
    assert.equal(cjs.Project, Project);
    const project = new Project('Installed', { stableId: 'installed' });
    project.addNote(Note.basic('npm', 'Rust', { stableId: 'note' }));
    (await project.validate()).ensureSuccess();
    const report = await project.writeApkg('installed.apkg', { inspect: true });
    report.ensureSuccess();
    assert.equal(report.inspect.notes, 1);
    assert.equal(bindingMetadata().bindingVersion, ${JSON.stringify(version)});
    console.log('Installed ESM + CJS → Rust → inspected APKG: passed');
  `,
  );
  const cleanEnv = {
    ...process.env,
    PATH: path.join(temporary, "no-executables"),
  };
  for (const key of Object.keys(cleanEnv))
    if (/^(ANKI_FORGE_|CARGO_|RUSTUP_|NODE_PATH)/.test(key))
      delete cleanEnv[key];
  process.stdout.write(await run(["smoke.mjs"], consumer, cleanEnv));
  // Execute the actual README snippets, copied unchanged into the installed
  // consumer. Fixtures are generated here; examples never depend on repository assets.
  function pngChunk(type, bytes) {
    const payload = Buffer.concat([Buffer.from(type), bytes]);
    let crc = 0xffffffff;
    for (const byte of payload) {
      crc ^= byte;
      for (let bit = 0; bit < 8; bit++)
        crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
    const length = Buffer.alloc(4),
      checksum = Buffer.alloc(4);
    length.writeUInt32BE(bytes.length);
    checksum.writeUInt32BE((crc ^ 0xffffffff) >>> 0);
    return Buffer.concat([length, payload, checksum]);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(100, 0);
  ihdr.writeUInt32BE(100, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  await fs.writeFile(
    path.join(consumer, "diagram.png"),
    Buffer.concat([
      Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
      pngChunk("IHDR", ihdr),
      pngChunk("IDAT", deflateSync(Buffer.alloc(100 * (1 + 100 * 3)))),
      pngChunk("IEND", Buffer.alloc(0)),
    ]),
  );
  const wav = Buffer.alloc(364);
  wav.write("RIFF");
  wav.writeUInt32LE(wav.length - 8, 4);
  wav.write("WAVEfmt ", 8);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(16000, 24);
  wav.writeUInt32LE(32000, 28);
  wav.writeUInt16LE(2, 32);
  wav.writeUInt16LE(16, 34);
  wav.write("data", 36);
  wav.writeUInt32LE(wav.length - 44, 40);
  await fs.writeFile(path.join(consumer, "hola.wav"), wav);
  const readme = await fs.readFile(path.join(root, "README.md"), "utf8");
  const examples = [...readme.matchAll(/```js\r?\n([\s\S]*?)```/g)];
  assert.ok(examples.length >= 3, "Expected runnable README examples");
  for (const [index, match] of examples.entries()) {
    const filename = `readme-${index}.mjs`;
    await fs.writeFile(path.join(consumer, filename), match[1]);
    await run([filename], consumer, cleanEnv);
  }
  console.log(`Installed README examples: ${examples.length} passed`);
  async function permissions(directory, readonly) {
    for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
      const filename = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        if (!readonly) await fs.chmod(filename, 0o755);
        await permissions(filename, readonly);
        if (readonly) await fs.chmod(filename, 0o555);
      } else if (entry.isFile())
        await fs.chmod(filename, readonly ? 0o444 : 0o644);
    }
  }
  if (process.platform !== "win32") {
    const modules = path.join(consumer, "node_modules");
    try {
      await permissions(modules, true);
      await fs.chmod(modules, 0o555);
      await run(["smoke.mjs"], consumer, cleanEnv);
      console.log("Read-only node_modules: passed");
    } finally {
      await fs.chmod(modules, 0o755);
      await permissions(modules, false);
    }
  }
  for (const extension of ["mts", "cts"]) {
    await fs.writeFile(
      path.join(consumer, `consumer.${extension}`),
      `
      import { Project, Note, Field, Template, NoteType, type BuildOptions, type BuildReport, type InspectLimits } from 'anki-forge-node';
      const project = new Project('Typed');
      project.addNote(Note.basic('front', 'back'));
      const options: BuildOptions = { output: 'typed.apkg' };
      const result: Promise<BuildReport> = project.build(options);
      // @ts-expect-error unknown product options must be rejected
      project.build({ output: 'x.apkg', launcherExecutable: 'cargo' });
      // @ts-expect-error fields and templates are distinct opaque values
      const field: Field = new Template('Card', { front: '{{Front}}', back: '{{Back}}' });
      const limits: InspectLimits = { maxMediaBytes: 1024 };
      // @ts-expect-error note types require Field objects
      NoteType.custom('bad', { fields: ['Front'], templates: [] });
    `,
    );
    await run(
      [
        path.join(root, "toolchain/node_modules/typescript/bin/tsc"),
        "--strict",
        "--noEmit",
        "--target",
        "ES2022",
        "--module",
        "NodeNext",
        "--moduleResolution",
        "NodeNext",
        "--typeRoots",
        path.join(root, "toolchain/node_modules/@types"),
        `consumer.${extension}`,
      ],
      consumer,
    );
  }
  console.log("Installed TypeScript ESM + CJS declarations: passed");
  await fs.rm(
    path.join(consumer, "node_modules", `anki-forge-node-${host.suffix}`),
    {
      recursive: true,
    },
  );
  const missing = await run(
    [
      "--input-type=module",
      "-e",
      `import { Project, NativeLoadError } from 'anki-forge-node';
    try { new Project('Missing'); process.exit(2); } catch (error) {
      if (!(error instanceof NativeLoadError) || !error.message.includes('--include=optional')) process.exit(3);
    }`,
    ],
    consumer,
    cleanEnv,
  );
  assert.equal(missing, "");
  console.log("Missing optional runtime reports actionable error: passed");
  const wrongRuntime = path.join(
    consumer,
    "node_modules",
    `anki-forge-node-${host.suffix}`,
  );
  await fs.mkdir(wrongRuntime);
  await fs.writeFile(
    path.join(wrongRuntime, "package.json"),
    '{"main":"index.cjs"}',
  );
  await fs.writeFile(
    path.join(wrongRuntime, "index.cjs"),
    `exports.NativeProject = class {}; exports.bindingMetadata = () => JSON.stringify({bindingVersion:'0.0.0'});`,
  );
  await run(
    [
      "--input-type=module",
      "-e",
      `import { Project, NativeLoadError } from 'anki-forge-node';
    try { new Project('Wrong version'); process.exit(2); } catch (error) {
      if (!(error instanceof NativeLoadError) || !error.message.includes('does not match SDK')) process.exit(3);
    }`,
    ],
    consumer,
    cleanEnv,
  );
  console.log("Mismatched native version is rejected: passed");
} finally {
  await new Promise((resolve) => server.close(resolve));
  await fs.rm(temporary, { recursive: true, force: true });
}
