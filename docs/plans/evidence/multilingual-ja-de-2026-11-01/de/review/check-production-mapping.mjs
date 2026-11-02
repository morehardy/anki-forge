import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { pages, localeIds } from '../../../../../../website/site-model.mjs';
import { reviewInputs } from '../../../../../../website/scripts/content-review-inputs.mjs';
import { sectionMapping } from '../../../../../../website/scripts/content-sections.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../../../..');
const baseline='270f26aa5653a0f56297f46fd35e586da444622b';
const basePath='docs/plans/evidence/multilingual-ja-de-2026-11-01';
const priorPath=`${basePath}/ja/linguistic-source-review-final.json`;
const priorCandidatePath=`${basePath}/ja/source-candidates-final-v2.json`;
const candidatePath=`${basePath}/de/source-candidates-preview.json`;
const oldLocales=['en','zh-CN','es','pt-BR','ja'];
const read=p=>fs.readFile(path.join(root,p),'utf8');
const json=async p=>JSON.parse(await read(p));
const hash=s=>createHash('sha256').update(s).digest('hex');
const old=p=>execFileSync('git',['show',`${baseline}:${p}`],{cwd:root,maxBuffer:16*1024*1024});
const issues=[];const assert=(cond,message)=>{if(!cond)issues.push(message);};
const priorRaw=old(priorPath),prior=JSON.parse(priorRaw),priorCandidateRaw=old(priorCandidatePath),priorCandidate=JSON.parse(priorCandidateRaw);
const candidateRaw=await read(candidatePath),candidate=JSON.parse(candidateRaw);
assert(hash(priorCandidateRaw)===prior.reviewMethod.snapshot.sha256,'Prior final linguistic report points at a different snapshot');
assert(prior.reviewerRun.includes('/root/de_full_author/'),'Prior Japanese report reviewer identity differs');
const oldFiles=[...new Set(Object.values(priorCandidate.entries).flatMap(languages=>Object.values(languages).flatMap(entry=>entry.inputFiles)))].sort();
const temp=await fs.mkdtemp(path.join(os.tmpdir(),'de-linguistic-270f26a-'));
const fileChecks=[];
for(const file of oldFiles){const before=old(file),current=await read(file);const dest=path.join(temp,file);await fs.mkdir(path.dirname(dest),{recursive:true});await fs.writeFile(dest,before);fileChecks.push({path:file,baselineSha256:hash(before),currentSha256:hash(current),unchanged:hash(before)===hash(current)});}
const priorBindings=[];
for(const page of pages)for(const locale of oldLocales){
 const actual=await reviewInputs(page,locale,temp),sealed=priorCandidate.entries[page.id]?.[locale],judgment=prior.entries[page.id]?.[locale];
 const match=sealed&&['sourceHash','targetHash','technicalDependencyHash'].every(k=>actual[k]===sealed[k]&&sealed[k]===judgment[k]);
 assert(match,`Baseline signature mismatch: ${page.id}/${locale}`);
 assert(judgment?.linguistic.status==='reviewed',`Missing prior linguistic review: ${page.id}/${locale}`);
 priorBindings.push({page:page.id,locale,immutableBaselineMatchesFinalV2:!!match,priorLinguisticStatus:judgment?.linguistic.status,priorScore:judgment?.linguistic.score});
}
const priorIssues=[];
for(const [page,languages] of Object.entries(prior.entries))for(const [locale,entry] of Object.entries(languages))for(const issue of entry.linguistic.openIssues){
 const stillPresent=(await read(issue.file)).includes(issue.span);assert(stillPresent,`Prior minor span changed: ${page}/${locale}/${issue.id}`);priorIssues.push({page,locale,id:issue.id,severity:issue.severity,file:issue.file,span:issue.span,stillPresent});
}
const baseNotes=await json(`${basePath}/de/review/linguistic-blind-read.json`),deltaNotes=await json(`${basePath}/de/review/linguistic-delta-tools-persistence.json`);
const reviewedHashes={...baseNotes.inputHashes};
for(const finding of baseNotes.findings)Object.assign(reviewedHashes,finding.resolution.files);
reviewedHashes[baseNotes.subsequentClarification.file]=baseNotes.subsequentClarification.currentSha256;
for(const entry of deltaNotes.entries)reviewedHashes[entry.file]=entry.currentSha256;
const prefix=`${basePath}/de/staging/`;
const germanMapping=[];
for(const [staged,reviewed] of Object.entries(reviewedHashes)){
 if(!staged.startsWith(prefix))continue;
 const production=staged.slice(prefix.length),actual=hash(await read(production)),stagedCurrent=hash(await read(staged));
 assert(actual===reviewed,`German production differs from read input: ${production}`);assert(stagedCurrent===actual,`German staging differs from production: ${production}`);
 germanMapping.push({production,staged,reviewedSha256:reviewed,currentSha256:actual,reviewedBytesMatch:actual===reviewed,stagingBytesMatch:stagedCurrent===actual});
}
const mappings=[];
for(const page of pages){
 const file=page.sources.de;
 if(page.type==='docs'||page.type==='article')try{mappings.push({page:page.id,file,sections:sectionMapping(page.id,'de',await read(file))});}catch(error){issues.push(error.message);}
}
const patch=await json(`${basePath}/de/authoring/de-data-patch.json`);
const projections=[];
for(const file of ['docs/ai-quickstart.json','website/language-briefs.json','website/search-terms.json','website/search-concepts.json']){
 const before=JSON.parse(old(file)),current=await json(file);
 for(const locale of oldLocales){let a,b;if(file==='docs/ai-quickstart.json'){a=Object.fromEntries(['prompts','requestPlaceholders','promptContracts'].map(k=>[k,before[k][locale]]));b=Object.fromEntries(['prompts','requestPlaceholders','promptContracts'].map(k=>[k,current[k][locale]]));}else if(file.includes('search-')){a=before.locales[locale];b=current.locales[locale];}else{a=before[locale];b=current[locale];}const equal=JSON.stringify(a)===JSON.stringify(b);assert(equal,`Existing locale data changed: ${file}/${locale}`);projections.push({file,locale,unchanged:equal});}
}
const germanProjectionChecks=[];
const prompt=await json('docs/ai-quickstart.json'),briefs=await json('website/language-briefs.json'),terms=await json('website/search-terms.json'),concepts=await json('website/search-concepts.json');
for(const [name,a,b] of [['prompts',prompt.prompts.de,patch.prompts.de],['requestPlaceholders',prompt.requestPlaceholders.de,patch.requestPlaceholders.de],['promptContracts',prompt.promptContracts.de,patch.promptContracts.de],['languageBrief',briefs.de,patch.languageBrief],['searchTerms',terms.locales.de,patch.searchTerms],['searchConcepts',concepts.locales.de,patch.searchConcepts]]){const same=JSON.stringify(a)===JSON.stringify(b);germanProjectionChecks.push({name,equalsReadPatch:same});}
const candidateBindings=[];
for(const page of pages)for(const locale of localeIds){const input=await reviewInputs(page,locale,root);const entry=candidate.entries[page.id]?.[locale];const same=entry&&['sourceHash','targetHash','technicalDependencyHash'].every(k=>input[k]===entry[k]);assert(same,`Preview candidate drift: ${page.id}/${locale}`);candidateBindings.push({page:page.id,locale,matchesCurrent:!!same});}
const output={schemaVersion:1,date:'2026-11-01',status:'pre-final-mapping-and-delta-check-only',finalSourceReport:false,releaseReady:false,reviewerTask:'/root/ja_full_author',baselineCommit:baseline,priorEvidence:{path:priorPath,sha256:hash(priorRaw),reviewerRun:prior.reviewerRun,scope:'All 170 old-locale judgments, including independent Japanese full review and the media/internal-tools delta.'},priorSnapshot:{path:priorCandidatePath,sha256:hash(priorCandidateRaw)},previewCandidate:{path:candidatePath,sha256:hash(candidateRaw)},oldLocales,oldInputFileChecks:fileChecks,changedOldDependencyFiles:fileChecks.filter(x=>!x.unchanged).map(x=>x.path),priorBaselineSignatureBindings:priorBindings,retainedMinorObservations:priorIssues,germanProductionMappings:germanMapping,sectionMappings:mappings,oldLocalizedProjections:projections,germanSharedPatchChecks:germanProjectionChecks,currentCandidateBindings:candidateBindings,issues,independenceBoundary:'Reviewer authored original Japanese translations. Existing Japanese prose is not approved anew: its exact immutable inputs and independent reviewer lineage are validated. This reviewer will assess only the independently authored German additions and new shared six-language delta.',limitations:['No final report is issued before Pagefind/term corrections settle.','This check does not claim a fresh full reading of unchanged five-locale prose.','No independent query content was read.','No source, dist or git mutation.']};
await fs.writeFile(path.join(here,'production-mapping-and-five-locale-delta-preview.json'),JSON.stringify(output,null,2)+'\n');
await fs.rm(temp,{recursive:true,force:true});
console.log(JSON.stringify({oldInputs:fileChecks.length,changed:output.changedOldDependencyFiles,priorBindings:priorBindings.length,priorMinorObservations:priorIssues.length,germanMappings:germanMapping.length,sectionMappings:mappings.length,germanPatch:germanProjectionChecks,candidateBindings:candidateBindings.length,issues},null,2));
if(issues.length)process.exitCode=1;
