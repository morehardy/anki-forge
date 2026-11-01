import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { isDeepStrictEqual } from 'node:util';
const [root, output = '/tmp/anki-forge-es-execution-reuse-audit.json'] = process.argv.slice(2);
assert.ok(root);
const hash = value => createHash('sha256').update(value).digest('hex');
const read = filename => readFile(path.join(root, filename));
const oldEvidence = 'docs/plans/evidence/multilingual-common-2026-10-27';
const snapshotPath = `${oldEvidence}/executions/execution-inputs.json`;
const snapshotBytes = await read(snapshotPath);
const snapshot = JSON.parse(snapshotBytes);
const comparisons = [];
for (const original of snapshot.inputs) {
  const currentSha256 = hash(await read(original.path));
  comparisons.push({ path: original.path, originalSha256: original.sha256, currentSha256, identical: currentSha256 === original.sha256 });
}
const controlPath = 'docs/ai-quickstart.json';
const originalControlSha256 = snapshot.inputs.find(input => input.path === controlPath)?.sha256;
assert.ok(originalControlSha256, 'Original real-execution snapshot must bind the control JSON');
const originalControlPath = `${oldEvidence}/technical-input-snapshot/${originalControlSha256}`;
const originalControlBytes = await read(originalControlPath);
assert.equal(hash(originalControlBytes), originalControlSha256, 'Original archived control JSON must match the original execution input hash');
const originalControl = JSON.parse(originalControlBytes);
const currentControlBytes = await read(controlPath);
const currentControl = JSON.parse(currentControlBytes);
const projection = value => ({ publicVersion: value.publicVersion, languages: value.languages, sitePromptUrl: value.sitePromptUrl });
const originalProjection = projection(originalControl);
const currentProjection = projection(currentControl);
const controlProjectionEquivalent = isDeepStrictEqual(originalProjection, currentProjection);
const changedControlFields = [...new Set([...Object.keys(originalControl), ...Object.keys(currentControl)])].filter(key => !isDeepStrictEqual(originalControl[key], currentControl[key]));
const originalManifestPath = 'docs/plans/evidence/multilingual-common-2026-10-28-stable-rebuild/root-origin-manifest.json';
const originalManifestBytes = await read(originalManifestPath);
const originalManifest = JSON.parse(originalManifestBytes);
assert.match(originalManifest.commit, /^[0-9a-f]{40}$/);
const englishSources = ['docs/installation.md', 'docs/node/quick-start.md', 'docs/python/quick-start.md', 'website/src/content/docs/docs/quickstart.mdx', 'docs/json-to-anki.md'];
const englishInstallationInputs = [];
for (const filename of englishSources) {
  const original = execFileSync('git', ['show', `${originalManifest.commit}:${filename}`], { cwd: root });
  const current = await read(filename);
  englishInstallationInputs.push({ path: filename, originalCommit: originalManifest.commit, originalSha256: hash(original), currentSha256: hash(current), identical: original.equals(current) });
}
const basicPrograms = [];
for (const [source, copy] of [['anki_forge/examples/target_api_basic.rs', 'rust/main.rs'], ['bindings/python/examples/quickstart.py', 'python/main.py'], ['bindings/node/examples/quickstart.mjs', 'node/main.mjs']]) {
  const program = await read(source);
  const archive = await read(`${oldEvidence}/executions/basic/${copy}`);
  basicPrograms.push({ source, actualExecutedCopy: `${oldEvidence}/executions/basic/${copy}`, sourceSha256: hash(program), actualExecutedCopySha256: hash(archive), identical: program.equals(archive) });
}
const sdkInventoryPath = 'docs/plans/evidence/multilingual-es-2026-10-28/technical-sdk-dependency-identity.json';
const sdkInventoryBytes = await read(sdkInventoryPath);
const sdkInventory = JSON.parse(sdkInventoryBytes);
const sdkImplementationComparisons = [];
for (const original of sdkInventory.entries) {
  const currentSha256 = hash(await read(original.path));
  sdkImplementationComparisons.push({ path: original.path, originalSha256: original.sha256, currentSha256, identical: currentSha256 === original.sha256 });
}
const originalInstallations = [];
for (const sdk of ['rust', 'python', 'node']) {
  const filename = `${oldEvidence}/executions/public-json/${sdk}/installation.json`;
  const bytes = await read(filename);
  const installation = JSON.parse(bytes);
  const version = sdk === 'rust' ? installation.source.version : sdk === 'python' ? installation.source.metadata.version : installation.source.version;
  originalInstallations.push({ sdk, path: filename, sha256: hash(bytes), checkedAt: installation.checkedAt, platform: installation.platform, arch: installation.arch, node: installation.node, actualPublicVersion: version, matchesCurrentPublicVersion: version === currentProjection.publicVersion });
}
const changedNonControl = comparisons.filter(entry => !entry.identical && entry.path !== controlPath);
const eligible = changedNonControl.length === 0 && controlProjectionEquivalent && changedControlFields.every(key => ['prompts', 'requestPlaceholders'].includes(key)) && basicPrograms.every(item => item.identical) && englishInstallationInputs.every(item => item.identical) && sdkImplementationComparisons.every(item => item.identical) && originalInstallations.every(item => item.matchesCurrentPublicVersion);
const report = {
  schemaVersion: 1, kind: 'execution-reuse-audit', scope: 'Pre-artifact SDK execution equivalence only; no release or localized task approval.',
  reviewerType: 'ai', reviewerRun: '/root/final_standards_review/independent-es-reuse-preparation-2026-10-28', observedAt: new Date().toISOString(),
  reuseEligible: eligible, sdkExecutionsRerun: false, browserTasksApproved: false,
  originalExecutionSnapshot: { path: snapshotPath, sha256: hash(snapshotBytes), count: comparisons.length },
  rawSnapshotComparison: { unchanged: comparisons.filter(entry => entry.identical).length, changed: comparisons.filter(entry => !entry.identical).length, entries: comparisons },
  controlJson: {
    sourcePath: controlPath, originalArchive: { path: originalControlPath, sha256: originalControlSha256 },
    originalSha256: originalControlSha256, currentSha256: hash(currentControlBytes), rawBytesIdentical: originalControlBytes.equals(currentControlBytes), changedTopLevelFields: changedControlFields,
    addedPromptLocales: Object.keys(currentControl.prompts || {}).filter(locale => !Object.hasOwn(originalControl.prompts || {}, locale)),
    changedExistingPromptLocales: Object.keys(originalControl.prompts || {}).filter(locale => !isDeepStrictEqual(originalControl.prompts[locale], currentControl.prompts?.[locale])),
    executionRelevantProjection: { original: originalProjection, current: currentProjection, equivalent: controlProjectionEquivalent },
    conclusion: 'The complete control JSON changed and is not an unchanged execution dependency. Only publicVersion, SDK language installation/runtime/entry metadata, and sitePromptUrl are relevant to these retained SDK executions; those projections were compared independently with the original archived bytes.',
  },
  englishInstallationInputs, basicPrograms,
  sdkImplementationInventory: { path: sdkInventoryPath, sha256: hash(sdkInventoryBytes), compared: sdkImplementationComparisons.length, identical: sdkImplementationComparisons.filter(entry => entry.identical).length, changed: sdkImplementationComparisons.filter(entry => !entry.identical) },
  originalManifestReference: { path: originalManifestPath, sha256: hash(originalManifestBytes), commit: originalManifest.commit },
  originalInstallations,
  currentObservationRuntime: { node: process.version, platform: process.platform, arch: process.arch },
  futureDependencyBindingPolicy: {
    unchangedVersionInstallSources: englishInstallationInputs.map(item => ({ path: item.path, sha256: item.currentSha256 })),
    unchangedExecutionInputs: comparisons.filter(item => item.identical).map(item => ({ path: item.path, sha256: item.currentSha256 })),
    excludeWholeFileAsAnUnchangedDependency: [controlPath],
    requirement: 'Formal artifact usage review must repeat real localized reading/navigation/copy/download for en/zh-CN/es and recheck this execution-relevant projection plus unchanged executable/dependency bytes. Bind real programs, fixtures, version/install documentation, manifest/lock/verifier inputs. Cite this audit for the changed control JSON rather than refreshing its whole-file hash and claiming all 20 original execution inputs were identical.',
  },
  limitations: [
    'No SDK installation, compilation, execution, package regeneration or Anki GUI import was performed for this preparation audit.',
    '19/20 or another exact raw count describes comparison with the original execution-input snapshot, not a new execution pass.',
    'A new Spanish prompt and localized placeholders require their own source/UI review; equivalence of SDK execution requirements does not approve their wording or behavior.',
    'No formal artifact was read or approved by this audit. Run the three-language helper against the frozen strict release artifacts when provided.',
  ],
};
await writeFile(output, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ output, reuseEligible: eligible, unchangedOriginalInputs: report.rawSnapshotComparison.unchanged, changedOriginalInputs: report.rawSnapshotComparison.changed, changedInputPaths: comparisons.filter(item => !item.identical).map(item => item.path), changedControlFields, controlProjectionEquivalent, comparedSdkFiles: sdkImplementationComparisons.length, identicalSdkFiles: report.sdkImplementationInventory.identical, englishInstallDocumentsIdentical: englishInstallationInputs.filter(item => item.identical).length, currentControlSha256: hash(currentControlBytes) }));
if (!eligible) process.exitCode = 1;
