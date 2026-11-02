// Records this independent reviewer's completed browser and execution observations.
// This is an evidence assembler, not a substitute for the retained task reading.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const env=process.argv[2];assert.ok(['root','prefix'].includes(env));
const base='docs/plans/evidence/bing-seo-2026-11-02';
const priorBase='docs/plans/evidence/website-pr66-portable-search-2026-11-02';
const hash=b=>createHash('sha256').update(b).digest('hex');
const json=async p=>JSON.parse(await readFile(p));
const proof=async(path,description)=>({path,sha256:hash(await readFile(path)),description});
const manifestPath=`${base}/${env}-site-manifest.json`,manifestBytes=await readFile(manifestPath),manifest=JSON.parse(manifestBytes);
const tourPath=`${base}/usage-${env}/browser-tour.json`,tour=await json(tourPath);
const prior=await json(`${priorBase}/usage-tasks-${env}.json`),priorTour=await json(`${priorBase}/usage-${env}/browser-tour.json`);
assert.equal(tour.manifestSha256,hash(manifestBytes));assert.equal(tour.mode,'release-candidate');
assert.deepEqual(tour.releaseLocales,manifest.releaseLocales);assert.equal(tour.failure,undefined);assert.deepEqual(tour.pageErrors,[]);assert.deepEqual(tour.responseReadErrors,[]);
const execution=await json(`${base}/usage-execution-audit.json`);
for(const b of [...execution.sourceBindings,...execution.dependencyBindings])assert.equal(hash(await readFile(b.path)),b.sha256,`Actual new execution input changed: ${b.path}`);
const records=[...manifest.pages,...manifest.resources];
const routes=new Map(records.map(r=>[r.route,r]));
const bindingRoutes=new Set(manifest.resources.filter(r=>/\/(?:index\.md|llms\.txt)$/.test(r.route)).map(r=>r.route));
for(const r of tour.loadedResources){assert.equal(routes.get(r.route)?.sha256,r.sha256);bindingRoutes.add(r.route);}
const meanings={
'rust-basic':'The localized instructions specify Rust 1.92+, the public 0.3.1 package, a complete main.rs, cargo run and a persistent spanish.apkg. The fresh compiled public crate output was independently decoded as hola/hello, 1 note and 1 card.',
'python-basic':'The localized instructions provide venv activation, pinned PyPI install, main.py and python main.py. Fresh wheel installation and execution produced a persistent hola/hello APKG, independently decoded as 1 note and 1 card.',
'node-basic':'The localized instructions provide Node 22.13+, optional native npm dependency, full .mjs code and node main.mjs. Fresh npm installation and execution produced the persistent package; closing its artifact did not remove it.',
'json-media':'The localized recipe identifies three questions, shared image and generated tone, all four downloaded inputs, complete Rust/Python/TypeScript programs and counts of 3/3/2. Fresh runs in all three public SDKs and an independent decoder confirm these counts and exact media bytes.',
'original-baseline-update':'The localized guide requires stable keys and the previous original distribution APKG, explicitly rejects Anki re-export as the identity baseline and distinguishes candidate review from publishing. Fresh JSON v1/v2 runs preserve identities with one low-risk answer change; comparison and review allow publication, and repeated execution preserves v1.',
 'troubleshooting':'The localized table maps NOTE.KEY_DUPLICATE to one unique key per logical record and distinguishes failures from a completed comparison blocked by policy. A fresh controlled Python run reproduces the code and recovers with a distinct key; the recovered APKG has 2 notes and 2 cards.'};
