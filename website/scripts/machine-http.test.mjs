import test from 'node:test';
import assert from 'node:assert/strict';
import { checkMachineHttp } from './machine-http.mjs';

test('machine resources require readable UTF-8 text without forced downloads', () => {
  const bytes = Buffer.from('中文\n');
  assert.doesNotThrow(() => checkMachineHttp('/docs/index.md', new Response(bytes, { headers: { 'content-type': 'text/markdown; charset=UTF-8' } }), bytes));
  assert.doesNotThrow(() => checkMachineHttp('/llms.txt', new Response(bytes, { headers: { 'content-type': 'text/plain; charset=UTF-8' } }), bytes));
  assert.throws(() => checkMachineHttp('/llms.txt', new Response(bytes, { headers: { 'content-type': 'text/html' } }), bytes), /Content-Type/);
  assert.throws(() => checkMachineHttp('/docs/index.md', new Response(bytes, { headers: { 'content-type': 'text/plain', 'content-disposition': 'attachment; filename=guide.md' } }), bytes), /attachment/);
  assert.throws(() => checkMachineHttp('/llms.txt', new Response(bytes, { headers: { 'content-type': 'text/plain' } }), Buffer.from([0xff])), /UTF-8/);
  assert.throws(() => checkMachineHttp('/docs/index.md', new Response(null, { status: 302 }), bytes), /status/);
});
