import path from 'node:path';
import { options, string } from './internal/validation';
import { outcome } from './internal/outcome';
import { nativeError } from './errors';
import { Content } from './content';
import type { NativeMediaRef } from './internal/native';

const handles = new WeakMap<MediaRef, NativeMediaRef>();
const token = Symbol('media');
export class MediaRef {
  readonly filename: string;
  /** @internal Only a successful media registration can create a reference. */
  constructor(key: symbol, filename: string, handle: NativeMediaRef) {
    if (key !== token) throw new TypeError('MediaRef must be obtained from project.media');
    this.filename = filename;
    handles.set(this, handle);
    Object.freeze(this);
  }
  image(): Content {
    return Content.image(this);
  }
  sound(): Content {
    return Content.sound(this);
  }
}
export function mediaHandle(media: MediaRef): NativeMediaRef {
  const handle = handles.get(media);
  if (!handle) throw new TypeError('Expected a registered MediaRef');
  return handle;
}
export function mediaFilename(media: MediaRef): string {
  mediaHandle(media);
  return media.filename;
}
export interface MediaOptions {
  exportAs?: string;
}
export interface MediaBackend {
  mediaRef(filename: string): NativeMediaRef;
  addMediaFile(path: string, exportAs: string): Promise<string>;
  addMediaBytes(label: string, exportAs: string, bytes: Buffer, spool: boolean): Promise<string>;
}
export class MediaRegistry {
  /** @internal Obtained from Project.media. */
  constructor(
    private readonly backend: MediaBackend,
    private readonly baseDir: string,
  ) {
    Object.freeze(this);
  }
  async addFile(filename: string, config: MediaOptions = {}): Promise<MediaRef> {
    string(filename, 'filename');
    const exported = this.exportName(filename, config);
    try {
      return this.reference(
        outcome(await this.backend.addMediaFile(path.resolve(this.baseDir, filename), exported)),
      );
    } catch (error) {
      nativeError(error);
    }
  }
  addBytes(sourceLabel: string, bytes: Uint8Array, config: MediaOptions = {}): Promise<MediaRef> {
    return this.bytes(sourceLabel, bytes, config, false);
  }
  addBuffer(sourceLabel: string, bytes: Uint8Array, config: MediaOptions = {}): Promise<MediaRef> {
    return this.bytes(sourceLabel, bytes, config, true);
  }
  private async bytes(
    label: string,
    bytes: Uint8Array,
    config: MediaOptions,
    spool: boolean,
  ): Promise<MediaRef> {
    string(label, 'sourceLabel');
    const exported = this.exportName(label, config);
    if (!(bytes instanceof Uint8Array)) throw new TypeError('bytes must be a Buffer or Uint8Array');
    // Snapshot before the first await; caller mutation cannot change registered data.
    try {
      return this.reference(
        outcome(await this.backend.addMediaBytes(label, exported, Buffer.from(bytes), spool)),
      );
    } catch (error) {
      nativeError(error);
    }
  }
  private exportName(label: string, config: MediaOptions): string {
    options(config, ['exportAs'], 'media');
    const name = config.exportAs ?? path.basename(label);
    string(name, 'exportAs');
    return name;
  }
  private reference(result: Record<string, unknown>): MediaRef {
    string(result.filename, 'native media filename');
    return new MediaRef(token, result.filename, this.backend.mediaRef(result.filename));
  }
}