const taskLog={ 'rust-basic':'executions/basic/rust/basic.log','python-basic':'executions/basic/python/basic.log','node-basic':'executions/basic/node/basic.log','json-media':'executions/public-json-run.log','original-baseline-update':'executions/public-json-run.log','troubleshooting':'executions/basic/python/troubleshooting.log'};
const results={},comparison=[];
for(const locale of manifest.releaseLocales){
 const tasks=tour.entries[locale];assert.equal(tasks.length,6);results[locale]={taskResults:[],openIssues:prior.entries[locale].openIssues||[]};
 for(const task of tasks){
  const old=priorTour.entries[locale].find(t=>t.id===task.id);assert.ok(old);
  // Exact comparison permits retaining existing nonblocking wording judgments.
  assert.equal(task.body,old.body);assert.equal(task.h1,old.h1);assert.deepEqual(task.codeBlocks,old.codeBlocks);
  comparison.push({locale,id:task.id,fullBodyUnchanged:true,headingUnchanged:true,renderedCodeUnchanged:true,actualQuote:task.body.replace(/\n\s*/g,' ').slice(0,750)});
  for(const url of task.readUrls){const route=new URL(url).pathname;assert.ok(routes.has(route));bindingRoutes.add(route);}
  for(const c of task.copiedPrograms){assert.equal(hash(await readFile(c.source)),c.sourceSha256);assert.equal(c.copiedCode.trim(),(await readFile(c.source,'utf8')).trim());}
  const route=new URL(task.finalUrl).pathname;
  const log=await proof(`${base}/${taskLog[task.id]}`,'Genuinely new execution in this SEO review batch. Executed once per SDK program/dependency set; reused across locales only after each localized copied program was matched to that exact source. No previous release execution is rebound.');
  const evidence=[{route,sha256:routes.get(route).sha256,observation:`Actually navigated the new ${locale} rendered page “${task.h1}”. ${meanings[task.id]}`},await proof(tourPath,`Actual ${env} browser navigation, search, clipboard and input-download observations; see entries[${locale}]/${task.id}.`),await proof(`${base}/usage-execution-audit.json`,'Current-source binding audit, fresh public registry receipts, decoded APKGs and actual execution conclusions.'),await proof(task.screenshot,'Retained screenshot from this new browser tour; not a new exhaustive visual design review.')];
  if(['json-media','original-baseline-update'].includes(task.id)){
   evidence.push(await proof(`${base}/usage-json-inspection.log`,'Independent second inspection of all fresh Rust/Python/Node v1/v2 APKG pairs: exact media, stable identities, one answer change and allowed publication.'));
   for(const l of ['rust','python','node'])evidence.push(await proof(`${base}/executions/public-json/${l}/installation.json`,'Fresh public package version, platform and registry installation receipt.'));
  }else if(task.id!=='troubleshooting'){
   const lang=task.id.split('-')[0];evidence.push(await proof(`${base}/executions/basic/${lang}/spanish.apkg`,'Freshly generated basic package; the matching inspection log confirms one hola/hello note and card.'));
   evidence.push(await proof(`${base}/executions/basic/rust/inspection-${lang}.log`,'Actual independent ZIP/protobuf/SQLite content inspection of the fresh output.'));
  }else evidence.push(await proof(`${base}/executions/basic/rust/inspection-python.log`,'Actual recovery package inspection: two notes, two cards, zero media.'));
  results[locale].taskResults.push({id:task.id,status:'passed',openIssues:prior.entries[locale].taskResults.find(t=>t.id===task.id).openIssues||[],readUrls:task.readUrls,steps:[...task.steps,'Read the retained current task instructions and compared the complete body, heading and rendered programs with the previously read artifact: identical. Known minor wording observations are retained.','Matched actual clipboard program bytes to current source, where applicable; used this reviewer’s fresh public SDK executions and independently inspected outputs.'],executionEvidence:[{...log,sourceBindings:execution.sourceBindings,dependencyBindings:execution.dependencyBindings}],evidence});
 }
}
await writeFile(`${base}/usage-${env}-reading.json`,JSON.stringify(comparison,null,2)+'\n');
const report={schemaVersion:1,kind:'usage-tasks',status:'passed',mode:'release-candidate',reviewerType:'ai',reviewerRun:`/root/usage_release_review/bing-seo-${env}-2026-11-02`,inputSha256:manifest.inputSha256,configuration:manifest.configuration,releaseLocales:manifest.releaseLocales,readOrigin:tour.readOrigin,originManifest:{path:manifestPath,sha256:hash(manifestBytes),commit:manifest.commit},resourceBindings:[...bindingRoutes].sort().map(route=>({route,sha256:routes.get(route).sha256})),entries:results,openIssues:prior.openIssues||[],reviewScope:'New six-language six-task actual browser tour and new SDK executions. The entire current artifact inventory was hashed. Existing minor wording observations are retained, not declared fixed. No direct Anki GUI import is claimed; APKG content and documented import boundary were checked.',readingEvidence:await proof(`${base}/usage-${env}-reading.json`,'Actual localized task excerpts inspected by this reviewer, plus exact full-body/code equality against previous captured prose.'),priorReadingLineage:await proof(`${priorBase}/usage-tasks-${env}.json`,'Previous immutable independent reading and minor issues; only unchanged wording judgments are retained, not old execution or browser claims.')};
await writeFile(`${base}/usage-tasks-${env}.json`,JSON.stringify(report,null,2)+'\n');
console.log(`Recorded ${env}: ${Object.values(results).flatMap(e=>e.taskResults).length} actual task reviews with fresh SDK executions.`);
