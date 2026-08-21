import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

import { ProtocolParseError, RuntimeInvocationError } from './errors.js';
import { validateContractPayload } from './contracts.js';
import { helperView } from './helpers.js';
import { runRaw } from './raw.js';

async function runStructured(command, request, runtimeOptions) {
  const raw = await runRaw(command, request, runtimeOptions);

  if (raw.exitStatus !== 0 && command !== 'product-build') {
    throw new RuntimeInvocationError(`${command} exited with status ${raw.exitStatus}`, {
      command,
      exitStatus: raw.exitStatus,
      stdout: raw.stdout,
      stderr: raw.stderr,
      resolvedRuntime: raw.resolvedRuntime,
      failurePhase: 'process-exit',
    });
  }

  let parsed;
  try {
    parsed = JSON.parse(raw.stdout);
  } catch (error) {
    if (raw.exitStatus !== 0) {
      throw new RuntimeInvocationError(`${command} exited with status ${raw.exitStatus}`, {
        command,
        exitStatus: raw.exitStatus,
        stdout: raw.stdout,
        stderr: raw.stderr,
        resolvedRuntime: raw.resolvedRuntime,
        failurePhase: 'process-exit',
      });
    }
    throw new ProtocolParseError(error.message, {
      command,
      exitStatus: raw.exitStatus,
      stdout: raw.stdout,
      stderr: raw.stderr,
      resolvedRuntime: raw.resolvedRuntime,
      parsePhase: 'json',
    });
  }

  try {
    validateContractPayload(command, parsed);
  } catch (error) {
    if (raw.exitStatus !== 0) {
      throw new RuntimeInvocationError(`${command} exited with status ${raw.exitStatus}`, {
        command,
        exitStatus: raw.exitStatus,
        stdout: raw.stdout,
        stderr: raw.stderr,
        resolvedRuntime: raw.resolvedRuntime,
        failurePhase: 'process-exit',
      });
    }
    throw new ProtocolParseError(error.message, {
      command,
      exitStatus: raw.exitStatus,
      stdout: raw.stdout,
      stderr: raw.stderr,
      resolvedRuntime: raw.resolvedRuntime,
      parsePhase: error.parsePhase ?? 'contract-shape',
    });
  }

  return {
    ...parsed,
    resolvedRuntime: raw.resolvedRuntime,
    rawCommand: {
      command: raw.command,
      argv: raw.argv,
      exitStatus: raw.exitStatus,
    },
    helper: helperView(command, parsed, request),
  };
}

export function normalize(request, runtimeOptions = {}) {
  return runStructured('normalize', request, runtimeOptions);
}

export function build(request, runtimeOptions = {}) {
  return runStructured('build', request, runtimeOptions);
}

async function runProductBuild(request, runtimeOptions) {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'anki-forge-node-product-'));
  try {
    let inputPath = request.inputPath;
    if (request.productDocument !== undefined) {
      inputPath = path.join(tempDir, 'project.product-v3.json');
      await fs.writeFile(inputPath, JSON.stringify(request.productDocument), 'utf8');
    }
    if (!inputPath) {
      throw new TypeError('productBuild requires inputPath or productDocument');
    }
    return await runStructured(
      'product-build',
      {
        ...request,
        inputPath,
        apkgOut: request.apkgOut ?? path.join(tempDir, 'deck.apkg'),
      },
      runtimeOptions,
    );
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

export async function productBuild(request, runtimeOptions = {}) {
  if (!request.apkgOut) {
    throw new TypeError('productBuild requires apkgOut so the built artifact can be retained');
  }
  return await runProductBuild(request, runtimeOptions);
}

export async function productValidate(request, runtimeOptions = {}) {
  const result = await runProductBuild(request, runtimeOptions);
  return { ...result, artifact: null };
}

export function templateValidate(request, runtimeOptions = {}) {
  return productValidate(request, runtimeOptions);
}

export function inspect(request, runtimeOptions = {}) {
  return runStructured('inspect', request, runtimeOptions);
}

export function diff(request, runtimeOptions = {}) {
  return runStructured('diff', request, runtimeOptions);
}
