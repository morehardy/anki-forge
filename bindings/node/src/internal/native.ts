import path from 'node:path';
import { NativeLoadError } from '../errors';
import type { BindingMetadata } from '../types';

export const VERSION = '0.2.0';
export interface NativeMediaRef {
  renderImage(): string;
  renderSound(): string;
}

export interface NativeBuildable {
  validate(): Promise<string>;
  build(input: string): Promise<string>;
  apkgBytes(): Promise<{ result: string; data: Buffer }>;
  diffAgainstApkg(path: string, limits: string): Promise<string>;
}
interface NativeProject extends NativeBuildable {
  addNote(input: string, references: NativeMediaRef[]): string;
  mediaRef(filename: string): NativeMediaRef;
  addNoteType(input: string): string;
  addMediaFile(path: string, exportAs: string): Promise<string>;
  addMediaBytes(label: string, exportAs: string, bytes: Buffer, spool: boolean): Promise<string>;
  importTemplateBundle(path: string): Promise<string>;
  apkgBytes(): Promise<{ result: string; data: Buffer }>;
  diffAgainstApkg(path: string, limits: string): Promise<string>;
  validate(): Promise<string>;
  build(input: string): Promise<string>;
}
interface NativeModule {
  bindingMetadata(): string;
  renderContent(text: string, html: boolean): string;
  validateTemplate(source: string, fields: string[]): string;
  defaultInspectLimits(): string;
  NativeProject: new (name: string, options: string) => NativeProject;
  NativeDeck: new (name: string, options: string) => NativeDeck;
}
interface NativeDeck extends NativeBuildable {
  addBasic(front: string, back: string, input: string): string;
  addCloze(text: string, input: string): string;
  addImageOcclusion(filename: string, input: string): string;
  addMediaFile(filename: string): Promise<string>;
  addMediaBytes(name: string, bytes: Buffer): Promise<string>;
}
let loaded: NativeModule | undefined;

export function platformSuffix(): string {
  if (process.platform === 'darwin' && ['arm64', 'x64'].includes(process.arch))
    return `darwin-${process.arch}`;
  if (process.platform === 'win32' && process.arch === 'x64') return 'win32-x64-msvc';
  if (process.platform === 'linux' && process.arch === 'x64') {
    const report = process.report?.getReport() as
      | { header?: { glibcVersionRuntime?: string } }
      | undefined;
    if (report?.header?.glibcVersionRuntime) return 'linux-x64-gnu';
  }
  throw new NativeLoadError(
    `Unsupported platform: ${process.platform}/${process.arch}. Supported: macOS arm64/x64, Windows x64, Linux x64 glibc.`,
  );
}

export function native(): NativeModule {
  if (loaded) return loaded;
  const suffix = platformSuffix();
  const override = process.env.ANKI_FORGE_NATIVE_PATH;
  if (override && !path.isAbsolute(override))
    throw new NativeLoadError(
      'ANKI_FORGE_NATIVE_PATH must be an absolute development artifact path.',
    );
  const packageName = `anki-forge-node-${suffix}`;
  try {
    const binding = require(override ?? packageName) as NativeModule;
    if (
      typeof binding.NativeProject !== 'function' ||
      typeof binding.bindingMetadata !== 'function'
    )
      throw new Error('Invalid native module exports');
    const metadata: BindingMetadata = JSON.parse(binding.bindingMetadata());
    if (metadata.bindingVersion !== VERSION)
      throw new Error(`Native version ${metadata.bindingVersion} does not match SDK ${VERSION}`);
    loaded = binding;
    return binding;
  } catch (cause) {
    throw new NativeLoadError(
      `Could not load ${packageName}@${VERSION}. Install with optional dependencies enabled (npm install --include=optional). ${cause instanceof Error ? cause.message : String(cause)}`,
      { cause },
    );
  }
}

export function bindingMetadata(): BindingMetadata {
  return Object.freeze(JSON.parse(native().bindingMetadata()));
}
