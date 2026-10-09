// Run with --expose-gc and ANKI_FORGE_NATIVE_PATH selecting a frozen binary.
import { performance } from 'node:perf_hooks';
import fs from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';

const [sdkPath, inputPath, operation, baseline, destination, control] = process.argv.slice(2);
const { Project, Note, BuildOptions, CompareOptions } = await import(pathToFileURL(sdkPath));
const operationOnly = control === '--operation-only';
const document = JSON.parse(await fs.readFile(inputPath, 'utf8'));
const project = new Project('publication-node');
const authorStarted = performance.now();
for (const [index, note] of document.notes.entries()) project.add(`note-${index}`, Note.basic(note.front, note.back));
const authorMs = performance.now() - authorStarted;
if (operation === 'baseline') {
  const output = await project.build(BuildOptions.to(destination));
  await output.artifact.close();
  process.exit(0);
}
global.gc();
const exclusiveStarted = performance.now();
project.defaultDeck('Default');
const exclusiveEditMs = performance.now() - exclusiveStarted;
const cloneStarted = performance.now();
const clone = operationOnly ? undefined : project.clone();
const cloneMs = performance.now() - cloneStarted;
await new Promise(resolve => setTimeout(resolve, 5));
const delays = [];
let tick = performance.now();
const timer = setInterval(() => {
  const now = performance.now();
  delays.push(Math.max(0, now - tick - 1));
  tick = now;
}, 1);
const started = performance.now();
const pending = operation === 'build'
  ? project.build(BuildOptions.to(destination))
  : operation === 'compare'
    ? project.compare(CompareOptions.against(baseline))
    : project.preparePublication(BuildOptions.to(destination).updateFrom(baseline));
const submitMs = performance.now() - started;
const first = performance.now();
if (!operationOnly) project.add('later', Note.basic('later', 'answer'));
const firstEditMs = performance.now() - first;
const second = performance.now();
if (!operationOnly) project.add('second', Note.basic('second', 'answer'));
const secondEditMs = performance.now() - second;
const rejected = performance.now();
if (!operationOnly) assert.throws(() => project.add('later', Note.basic('duplicate', 'answer')));
const failedAddMs = performance.now() - rejected;
const result = await pending;
const completionMs = performance.now() - started;
await new Promise(resolve => setTimeout(resolve, 2));
clearInterval(timer);
let report;
if (operation === 'compare') report = result.snapshot();
else {
  const output = operation === 'prepare' ? await result.publish() : result;
  report = output.report.snapshot();
  assert.equal(output.report.counts.notes, document.note_count);
  await output.artifact.close();
}
if (clone) assert.equal(clone.length, document.note_count);
console.log(JSON.stringify({ operation_only: operationOnly, author_ms: authorMs, submit_ms: submitMs, completion_ms: completionMs,
  clone_ms: cloneMs, exclusive_edit_ms: exclusiveEditMs, first_shared_edit_ms: firstEditMs,
  subsequent_edit_ms: secondEditMs, failed_add_ms: failedAddMs,
  timer_max_ms: Math.max(0, ...delays), timer_samples_ms: delays, report }));
