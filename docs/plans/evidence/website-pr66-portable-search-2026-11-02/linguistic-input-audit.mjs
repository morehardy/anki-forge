import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir,mkdtemp,rm,symlink} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {reviewInputs,reviewInputFiles} from '../../../../website/scripts/content-review-inputs.mjs';
import {pages} from '../../../../website/site-model.mjs';
const base='e9fbc7b4a31dbad934455b0ec1b58695be4dc54e',batch='docs/plans/evidence/website-pr66-portable-search-2026-11-02';
const previousPaths=['docs/plans/evidence/website-pr66-final-2026-11-02/linguistic-source-review.json','docs/plans/evidence/website-pr66-rebuild-2026-11-02/linguistic-source-review.json'];
const hash=b=>createHash('sha256').update(b).digest('hex');
const binding=async p=>({path:p,sha256:hash(await readFile(p))});
const git=p=>execFileSync('git',['show',`${base}:${p}`],{maxBuffer:30e6});
const prior={};
for(const p of previousPaths){const bytes=await readFile(p);assert.deepEqual(bytes,git(p));const report=JSON.parse(bytes);for(const [id,ls] of Object.entries(report.entries))for(const [l,e] of Object.entries(ls)){prior[id]||={};prior[id][l]={entry:e,report:p};}}
const candidate=JSON.parse(await readFile(batch+'/source-candidate.json'));
const temp=await mkdtemp('/private/tmp/anki-forge-portable-linguistic-');
const paths=[...new Set(Object.values(candidate.entries).flatMap(ls=>Object.values(ls).flatMap(e=>e.inputFiles)))].sort();
const changes=new Set(['website/astro.config.mjs','website/scripts/search-documents.mjs']);
const added='website/vendor/pagefind/1.5.2/runtime.json';const fileAudit=[];
for(const p of paths){const current=await readFile(p);if(p===added){fileAudit.push({path:p,added:true,currentSha256:hash(current),currentContent:current.toString()});continue;}const before=git(p),changed=!before.equals(current);assert.equal(changed,changes.has(p),p);fileAudit.push({path:p,baselineSha256:hash(before),currentSha256:hash(current),changed,...(changed?{before:before.toString(),current:current.toString()}: {})});await mkdir(path.dirname(path.join(temp,p)),{recursive:true});await writeFile(path.join(temp,p),before);}
for(const p of ['website/scripts/content-review-inputs.mjs','website/scripts/content-sections.mjs']){await mkdir(path.dirname(path.join(temp,p)),{recursive:true});await writeFile(path.join(temp,p),git(p));}
await symlink(path.resolve('website/node_modules'),path.join(temp,'website/node_modules'),'dir');
const oldCalculator=await import(pathToFileURL(path.join(temp,'website/scripts/content-review-inputs.mjs')));
const affected=[],unchanged=[];
for(const [id,ls] of Object.entries(candidate.entries))for(const [locale,c] of Object.entries(ls)){
 const page=pages.find(p=>p.id===id);assert.ok(page,id);const previous=prior[id][locale];const old=await oldCalculator.reviewInputs(page,locale,temp),now=await reviewInputs(page,locale,process.cwd());
 const oldFiles=await oldCalculator.reviewInputFiles(page,locale,temp),newFiles=await reviewInputFiles(page,locale,process.cwd());
 for(const k of ['sourceHash','targetHash','technicalDependencyHash']){assert.equal(old[k],previous.entry[k],id+'/'+locale+'/'+k);assert.equal(now[k],c[k],id+'/'+locale+'/'+k);}
 assert.equal(old.sourceHash,now.sourceHash);assert.equal(old.technicalDependencyHash,now.technicalDependencyHash);
 const addedFiles=newFiles.filter(p=>!oldFiles.includes(p));assert.deepEqual(addedFiles,page.type==='docs'?['website/astro.config.mjs',added]:[]);
 (old.targetHash===now.targetHash?unchanged:affected).push({pageId:id,locale,previousReview:previous.report,previous:old,current:now,addedInputFiles:addedFiles});
}
assert.equal(affected.length,174);assert.equal(unchanged.length,30);
const runtime=JSON.parse(await readFile(added)),lock=JSON.parse(await readFile('website/package-lock.json'));const pkg=lock.packages['node_modules/@pagefind/linux-x64'];
assert.equal(pkg.integrity,runtime.sourceIntegrity);assert.equal(pkg.resolved,runtime.sourceTarball);assert.equal(pkg.version,runtime.version);
const tarball='/private/tmp/pagefind-linux-x64-1.5.2.tgz',binary='/private/tmp/pr66-pagefind-linux-x64/package/bin/pagefind_extended';
assert.equal('sha512-'+createHash('sha512').update(await readFile(tarball)).digest('base64'),runtime.sourceIntegrity);assert.equal(hash(await readFile(binary)),runtime.binarySha256);
const wasm=[];for(const [name,expected] of Object.entries(runtime.files)){const p='website/vendor/pagefind/1.5.2/'+name,bytes=await readFile(p),upstream=await readFile('/private/tmp/pr66-pagefind-output-linux-x64/'+name);assert.equal(bytes.length,expected.bytes);assert.equal(hash(bytes),expected.sha256);assert.deepEqual(bytes,upstream);wasm.push({...await binding(p),bytes:bytes.length,officialGeneratedBytesEqual:true});}
const starlight='website/node_modules/@astrojs/starlight/dist/components/Page.astro',sl=await readFile(starlight,'utf8');assert.ok(sl.includes("starlightRoute.entry.data.pagefind !== false"));assert.ok(!sl.includes('config.pagefind'));
const out={schemaVersion:1,reviewerRun:'/root/spec_review/pr66-portable-search-linguistic-2026-11-02',baseline:base,priorReviews:await Promise.all(previousPaths.map(binding)),candidate:await binding(batch+'/source-candidate.json'),calculation:'Prior triples computed using immutable baseline content-review-inputs.mjs and its original closure in an isolated temporary root; current triples use current calculator and expanded closure. Every prior/current triple matches its real report/candidate. No old hash is recomputed using the new closure.',calculators:{priorSha256:hash(git('website/scripts/content-review-inputs.mjs')),current:await binding('website/scripts/content-review-inputs.mjs')},inputFiles:fileAudit,affected,unchanged,runtimeProvenance:{manifest:await binding(added),tarball:{path:tarball,integrity:runtime.sourceIntegrity,verified:true},binary:{path:binary,sha256:runtime.binarySha256,verified:true},wasm,lock:await binding('website/package-lock.json')},sourceObservations:['Sorted relative HTML paths are sequentially awaited into the same official Pagefind 1.5.2 HTML parser. URL mapping retains root-relative index paths. Existing artifact pagefind directory is skipped and removed before writing so stale shards cannot persist.','Global Starlight pagefind:false disables its addDirectory hook only. Read the actual installed Page.astro: body indexing still depends on each entry pagefind flag and excludes 404; DocsHeader renders independent DocsSearch and unchanged searchOptions.','Both astro config and runtime inventory now belong to each document target closure. Runtime version, every required language inventory entry, byte length and SHA are checked before copying official bytes. New languages fail closed until reviewed runtime inventory supports them.','No body/source/UI/metadata/glossary/program text changed. Sorted numeric page IDs can change equal-score ordering; fresh six-language search thresholds and actual browser tasks must validate the new artifact. Prior ordering or WASM byte equivalence is not claimed.'],limits:'Source linguistic delta only; no new built artifact pass, SDK execution reuse, native client import or deployment claim.'};
await writeFile(batch+'/linguistic-input-audit.json',JSON.stringify(out,null,2)+'\n');
await rm(temp,{recursive:true,force:true});console.log(`Independent old/new calculators verified ${paths.length} input files, 174 affected and 30 unchanged entries; five official runtime files verified.`);
