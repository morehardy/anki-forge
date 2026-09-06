import type { CoreBuildReport, CoreDiffReport, Diagnostic } from './types';
import { BuildError, BindingProtocolError, ValidationError, ProjectDiffError } from './errors';
import { deepFreeze } from './internal/validation';

export function diagnostics(value: unknown): readonly Diagnostic[] {
  if (
    !Array.isArray(value) ||
    value.some(
      (item) =>
        !item ||
        typeof item.code !== 'string' ||
        typeof item.message !== 'string' ||
        !['info', 'warning', 'error'].includes(item.severity),
    )
  )
    throw new BindingProtocolError('Invalid native diagnostics');
  return deepFreeze(value as Diagnostic[]);
}

export class ProjectDiffReport {
  readonly raw: CoreDiffReport;
  constructor(value: unknown) {
    if (
      !value ||
      typeof value !== 'object' ||
      !['success', 'blocked', 'invalid', 'error'].includes((value as CoreDiffReport).status)
    )
      throw new BindingProtocolError('Invalid native diff report');
    diagnostics((value as CoreDiffReport).diagnostics);
    this.raw = deepFreeze(value as CoreDiffReport);
    Object.freeze(this);
  }
  get status() {
    return this.raw.status;
  }
  get comparison() {
    return this.raw.comparison;
  }
  get diagnostics() {
    return this.raw.diagnostics;
  }
  get currentInspect() {
    return this.raw.current_inspect;
  }
  get previousInspect() {
    return this.raw.previous_inspect;
  }
  get updateSafety() {
    return this.raw.update_safety;
  }
  get diff() {
    return this.raw.diff;
  }
  get risk() {
    return this.raw.risk;
  }
  get metrics() {
    return this.raw.metrics;
  }
  ensureSuccess(): void {
    if (this.status !== 'success' || this.diagnostics.some((item) => item.severity === 'error'))
      throw new ProjectDiffError(
        'Comparison failed',
        this.diagnostics[0]?.code ?? 'PROJECT.DIFF_FAILED',
        this,
      );
  }
}

export class ValidationReport {
  readonly diagnostics: readonly Diagnostic[];
  constructor(items: unknown) {
    this.diagnostics = diagnostics(items);
    Object.freeze(this);
  }
  get hasErrors(): boolean {
    return this.diagnostics.some((item) => item.severity === 'error');
  }
  get warningCount(): number {
    return this.diagnostics.filter((item) => item.severity === 'warning').length;
  }
  ensureSuccess(): void {
    if (this.hasErrors) throw new ValidationError(this);
  }
}

export class BuildReport {
  readonly raw: CoreBuildReport;
  constructor(
    raw: unknown,
    private readonly pretty: string,
  ) {
    if (!raw || typeof raw !== 'object')
      throw new BindingProtocolError('Missing native build report');
    const value = raw as CoreBuildReport;
    if (
      value.kind !== 'anki-forge-build-report' ||
      value.schema_version !== 'phase4-build-report-v2' ||
      !['success', 'blocked', 'invalid', 'error'].includes(value.status) ||
      !value.counts ||
      !['notes', 'cards', 'media'].every((key) =>
        Number.isSafeInteger(value.counts[key as keyof typeof value.counts]),
      )
    )
      throw new BindingProtocolError('Invalid native build report');
    diagnostics(value.diagnostics);
    this.raw = deepFreeze(value);
    Object.freeze(this);
  }
  get status() {
    return this.raw.status;
  }
  get comparison() {
    return this.raw.comparison;
  }
  get artifact() {
    return this.raw.artifact;
  }
  get counts() {
    return this.raw.counts;
  }
  get diagnostics() {
    return this.raw.diagnostics;
  }
  get media() {
    return this.raw.media;
  }
  get metrics() {
    return this.raw.metrics;
  }
  get policy() {
    return this.raw.policy;
  }
  get inspect() {
    return this.raw.inspect;
  }
  get previousInspect() {
    return this.raw.previous_inspect;
  }
  get diff() {
    return this.raw.diff;
  }
  get risk() {
    return this.raw.risk;
  }
  get updateSafety() {
    return this.raw.update_safety;
  }
  get warningCount(): number {
    return this.diagnostics.filter((item) => item.severity === 'warning').length;
  }
  get diagnosticCodes(): readonly string[] {
    return this.diagnostics.map((item) => item.code);
  }
  prettyReport(): string {
    return this.pretty;
  }
  ensureSuccess(): void {
    const failureCause = this.diagnostics.some((item) => item.severity === 'error')
      ? 'Diagnostics'
      : this.status === 'invalid'
        ? 'Invalid'
        : this.status === 'blocked'
          ? 'PolicyBlocked'
          : this.status === 'error'
            ? 'Internal'
            : !this.artifact
              ? 'MissingArtifact'
              : undefined;
    if (!failureCause) return;
    const codes = {
      Diagnostics: 'PROJECT.BUILD_DIAGNOSTICS',
      Invalid: 'PROJECT.BUILD_INVALID',
      PolicyBlocked: 'PROJECT.BUILD_POLICY_BLOCKED',
      Internal: 'PROJECT.BUILD_INTERNAL',
      MissingArtifact: 'PROJECT.BUILD_MISSING_ARTIFACT',
    };
    const code =
      this.diagnostics.find((item) => item.severity === 'error')?.code ??
      this.diagnostics[0]?.code ??
      codes[failureCause];
    throw new BuildError(`Build ${this.status}`, code, this, failureCause);
  }
}
