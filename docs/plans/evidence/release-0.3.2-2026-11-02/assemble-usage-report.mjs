// Assembles this independent reviewer's usage-tasks report for the release-0.3.2 artifact from the
// fresh browser tour (usage-observations.mjs), fresh SDK executions and inspections.
// Prior nonblocking wording judgments are retained ONLY after exact body/heading/code equality
// with the previously reviewed tour. Usage: node assemble-usage-report.mjs root|prefix
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const env=process.argv[2];assert.ok(['root','prefix'].includes(env));
const base='docs/plans/evidence/release-0.3.2-2026-11-02';
const priorBase='docs/plans/evidence/bing-seo-2026-11-02';
const hash=b=>createHash('sha256').update(b).digest('hex');
const json=async p=>JSON.parse(await readFile(p));
const proof=async(path,description)=>({path,sha256:hash(await readFile(path)),description});
const manifestPath=`${base}/${env}-site-manifest.json`,manifestBytes=await readFile(manifestPath),manifest=JSON.parse(manifestBytes);
assert.equal(manifest.commit,'75a8748cc694f409c0060ccd17fc1d3270a8e077');
const tourPath=`${base}/usage-${env}/browser-tour.json`,tour=await json(tourPath);
const prior=await json(`${priorBase}/usage-tasks-${env}.json`),priorTour=await json(`${priorBase}/usage-${env}/browser-tour.json`);
assert.equal(tour.manifestSha256,hash(manifestBytes));assert.equal(tour.mode,'release-candidate');assert.equal(tour.commit,manifest.commit);
assert.deepEqual(tour.releaseLocales,manifest.releaseLocales);assert.equal(tour.failure,undefined);assert.deepEqual(tour.pageErrors,[]);assert.deepEqual(tour.responseReadErrors,[]);
const integrity=await json(`${base}/usage-${env}/artifact-integrity.json`);assert.equal(integrity.manifestSha256,hash(manifestBytes));assert.equal(integrity.matched,manifest.pages.length+manifest.resources.length);
const execution=await json(`${base}/usage-execution-audit.json`);
for(const b of [...execution.sourceBindings,...execution.dependencyBindings])assert.equal(hash(await readFile(b.path)),b.sha256,`Execution input changed: ${b.path}`);
for(const f of execution.evidenceFiles)assert.equal(hash(await readFile(f.path)),f.sha256,`Execution evidence changed: ${f.path}`);
const comparisonFile=await json(`${base}/artifact-comparison.json`),downloads=await json(`${base}/generated-download-inspection.json`);
assert.ok(comparisonFile.environments[env].changedManifestRecords.every(r=>r.includes('/generated/')));assert.equal(downloads.onlyMetadataChanged,true);
const records=[...manifest.pages,...manifest.resources];
const routes=new Map(records.map(r=>[r.route,r]));
const bindingRoutes=new Set(manifest.resources.filter(r=>/\/(?:index\.md|llms\.txt)$/.test(r.route)).map(r=>r.route));
for(const r of tour.loadedResources){assert.equal(routes.get(r.route)?.sha256,r.sha256);bindingRoutes.add(r.route);}
const meanings={
'rust-basic':'The localized instructions specify Rust 1.92+, `cargo add ankiforge@0.3.1` (the published version), a complete main.rs, cargo run and a persistent spanish.apkg. This reviewer freshly ran the documented cargo add against crates.io (resolved ankiforge 0.3.1 from the crates.io index), compiled and ran the copied program; the output was independently decoded as hola/hello, 1 note and 1 card.',
'python-basic':'The localized instructions provide venv activation, `python -m pip install ankiforge==0.3.1`, main.py and python main.py. A fresh no-cache PyPI wheel install (files.pythonhosted.org) and execution produced a persistent hola/hello APKG, independently decoded as 1 note and 1 card.',
'node-basic':'The localized instructions provide Node 22.13+, `npm install --include=optional ankiforge@0.3.1`, full .mjs code and node main.mjs. A fresh npm install from registry.npmjs.org and execution produced the persistent package; closing its artifact did not remove it; decoded as 1 note and 1 card.',
'json-media':'The localized recipe identifies three questions, shared image and generated tone, all four downloaded inputs (downloaded in this tour and byte-matched to docs/recipes/json-to-anki), complete Rust/Python/TypeScript programs and counts of 3/3/2. Fresh public 0.3.1 runs in all three SDKs, the consumer inspector and this reviewer\'s own Python decoder confirm these counts and exact media bytes.',
'original-baseline-update':'The localized guide requires stable keys and the previous original distribution APKG, explicitly rejects Anki re-export as the identity baseline and distinguishes candidate review from publishing. Fresh JSON v1/v2 runs preserve GUIDs with exactly one changed note; comparison and review report RISK.NOTE_CHANGED (low) and allow publication; a repeated run refused to overwrite v1. The changed /generated/spanish-v1/v2 downloads differ from the previous artifact only by build timestamps.',
'troubleshooting':'The localized table maps NOTE.KEY_DUPLICATE to one unique key per logical record and distinguishes failures from a completed comparison blocked by policy. A fresh controlled Python (public 0.3.1) run reproduced the code and recovered with a distinct key; the recovered APKG has 2 notes and 2 cards.'};
const taskLog={'rust-basic':'executions/basic/rust/basic.log','python-basic':'executions/basic/python/basic.log','node-basic':'executions/basic/node/basic.log','json-media':'executions/public-json-run.log','original-baseline-update':'executions/public-json-run.log','troubleshooting':'executions/basic/python/troubleshooting.log'};
const results={},comparison=[];
for(const locale of manifest.releaseLocales){
 const tasks=tour.entries[locale];assert.equal(tasks.length,6);results[locale]={taskResults:[],openIssues:prior.entries[locale].openIssues||[]};
 for(const task of tasks){
  const old=priorTour.entries[locale].find(t=>t.id===task.id);assert.ok(old);
  // Exact comparison is the only condition under which prior nonblocking wording judgments are retained.
  assert.equal(task.body,old.body);assert.equal(task.h1,old.h1);assert.deepEqual(task.codeBlocks,old.codeBlocks);
  comparison.push({locale,id:task.id,fullBodyUnchanged:true,headingUnchanged:true,renderedCodeUnchanged:true,actualQuote:task.body.replace(/\n\s*/g,' ').slice(0,750)});
  for(const url of task.readUrls){const route=new URL(url).pathname;assert.ok(routes.has(route));bindingRoutes.add(route);}
  for(const c of task.copiedPrograms){assert.equal(hash(await readFile(c.source)),c.sourceSha256);assert.equal(c.copiedCode.trim(),(await readFile(c.source,'utf8')).trim());}
  for(const d of task.downloadedResources)assert.equal(hash(await readFile(d.path)),d.sha256);
  const route=new URL(task.finalUrl).pathname;
  const log=await proof(`${base}/${taskLog[task.id]}`,'Genuinely new execution in this release-0.3.2 review (2026-11-02), installing the published 0.3.1 packages the website teaches. Executed once per SDK program/dependency set; reused across locales only after each localized copied program was matched to that exact source. No previous-cycle execution is rebound.');
  const evidence=[{route,sha256:routes.get(route).sha256,observation:`Actually navigated the ${locale} rendered page “${task.h1}” in the frozen release-0.3.2 ${env} artifact. ${meanings[task.id]}`},
   await proof(tourPath,`Actual fresh ${env} browser navigation, search, clipboard and input-download observations; see entries[${locale}]/${task.id}.`),
   await proof(`${base}/usage-execution-audit.json`,'Current-source binding audit, fresh public registry receipts, decoded APKGs and actual execution conclusions.'),
   await proof(`${base}/artifact-comparison.json`,'Independent full manifest and dist-tree byte comparison with the previously reviewed artifact: all HTML/Markdown/llms bytes identical, only /generated downloads differ.'),
   await proof(`${base}/rebuild-input-verification.json`,'Recomputed input hash: the rebuilt artifact differs from the first release-0.3.2 build only by website/post-build-reviews.config.json.'),
   await proof(task.screenshot,'Screenshot from this fresh browser tour; not a new exhaustive visual design review.')];
  if(['json-media','original-baseline-update'].includes(task.id)){
   evidence.push(await proof(`${base}/usage-json-inspection.log`,'Second inspection of all fresh Rust/Python/Node v1/v2 APKG pairs by the consumer inspector and this reviewer\'s own Python decoder: exact media, stable GUIDs, one changed note and allowed publication.'));
   evidence.push(await proof(`${base}/generated-download-inspection.json`,'Changed website /generated downloads decoded old vs new: only build timestamps (and showcase.json crateVersion/sourceCommit/digests) changed; deck content identical.'));
   for(const l of ['rust','python','node'])evidence.push(await proof(`${base}/executions/public-json/${l}/installation.json`,'Fresh public package version, platform and registry installation receipt.'));
  }else if(task.id!=='troubleshooting'){
   const lang=task.id.split('-')[0];evidence.push(await proof(`${base}/executions/basic/${lang}/spanish.apkg`,'Freshly generated basic package; the matching inspection log confirms one hola/hello note and card.'));
   evidence.push(await proof(`${base}/executions/basic/rust/inspection-${lang}.log`,'Actual independent ZIP/protobuf/SQLite content inspection of the fresh output.'));
  }else evidence.push(await proof(`${base}/executions/basic/rust/inspection-python.log`,'Actual recovery package inspection: two notes, two cards, zero media.'));
  results[locale].taskResults.push({id:task.id,status:'passed',openIssues:prior.entries[locale].taskResults.find(t=>t.id===task.id).openIssues||[],readUrls:task.readUrls,steps:[...task.steps,'Compared the complete rendered body, heading and code blocks with the previously reviewed artifact\'s tour: identical (the HTML bytes are identical). Known minor wording observations are retained, not declared fixed.','Matched actual clipboard program bytes to current source, where applicable; used this reviewer\'s fresh public 0.3.1 SDK executions and independently inspected outputs.'],executionEvidence:[{...log,sourceBindings:execution.sourceBindings,dependencyBindings:execution.dependencyBindings}],evidence});
 }
}
await writeFile(`${base}/usage-${env}-reading.json`,JSON.stringify(comparison,null,2)+'\n');
const method='FRESH in this review (2026-11-02, commit 75a8748c): (1) independent full manifest and dist-tree byte comparison with the previously reviewed bing-seo artifact (artifact-comparison.json) and decoding of every changed /generated download (generated-download-inspection.json); (2) a new real Chromium tour of the frozen '+env+' artifact served over loopback, 6 tasks x 6 locales, including homepage navigation, localized Pagefind search, sidebar navigation, real clipboard copies matched to current sources, the four JSON recipe input downloads, editorial pages, Japanese clipboard denial/recovery and word-form searches, with every loaded response hashed against the manifest; (3) new executions installing the published 0.3.1 packages from crates.io/PyPI/npm (basic Rust/Python/Node, duplicate-key reproduction, JSON recipe v1/v2 in all three SDKs) and two independent APKG inspections. RETAINED: only the earlier nonblocking minor wording observations (openIssues) and the detailed linguistic judgments behind them, and only after asserting exact equality of every task page body, heading and rendered code with the previous tour. No previous execution, browser or registry claim is rebound. REBIND: the browser tour/HTTP reads, artifact comparison and download inspection were re-run against the rebuilt artifact (inputSha256 57aae622...) whose only input change is website/post-build-reviews.config.json pointing at these reports (rebuild-input-verification.json; all HTML/Markdown/llms bytes identical to the first release-0.3.2 build, only /generated timestamps differ). The SDK executions of this review were kept because every source/dependency binding still matches the current files.';
const report={schemaVersion:1,kind:'usage-tasks',status:'passed',mode:'release-candidate',reviewerType:'ai',reviewerRun:`/root/usage_release_review/release-0.3.2-${env}-2026-11-02`,inputSha256:manifest.inputSha256,configuration:manifest.configuration,releaseLocales:manifest.releaseLocales,readOrigin:tour.readOrigin,originManifest:{path:manifestPath,sha256:hash(manifestBytes),commit:manifest.commit},method,resourceBindings:[...bindingRoutes].sort().map(route=>({route,sha256:routes.get(route).sha256})),entries:results,openIssues:prior.openIssues||[],reviewScope:'New six-language six-task actual browser tour and new public-package SDK executions for the release-0.3.2 artifact. The entire artifact inventory was hashed against the manifest. Existing minor wording observations are retained, not declared fixed. No direct Anki GUI import is claimed; APKG content and the documented import boundary were checked.',limitations:['Local loopback serving does not establish production hosting headers or live publication.','No Anki GUI import/playback was performed.','The website deliberately still teaches the published 0.3.1 packages; this review verified that path, not the unreleased 0.3.2 packages.'],readingEvidence:await proof(`${base}/usage-${env}-reading.json`,'Actual localized task excerpts from this fresh tour, plus exact full-body/heading/code equality against the previously reviewed tour.'),priorReadingLineage:await proof(`${priorBase}/usage-tasks-${env}.json`,'Previous immutable independent review whose minor wording observations are retained; its execution and browser claims are not reused.')};
await writeFile(`${base}/usage-tasks-${env}.json`,JSON.stringify(report,null,2)+'\n');
console.log(`Recorded ${env}: ${Object.values(results).flatMap(e=>e.taskResults).length} actual task reviews with fresh SDK executions; ${report.resourceBindings.length} resource bindings.`);
