import path from 'node:path';
import { Buildable } from './buildable';
import { native, type NativeBuildable } from './internal/native';
import { options, string, strings } from './internal/validation';
import { outcome } from './internal/outcome';
import { nativeError } from './errors';
import type { IoMode, Rect } from './types';

export interface DeckOptions {
  baseDir?: string;
  stableId?: string;
  basicIdentity?: readonly ('front' | 'back')[];
}
export interface DeckNoteOptions {
  stableId?: string;
  tags?: readonly string[];
}
export interface DeckBasicOptions extends DeckNoteOptions {
  identityOverride?: { fields: readonly ('front' | 'back')[]; reasonCode: string };
}
export interface DeckClozeOptions extends DeckNoteOptions {
  extra?: string;
}
export interface DeckImageOcclusionOptions extends DeckNoteOptions {
  mode?: IoMode;
  rects: readonly Rect[];
  header?: string;
  backExtra?: string;
  comments?: string;
}
const references = new WeakSet<DeckMediaRef>();
const token = Symbol('deck-media');
export class DeckMediaRef {
  /** @internal Obtained by registering media on a Deck. */
  constructor(
    key: symbol,
    readonly filename: string,
  ) {
    if (key !== token) throw new TypeError('Obtain media from Deck.media');
    references.add(this);
    Object.freeze(this);
  }
}
function identityFields(value: unknown): void {
  strings(value, 'identity fields');
  if (value.some((field) => !['front', 'back'].includes(field)))
    throw new TypeError('Deck identity fields must be front or back');
}
function noteOptions(config: DeckNoteOptions, extraKeys: string[]): void {
  options(config, ['stableId', 'tags', ...extraKeys], 'deck note');
  if (config.stableId !== undefined) string(config.stableId, 'stableId');
  if (config.tags !== undefined) strings(config.tags, 'tags');
}
export class Deck extends Buildable {
  readonly baseDir: string;
  readonly media: {
    addFile: (filename: string) => Promise<DeckMediaRef>;
    addBytes: (name: string, bytes: Uint8Array) => Promise<DeckMediaRef>;
  };
  #deck: InstanceType<ReturnType<typeof native>['NativeDeck']>;
  constructor(
    readonly name: string,
    config: DeckOptions = {},
  ) {
    super();
    string(name, 'name');
    options(config, ['baseDir', 'stableId', 'basicIdentity'], 'deck');
    if (config.stableId !== undefined) string(config.stableId, 'stableId');
    const baseDir = config.baseDir ?? process.cwd();
    string(baseDir, 'baseDir');
    this.baseDir = path.resolve(baseDir);
    if (config.basicIdentity !== undefined) identityFields(config.basicIdentity);
    try {
      this.#deck = new (native().NativeDeck)(
        name,
        JSON.stringify({ stableId: config.stableId, basicIdentity: config.basicIdentity }),
      );
    } catch (error) {
      if (error instanceof Error && error.message.startsWith('{')) outcome(error.message);
      throw error;
    }
    const reference = (input: string) => {
      const result = outcome(input);
      string(result.filename, 'native media filename');
      return new DeckMediaRef(token, result.filename);
    };
    this.media = Object.freeze({
      addFile: async (filename: string) => {
        string(filename, 'filename');
        try {
          return reference(await this.#deck.addMediaFile(path.resolve(this.baseDir, filename)));
        } catch (error) {
          nativeError(error);
        }
      },
      addBytes: async (name: string, bytes: Uint8Array) => {
        string(name, 'name');
        if (!(bytes instanceof Uint8Array)) throw new TypeError('Expected Buffer or Uint8Array');
        try {
          return reference(await this.#deck.addMediaBytes(name, Buffer.from(bytes)));
        } catch (error) {
          nativeError(error);
        }
      },
    });
    Object.freeze(this);
  }
  basic(front: string, back: string, config: DeckBasicOptions = {}): void {
    string(front, 'front');
    string(back, 'back');
    noteOptions(config, ['identityOverride']);
    if (config.identityOverride !== undefined) {
      options(config.identityOverride, ['fields', 'reasonCode'], 'identityOverride');
      identityFields(config.identityOverride.fields);
      string(config.identityOverride.reasonCode, 'reasonCode');
    }
    try {
      outcome(this.#deck.addBasic(front, back, JSON.stringify(config)));
    } catch (error) {
      nativeError(error);
    }
  }
  cloze(text: string, config: DeckClozeOptions = {}): void {
    string(text, 'text');
    noteOptions(config, ['extra']);
    if (config.extra !== undefined) string(config.extra, 'extra');
    try {
      outcome(this.#deck.addCloze(text, JSON.stringify(config)));
    } catch (error) {
      nativeError(error);
    }
  }
  imageOcclusion(image: DeckMediaRef, config: DeckImageOcclusionOptions): void {
    if (!references.has(image)) throw new TypeError('Expected a DeckMediaRef');
    noteOptions(config, ['mode', 'rects', 'header', 'backExtra', 'comments']);
    if (!Array.isArray(config.rects)) throw new TypeError('rects must be an array');
    for (const rect of config.rects) {
      options(rect, ['x', 'y', 'width', 'height'], 'rect');
      for (const key of ['x', 'y', 'width', 'height'])
        if (!Number.isInteger(rect[key]) || Number(rect[key]) < 0 || Number(rect[key]) > 0xffffffff)
          throw new TypeError(`rect.${key} must be a uint32`);
    }
    if (
      config.mode !== undefined &&
      !['hide-all-guess-one', 'hide-one-guess-one'].includes(config.mode)
    )
      throw new TypeError('Invalid image occlusion mode');
    for (const key of ['header', 'backExtra', 'comments'] as const)
      if (config[key] !== undefined) string(config[key], key);
    try {
      outcome(
        this.#deck.addImageOcclusion(
          image.filename,
          JSON.stringify({ tags: [], mode: 'hide-all-guess-one', ...config }),
        ),
      );
    } catch (error) {
      nativeError(error);
    }
  }
  protected nativeProject(): NativeBuildable {
    return this.#deck;
  }
}
