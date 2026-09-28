import path from "node:path";

/** Anchor a native path without erasing symlinks followed by `..`. */
export function absolutePath(filename: string, baseDir = process.cwd()): string {
  if (filename === "") return filename;
  const root = path.parse(filename).root;
  const rootedWithoutDrive =
    process.platform === "win32" && (root === "\\" || root === "/");
  if (path.isAbsolute(filename) && !rootedWithoutDrive) return filename;

  // On Windows, resolve only the drive/root prefix (C: or \), preserving
  // every supplied path component. Other relative paths just need the base.
  const base = root ? path.resolve(baseDir, root) : baseDir;
  const separator = base.endsWith(path.sep) ? "" : path.sep;
  return base + separator + filename.slice(root.length);
}
