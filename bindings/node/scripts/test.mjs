import path from "node:path";
import { spawnSync } from "node:child_process";
import { root, targets } from "./platforms.mjs";
const platform = targets.find(
  (item) => item.os === process.platform && item.cpu === process.arch,
);
if (!platform) throw new Error("Unsupported test platform");
const parity = process.argv.includes("--parity");
const evidenceArg = process.argv.indexOf("--evidence");
if (evidenceArg >= 0 && (!parity || !process.argv[evidenceArg + 1]))
  throw new Error("--evidence requires --parity and an output directory");
let observer;
if (parity) {
  const built = spawnSync(
    "cargo",
    [
      "build",
      "-p",
      "anki_forge_node_native",
      "--example",
      "sdk_parity",
      "--locked",
      "--message-format=json",
    ],
    {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "inherit"],
    },
  );
  if (built.error) throw built.error;
  if (built.status !== 0) process.exit(built.status ?? 1);
  observer = built.stdout
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line))
    .find(
      (item) => item.target?.name === "sdk_parity" && item.executable,
    )?.executable;
  if (!observer) throw new Error("Rust parity observer was not built");
}
const result = spawnSync(
  process.execPath,
  [
    "--expose-gc",
    "--test",
    parity ? "test/parity.test.mjs" : "test/product.test.mjs",
  ],
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
      ...(observer ? { ANKI_FORGE_TEST_OBSERVER: observer } : {}),
      ...(evidenceArg >= 0
        ? {
            ANKI_FORGE_NODE_EVIDENCE_DIR: path.resolve(
              root,
              process.argv[evidenceArg + 1],
            ),
          }
        : {}),
    },
  },
);
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
