import { spawn } from 'node:child_process';
import path from 'node:path';

import { RuntimeInvocationError } from './errors.js';
import { resolveRuntime } from './runtime.js';

function buildArgs(command, request, runtime) {
  switch (command) {
    case 'normalize':
      return [
        ...runtime.launcherPrefix,
        'normalize',
        '--manifest',
        runtime.manifestPath,
        '--input',
        request.inputPath,
        '--output',
        request.output ?? 'contract-json',
      ];
    case 'build':
      return [
        ...runtime.launcherPrefix,
        'build',
        '--manifest',
        runtime.manifestPath,
        '--input',
        request.inputPath,
        '--writer-policy',
        request.writerPolicy ?? 'default',
        '--build-context',
        request.buildContext ?? 'default',
        '--artifacts-dir',
        request.artifactsDir,
        '--output',
        request.output ?? 'contract-json',
      ];
    case 'inspect':
      return request.stagingPath
        ? [
            ...runtime.launcherPrefix,
            'inspect',
            '--staging',
            request.stagingPath,
            '--output',
            request.output ?? 'contract-json',
          ]
        : [
            ...runtime.launcherPrefix,
            'inspect',
            '--apkg',
            request.apkgPath,
            '--output',
            request.output ?? 'contract-json',
          ];
    case 'diff':
      return [
        ...runtime.launcherPrefix,
        'diff',
        '--left',
        request.leftPath,
        '--right',
        request.rightPath,
        '--output',
        request.output ?? 'contract-json',
      ];
    default:
      throw new Error(`unsupported command: ${command}`);
  }
}

export async function runRaw(command, request, runtimeOptions = {}) {
  let resolvedRuntime;
  try {
    resolvedRuntime = resolveRuntime(runtimeOptions);
  } catch (error) {
    throw new RuntimeInvocationError(error.message, {
      command,
      exitStatus: null,
      stdout: '',
      stderr: '',
      resolvedRuntime: null,
      failurePhase: 'runtime-resolution',
    });
  }

  const argv = buildArgs(command, request, resolvedRuntime);

  return await new Promise((resolve, reject) => {
    const child = spawn(resolvedRuntime.launcherExecutable, argv, {
      cwd:
        resolvedRuntime.mode === 'workspace'
          ? path.dirname(path.dirname(resolvedRuntime.manifestPath))
          : undefined,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
    });
    child.on('error', (error) => {
      reject(
        new RuntimeInvocationError(error.message, {
          command,
          exitStatus: null,
          stdout,
          stderr,
          resolvedRuntime,
          failurePhase: 'spawn',
        }),
      );
    });
    child.on('close', (code) => {
      resolve({
        command,
        argv: [resolvedRuntime.launcherExecutable, ...argv],
        exitStatus: code ?? -1,
        stdout,
        stderr,
        resolvedRuntime,
      });
    });
  });
}
