import type { Diagnostic } from './types';
import type { BuildReport, ValidationReport, ProjectDiffReport } from './report';

export class NativeLoadError extends Error {
  readonly code = 'BINDING.NATIVE_LOAD_FAILED';
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = 'NativeLoadError';
  }
}
export class BindingProtocolError extends Error {
  readonly code = 'BINDING.PROTOCOL_ERROR';
  constructor(message: string) {
    super(message);
    this.name = 'BindingProtocolError';
  }
}
export class ProjectBusyError extends Error {
  readonly code = 'BINDING.PROJECT_BUSY';
  constructor() {
    super('This project has an operation in progress. Await it before using the project again.');
    this.name = 'ProjectBusyError';
  }
}
export class ProjectFailedError extends Error {
  readonly code = 'BINDING.PROJECT_FAILED';
  constructor() {
    super('The native project encountered an unrecoverable error. Create a new project.');
    this.name = 'ProjectFailedError';
  }
}
export class ProjectAddError extends Error {
  readonly code: string;
  constructor(readonly diagnostic: Diagnostic) {
    super(diagnostic.message);
    this.name = 'ProjectAddError';
    this.code = diagnostic.code;
  }
}
export class BuildError extends Error {
  constructor(
    message: string,
    readonly code: string,
    readonly report: BuildReport,
    readonly failureCause?: string,
  ) {
    super(message);
    this.name = 'BuildError';
  }
}
export class MediaError extends Error {
  constructor(
    message: string,
    readonly code: string,
  ) {
    super(message);
    this.name = 'MediaError';
  }
}
export class ProjectDiffError extends Error {
  constructor(
    message: string,
    readonly code: string,
    readonly report: ProjectDiffReport,
    readonly failureCause?: string,
  ) {
    super(message);
    this.name = 'ProjectDiffError';
  }
}
export class ProductNoteError extends Error {
  constructor(
    message: string,
    readonly code: string,
  ) {
    super(message);
    this.name = 'ProductNoteError';
  }
}
export class DeckError extends Error {
  constructor(
    message: string,
    readonly code: string,
  ) {
    super(message);
    this.name = 'DeckError';
  }
}
export class TemplateBundleError extends Error {
  constructor(
    message: string,
    readonly code: string,
    readonly path: string | null,
    readonly byteOffset: number | null,
  ) {
    super(message);
    this.name = 'TemplateBundleError';
  }
}
export class ValidationError extends Error {
  readonly code: string;
  constructor(readonly report: ValidationReport) {
    const diagnostic = report.diagnostics.find((item) => item.severity === 'error');
    super(diagnostic?.message ?? 'Validation failed');
    this.name = 'ValidationError';
    this.code = diagnostic?.code ?? 'BINDING.VALIDATION_FAILED';
  }
}

export function nativeError(error: unknown): never {
  if (error instanceof Error && error.message === 'BINDING.PROJECT_BUSY')
    throw new ProjectBusyError();
  if (error instanceof Error && error.message === 'BINDING.PROJECT_FAILED')
    throw new ProjectFailedError();
  throw error;
}
