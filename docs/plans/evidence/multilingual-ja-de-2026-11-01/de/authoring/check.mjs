import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
const authoring=path.dirname(fileURLToPath(import.meta.url));
const base=path.dirname(authoring);
const root=path.resolve(base,'../../../../..');
const { sectionMapping, headings }=await import(path.join(root,'website/scripts/content-sections.mjs'));
const read=p=>fs.readFile(path.join(root,p),'utf8');
const json=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const sha=s=>createHash('sha256').update(s).digest('hex');
const contracts=await json(path.join(root,'website/section-contracts.json'));
const sources=await json(path.join(authoring,'sources.json'));
const patch=await json(path.join(authoring,'de-data-patch.json'));
const manifest={};const inputs={};const failures=[];
const fences=s=>[...s.matchAll(/^```[^\n]*\n[\s\S]*?^```/gm)].map(m=>m[0]);
const strip=s=>s.replace(/^```[^\n]*\n[\s\S]*?^```/gm,'');
const inline=s=>[...strip(s).matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map(m=>m[1]);
const expanded=async s=>s.replace('<CodeBlock code={quickstartSource} />','```rust\n'+(await read('anki_forge/examples/target_api_basic.rs')).trimEnd()+'\n```');
const report={schemaVersion:1,locale:'de',kind:'author-static-checks',reviewerApproval:false,generatedAt:new Date().toISOString(),documents:[],blogs:[],dictionaries:[],dataPatch:{},limitations:['No independent linguistic or technical approval.','No site build, Pagefind ranking, browser layout or package execution performed.','Paragraph and byte statistics are authoring evidence, not proof of semantic equivalence.']};
for(const d of sources){
 const source=await read(d.source);inputs[d.source]=sha(source);
 const targetPath=path.join(base,'staging/docs/de',d.name==='docs'?'index.md':d.name+'.md');
 const target=await fs.readFile(targetPath,'utf8');manifest[path.relative(base,targetPath)]=sha(target);
 const effective=await expanded(source);const a=fences(effective),b=fences(target);
 const unchanged=JSON.stringify(a)===JSON.stringify(b);if(!unchanged)failures.push(d.name+': changed code fences');
 let sections;try{sections=sectionMapping(d.name,'de',target,{},contracts);}catch(e){failures.push(e.message);}
 const dynamicLinks=inline(effective).filter(s=>s.startsWith('${repository}/')).map(expression=>({expression,resolved:expression.replace('${repository}','https://github.com/morehardy/anki-forge')}));
 for(const link of dynamicLinks)if(!target.includes(']('+link.resolved+')'))failures.push(d.name+': missing expanded repository link '+link.resolved);
 const missingInline=inline(effective).filter(s=>!s.startsWith('${repository}/')&&!inline(target).includes(s));
 if(missingInline.length)failures.push(d.name+': missing inline code '+JSON.stringify(missingInline));
 if(/\[\[CODE\d+\]\]/.test(target))failures.push(d.name+': unresolved code placeholder');
 report.documents.push({name:d.name,source:d.source,target:path.relative(base,targetPath),sourceSha256:sha(source),targetSha256:sha(target),codeBlocks:a.length,codeBytesUnchanged:unchanged,sectionMapping:sections,dynamicRepositoryLinks:dynamicLinks,missingInlineCode:missingInline,sourceProseBytes:Buffer.byteLength(strip(effective)),targetProseBytes:Buffer.byteLength(strip(target))});
}
for(const name of ['your-first-anki-deck','stable-ids-and-deck-updates']){
 const sourcePath=`website/src/content/blog/${name}.mdx`;const targetPath=path.join(base,'staging/website/src/content/blog/de',name+'.mdx');
 const source=await read(sourcePath);const target=await fs.readFile(targetPath,'utf8');inputs[sourcePath]=sha(source);manifest[path.relative(base,targetPath)]=sha(target);
 const componentSource=name==='your-first-anki-deck'?'anki_forge/examples/target_api_basic.rs':'anki_forge/examples/website_showcase.rs';
 const raw=await read(componentSource);inputs[componentSource]=sha(raw);
 const expected=name==='your-first-anki-deck'?'<CodeBlock code={quickstartSource} />':'<CodeBlock code={updateExample.source} />';
 const dynamicCodeSame=source.includes(expected)&&target.includes(expected);if(!dynamicCodeSame)failures.push(name+': dynamic code component differs');
 for(const field of ['translationKey','date','category'])if(source.match(new RegExp('^'+field+': (.+)$','m'))?.[1]!==target.match(new RegExp('^'+field+': (.+)$','m'))?.[1])failures.push(name+': changed '+field);
 let sections;try{sections=sectionMapping('blog:'+name,'de',target,{},contracts);}catch(e){failures.push(e.message);}
 report.blogs.push({name,sourceSha256:sha(source),targetSha256:sha(target),dynamicCodeSame,componentSource,componentSourceSha256:sha(raw),sectionMapping:sections});
}
function leaves(v,p=''){if(typeof v==='string')return [[p,v]];return Object.entries(v).flatMap(([k,x])=>leaves(x,p?`${p}.${k}`:k));}
const variables=s=>[...s.matchAll(/\{\{?([A-Za-z][\w]*)\}?\}|\[([A-Z_]+)\]/g)].map(m=>m[0]).sort();
for(const rel of ['website/src/i18n','website/src/content/i18n']){
 const source=await read(`${rel}/en.json`);const targetPath=path.join(base,'staging',rel,'de.json');const target=await fs.readFile(targetPath,'utf8');
 const en=JSON.parse(source),de=JSON.parse(target);inputs[`${rel}/en.json`]=sha(source);manifest[path.relative(base,targetPath)]=sha(target);
 const sameKeys=JSON.stringify(Object.keys(en).sort())===JSON.stringify(Object.keys(de).sort());if(!sameKeys)failures.push(rel+': key mismatch');
 const targetLeaves=new Map(leaves(de));const errors=[];
 for(const [k,v] of leaves(en))if(JSON.stringify(variables(v))!==JSON.stringify(variables(targetLeaves.get(k)??'')))errors.push(k);
 if(errors.length)failures.push(rel+': interpolation mismatch '+errors.join(','));
 const plurals=Object.entries(de).filter(([,v])=>v?.type==='plural').map(([key,v])=>({key,forms:Object.keys(v.forms),samples:Object.fromEntries([0,1,2,1.5].map(n=>[n,new Intl.PluralRules('de').select(n)]))}));
 for(const p of plurals)for(const category of new Intl.PluralRules('de').resolvedOptions().pluralCategories)if(!p.forms.includes(category))failures.push(p.key+': missing plural '+category);
 report.dictionaries.push({path:path.relative(base,targetPath),keys:Object.keys(de).length,sameKeys,interpolationErrors:errors,plurals});
}
const names=sources.map(d=>d.name).sort();
if(JSON.stringify(Object.keys(patch.documents).sort())!==JSON.stringify(names))failures.push('metadata document keys mismatch');
const searchNames=[...names,'blog:your-first-anki-deck','blog:stable-ids-and-deck-updates'].sort();
if(JSON.stringify(Object.keys(patch.searchTerms).sort())!==JSON.stringify(searchNames))failures.push('search terms keys mismatch');
const ai=await json(path.join(root,'docs/ai-quickstart.json'));
if(JSON.stringify(variables(ai.prompts.en))!==JSON.stringify(variables(patch.prompts.de)))failures.push('prompt interpolation mismatch');
for(const required of patch.promptContracts.de.required)if(!patch.prompts.de.includes(required))failures.push('prompt missing '+required);
const words=patch.prompts.de.trim().split(/\s+/u).length;if(words>patch.promptContracts.de.maxWords)failures.push('prompt too long');
report.dataPatch={documents:Object.keys(patch.documents).length,docGroups:patch.docGroups.length,navigation:Object.keys(patch.navigation).length,searchTerms:Object.keys(patch.searchTerms).length,searchConcepts:Object.keys(patch.searchConcepts).length,promptWords:words,independentSearchQueriesRead:false};
const docCount=(await fs.readdir(path.join(base,'staging/docs/de'))).length;const blogCount=(await fs.readdir(path.join(base,'staging/website/src/content/blog/de'))).length;
report.counts={documents:docCount,blogs:blogCount,stagingFiles:Object.keys(manifest).length};if(docCount!==29||blogCount!==2||Object.keys(manifest).length!==33)failures.push('directory count mismatch');
report.failures=failures;report.status=failures.length?'failed':'passed';
await fs.writeFile(path.join(authoring,'authoring-checks.json'),JSON.stringify(report,null,2)+'\n');
await fs.writeFile(path.join(authoring,'source-input-hashes.json'),JSON.stringify(inputs,null,2)+'\n');
await fs.writeFile(path.join(authoring,'staging-sha256.json'),JSON.stringify(manifest,null,2)+'\n');
console.log(JSON.stringify({status:report.status,counts:report.counts,dataPatch:report.dataPatch,failures},null,2));if(failures.length)process.exitCode=1;
