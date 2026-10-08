import { deepFreeze } from "./internal/validation";
import { BuildReport } from "./report";
import type {
  BuildSnapshot,
  ReportSnapshot,
  PublicationSnapshot,
  PathSnapshot,
} from "./snapshots";
export interface ErrorSourceDetail {
  readonly type: string;
  readonly kind?: string;
  readonly code?: string;
  readonly path?: PathSnapshot | null;
  readonly [key: string]: unknown;
}
/** Location in the caller's original note/content, before rendering. */
export type AddTarget =
  | { readonly type: "note_key" | "note" | "model" | "unknown" }
  | { readonly type: "field"; readonly fieldKey: string; readonly contentPath: readonly number[] | null; readonly byteRange: { readonly start: number; readonly end: number } | null }
  | { readonly type: "tag"; readonly index: number; readonly value: string }
  | { readonly type: "deck"; readonly name: string; readonly inherited: boolean }
  | { readonly type: "model_asset" | "occlusion_image" | "explicit_asset"; readonly mediaName: string }
  | { readonly type: "project_default_deck"; readonly name: string };
export interface AddContext {
  readonly noteKey: string | null;
  readonly modelKey: string | null;
  readonly target: AddTarget;
}
export type MediaUsage = "image" | "sound" | "unknown";
export type MediaConflictKind = "portable_name_collision" | "different_content" | "unknown";
export type AddDetail =
  | { readonly type: "model_definition_conflict" | "unknown" }
  | { readonly type: "model_name_conflict"; readonly existingModelKey: string; readonly conflictingName: string }
  | { readonly type: "media_conflict"; readonly kind: MediaConflictKind; readonly existingName: string; readonly incomingName: string }
  | { readonly type: "media_usage"; readonly requested: MediaUsage; readonly mediaName: string; readonly mediaType: string };
/** Deeply frozen facts; null detail means no additional conflict evidence. */
export interface AddErrorDetails extends Readonly<Record<string, unknown>> {
  readonly context: AddContext;
  readonly detail: AddDetail | null;
}
export class NativeLoadError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "NativeLoadError";
  }
}
export class ForgeError extends Error {
  readonly kind: string;
  readonly code: string;
  readonly domain: string;
  readonly causes: readonly string[];
  readonly sourceDetails: readonly ErrorSourceDetail[];
  readonly details: Readonly<Record<string, unknown>>;
  constructor(
    data: {
      message: string;
      kind: string;
      code: string;
      domain: string;
      causes?: string[];
      sourceDetails?: ErrorSourceDetail[];
      details?: Record<string, unknown>;
    },
    cause?: unknown,
  ) {
    super(data.message, { cause });
    this.name = new.target.name;
    this.kind = data.kind;
    this.code = data.code;
    this.domain = data.domain;
    this.causes = Object.freeze(data.causes ?? []);
    this.sourceDetails = deepFreeze(data.sourceDetails ?? []);
    this.details = deepFreeze(data.details ?? {});
  }
}
export class SchemaError extends ForgeError {}
export class AddError extends ForgeError {
  declare readonly details: AddErrorDetails;
}
export class MediaError extends ForgeError {}
export class TemplateBundleError extends ForgeError {}
export class ImageOcclusionError extends ForgeError {}
export class PolicyError extends ForgeError {}
export class ConfigurationError extends ForgeError {}
export class CompareError extends ForgeError {
  get report(): BuildReport {
    return new BuildReport(this.details.report as ReportSnapshot);
  }
}
export class BuildError extends ForgeError {
  snapshot(): BuildSnapshot {
    return this.details.snapshot as BuildSnapshot;
  }
  get report(): BuildReport {
    return new BuildReport(this.snapshot().report);
  }
}
export class PersistError extends ForgeError {
  get publication(): PublicationSnapshot {
    return this.details.publication as PublicationSnapshot;
  }
}
export class PreparedPublicationStateError extends ForgeError {
  get reason(): "closed" | "consumed" { return this.details.reason as "closed" | "consumed"; }
}
export class ArtifactClosedError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "ArtifactClosedError";
  }
  readonly code = "BINDING.ARTIFACT_CLOSED";
  readonly kind = "Closed";
}
export function nativeError(error: unknown): never {
  if (error instanceof ForgeError) throw error;
  const message = error instanceof Error ? error.message : String(error);
  if (message === "BINDING.ARTIFACT_CLOSED")
    throw new ArtifactClosedError(message, { cause: error });
  let data;
  try {
    data = JSON.parse(message);
  } catch {
    throw error;
  }
  if (!data || typeof data.code !== "string") throw error;
  const constructors: Record<string, typeof ForgeError> = {
    schema: SchemaError,
    add: AddError,
    media: MediaError,
    bundle: TemplateBundleError,
    occlusion: ImageOcclusionError,
    policy: PolicyError,
    configuration: ConfigurationError,
    compare: CompareError,
    build: BuildError,
    persist: PersistError,
    prepared: PreparedPublicationStateError,
  };
  throw new (constructors[data.domain] ?? ForgeError)(data, error);
}
export function call<T>(fn: () => T): T {
  try {
    return fn();
  } catch (e) {
    return nativeError(e);
  }
}
export async function asyncCall<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    return nativeError(e);
  }
}
