// P1 SDK workflow control: no intervening authoring, fresh process each sample.
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import { performance } from 'node:perf_hooks';
import { pathToFileURL } from 'node:url';

const [sdk, input, destination, mode, baseline] = process.argv.slice(2);
const { Project, Note, BuildOptions, CompareOptions } = await import(pathToFileURL(sdk));
const started = performance.now();
const document = JSON.parse(await fs.readFile(input, 'utf8'));
const project = new Project('publication-node');
for (const [index, note] of document.notes.entries()) {
  project.add(`note-${index}`, Note.basic(note.front, note.back));
}
const inputMs = performance.now() - started;
const operationStarted = performance.now();
const options = BuildOptions.to(destination).updateFrom(baseline);
let comparison;
let output;
if (mode === 'compare-build') {
  comparison = (await project.compare(CompareOptions.against(baseline))).snapshot();
  output = await project.build(options);
} else {
  const prepared = await project.preparePublication(options);
  comparison = prepared.report.comparison.snapshot();
  output = await prepared.publish();
}
assert.equal(output.report.counts.notes, document.note_count);
const operationMs = performance.now() - operationStarted;
await output.artifact.close();
console.log(JSON.stringify({ input_ms: inputMs, operation_ms: operationMs,
  comparison, report: output.report.snapshot() }));
