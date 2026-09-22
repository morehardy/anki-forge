import path from 'node:path';
import type { NativeApkgArtifact } from './internal/native';
import { nativeError } from './errors';
import { string } from './internal/validation';

let adopt: (native: NativeApkgArtifact, baseDir: string) => ApkgArtifact;
const token = Symbol('artifact');

/** Owns an APKG independently of its path and the project that built it. */
export class ApkgArtifact {
  readonly path: string;
  #handle: NativeApkgArtifact;
  #baseDir: string;

  private constructor(key: symbol, handle: NativeApkgArtifact, baseDir: string) {
    if (key !== token)
      throw new TypeError('Obtain artifact handles from a native build report');
    this.#handle = handle;
    this.#baseDir = baseDir;
    this.path = handle.path;
    Object.freeze(this);
  }

  static {
    adopt = (handle, baseDir) => new ApkgArtifact(token, handle, baseDir);
  }

  clone(): ApkgArtifact {
    try {
      return adopt(this.#handle.cloneHandle(), this.#baseDir);
    } catch (error) {
      nativeError(error);
    }
  }

  async persistTo(filename: string): Promise<ApkgArtifact> {
    string(filename, 'filename');
    try {
      return adopt(
        await this.#handle.persistTo(path.resolve(this.#baseDir, filename)),
        this.#baseDir,
      );
    } catch (error) {
      nativeError(error);
    }
  }

  async close(): Promise<void> {
    try {
      await this.#handle.close();
    } catch (error) {
      nativeError(error);
    }
  }
}

/** @internal Only native build results may supply an owned artifact. */
export function artifactFromNative(handle: NativeApkgArtifact, baseDir: string): ApkgArtifact {
  return adopt(handle, baseDir);
}
