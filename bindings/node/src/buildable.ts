import path from 'node:path';
import { Writable } from 'node:stream';
import type { BuildOptions, InspectLimits } from './types';
import type { NativeBuildable } from './internal/native';
import { writeBuffer } from './internal/stream';
import { checkInspectLimits } from './options';
import { options, string } from './internal/validation';
import { BuildReport, ValidationReport, ProjectDiffReport } from './report';
import { nativeError } from './errors';
import { outcome } from './internal/outcome';

const buildKeys = [
  'output',
  'artifactsDir',
  'inspect',
  'compareTo',
  'failOn',
  'reportJson',
  'identityLockfile',
  'writeIdentityLockfile',
  'updateSafety',
  'selfContained',
  'inspectLimits',
  'mediaMode',
  'mediaPolicy',
  'mediaStoreDir',
];
const pathKeys = [
  'output',
  'artifactsDir',
  'compareTo',
  'reportJson',
  'identityLockfile',
  'mediaStoreDir',
];

/** Shared product operations; all work is performed by the owned native object. */
export abstract class Buildable {
  abstract readonly baseDir: string;
  protected abstract nativeProject(): NativeBuildable;

  async validate(): Promise<ValidationReport> {
    try {
      return new ValidationReport(outcome(await this.nativeProject().validate()).diagnostics);
    } catch (error) {
      nativeError(error);
    }
  }

  async build(config: BuildOptions): Promise<BuildReport> {
    options(config, buildKeys, 'build');
    string(config.output, 'output');
    const resolved: Record<string, unknown> = { ...config };
    for (const key of pathKeys)
      if (resolved[key] !== undefined) {
        string(resolved[key], key);
        resolved[key] = path.resolve(this.baseDir, resolved[key]);
      }
    for (const key of ['inspect', 'writeIdentityLockfile', 'selfContained'])
      if (resolved[key] !== undefined && typeof resolved[key] !== 'boolean')
        throw new TypeError(`${key} must be a boolean`);
    if (
      config.failOn !== undefined &&
      !['info', 'low', 'medium', 'high', 'critical'].includes(config.failOn)
    )
      throw new TypeError('Invalid failOn');
    if (
      config.updateSafety !== undefined &&
      !['strict', 'report-only', 'disabled'].includes(config.updateSafety)
    )
      throw new TypeError('Invalid updateSafety');
    if (config.inspectLimits !== undefined) checkInspectLimits(config.inspectLimits);
    if (
      config.mediaMode !== undefined &&
      !['path-backed', 'self-contained'].includes(config.mediaMode)
    )
      throw new TypeError('Invalid mediaMode');
    if (config.mediaMode === 'path-backed' && config.selfContained)
      throw new TypeError('Conflicting mediaMode and selfContained');
    if (config.mediaPolicy !== undefined) {
      options(
        config.mediaPolicy,
        ['unusedBinding', 'unknownMime', 'declaredMimeMismatch'],
        'mediaPolicy',
      );
      for (const [key, value] of Object.entries(config.mediaPolicy))
        if (
          value !== undefined &&
          !(
            key === 'declaredMimeMismatch'
              ? ['warning', 'error']
              : ['ignore', 'info', 'warning', 'error']
          ).includes(String(value))
        )
          throw new TypeError(`Invalid mediaPolicy.${key}`);
    }
    try {
      const result = outcome(await this.nativeProject().build(JSON.stringify(resolved)));
      return new BuildReport(result.report, String(result.pretty));
    } catch (error) {
      nativeError(error);
    }
  }

  writeApkg(output: string, config: Omit<BuildOptions, 'output'> = {}): Promise<BuildReport> {
    options(
      config,
      buildKeys.filter((key) => key !== 'output'),
      'writeApkg',
    );
    return this.build({ ...config, output });
  }

  async toApkgBuffer(): Promise<Buffer> {
    try {
      const result = await this.nativeProject().apkgBytes();
      outcome(result.result);
      return result.data;
    } catch (error) {
      nativeError(error);
    }
  }

  async writeTo(stream: Writable): Promise<void> {
    if (!(stream instanceof Writable) || stream.destroyed || stream.writableEnded)
      throw new TypeError('Expected an open Writable');
    let failure: Error | undefined;
    const failed = (error: Error) => {
      failure ??= error;
    };
    const closed = () => failed(new Error('Writable closed while generating the archive'));
    stream.on('error', failed);
    stream.once('close', closed);
    try {
      const bytes = await this.toApkgBuffer();
      if (failure) throw failure;
      if (stream.destroyed || stream.writableEnded)
        throw new Error('Writable closed while generating the archive');
      await writeBuffer(stream, bytes);
    } finally {
      stream.off('error', failed);
      stream.off('close', closed);
    }
  }

  async diffAgainstApkg(
    filename: string,
    config: { inspectLimits?: InspectLimits } = {},
  ): Promise<ProjectDiffReport> {
    string(filename, 'filename');
    options(config, ['inspectLimits'], 'diff');
    if (config.inspectLimits !== undefined) checkInspectLimits(config.inspectLimits);
    try {
      return new ProjectDiffReport(
        outcome(
          await this.nativeProject().diffAgainstApkg(
            path.resolve(this.baseDir, filename),
            JSON.stringify(config.inspectLimits ?? {}),
          ),
        ),
      );
    } catch (error) {
      nativeError(error);
    }
  }
}
