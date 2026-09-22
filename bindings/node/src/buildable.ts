import path from 'node:path';
import { createReadStream } from 'node:fs';
import { Writable } from 'node:stream';
import type { BuildOptions, InspectLimits } from './types';
import type { NativeBuildable } from './internal/native';
import { writeBuffer } from './internal/stream';
import { encodeInspectLimits } from './options';
import { options, string } from './internal/validation';
import { BuildReport, ValidationReport, ProjectDiffReport } from './report';
import { nativeError, BindingProtocolError } from './errors';
import type { ApkgArtifact } from './artifact';
import { outcome, buildOutcome } from './internal/outcome';

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

  async build(config: BuildOptions = {}): Promise<BuildReport> {
    options(config, buildKeys, 'build');
    if (config.output !== undefined) string(config.output, 'output');
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
    if (config.inspectLimits !== undefined)
      resolved.inspectLimits = encodeInspectLimits(config.inspectLimits);
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
      return buildOutcome(
        await this.nativeProject().build(JSON.stringify(resolved)),
        this.baseDir,
      );
    } catch (error) {
      nativeError(error);
    }
  }

  writeApkg(output: string, config: Omit<BuildOptions, 'output'> = {}): Promise<BuildReport> {
    string(output, 'output');
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
    let artifact: ApkgArtifact | null = null;
    let source: ReturnType<typeof createReadStream> | undefined;
    let operationFailed = false;
    try {
      const report = await this.build();
      report.ensureSuccess();
      artifact = report.artifactHandle;
      if (!artifact) throw new BindingProtocolError('Native build did not retain its artifact');
      if (failure) throw failure;
      if (stream.destroyed || stream.writableEnded)
        throw new Error('Writable closed while generating the archive');
      source = createReadStream(artifact.path, { highWaterMark: 64 * 1024 });
      for await (const chunk of source) {
        if (failure) throw failure;
        await writeBuffer(stream, chunk);
      }
      if (failure) throw failure;
    } catch (error) {
      operationFailed = true;
      throw error;
    } finally {
      if (source) {
        // Wait for descriptor closure before releasing a temporary file on Windows.
        const closed = source.closed
          ? Promise.resolve()
          : new Promise<void>((resolve) => source!.once('close', resolve));
        source.destroy();
        await closed;
      }
      if (artifact) {
        try {
          await artifact.close();
        } catch (error) {
          if (!operationFailed) {
            stream.off('error', failed);
            stream.off('close', closed);
            throw error;
          }
        }
      }
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
    const limits = encodeInspectLimits(config.inspectLimits ?? {});
    try {
      return new ProjectDiffReport(
        outcome(
          await this.nativeProject().diffAgainstApkg(
            path.resolve(this.baseDir, filename),
            JSON.stringify(limits),
          ),
        ),
      );
    } catch (error) {
      nativeError(error);
    }
  }
}
