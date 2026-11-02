import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir,mkdtemp,rm} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {reviewInputs} from '../../../../website/scripts/content-review-inputs.mjs';
import {pages} from '../../../../website/site-model.mjs';
import {canonicalizePagefindEntry} from '../../../../website/scripts/search-documents.mjs';
const base='0001985b',batch='docs/plans/evidence/website-pr66-rebuild-2026-11-02',previous='docs/plans/evidence/website-pr66-final-2026-11-02/linguistic-source-review.json';
const hash=b=>createHash('sha256').update(b).digest('hex');
const binding=async p=>({path:p,sha256:hash(await readFile(p))});
const priorBytes=await readFile(previous);assert.deepEqual(priorBytes,execFileSync('git',['show',`${base}:${previous}`],{maxBuffer:20e6}));
const prior=JSON.parse(priorBytes),candidate=JSON.parse(await readFile(batch+'/source-candidate.json'));
const temp=await mkdtemp('/private/tmp/anki-forge-linguistic-pagefind-');
const inputPaths=[...new Set(Object.values(candidate.entries).flatMap(ls=>Object.values(ls).flatMap(e=>e.inputFiles)))].sort();
const inputAudit=[];
for(const p of inputPaths){const old=execFileSync('git',['show',`${base}:${p}`],{maxBuffer:10e6}),current=await readFile(p);const different=!old.equals(current);assert.equal(different,p==='website/scripts/search-documents.mjs',p);inputAudit.push({path:p,baselineSha256:hash(old),currentSha256:hash(current),changed:different});await mkdir(path.dirname(path.join(temp,p)),{recursive:true});await writeFile(path.join(temp,p),old);}
const changed=[],unchanged=[];
for(const [id,ls] of Object.entries(candidate.entries))for(const [locale,e] of Object.entries(ls)){
 const page=pages.find(p=>p.id===id);assert.ok(page,id);const old=await reviewInputs(page,locale,temp),now=await reviewInputs(page,locale,process.cwd());
 for(const k of ['sourceHash','targetHash','technicalDependencyHash']){assert.equal(old[k],prior.entries[id][locale][k]);assert.equal(now[k],e[k]);}
 assert.equal(old.sourceHash,now.sourceHash);assert.equal(old.technicalDependencyHash,now.technicalDependencyHash);
 (old.targetHash===now.targetHash?unchanged:changed).push({pageId:id,locale,previous:old,current:now});
}
assert.equal(changed.length,174);assert.equal(unchanged.length,30);
const originalPath=batch+'/pagefind-original.json',rebuiltPath=batch+'/pagefind-commit-rebuild.json';
const a=JSON.parse(await readFile(originalPath)),b=JSON.parse(await readFile(rebuiltPath));assert.deepEqual(a,b);
const probe=path.join(temp,'artifact');await mkdir(path.join(probe,'pagefind'),{recursive:true});const target=path.join(probe,'pagefind/pagefind-entry.json');let canonical;
const keys=Object.keys(a.languages);const observations=[];
for(let n=0;n<keys.length;n++){
 const reordered=[...keys.slice(n),...keys.slice(0,n)];await writeFile(target,JSON.stringify({...a,languages:Object.fromEntries(reordered.map(k=>[k,a.languages[k]]))}));await canonicalizePagefindEntry(probe);const bytes=await readFile(target);assert.deepEqual(JSON.parse(bytes),a);if(canonical)assert.deepEqual(bytes,canonical);canonical=bytes;await canonicalizePagefindEntry(probe);assert.deepEqual(await readFile(target),bytes);observations.push({inputOrder:reordered,canonicalSha256:hash(bytes),allParsedValuesEqual:true,idempotent:true});
}
const localeMapping=['en','zh-CN','es','pt-BR','ja','de'].map(locale=>{const key=locale.toLocaleLowerCase();assert.ok(a.languages[key]);assert.deepEqual(a.languages[key],JSON.parse(canonical).languages[key]);return{locale,pagefindKey:key,exactMatch:true,index:a.languages[key]};});
const jsPath='target/website-pr66-final/root-dist/pagefind/pagefind.js';const js=await readFile(jsPath,'utf8');
assert.ok(js.includes('const langCode=document.querySelector("html")?.getAttribute("lang")||"unknown";this.primaryLanguage=langCode.toLocaleLowerCase();'));
assert.ok(js.includes('let index=this.languages[language];if(index)return index;'));
await writeFile(batch+'/linguistic-delta-input-audit.json',JSON.stringify({schemaVersion:1,reviewerRun:'/root/spec_review/pagefind-stability-linguistic-2026-11-02',baseline:execFileSync('git',['rev-parse',base],{encoding:'utf8'}).trim(),priorReview:await binding(previous),candidate:await binding(batch+'/source-candidate.json'),inputFiles:inputAudit,changed,unchanged,probe:{original:await binding(originalPath),commitRebuild:await binding(rebuiltPath),beforeAfterParsedEqual:true,canonicalBytes:canonical.toString(),permutations:observations,supportedLanguages:localeMapping,pagefindImplementation:await binding(jsPath),observations:['Read docs-search createInstance and init: no language override. Actual Pagefind 1.5.2 obtains html lang and lowercases it, then takes exact language-map key. All six published locales have exact matching keys; hash, wasm, page_count and include_characters remain equal.','Unknown-language Pagefind fallback sorts indexes by page_count; equal-count tie resolution can become deterministic after canonical ordering. This audit does not claim identical unsupported-locale fallback. No released page uses that path.']},scope:'Actual shared-producer delta only; every prior/current per-topic triple recomputed and all 259 input files compared against immutable baseline. Prose/UI/catalog/program inputs unchanged; prior independent full-reading judgments and unresolved issues retained.'},null,2)+'\n');
await rm(temp,{recursive:true,force:true});console.log(`Verified ${inputPaths.length} inputs, ${changed.length} changed docs targets, ${unchanged.length} unchanged entries, six actual-entry permutations and exact language selections.`);
