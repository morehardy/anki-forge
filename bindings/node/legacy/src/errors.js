export class RuntimeInvocationError extends Error {
  constructor(message, details) {
    super(message);
    this.name = 'RuntimeInvocationError';
    this.command = details.command;
    this.exitStatus = details.exitStatus ?? null;
    this.stdout = details.stdout ?? '';
    this.stderr = details.stderr ?? '';
    this.resolvedRuntime = details.resolvedRuntime ?? null;
    this.failurePhase = details.failurePhase ?? null;
    this.parsePhase = details.parsePhase ?? null;
  }
}

export class ProtocolParseError extends Error {
  constructor(message, details) {
    super(message);
    this.name = 'ProtocolParseError';
    this.command = details.command;
    this.exitStatus = details.exitStatus ?? null;
    this.stdout = details.stdout ?? '';
    this.stderr = details.stderr ?? '';
    this.resolvedRuntime = details.resolvedRuntime ?? null;
    this.parsePhase = details.parsePhase ?? 'json';
  }
}
