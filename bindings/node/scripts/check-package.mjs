import fs from "node:fs/promises";
import path from "node:path";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { root, targets } from "./platforms.mjs";

const manifest = JSON.parse(
  await fs.readFile(path.join(root, "package.json"), "utf8"),
);
const cargo = await fs.readFile(path.join(root, "native/Cargo.toml"), "utf8");
const packageSection = cargo.split("[package]")[1].split(/^\[/m)[0];
assert.equal(packageSection.match(/^version\s*=\s*"([^"]+)"/m)?.[1], manifest.version);
const loader = await fs.readFile(path.join(root, "src/internal/native.ts"), "utf8");
assert.equal(loader.match(/export const VERSION = "([^"]+)"/)?.[1], manifest.version);
assert.equal(manifest.name, "ankiforge");
assert.deepEqual(manifest.optionalDependencies, Object.fromEntries(
  targets.map(target => [`${manifest.name}-${target.suffix}`, manifest.version]),
));
const require = createRequire(import.meta.url);
const all = process.argv.includes("--all");
for (const target of targets) {
  const directory = path.join(root, "npm", target.suffix);
  const platform = JSON.parse(
    await fs.readFile(path.join(directory, "package.json"), "utf8"),
  );
  assert.equal(platform.version, manifest.version);
  assert.equal(platform.name, `${manifest.name}-${target.suffix}`);
  assert.deepEqual(platform.os, [target.os]);
  assert.deepEqual(platform.cpu, [target.cpu]);
  assert.deepEqual(platform.libc, target.libc ? [target.libc] : undefined);
  assert.deepEqual(platform.publishConfig, manifest.publishConfig);
  assert.equal(manifest.optionalDependencies[platform.name], manifest.version);
  const binary = path.join(directory, "anki-forge.node");
  const host = target.os === process.platform && target.cpu === process.arch;
  if (all || host)
    assert.ok(
      (await fs.stat(binary)).size > 0,
      `Missing ${target.target} binary`,
    );
  if (host) {
    const metadata = JSON.parse(require(binary).bindingMetadata());
    assert.equal(metadata.bindingVersion, manifest.version);
    assert.equal(metadata.target, target.target);
    assert.equal(metadata.nodeApiVersion, 8);
    assert.equal(metadata.bindingProtocolVersion, 6);
  }
  assert.equal(platform.scripts, undefined);
  assert.ok(platform.files.includes("THIRD_PARTY_NOTICES.md"));
  assert.equal(
    await fs.readFile(path.join(directory, "THIRD_PARTY_NOTICES.md"), "utf8"),
    await fs.readFile(path.join(root, "THIRD_PARTY_NOTICES.md"), "utf8"),
  );
}
for (const hook of ["preinstall", "install", "postinstall", "prepare", "prepack", "postpack", "prepublish", "prepublishOnly", "publish", "postpublish"])
  assert.equal(manifest.scripts?.[hook], undefined, `Unexpected ${hook} hook`);
for (const file of [
  "dist/index.mjs",
  "dist/index.d.mts",
  "dist/cjs/index.js",
  "dist/cjs/index.d.ts",
  "README.md",
  "LICENSE",
  "THIRD_PARTY_NOTICES.md",
])
  await fs.access(path.join(root, file));
console.log(
  `Package metadata, versions and ${all ? "all" : "host"} native artifacts: passed`,
);
