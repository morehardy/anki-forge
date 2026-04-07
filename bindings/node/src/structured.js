import { ProtocolParseError, RuntimeInvocationError } from './errors.js';
import { validateContractPayload } from './contracts.js';
import { helperView } from './helpers.js';
import { runRaw } from './raw.js';

async function runStructured(command, request, runtimeOptions) {
  const raw = await runRaw(command, request, runtimeOptions);

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

  let parsed;
  try {
    parsed = JSON.parse(raw.stdout);
  } catch (error) {
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

export function inspect(request, runtimeOptions = {}) {
  return runStructured('inspect', request, runtimeOptions);
}

export function diff(request, runtimeOptions = {}) {
  return runStructured('diff', request, runtimeOptions);
}
