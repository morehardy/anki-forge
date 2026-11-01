import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
const out=path.resolve(process.argv[2]);
const repo='/Users/hp/Desktop/2026/anki-forge/.worktrees/spanish-integration';
const node='/Users/hp/.nvm/versions/node/v24.19.0/bin/node';
const read=async()=>JSON.parse(await fs.readFile(path.join(out,'session.json')));
const s=await read(),m=JSON.parse(await fs.readFile(path.join(repo,s.originManifest.path)));
const tasks={'python-install':['python-quickstart','compatibility','python-api'],'node-api':['node-api','node-quickstart'],'json-update':['json-to-anki','updates','media'],'compatibility-constraints':['compatibility','build-guarantees']};
const plan={reviewerRun:'/root/spanish_spec/machine-reading-2026-10-31',readOrigin:s.readOrigin,method:'Fetch this locale llms afresh for each task, parse actual linked URLs, follow matching same-locale Markdown links on loopback with canonical path preserved. Content judgement and answers are authored separately after reading the captured bodies.',tasks:[]};
for(const locale of m.releaseLocales){
 const home=m.pages.find(p=>p.id==='home'&&p.locale===locale);
 for(const [id,topics] of Object.entries(tasks)){
  const llms=home.route+'llms.txt';
  execFileSync(node,[path.join(out,'fetch.mjs'),out,locale+' '+id+': start with locale llms index',llms],{cwd:repo,maxBuffer:16*1024*1024});
  const entry=(await read()).fetches.at(-1).results[0];
  assert.equal(entry.status,200);assert.ok(entry.checks.matchesManifest);
  const body=await fs.readFile(path.join(out,entry.bodyFile),'utf8');
  const links=[...body.matchAll(/\[([^\]]+)\]\((https?:\/\/[^)]+)\)/g)].map(x=>({label:x[1],canonicalUrl:x[2],route:new URL(x[2]).pathname}));
  const followed=topics.map(topic=>{
   const page=m.pages.find(p=>p.id===topic&&p.locale===locale);assert.ok(page);
   const link=links.find(l=>l.route===page.route+'index.md');assert.ok(link,locale+' '+id+' missing llms link '+topic);return{topic,...link};
  });
  execFileSync(node,[path.join(out,'fetch.mjs'),out,locale+' '+id+': follow llms links '+followed.map(f=>f.label).join(' | '),...followed.map(f=>f.route)],{cwd:repo,maxBuffer:16*1024*1024});
  const results=(await read()).fetches.at(-1).results;
  for(const r of results){assert.equal(r.status,200);assert.ok(r.checks.matchesManifest&&r.checks.utf8Decodes&&r.checks.utf8Charset&&r.checks.machineContentType);}
  plan.tasks.push({locale,id,entryUrl:s.readOrigin+llms,entryBody:entry.bodyFile,followed,bodyFiles:results.map(r=>r.bodyFile)});
  console.log(locale+' '+id+': '+followed.length+' linked Markdown responses captured');
 }
}
await fs.writeFile(path.join(out,'reading-plan.json'),JSON.stringify(plan,null,2)+'\n');
