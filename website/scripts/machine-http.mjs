import assert from 'node:assert/strict';

export function checkMachineHttp(route, response, bytes) {
  assert.equal(response.status, 200, `${route}: machine resource status`);
  const type = (response.headers.get('content-type') || '').split(';', 1)[0].trim().toLowerCase();
  const accepted = route.endsWith('.md') ? ['text/markdown', 'text/plain'] : ['text/plain'];
  assert.ok(accepted.includes(type), `${route}: invalid machine resource Content-Type ${type || '(missing)'}`);
  assert.doesNotMatch(response.headers.get('content-disposition') || '', /^\s*attachment(?:\s*;|\s*$)/i, `${route}: forced attachment download`);
  assert.doesNotThrow(() => new TextDecoder('utf-8', { fatal: true }).decode(bytes), `${route}: invalid UTF-8`);
}
