import fs from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { root } from "./platforms.mjs";
await fs.rm(path.join(root, "dist"), { recursive: true, force: true });
const result = spawnSync(
  process.execPath,
  ["toolchain/node_modules/typescript/bin/tsc", "-p", "tsconfig.json"],
  { cwd: root, stdio: "inherit" },
);
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
await import("./entries.mjs");
