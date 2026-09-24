import { deepFreeze } from "./internal/validation";
import type {
  BuildSnapshot,
  ReportSnapshot,
  PublicationSnapshot,
} from "./snapshots";
export interface ErrorSourceDetail {
  readonly type: string;
  readonly kind?: string;
  readonly code?: string;
  readonly [key: string]: unknown;
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
export class AddError extends ForgeError {}
export class MediaError extends ForgeError {}
export class TemplateBundleError extends ForgeError {}
export class ImageOcclusionError extends ForgeError {}
export class PolicyError extends ForgeError {}
export class ConfigurationError extends ForgeError {}
export class CompareError extends ForgeError {
  get report(): ReportSnapshot {
    return this.details.report as ReportSnapshot;
  }
}
export class BuildError extends ForgeError {
  snapshot(): BuildSnapshot {
    return this.details.snapshot as BuildSnapshot;
  }
  get report(): ReportSnapshot {
    return this.snapshot().report;
  }
}
export class PersistError extends ForgeError {
  get publication(): PublicationSnapshot {
    return this.details.publication as PublicationSnapshot;
  }
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
