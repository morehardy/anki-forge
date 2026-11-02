import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import path from 'node:path';
const root=process.cwd();
const env=process.argv[2];
const directory=path.resolve(process.argv[3]);
const evidence='docs/plans/evidence/bing-seo-2026-11-02';
const oldEvidence='docs/plans/evidence/website-pr66-portable-search-2026-11-02';
const manifestPath=`${evidence}/${env}-site-manifest.json`;
const manifestBytes=await readFile(manifestPath);
const manifest=JSON.parse(manifestBytes);
const previous=JSON.parse(await readFile(`${oldEvidence}/${env}-site-manifest.json`));
const oldDir=path.resolve(`target/website-pr66-portable-search/${env}-dist`);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const strip=route=>route.slice(manifest.configuration.base==='/'?1:manifest.configuration.base.length+1);
const oldRecords=new Map(previous.resources.map(r=>[r.route,r]));
const machine=manifest.resources.filter(r=>/\/(?:index\.md|llms\.txt)$/.test(r.route));
const compared=[];
for(const r of machine){
 const bytes=await readFile(path.join(directory,strip(r.route)));
 const before=await readFile(path.join(oldDir,strip(r.route)));
 assert.equal(sha(bytes),r.sha256);
 assert.equal(sha(before),oldRecords.get(r.route).sha256);
 compared.push({route:r.route,sha256:r.sha256,previousSha256:sha(before),byteIdentical:bytes.equals(before)});
}
const responses=[];
const server=createServer(async(req,res)=>{
 try {
  const route=new URL(req.url,'http://localhost').pathname;
  const file=path.resolve(directory,strip(route));
  assert.ok(file.startsWith(directory+path.sep));
  res.setHeader('Content-Type',route.endsWith('.md')?'text/markdown; charset=utf-8':'text/plain; charset=utf-8');
  res.end(await readFile(file));
 } catch {res.statusCode=404;res.end('Not found');}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin=`http://127.0.0.1:${server.address().port}`;
const captureDir=`${evidence}/machine-${env}`;
await mkdir(`${captureDir}/bodies`,{recursive:true});
const captures=new Map();
async function get(route){
 if(captures.has(route))return captures.get(route);
 const response=await fetch(origin+route);
 assert.equal(response.status,200);
 const bytes=Buffer.from(await response.arrayBuffer());
 const record=machine.find(r=>r.route===route);
 assert.ok(record,route);
 assert.equal(sha(bytes),record.sha256);
 const file=`${captureDir}/bodies/${route.slice(1).replaceAll('/','__')}`;
 await writeFile(file,bytes);
 const captured={url:origin+route,route,status:response.status,sha256:sha(bytes),bytes:bytes.length,path:file};
 responses.push(captured);captures.set(route,{...captured,text:bytes.toString()});return captures.get(route);
}
const paths={
 'python-install':['python-quickstart','compatibility'],
 'node-api':['node-api','node-quickstart'],
 'json-update':['json-to-anki','updates','media'],
 'compatibility-constraints':['compatibility','build-guarantees'],
};
const tasks={};
try{
 for(const locale of manifest.releaseLocales){
  const home=manifest.pages.find(p=>p.id==='home'&&p.locale===locale);
  const entry=await get(home.route+'llms.txt');
  const links=[...entry.text.matchAll(/\]\((https?:[^)]+)\)/g)].map(m=>new URL(m[1]));
  tasks[locale]={};
  for(const [id,slugs]of Object.entries(paths)){
   const reads=[entry.url];
   for(const slug of slugs){
    const url=links.find(u=>u.pathname===home.route+`docs/${slug}/index.md`);
    assert.ok(url,`${locale}/${id}: not discovered in llms`);
    const page=await get(url.pathname);
    assert.ok(page.text.includes(`Language: ${locale}`));
    reads.push(page.url);
   }
   tasks[locale][id]={readUrls:reads};
  }
 }
 for(const record of compared.filter(r=>!r.byteIdentical)) await get(record.route);
}finally{server.close();}
const audit={schemaVersion:1,reviewerRun:'/root/standards_review/bing-seo-machine-'+env+'-2026-11-02',status:'bytes-and-discovery-verified-awaiting-semantic-review',readOrigin:origin,manifest:{path:manifestPath,sha256:sha(manifestBytes)},configuration:manifest.configuration,inputSha256:manifest.inputSha256,comparisons:compared,tasks,responses};
await writeFile(`${captureDir}/audit.json`,JSON.stringify(audit,null,2)+'\n');
console.log(JSON.stringify({env,machineResources:compared.length,changed:compared.filter(r=>!r.byteIdentical).map(r=>r.route),httpReads:responses.length,origin,audit:`${captureDir}/audit.json`},null,2));
