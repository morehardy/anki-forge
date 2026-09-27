import path from "node:path";
import { NativeLoadError } from "../errors";
export const VERSION = "0.2.0";
export interface BindingMetadata {
  readonly bindingVersion: string;
  readonly bindingProtocolVersion: number;
}
export interface NativeContent {}
export interface NativeMedia {
  readonly filename: string;
  readonly mediaType: string;
  readonly byteLength: number;
  withExportName(name: string): NativeMedia;
  image(): NativeContent;
  sound(): NativeContent;
}
export interface NativeNote {
  field(key: string, value: NativeContent): NativeNote;
  deck(name: string): NativeNote;
  tag(tag: string): NativeNote;
}
export interface NativeNoteType {
  readonly key: string;
  readonly displayName: string;
  note(): NativeNote;
}
export interface NativeProject {
  name(name: string): void;
  defaultDeck(name: string): void;
  add(key: string, note: NativeNote): void;
  addAsset(media: NativeMedia): void;
  readonly length: number;
  cloneState(): NativeProject;
  build(
    input: string,
  ): Promise<{ snapshot: string; artifact: NativeApkgArtifact }>;
  compare(input: string): Promise<string>;
}
export interface NativeApkgArtifact {
  readonly path: string;
  cloneHandle(): NativeApkgArtifact;
  persistTo(path: string): Promise<NativeApkgArtifact>;
  close(): Promise<void>;
}
interface NativeModule {
  bindingMetadata(): string;
  NativeContent: {
    text(value: string): NativeContent;
    html(value: string): NativeContent;
    sequence(items: NativeContent[]): NativeContent;
  };
  NativeMedia: {
    file(path: string, limits: string): Promise<NativeMedia>;
    bytes(bytes: Buffer, mime: string, limits: string): Promise<NativeMedia>;
  };
  NativeNoteType: {
    build(input: string, assets: NativeMedia[]): NativeNoteType;
    fromBundle(path: string, limits: string): Promise<NativeNoteType>;
  };
  NativeNote: {
    basic(front: NativeContent, back: NativeContent): NativeNote;
    cloze(text: NativeContent): NativeNote;
    imageOcclusion(image: NativeMedia, masks: readonly {
      readonly key: string;
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    }[], mode: string): NativeNote;
  };
  NativeProject: new (namespace: string) => NativeProject;
}
let loaded: NativeModule | undefined;

export function platformSuffix(): string {
  if (process.platform === "darwin" && ["arm64", "x64"].includes(process.arch))
    return `darwin-${process.arch}`;
  if (process.platform === "win32" && process.arch === "x64")
    return "win32-x64-msvc";
  if (process.platform === "linux" && process.arch === "x64") {
    const report = process.report?.getReport() as
      { header?: { glibcVersionRuntime?: string } } | undefined;
    if (report?.header?.glibcVersionRuntime) return "linux-x64-gnu";
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
      "ANKI_FORGE_NATIVE_PATH must be an absolute development artifact path.",
    );
  const packageName = `anki-forge-node-${suffix}`;
  try {
    const binding = require(override ?? packageName) as NativeModule;
    if (
      typeof binding.NativeProject !== "function" ||
      typeof binding.bindingMetadata !== "function"
    )
      throw new Error("Invalid native module exports");
    const metadata: BindingMetadata = JSON.parse(binding.bindingMetadata());
    if (metadata.bindingVersion !== VERSION)
      throw new Error(
        `Native version ${metadata.bindingVersion} does not match SDK ${VERSION}`,
      );
    if (metadata.bindingProtocolVersion !== 4)
      throw new Error(
        `Native protocol ${metadata.bindingProtocolVersion ?? "missing"} does not match SDK protocol 4`,
      );
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
