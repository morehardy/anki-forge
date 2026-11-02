// Independent raw HTTP capture. No task verdicts or passing answers are generated.
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
const [artifactArg,environment,locale]=process.argv.slice(2),root=process.cwd(),artifact=path.resolve(artifactArg);
const output=path.join(root,'docs/plans/evidence/multilingual-ja-de-2026-11-01',locale,`machine-${environment}`);
const sha=b=>createHash('sha256').update(b).digest('hex');
const manifestBytes=await readFile(path.join(artifact,'generated/site-manifest.json')),manifest=JSON.parse(manifestBytes);
assert.ok(['ja','de'].includes(locale));
assert.deepEqual(manifest.releaseLocales,locale==='ja'?['en','zh-CN','es','pt-BR','ja']:['en','zh-CN','es','pt-BR','ja','de']);
process.env.SITE_URL=manifest.configuration.site;process.env.SITE_BASE=manifest.configuration.base;process.env.SITE_CUSTOM_DOMAIN=manifest.configuration.customDomain||'';
const {serveBuiltSite}=await import(pathToFileURL(path.join(root,'website/scripts/serve-built-site.mjs')));
const {inspectReleaseArtifact}=await import(pathToFileURL(path.join(root,'website/scripts/release-evidence.mjs')));
const {buildInputIdentity}=await import(pathToFileURL(path.join(root,'website/scripts/site-manifest.mjs')));
const integrity=await inspectReleaseArtifact(artifact,await buildInputIdentity(root));
await mkdir(output,{recursive:true});await writeFile(path.join(output,'artifact-integrity.json'),JSON.stringify(integrity,null,2)+'\n');await writeFile(path.join(output,'origin-manifest.json'),manifestBytes);
const records=new Map([...manifest.pages,...manifest.resources].map(r=>[r.route,r]));
const server=await serveBuiltSite({directory:artifact});
const observations={schemaVersion:1,observerRun:'/root/ja-de-observation-helper',scope:'fresh-http-observations-only',manifestSha256:sha(manifestBytes),configuration:manifest.configuration,releaseLocales:manifest.releaseLocales,inputSha256:manifest.inputSha256,readOrigin:server.origin,entries:{}};
const tasks={'python-install':['python-quickstart','python-api','compatibility'],'node-api':['node-api','node-quickstart'],'json-update':['json-to-anki','updates','media'],'compatibility-constraints':['compatibility','build-guarantees']};
let number=0;
async function get(route){const response=await fetch(server.origin+route,{redirect:'manual',headers:{accept:'text/markdown, text/plain'}});const bytes=Buffer.from(await response.arrayBuffer());assert.equal(response.status,200);assert.equal(response.url,server.origin+route);assert.match(response.headers.get('content-type'),/^text\/(?:plain|markdown); charset=utf-8$/);assert.equal(response.headers.get('content-disposition'),null);assert.equal(sha(bytes),records.get(route)?.sha256);assert.equal(bytes.length,records.get(route)?.bytes);const body=new TextDecoder('utf-8',{fatal:true}).decode(bytes);const filename=`${String(++number).padStart(3,'0')}-${route.slice(1).replaceAll('/','_')}.body`;await writeFile(path.join(output,filename),bytes);return {route,url:response.url,status:response.status,headers:Object.fromEntries(response.headers),sha256:sha(bytes),bytes:bytes.length,path:path.relative(root,path.join(output,filename)),body};}
try{for(const locale of manifest.releaseLocales){observations.entries[locale]=[];const home=manifest.pages.find(p=>p.id==='home'&&p.locale===locale);for(const[id,topics]of Object.entries(tasks)){const index=await get(home.route+'llms.txt');const links=[...index.body.matchAll(/\[([^\]]+)\]\((https?:\/\/[^)]+)\)/g)].map(m=>({title:m[1],url:m[2],route:new URL(m[2]).pathname}));const followed=[];for(const topic of topics){const p=manifest.pages.find(p=>p.id===topic&&p.locale===locale),route=p.route+'index.md',link=links.find(l=>l.route===route);assert.ok(link,`${locale}/${id}: actual llms entry must link ${topic}`);assert.equal(new URL(link.url).origin,manifest.configuration.site);followed.push({topic,linkLabel:link.title,entryLink:link.url,...await get(route)});}observations.entries[locale].push({id,index,followed,readUrls:[index.url,...followed.map(r=>r.url)]});console.log(`${environment}/${locale}/${id}: actual llms entry plus ${followed.length} linked Markdown responses captured`);}}
await writeFile(path.join(output,'http-observations.json'),JSON.stringify(observations,null,2)+'\n');}finally{await server.close();}
