import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {pages} from '../../../../website/site-model.mjs';
import {reviewInputs,reviewInputFiles,digest} from '../../../../website/scripts/content-review-inputs.mjs';

const root=process.cwd();
const oldRoot='/private/tmp/anki-forge-multilingual-fixes-20261101';
const batch='docs/plans/evidence/website-pr66-final-2026-11-02';
const baseline='6d4625639fc1477222b41880d29724c162baa3be';
const read=async name=>JSON.parse(await readFile(root+'/'+name,'utf8'));
const candidate=await read(batch+'/source-candidate.json');
const priorIndex=await read('website/content-reviews.json');
const oldDelta=await read('docs/plans/evidence/multilingual-review-fixes-2026-11-01/linguistic-delta-inputs.json');
const files={},reports={},entries={};
for(const [id,ls] of Object.entries(candidate.entries)) {
  const page=pages.find(p=>p.id===id); assert.ok(page);
  entries[id]={};
  for(const [locale,current] of Object.entries(ls)) {
    const actual=await reviewInputs(page,locale,root);
    for(const key of ['sourceHash','targetHash','technicalDependencyHash'])assert.equal(actual[key],current[key],id+'/'+locale+'/'+key);
    const previousEvidence=priorIndex.entries[id][locale].linguistic.evidence;
    const reportBytes=await readFile(root+'/'+previousEvidence.path);
    assert.equal(digest(reportBytes),previousEvidence.sha256);
    reports[previousEvidence.path]={sha256:previousEvidence.sha256,reviewerRun:JSON.parse(reportBytes).reviewerRun};
    const previous=JSON.parse(reportBytes).entries[id][locale];
    const oldActual=await reviewInputs(page,locale,oldRoot);
    for(const key of ['sourceHash','targetHash','technicalDependencyHash'])assert.equal(oldActual[key],previous[key],id+'/'+locale+'/previous/'+key);
    const names=await reviewInputFiles(page,locale,root);
    assert.deepEqual(names,current.inputFiles,id+'/'+locale+'/files');
    assert.deepEqual(names,await reviewInputFiles(page,locale,oldRoot));
    for(const name of names)if(!files[name]) {
      const prior=await readFile(oldRoot+'/'+name,'utf8'),now=await readFile(root+'/'+name,'utf8');
      const git=execFileSync('git',['show',baseline+':'+name],{cwd:root}).toString();
      const expected=oldDelta.deltaSources[name]?.currentContent??git;
      assert.equal(prior,expected,'Previous input not exactly frozen baseline+recorded delta: '+name);
      const changed=now!==prior;
      files[name]={previousSha256:digest(prior),currentSha256:digest(now),changed,previousContentOrigin:oldDelta.deltaSources[name]?{path:'docs/plans/evidence/multilingual-review-fixes-2026-11-01/linguistic-delta-inputs.json',field:'deltaSources['+name+'].currentContent'}:{commit:baseline,path:name},...(changed?{previousContent:prior,currentContent:now}:{})};
    }
    entries[id][locale]={...actual,sourceFile:page.sources.en,targetFile:page.sources[locale],pageType:page.type,previousEvidence,previousHashes:oldActual,previousScore:previous.linguistic.score,previousOpenIssues:previous.linguistic.openIssues,changedInputs:names.filter(n=>files[n].changed),unchangedInputs:names.filter(n=>!files[n].changed)};
  }
}
const audit={schemaVersion:1,reviewerRun:'/root/spec_review/pr66-final-linguistic-2026-11-02',scope:'Current candidate hashes independently recomputed. Each immutable prior report SHA and prior entry hash verified against the exact older baseline plus four recorded production fixes. Every prior input byte checked against Git baseline or immutable prior full-content snapshot. Full current and prior content retained for changed inputs; unchanged inputs are pinned by exact digests and immutable origin. This is evidence of eligible delta reuse, not a claim to reread unchanged long pages.',baselineCommit:baseline,candidate:{path:batch+'/source-candidate.json',sha256:digest(await readFile(root+'/'+batch+'/source-candidate.json'))},priorReports:reports,files,entries};
await writeFile(root+'/'+batch+'/linguistic-input-audit.json',JSON.stringify(audit,null,2)+'\n');
console.log(JSON.stringify({entries:Object.values(entries).reduce((n,e)=>n+Object.keys(e).length,0),files:Object.keys(files).length,changed:Object.keys(files).filter(f=>files[f].changed),priorReports:reports},null,2));
