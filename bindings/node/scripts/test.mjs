import path from "node:path";
import { spawnSync } from "node:child_process";
import { root, targets } from "./platforms.mjs";
const platform = targets.find(
  (p) => p.os === process.platform && p.cpu === process.arch,
);
if (!platform) throw new Error("Unsupported test platform");
const build = spawnSync(
  "cargo",
  [
    "build",
    "--offline",
    "--locked",
    "-p",
    "anki_forge_node_native",
    "--example",
    "sdk_parity",
    "--message-format=json",
  ],
  { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] },
);
if (build.status !== 0) process.exit(build.status ?? 1);
const observer = build.stdout
  .trim()
  .split("\n")
  .map((l) => JSON.parse(l))
  .find((x) => x.target?.name === "sdk_parity" && x.executable)?.executable;
if (!observer) throw new Error("Missing semantic observer");
const result = spawnSync(
  process.execPath,
  ["--expose-gc", "--test", "test/public-api.test.mjs"],
  {
    cwd: root,
    stdio: "inherit",
    env: {
      ...process.env,
      ANKI_FORGE_NATIVE_PATH: path.join(
        root,
        "npm",
        platform.suffix,
        "anki-forge.node",
      ),
      ANKI_FORGE_TEST_OBSERVER: observer,
    },
  },
);
process.exitCode = result.status ?? 1;
